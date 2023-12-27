#[cfg(not(target_arch = "wasm32"))]
use napi::bindgen_prelude::Status;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::JsValue;

pub(crate) struct UrlError(pub(crate) css_inline::ParseError);

#[cfg(not(target_arch = "wasm32"))]
impl From<UrlError> for napi::Error {
    fn from(err: UrlError) -> Self {
        napi::Error::new(Status::InvalidArg, err.0.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
impl From<UrlError> for JsValue {
    fn from(error: UrlError) -> Self {
        JsValue::from_str(error.0.to_string().as_str())
    }
}

pub(crate) struct InlineError(pub(crate) css_inline::InlineError);

#[cfg(not(target_arch = "wasm32"))]
impl From<InlineError> for napi::Error {
    fn from(error: InlineError) -> Self {
        napi::Error::from_reason(error.0.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
impl From<InlineError> for JsValue {
    fn from(error: InlineError) -> Self {
        if let css_inline::InlineError::ParseError(e) = error.0 {
            JsValue::from_str(&e)
        } else {
            JsValue::from_str(error.0.to_string().as_str())
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) type JsError = JsValue;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type JsError = napi::Error;
