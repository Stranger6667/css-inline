#[cfg(not(target_arch = "wasm32"))]
use napi_derive::napi;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

mod errors;
mod options;
use options::Options;

#[cfg(not(target_arch = "wasm32"))]
#[napi]
/// Inline CSS styles from <style> tags to matching elements in the HTML tree and return a string.
pub fn inline(
    html: String,
    options: Option<Options>,
) -> std::result::Result<String, errors::JsError> {
    let options = options.unwrap_or_default();
    inline_inner(html, options)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(skip_typescript)]
/// Inline CSS styles from <style> tags to matching elements in the HTML tree and return a string.
pub fn inline(html: String, options: JsValue) -> std::result::Result<String, errors::JsError> {
    let options: Options = if options.is_undefined() {
        Options::default()
    } else {
        serde_wasm_bindgen::from_value(options)?
    };
    inline_inner(html, options)
}

// TODO: Re-check + adjust naming to camel case
#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
const INLINE: &'static str = r#"
interface InlineOptions {
    inline_style_tags?: boolean,
    keep_style_tags?: boolean,
    keep_link_tags?: boolean,
    base_url?: string,
    load_remote_stylesheets?: boolean,
    extra_css?: string,
    preallocate_node_capacity?: number,
}

export function inline(html: string, options?: InlineOptions): string;
"#;

fn inline_inner(html: String, options: Options) -> std::result::Result<String, errors::JsError> {
    let inliner = css_inline::CSSInliner::new(options.try_into()?);
    Ok(inliner.inline(&html).map_err(errors::InlineError)?)
}
