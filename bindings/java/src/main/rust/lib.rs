use core::fmt;
use css_inline::{CSSInliner, StylesheetCache};
use jni::{
    JNIEnv,
    errors::Result as JNIResult,
    objects::{JClass, JObject, JString},
    sys::jstring,
};
use std::{borrow::Cow, num::NonZeroUsize};

trait JNIExt {
    fn get_rust_string(&mut self, obj: &JString) -> String;
    fn to_jstring(&mut self, obj: String) -> jstring;
    fn get_bool_field(&mut self, obj: &JObject, name: &str) -> JNIResult<bool>;
    fn get_int_field(&mut self, obj: &JObject, name: &str) -> JNIResult<i32>;
    fn get_string_field_opt(&mut self, obj: &JObject, name: &str) -> JNIResult<Option<String>>;
}

impl<'a> JNIExt for JNIEnv<'a> {
    fn get_rust_string(&mut self, obj: &JString) -> String {
        self.get_string(&obj)
            .expect("Failed to get Java String")
            .into()
    }

    fn to_jstring(&mut self, obj: String) -> jstring {
        self.new_string(obj)
            .expect("Failed to get Java String")
            .into_raw()
    }

    fn get_bool_field(&mut self, obj: &JObject, name: &str) -> JNIResult<bool> {
        self.get_field(obj, name, "Z")?.z()
    }

    fn get_int_field(&mut self, obj: &JObject, name: &str) -> JNIResult<i32> {
        self.get_field(obj, name, "I")?.i()
    }

    fn get_string_field_opt(&mut self, cfg: &JObject, name: &str) -> JNIResult<Option<String>> {
        let value = self.get_field(cfg, name, "Ljava/lang/String;")?.l()?;
        if value.is_null() {
            Ok(None)
        } else {
            Ok(Some(self.get_string(&JString::from(value))?.into()))
        }
    }
}

enum Error<E> {
    Jni(jni::errors::Error),
    Other(E),
}

impl<E> From<jni::errors::Error> for Error<E> {
    fn from(value: jni::errors::Error) -> Self {
        Error::Jni(value)
    }
}

impl<E: fmt::Display> fmt::Display for Error<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Jni(error) => error.fmt(f),
            Error::Other(error) => error.fmt(f),
        }
    }
}

fn build_inliner(
    env: &mut JNIEnv,
    cfg: JObject,
) -> Result<CSSInliner<'static>, Error<css_inline::ParseError>> {
    let inline_style_tags = env.get_bool_field(&cfg, "inlineStyleTags")?;
    let keep_style_tags = env.get_bool_field(&cfg, "keepStyleTags")?;
    let keep_link_tags = env.get_bool_field(&cfg, "keepLinkTags")?;
    let load_remote_stylesheets = env.get_bool_field(&cfg, "loadRemoteStylesheets")?;
    let cache_size = env.get_int_field(&cfg, "cacheSize")?;
    let preallocate_node_capacity = env.get_int_field(&cfg, "preallocateNodeCapacity")?;

    let extra_css = env.get_string_field_opt(&cfg, "extraCss")?;
    let base_url = env.get_string_field_opt(&cfg, "baseUrl")?;
    let mut builder = CSSInliner::options()
        .inline_style_tags(inline_style_tags)
        .keep_style_tags(keep_style_tags)
        .keep_link_tags(keep_link_tags)
        .load_remote_stylesheets(load_remote_stylesheets)
        .extra_css(extra_css.map(Cow::Owned))
        .preallocate_node_capacity(preallocate_node_capacity as usize);

    if let Some(url) = base_url {
        match css_inline::Url::parse(&url) {
            Ok(url) => {
                builder = builder.base_url(Some(url));
            }
            Err(error) => return Err(Error::Other(error)),
        }
    }

    if cache_size > 0 {
        builder = builder.cache(StylesheetCache::new(
            NonZeroUsize::new(cache_size as usize).expect("Cache size is not null"),
        ));
    }

    Ok(builder.build())
}

fn throw(mut env: JNIEnv, message: String) -> jstring {
    let exception = env
        .find_class("org/cssinline/CssInlineException")
        .expect("CssInlineException class not found");
    env.throw_new(exception, message)
        .expect("Failed to throw CssInlineException");
    std::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_org_cssinline_CssInline_nativeInline(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
    cfg: JObject,
) -> jstring {
    let html = env.get_rust_string(&input);
    let inliner = match build_inliner(&mut env, cfg) {
        Ok(inliner) => inliner,
        Err(error) => return throw(env, error.to_string()),
    };
    match inliner.inline(&html) {
        Ok(out) => env.to_jstring(out),
        Err(error) => throw(env, error.to_string()),
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_org_cssinline_CssInline_nativeInlineFragment(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
    css: JString,
    cfg: JObject,
) -> jstring {
    let html = env.get_rust_string(&input);
    let css = env.get_rust_string(&css);
    let inliner = match build_inliner(&mut env, cfg) {
        Ok(inliner) => inliner,
        Err(error) => return throw(env, error.to_string()),
    };
    match inliner.inline_fragment(&html, &css) {
        Ok(out) => env.to_jstring(out),
        Err(error) => throw(env, error.to_string()),
    }
}
