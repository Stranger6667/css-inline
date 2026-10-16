#[cfg(not(target_arch = "wasm32"))]
use napi_derive::napi;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::{JsValue, wasm_bindgen};

mod errors;
mod options;
use options::Options;

#[cfg(not(target_family = "wasm"))]
#[cfg(not(all(target_os = "linux", target_arch = "arm")))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(not(target_arch = "wasm32"))]
#[napi]
/// Inline CSS styles from <style> tags to matching elements in the HTML tree and return a string.
pub fn inline(html: String, options: Option<Options>) -> Result<String, errors::JsError> {
    let options = options.unwrap_or_default();
    inline_inner(html, options)
}

#[cfg(not(target_arch = "wasm32"))]
#[napi]
/// Inline CSS styles into an HTML fragment.
pub fn inline_fragment(
    html: String,
    css: String,
    options: Option<Options>,
) -> Result<String, errors::JsError> {
    let options = options.unwrap_or_default();
    inline_fragment_inner(html, css, options)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(skip_typescript)]
/// Inline CSS styles from <style> tags to matching elements in the HTML tree and return a string.
pub fn inline(html: String, options: JsValue) -> Result<String, errors::JsError> {
    inline_inner(html, parse_options(options)?)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = "inlineFragment", skip_typescript)]
/// Inline CSS styles into an HTML fragment.
pub fn inline_fragment(
    html: String,
    css: String,
    options: JsValue,
) -> Result<String, errors::JsError> {
    inline_fragment_inner(html, css, parse_options(options)?)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = "inlineAsync", skip_typescript)]
/// Inline CSS styles into the HTML, fetching remote stylesheets without blocking.
pub async fn inline_async(html: String, options: JsValue) -> Result<String, errors::JsError> {
    let inliner = css_inline::CSSInliner::new(parse_options(options)?.try_into()?);
    Ok(inliner
        .inline_async(&html)
        .await
        .map_err(errors::InlineError)?)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = "inlineFragmentAsync", skip_typescript)]
/// Inline CSS styles into an HTML fragment, fetching remote stylesheets without blocking.
pub async fn inline_fragment_async(
    html: String,
    css: String,
    options: JsValue,
) -> Result<String, errors::JsError> {
    let inliner = css_inline::CSSInliner::new(parse_options(options)?.try_into()?);
    Ok(inliner
        .inline_fragment_async(&html, &css)
        .await
        .map_err(errors::InlineError)?)
}

#[cfg(target_arch = "wasm32")]
fn parse_options(options: JsValue) -> Result<Options, errors::JsError> {
    if options.is_undefined() {
        Ok(Options::default())
    } else {
        Ok(serde_wasm_bindgen::from_value(options)?)
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip_typescript))]
#[cfg_attr(not(target_arch = "wasm32"), napi)]
/// Get the package version.
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Manually write TypeScript section to provide proper definitions for `InlineOptions`.
#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(typescript_custom_section))]
const INLINE: &'static str = r#"export interface InlineOptions {
    inlineStyleTags?: boolean,
    keepStyleTags?: boolean,
    keepLinkTags?: boolean,
    keepAtRules?: boolean,
    minifyCss?: boolean,
    baseUrl?: string,
    loadRemoteStylesheets?: boolean,
    extraCss?: string,
    preallocateNodeCapacity?: number,
    removeInlinedSelectors?: boolean,
    applyWidthAttributes?: boolean,
    applyHeightAttributes?: boolean,
}

export function inline(html: string, options?: InlineOptions): string;
export function inlineFragment(html: string, css: string, options?: InlineOptions): string;
export function inlineAsync(html: string, options?: InlineOptions): Promise<string>;
export function inlineFragmentAsync(html: string, css: string, options?: InlineOptions): Promise<string>;
export function version(): string;"#;

fn inline_inner(html: String, options: Options) -> std::result::Result<String, errors::JsError> {
    let inliner = css_inline::CSSInliner::new(options.try_into()?);
    Ok(inliner.inline(&html).map_err(errors::InlineError)?)
}

fn inline_fragment_inner(
    html: String,
    css: String,
    options: Options,
) -> std::result::Result<String, errors::JsError> {
    let inliner = css_inline::CSSInliner::new(options.try_into()?);
    Ok(inliner
        .inline_fragment(&html, &css)
        .map_err(errors::InlineError)?)
}
