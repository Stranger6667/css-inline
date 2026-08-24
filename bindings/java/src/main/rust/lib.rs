use core::fmt;
use css_inline::{CSSInliner, StylesheetCache};
use jni::{
    Env, EnvUnowned,
    errors::{ErrorPolicy, Result as JNIResult},
    jni_sig, jni_str,
    objects::{JClass, JObject, JString},
    strings::{JNIStr, JNIString},
};
use std::{any::Any, borrow::Cow, num::NonZeroUsize};

trait JNIExt {
    fn get_rust_string(&mut self, obj: &JString) -> JNIResult<String>;
    fn get_bool_field(&mut self, obj: &JObject, name: &JNIStr) -> JNIResult<bool>;
    fn get_int_field(&mut self, obj: &JObject, name: &JNIStr) -> JNIResult<i32>;
    fn get_string_field_opt(&mut self, obj: &JObject, name: &JNIStr) -> JNIResult<Option<String>>;
}

impl<'a> JNIExt for Env<'a> {
    fn get_rust_string(&mut self, obj: &JString) -> JNIResult<String> {
        Ok(obj.mutf8_chars(self)?.to_string())
    }

    fn get_bool_field(&mut self, obj: &JObject, name: &JNIStr) -> JNIResult<bool> {
        self.get_field(obj, name, jni_sig!("Z"))?.z()
    }

    fn get_int_field(&mut self, obj: &JObject, name: &JNIStr) -> JNIResult<i32> {
        self.get_field(obj, name, jni_sig!("I"))?.i()
    }

    fn get_string_field_opt(&mut self, cfg: &JObject, name: &JNIStr) -> JNIResult<Option<String>> {
        let value = self
            .get_field(cfg, name, jni_sig!("Ljava/lang/String;"))?
            .l()?;
        if value.is_null() {
            Ok(None)
        } else {
            let value = self.new_cast_local_ref::<JString>(&value)?;
            self.get_rust_string(&value).map(Some)
        }
    }
}

enum Error {
    Jni(jni::errors::Error),
    Other(String),
}

impl From<jni::errors::Error> for Error {
    fn from(value: jni::errors::Error) -> Self {
        Error::Jni(value)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Jni(error) => error.fmt(f),
            Error::Other(error) => f.write_str(error),
        }
    }
}

/// Maps Rust errors & panics onto `CssInlineException`.
struct ThrowCssInlineException;

impl<T: Default, E: fmt::Display> ErrorPolicy<T, E> for ThrowCssInlineException {
    type Captures<'local: 'method, 'method> = ();

    fn on_error<'local: 'method, 'method>(
        env: &mut Env<'local>,
        _: &mut Self::Captures<'local, 'method>,
        error: E,
    ) -> JNIResult<T> {
        throw(env, error.to_string())
    }

    fn on_panic<'local: 'method, 'method>(
        env: &mut Env<'local>,
        _: &mut Self::Captures<'local, 'method>,
        payload: Box<dyn Any + Send + 'static>,
    ) -> JNIResult<T> {
        let message = match payload.downcast_ref::<&'static str>() {
            Some(message) => (*message).to_string(),
            None => match payload.downcast_ref::<String>() {
                Some(message) => message.clone(),
                None => "Unknown panic".to_string(),
            },
        };
        throw(env, format!("Panic: {message}"))
    }
}

fn throw<T: Default>(env: &mut Env, message: String) -> JNIResult<T> {
    if env.exception_check() {
        return Ok(T::default());
    }
    let exception = env.find_class(jni_str!("org/cssinline/CssInlineException"))?;
    // `throw_new` reports the exception it just threw via `Err(JavaException)`
    let _ = env.throw_new(exception, JNIString::new(message));
    Ok(T::default())
}

fn build_inliner(env: &mut Env, cfg: &JObject) -> Result<CSSInliner<'static>, Error> {
    let inline_style_tags = env.get_bool_field(cfg, jni_str!("inlineStyleTags"))?;
    let keep_style_tags = env.get_bool_field(cfg, jni_str!("keepStyleTags"))?;
    let keep_link_tags = env.get_bool_field(cfg, jni_str!("keepLinkTags"))?;
    let keep_at_rules = env.get_bool_field(cfg, jni_str!("keepAtRules"))?;
    let minify_css = env.get_bool_field(cfg, jni_str!("minifyCss"))?;
    let load_remote_stylesheets = env.get_bool_field(cfg, jni_str!("loadRemoteStylesheets"))?;
    let cache_size = env.get_int_field(cfg, jni_str!("cacheSize"))?;
    let preallocate_node_capacity = env.get_int_field(cfg, jni_str!("preallocateNodeCapacity"))?;
    let remove_inlined_selectors = env.get_bool_field(cfg, jni_str!("removeInlinedSelectors"))?;
    let apply_width_attributes = env.get_bool_field(cfg, jni_str!("applyWidthAttributes"))?;
    let apply_height_attributes = env.get_bool_field(cfg, jni_str!("applyHeightAttributes"))?;

    let extra_css = env.get_string_field_opt(cfg, jni_str!("extraCss"))?;
    let base_url = env.get_string_field_opt(cfg, jni_str!("baseUrl"))?;
    let mut builder = CSSInliner::options()
        .inline_style_tags(inline_style_tags)
        .keep_style_tags(keep_style_tags)
        .keep_link_tags(keep_link_tags)
        .keep_at_rules(keep_at_rules)
        .minify_css(minify_css)
        .load_remote_stylesheets(load_remote_stylesheets)
        .extra_css(extra_css.map(Cow::Owned))
        .preallocate_node_capacity(preallocate_node_capacity as usize)
        .remove_inlined_selectors(remove_inlined_selectors)
        .apply_width_attributes(apply_width_attributes)
        .apply_height_attributes(apply_height_attributes);

    if let Some(url) = base_url {
        match css_inline::Url::parse(&url) {
            Ok(url) => {
                builder = builder.base_url(Some(url));
            }
            Err(error) => return Err(Error::Other(error.to_string())),
        }
    }

    if cache_size > 0 {
        builder = builder.cache(StylesheetCache::new(
            NonZeroUsize::new(cache_size as usize).expect("Cache size is not null"),
        ));
    }

    Ok(builder.build())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_org_cssinline_CssInline_nativeInline<'caller>(
    mut env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    input: JString<'caller>,
    cfg: JObject<'caller>,
) -> JObject<'caller> {
    env.with_env(|env| -> Result<JObject, Error> {
        let html = env.get_rust_string(&input)?;
        let inliner = build_inliner(env, &cfg)?;
        let out = inliner
            .inline(&html)
            .map_err(|error| Error::Other(error.to_string()))?;
        Ok(JString::from_str(env, out)?.into())
    })
    .resolve::<ThrowCssInlineException>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_org_cssinline_CssInline_nativeInlineFragment<'caller>(
    mut env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    input: JString<'caller>,
    css: JString<'caller>,
    cfg: JObject<'caller>,
) -> JObject<'caller> {
    env.with_env(|env| -> Result<JObject, Error> {
        let html = env.get_rust_string(&input)?;
        let css = env.get_rust_string(&css)?;
        let inliner = build_inliner(env, &cfg)?;
        let out = inliner
            .inline_fragment(&html, &css)
            .map_err(|error| Error::Other(error.to_string()))?;
        Ok(JString::from_str(env, out)?.into())
    })
    .resolve::<ThrowCssInlineException>()
}
