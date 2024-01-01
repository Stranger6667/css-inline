#[cfg(not(target_arch = "wasm32"))]
use napi_derive::napi;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

mod errors;
mod options;
use options::Options;

#[cfg(all(
    not(target_arch = "wasm32"),
    not(debug_assertions),
    not(all(target_os = "windows", target_arch = "aarch64")),
    not(all(target_os = "linux", target_arch = "aarch64", target_env = "musl")),
))]
#[global_allocator]
static ALLOC: mimalloc_rust::GlobalMiMalloc = mimalloc_rust::GlobalMiMalloc;

#[cfg(not(target_arch = "wasm32"))]
#[napi]
/// Inline CSS styles from <style> tags to matching elements in the HTML tree and return a string.
pub fn inline(html: String, options: Option<Options>) -> Result<String, errors::JsError> {
    let options = options.unwrap_or_default();
    inline_inner(html, options)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(skip_typescript)]
/// Inline CSS styles from <style> tags to matching elements in the HTML tree and return a string.
pub fn inline(html: String, options: JsValue) -> Result<String, errors::JsError> {
    let options: Options = if options.is_undefined() {
        Options::default()
    } else {
        serde_wasm_bindgen::from_value(options)?
    };
    inline_inner(html, options)
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
    baseUrl?: string,
    loadRemoteStylesheets?: boolean,
    extraCss?: string,
    preallocateNodeCapacity?: number,
}

export function inline(html: string, options?: InlineOptions): string;
export function version(): string;"#;

fn inline_inner(html: String, options: Options) -> std::result::Result<String, errors::JsError> {
    let inliner = css_inline::blocking::CSSInliner::new(options.try_into()?);
    Ok(inliner.inline(&html).map_err(errors::InlineError)?)
}
