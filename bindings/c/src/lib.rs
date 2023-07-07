use css_inline::{CSSInliner, InlineError, InlineOptions, Url};
use libc::{c_char, size_t};
use std::borrow::Cow;
use std::ffi::CStr;
use std::io::Write;
use std::ptr::null;

/// Result of CSS inlining operations
#[repr(C)]
pub enum CssResult {
    /// No error.
    Ok,
    /// Missing a stylesheet file.
    MissingStylesheet,
    /// When loading a remote stylesheet, the file is not available.
    RemoteStylesheetNotAvailable,
    /// Error in the IO layer. This error also happens when the output array is too small to fit
    /// the inlined CSS.
    IoError,
    /// Error while parsing the CSS.
    InternalSelectorParseError,
    /// options pointer is null.
    NullOptions,
    /// Invalid base_url parameter.
    InvalidUrl,
    /// Invalid extra_css parameter.
    InvalidExtraCss,
    /// input string not in UTF-8.
    InvalidInputString,
}

// must be public because the impl From<&CssInlinerOptions> for InlineOptions would leak this type
/// Error to convert to CssResult later
/// cbindgen:ignore
pub enum InlineOptionsError {
    InvalidUrl,
    InvalidExtraCss,
}

/// Configuration options for CSS inlining process.
#[repr(C)]
pub struct CssInlinerOptions {
    /// Keep "style" tags after inlining.
    pub keep_style_tags: bool,
    /// Keep "link" tags after inlining.
    pub keep_link_tags: bool,
    /// Whether remote stylesheets should be loaded or not.
    pub load_remote_stylesheets: bool,
    /// Used for loading external stylesheets via relative URLs.
    pub base_url: *const c_char,
    /// Additional CSS to inline.
    pub extra_css: *const c_char,
    /// Pre-allocate capacity for HTML nodes during parsing.
    /// It can improve performance when you have an estimate of the number of nodes in your HTML document.
    pub preallocate_node_capacity: size_t,
}

/// @brief Inline CSS from @p input & write the result to @p output with @p options.
/// @param options configuration for the inliner.
/// @param input html to inline.
/// @param output buffer to save the inlined CSS.
/// @param output_size size of @p output in bytes.
/// @return a CSS_RESULT enum variant regarding if the operation was a success or an error occurred
#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn inline_to(
    options: *const CssInlinerOptions,
    input: *const c_char,
    output: *mut c_char,
    output_size: size_t,
) -> CssResult {
    let options = CSSInliner::new(
        match InlineOptions::try_from(match options.as_ref() {
            Some(ptr) => ptr,
            None => return CssResult::NullOptions,
        }) {
            Ok(inline_options) => inline_options,
            Err(e) => return CssResult::from(e),
        },
    );
    let html = match CStr::from_ptr(input).to_str() {
        Ok(val) => val,
        Err(_) => return CssResult::InvalidInputString,
    };
    let mut buffer = CBuffer::new(output, output_size);
    if let Err(e) = options.inline_to(html, &mut buffer) {
        match e {
            InlineError::IO(_) => return CssResult::IoError,
            InlineError::Network(_) => return CssResult::RemoteStylesheetNotAvailable,
            InlineError::ParseError(_) => return CssResult::InternalSelectorParseError,
            InlineError::MissingStyleSheet { .. } => return CssResult::MissingStylesheet,
        }
    };
    // Null terminate the pointer
    let ptr: *mut c_char = buffer.buffer.add(buffer.pos);
    *ptr = 0;
    CssResult::Ok
}

/// @brief Creates an instance of CssInlinerOptions with the default parameters.
/// @return a CssInlinerOptions struct
#[no_mangle]
pub extern "C" fn css_inliner_default_options() -> CssInlinerOptions {
    CssInlinerOptions {
        keep_style_tags: false,
        keep_link_tags: false,
        base_url: null(),
        load_remote_stylesheets: true,
        extra_css: null(),
        preallocate_node_capacity: 32,
    }
}

struct CBuffer {
    buffer: *mut c_char,
    size: size_t,
    pos: usize,
}

impl TryFrom<&CssInlinerOptions> for InlineOptions<'_> {
    type Error = InlineOptionsError;

    fn try_from(value: &CssInlinerOptions) -> Result<Self, Self::Error> {
        let base_url: Option<&str> = unsafe {
            // .as_ref() returns None when the pointer is null
            match value.base_url.as_ref() {
                Some(val) => Some(match CStr::from_ptr(val).to_str() {
                    Ok(val) => val,
                    Err(_) => return Err(InlineOptionsError::InvalidUrl),
                }),
                None => None,
            }
        };
        let extra_css: Option<&str> = unsafe {
            // .as_ref() returns None when the pointer is null
            match value.extra_css.as_ref() {
                Some(val) => Some(match CStr::from_ptr(val).to_str() {
                    Ok(val) => val,
                    Err(_) => return Err(InlineOptionsError::InvalidExtraCss),
                }),
                None => None,
            }
        };
        Ok(Self {
            keep_style_tags: value.keep_style_tags,
            keep_link_tags: value.keep_link_tags,
            base_url: match base_url {
                Some(url) => Some(Url::parse(url).map_err(|_| InlineOptionsError::InvalidUrl)?),
                None => None,
            },
            load_remote_stylesheets: value.load_remote_stylesheets,
            extra_css: extra_css.map(Cow::Borrowed),
            preallocate_node_capacity: value.preallocate_node_capacity,
        })
    }
}

impl From<InlineOptionsError> for CssResult {
    fn from(value: InlineOptionsError) -> Self {
        match value {
            InlineOptionsError::InvalidUrl => CssResult::InvalidUrl,
            InlineOptionsError::InvalidExtraCss => CssResult::InvalidExtraCss,
        }
    }
}

impl CBuffer {
    fn new(buffer: *mut c_char, size: size_t) -> Self {
        Self {
            buffer,
            size,
            pos: 0,
        }
    }
}

impl Write for CBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // Because write() attempts to write regardless of buf.len() being higher than self.size,
        // we are going to check at each byte that we didn't cross the bounds
        let mut bytes_written: usize = 0;
        for byte in buf {
            // size-1 because the buffer needs to be null terminated
            if self.pos >= self.size - 1 {
                return Ok(bytes_written);
            }
            unsafe {
                let ptr: *mut c_char = self.buffer.add(self.pos);
                *ptr = *byte as i8;
            }
            self.pos += 1;
            bytes_written += 1;
        }
        Ok(bytes_written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // flush() is not used in this context because we are not using a BufWriter or similar
        // types, so only write() gets a call
        unimplemented!();
    }
}
