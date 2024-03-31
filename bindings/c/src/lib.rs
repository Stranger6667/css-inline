use css_inline::{CSSInliner, DefaultStylesheetResolver, InlineError, InlineOptions, Url};
use libc::{c_char, size_t};
use std::{
    borrow::Cow,
    cmp,
    ffi::CStr,
    io::Write,
    num::NonZeroUsize,
    ptr,
    sync::{Arc, Mutex},
};

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
    /// Options pointer is null.
    NullOptions,
    /// Invalid base_url parameter.
    InvalidUrl,
    /// Invalid extra_css parameter.
    InvalidExtraCss,
    /// Input string not in UTF-8.
    InvalidInputString,
    /// Invalid cache size.
    InvalidCacheSize,
}

impl From<InlineError> for CssResult {
    fn from(value: InlineError) -> Self {
        match value {
            InlineError::IO(_) => CssResult::IoError,
            InlineError::Network { .. } => CssResult::RemoteStylesheetNotAvailable,
            InlineError::ParseError(_) => CssResult::InternalSelectorParseError,
            InlineError::MissingStyleSheet { .. } => CssResult::MissingStylesheet,
        }
    }
}

// must be public because the impl From<&CssInlinerOptions> for InlineOptions would leak this type
/// Error to convert to CssResult later
/// cbindgen:ignore
pub enum InlineOptionsError {
    /// Invalid base_url parameter.
    InvalidUrl,
    /// Invalid extra_css parameter.
    InvalidExtraCss,
    /// Invalid cache size.
    InvalidCacheSize,
}

/// An LRU Cache for external stylesheets.
#[repr(C)]
pub struct StylesheetCache {
    /// Cache size.
    size: size_t,
}

/// @brief Creates an instance of StylesheetCache.
/// @return a StylesheetCache struct
#[no_mangle]
pub extern "C" fn css_inliner_stylesheet_cache(size: size_t) -> StylesheetCache {
    StylesheetCache { size }
}

/// Configuration options for CSS inlining process.
#[repr(C)]
pub struct CssInlinerOptions {
    /// Whether to inline CSS from "style" tags.
    pub inline_style_tags: bool,
    /// Keep "style" tags after inlining.
    pub keep_style_tags: bool,
    /// Keep "link" tags after inlining.
    pub keep_link_tags: bool,
    /// Whether remote stylesheets should be loaded or not.
    pub load_remote_stylesheets: bool,
    /// Cache for external stylesheets.
    pub cache: *const StylesheetCache,
    /// Used for loading external stylesheets via relative URLs.
    pub base_url: *const c_char,
    /// Additional CSS to inline.
    pub extra_css: *const c_char,
    /// Pre-allocate capacity for HTML nodes during parsing.
    /// It can improve performance when you have an estimate of the number of nodes in your HTML document.
    pub preallocate_node_capacity: size_t,
}

macro_rules! inliner {
    ($options:expr) => {
        CSSInliner::new(
            match InlineOptions::try_from(match $options.as_ref() {
                Some(ptr) => ptr,
                None => return CssResult::NullOptions,
            }) {
                Ok(inline_options) => inline_options,
                Err(e) => return CssResult::from(e),
            },
        )
    };
}

macro_rules! to_str {
    ($input:expr) => {
        match CStr::from_ptr($input).to_str() {
            Ok(val) => val,
            Err(_) => return CssResult::InvalidInputString,
        }
    };
}

/// @brief Inline CSS from @p input & write the result to @p output with @p options.
/// @param options configuration for the inliner.
/// @param input html to inline.
/// @param output buffer to save the inlined CSS.
/// @param output_size size of @p output in bytes.
/// @return a CSS_RESULT enum variant regarding if the operation was a success or an error occurred
#[allow(clippy::missing_safety_doc)]
#[must_use]
#[no_mangle]
pub unsafe extern "C" fn css_inline_to(
    options: *const CssInlinerOptions,
    input: *const c_char,
    output: *mut c_char,
    output_size: size_t,
) -> CssResult {
    let inliner = inliner!(options);
    let html = to_str!(input);
    let mut buffer = CBuffer::new(output, output_size);
    if let Err(e) = inliner.inline_to(html, &mut buffer) {
        return e.into();
    };
    // Null terminate the pointer
    let ptr: *mut c_char = buffer.buffer.add(buffer.pos);
    *ptr = 0;
    CssResult::Ok
}

/// @brief Inline CSS @p fragment into @p input & write the result to @p output with @p options.
/// @param options configuration for the inliner.
/// @param input html to inline.
/// @param css css to inline.
/// @param output buffer to save the inlined CSS.
/// @param output_size size of @p output in bytes.
/// @return a CSS_RESULT enum variant regarding if the operation was a success or an error occurred
#[allow(clippy::missing_safety_doc)]
#[must_use]
#[no_mangle]
pub unsafe extern "C" fn css_inline_fragment_to(
    options: *const CssInlinerOptions,
    input: *const c_char,
    css: *const c_char,
    output: *mut c_char,
    output_size: size_t,
) -> CssResult {
    let inliner = inliner!(options);
    let html = to_str!(input);
    let css = to_str!(css);
    let mut buffer = CBuffer::new(output, output_size);
    if let Err(e) = inliner.inline_fragment_to(html, css, &mut buffer) {
        return e.into();
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
        inline_style_tags: true,
        keep_style_tags: false,
        keep_link_tags: false,
        base_url: ptr::null(),
        load_remote_stylesheets: true,
        cache: std::ptr::null(),
        extra_css: ptr::null(),
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
            inline_style_tags: value.inline_style_tags,
            keep_style_tags: value.keep_style_tags,
            keep_link_tags: value.keep_link_tags,
            base_url: match base_url {
                Some(url) => Some(Url::parse(url).map_err(|_| InlineOptionsError::InvalidUrl)?),
                None => None,
            },
            load_remote_stylesheets: value.load_remote_stylesheets,
            cache: {
                if value.cache.is_null() {
                    None
                } else if let Some(size) = NonZeroUsize::new(unsafe { (*value.cache).size }) {
                    Some(Mutex::new(css_inline::StylesheetCache::new(size)))
                } else {
                    return Err(InlineOptionsError::InvalidCacheSize);
                }
            },
            extra_css: extra_css.map(Cow::Borrowed),
            preallocate_node_capacity: value.preallocate_node_capacity,
            resolver: Arc::new(DefaultStylesheetResolver),
        })
    }
}

impl From<InlineOptionsError> for CssResult {
    fn from(value: InlineOptionsError) -> Self {
        match value {
            InlineOptionsError::InvalidUrl => CssResult::InvalidUrl,
            InlineOptionsError::InvalidExtraCss => CssResult::InvalidExtraCss,
            InlineOptionsError::InvalidCacheSize => CssResult::InvalidCacheSize,
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
        // we are going to write all it's possible
        let end = self.size - 1; // size-1 because the buffer needs to be null terminated
        let num_bytes_to_write = cmp::min(end - self.pos, buf.len());
        if num_bytes_to_write != 0 {
            unsafe {
                let dst = self.buffer.add(self.pos);
                ptr::copy_nonoverlapping(buf.as_ptr() as *const c_char, dst, num_bytes_to_write);
            }
            self.pos += num_bytes_to_write;
        }
        Ok(num_bytes_to_write)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // flush() is not used in this context because we are not using a BufWriter or similar
        // types, so only write() gets a call
        unimplemented!();
    }
}
