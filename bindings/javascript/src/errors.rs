#[cfg(not(target_arch = "wasm32"))]
use napi::bindgen_prelude::Status;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::JsValue;

pub(crate) struct UrlError {
    pub(crate) error: css_inline::ParseError,
    pub(crate) url: String,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<UrlError> for napi::Error {
    fn from(error: UrlError) -> Self {
        napi::Error::new(
            Status::InvalidArg,
            format!("{}: {}", error.error, error.url),
        )
    }
}

#[cfg(target_arch = "wasm32")]
impl From<UrlError> for JsValue {
    fn from(error: UrlError) -> Self {
        JsValue::from_str(format!("{}: {}", error.error, error.url).as_str())
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
        match &error.0 {
            css_inline::InlineError::ParseError(e) => JsValue::from_str(e),
            css_inline::InlineError::IO(io_error)
                if io_error.kind() == std::io::ErrorKind::Unsupported =>
            {
                JsValue::from_str(io_error.to_string().as_str())
            }
            _ => JsValue::from_str(error.0.to_string().as_str()),
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) type JsError = JsValue;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) type JsError = napi::Error;
