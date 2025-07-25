#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("`css-inline` CLI is only available with the `cli` feature");
    std::process::exit(1);
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use core::fmt;
    use css_inline::{CSSInliner, DefaultStylesheetResolver, InlineOptions};
    use rayon::prelude::*;
    use std::{
        borrow::Cow,
        env,
        error::Error,
        ffi::OsString,
        fmt::Write as FmtWrite,
        fs::{read_to_string, File},
        io::{self, Read, Write},
        path::Path,
        str::FromStr,
        sync::{
            atomic::{AtomicI32, Ordering},
            Arc,
        },
    };

    fn parse_url(url: Option<&str>) -> Result<Option<url::Url>, url::ParseError> {
        Ok(if let Some(url) = url {
            Some(url::Url::parse(url)?)
        } else {
            None
        })
    }

    #[derive(Debug)]
    struct ParseError {
        message: String,
    }

    impl fmt::Display for ParseError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl Error for ParseError {}

    struct ParsedArgs {
        help: bool,
        version: bool,
        files: Vec<String>,
        inline_style_tags: bool,
        keep_style_tags: bool,
        keep_link_tags: bool,
        keep_at_rules: bool,
        base_url: Option<String>,
        extra_css: Option<String>,
        extra_css_files: Vec<String>,
        output_filename_prefix: Option<OsString>,
        load_remote_stylesheets: bool,
        #[cfg(feature = "stylesheet-cache")]
        cache_size: Option<usize>,
    }

    impl Default for ParsedArgs {
        fn default() -> Self {
            Self {
                help: false,
                version: false,
                files: Vec::new(),
                inline_style_tags: true,
                keep_style_tags: false,
                keep_link_tags: false,
                keep_at_rules: false,
                base_url: None,
                extra_css: None,
                extra_css_files: Vec::new(),
                output_filename_prefix: None,
                load_remote_stylesheets: false,
                #[cfg(feature = "stylesheet-cache")]
                cache_size: None,
            }
        }
    }

    #[cfg(feature = "stylesheet-cache")]
    macro_rules! if_cfg_feature_stylesheet_cache {
        ($val:expr) => {
            $val
        };
    }

    #[cfg(not(feature = "stylesheet-cache"))]
    macro_rules! if_cfg_feature_stylesheet_cache {
        // Empty string that won't match
        ($val:expr) => {
            ""
        };
    }

    fn requires_value(flag: &str) -> bool {
        matches!(
            flag,
            "inline-style-tags"
                | "base-url"
                | "extra-css"
                | "extra-css-file"
                | "output-filename-prefix"
                | if_cfg_feature_stylesheet_cache!("cache-size")
        )
    }

    fn parse_value<T>(value: &str, flag: &str) -> Result<T, ParseError>
    where
        T: FromStr,
        T::Err: fmt::Display,
    {
        value.parse::<T>().map_err(|e| ParseError {
            message: format!("Failed to parse value '{value}' for flag '{flag}': {e}"),
        })
    }

    fn handle_flag_with_value(
        parsed: &mut ParsedArgs,
        flag: &str,
        value: &str,
    ) -> Result<(), ParseError> {
        match flag {
            "inline-style-tags" => parsed.inline_style_tags = parse_value(value, flag)?,
            "load-remote-stylesheets" => parsed.load_remote_stylesheets = parse_value(value, flag)?,
            "base-url" => parsed.base_url = Some(value.to_string()),
            "extra-css" => parsed.extra_css = Some(value.to_string()),
            "extra-css-file" => parsed.extra_css_files.push(value.to_string()),
            "output-filename-prefix" => {
                parsed.output_filename_prefix = Some(value.to_string().into());
            }
            #[cfg(feature = "stylesheet-cache")]
            "cache-size" => parsed.cache_size = Some(parse_value(value, flag)?),
            _ => {
                return Err(ParseError {
                    message: format!("Unknown flag: --{flag}"),
                })
            }
        }
        Ok(())
    }

    fn handle_boolean_flag(parsed: &mut ParsedArgs, flag: &str) -> Result<(), ParseError> {
        match flag {
            "help" | "h" => parsed.help = true,
            "version" | "v" => parsed.version = true,
            "keep-style-tags" => parsed.keep_style_tags = true,
            "keep-link-tags" => parsed.keep_link_tags = true,
            _ => {
                return Err(ParseError {
                    message: format!("Unknown flag: {flag}"),
                })
            }
        }
        Ok(())
    }

    fn combine_extra_css(
        extra_css: Option<String>,
        extra_css_files: Vec<String>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let mut buffer = extra_css.unwrap_or_default();

        if !buffer.is_empty() {
            buffer.push('\n');
        }

        for path in extra_css_files {
            let mut file =
                File::open(&path).map_err(|e| format!("Failed to read CSS file '{path}': {e}"))?;
            file.read_to_string(&mut buffer)?;
            if !buffer.is_empty() {
                buffer.push('\n');
            }
        }

        Ok(if buffer.is_empty() {
            None
        } else {
            Some(buffer)
        })
    }

    fn format_error(filename: Option<&str>, error: impl fmt::Display) {
        let mut buffer = String::with_capacity(128);
        if let Some(filename) = filename {
            writeln!(buffer, "Filename: {filename}").expect("Failed to write to buffer");
        }
        buffer.push_str("Status: ERROR\n");
        writeln!(buffer, "Details: {error}").expect("Failed to write to buffer");
        eprintln!("{}", buffer.trim());
    }

    const VERSION_MESSAGE: &[u8] =
        concat!("css-inline ", env!("CARGO_PKG_VERSION"), "\n").as_bytes();
    const HELP_MESSAGE: &[u8] = concat!(
        "css-inline ",
        env!("CARGO_PKG_VERSION"),
        r#"
Dmitry Dygalo <dmitry@dygalo.dev>

css-inline inlines CSS into HTML 'style' attributes.

USAGE:
   css-inline [OPTIONS] [PATH ...]
   command | css-inline [OPTIONS]

ARGS:
    <PATH>...
        An HTML document to process. In each specified document "css-inline" will look for
        all relevant "style" and "link" tags, will load CSS from them and then inline it
        to the HTML tags, according to the corresponding CSS selectors.
        When multiple documents are specified, they will be processed in parallel, and each inlined
        file will be saved with "inlined." prefix. E.g., for "example.html", there will be
        "inlined.example.html".

OPTIONS:

    --inline-style-tags
        Whether to inline CSS from "style" tags. The default value is `true`. To disable inlining
        from "style" tags use `--inline-style-tags=false`.

    --keep-style-tags
        Keep "style" tags after inlining.

    --keep-link-tags
        Keep "link" tags after inlining.
        
    --keep-at-rules
        Keep "at-rules" after inlining.

    --base-url
        Used for loading external stylesheets via relative URLs.

    --load-remote-stylesheets
        Whether remote stylesheets should be loaded or not.

    --cache-size
        Set the cache size for remote stylesheets.

    --extra-css
        Additional CSS to inline.

    --extra-css-file <PATH>
        Load additional CSS from a file to inline. Can be used multiple times to load
        from several files. The CSS will be processed alongside any existing styles.

    --output-filename-prefix
        Custom prefix for output files. Defaults to `inlined.`.
"#
    )
    .as_bytes();

    let mut raw_args = env::args().skip(1);
    let mut args = ParsedArgs::default();

    while let Some(arg) = raw_args.next() {
        if let Some(flag) = arg.strip_prefix("--") {
            // Handle --key=value format
            if let Some((flag, value)) = flag.split_once('=') {
                if let Err(error) = handle_flag_with_value(&mut args, flag, value) {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            } else {
                // Handle --key format (boolean or expecting value)
                if requires_value(flag) {
                    // Expects a value
                    if let Some(value) = raw_args.next() {
                        if let Err(error) = handle_flag_with_value(&mut args, flag, &value) {
                            eprintln!("{error}");
                            std::process::exit(1);
                        }
                    } else {
                        eprintln!("Error parsing arguments: Flag --{flag} requires a value");
                        std::process::exit(1);
                    }
                } else {
                    // Boolean flag
                    if let Err(error) = handle_boolean_flag(&mut args, flag) {
                        eprintln!("{error}");
                        std::process::exit(1);
                    }
                }
            }
        } else if let Some(flag) = arg.strip_prefix('-') {
            if flag.len() == 1 {
                // Single character short flag
                if let Err(error) = handle_boolean_flag(&mut args, flag) {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error parsing arguments: Invalid flag: -{flag}");
                std::process::exit(1);
            }
        } else {
            // Positional argument (file)
            args.files.push(arg);
        }
    }

    let exit_code = AtomicI32::new(0);
    if args.help {
        io::stdout().write_all(HELP_MESSAGE)?;
    } else if args.version {
        io::stdout().write_all(VERSION_MESSAGE)?;
    } else {
        let base_url = match parse_url(args.base_url.as_deref()) {
            Ok(base_url) => base_url,
            Err(error) => {
                format_error(None, error);
                std::process::exit(1);
            }
        };
        #[cfg(feature = "stylesheet-cache")]
        let cache = if let Some(size) = args.cache_size {
            if size == 0 {
                eprintln!("ERROR: Cache size must be an integer greater than zero");
                std::process::exit(1);
            }
            std::num::NonZeroUsize::new(size)
                .map(css_inline::StylesheetCache::new)
                .map(std::sync::Mutex::new)
        } else {
            None
        };
        let extra_css = match combine_extra_css(args.extra_css, args.extra_css_files) {
            Ok(css) => css,
            Err(error) => {
                format_error(None, error);
                std::process::exit(1);
            }
        };
        let options = InlineOptions {
            inline_style_tags: args.inline_style_tags,
            keep_style_tags: args.keep_style_tags,
            keep_link_tags: args.keep_link_tags,
            keep_at_rules: args.keep_at_rules,
            base_url,
            load_remote_stylesheets: args.load_remote_stylesheets,
            #[cfg(feature = "stylesheet-cache")]
            cache,
            extra_css: extra_css.as_deref().map(Cow::Borrowed),
            preallocate_node_capacity: 32,
            resolver: Arc::new(DefaultStylesheetResolver),
        };
        let inliner = CSSInliner::new(options);
        if args.files.is_empty() {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            if let Err(error) = inliner.inline_to(buffer.as_str().trim(), &mut io::stdout()) {
                format_error(None, error);
                exit_code.store(1, Ordering::SeqCst);
            }
        } else {
            args.files
                .par_iter()
                .map(|file_path| {
                    read_to_string(file_path)
                        .and_then(|contents| {
                            let path = Path::new(file_path);
                            let mut new_filename = args
                                .output_filename_prefix
                                .clone()
                                .unwrap_or_else(|| OsString::from("inlined."));
                            new_filename.push(
                                path.to_path_buf()
                                    .file_name()
                                    .expect("It is already read, therefore it is a file"),
                            );
                            let new_path = path.with_file_name(new_filename);
                            File::create(new_path).map(|file| (file, contents))
                        })
                        .map(|(mut file, contents)| {
                            (file_path, inliner.inline_to(contents.as_str(), &mut file))
                        })
                        .map_err(|error| (file_path, error))
                })
                .for_each(|result| match result {
                    Ok((filename, result)) => match result {
                        Ok(()) => println!("{filename}: SUCCESS"),
                        Err(error) => {
                            format_error(Some(filename.as_str()), error);
                            exit_code.store(1, Ordering::SeqCst);
                        }
                    },
                    Err((filename, error)) => {
                        format_error(Some(filename.as_str()), error);
                        exit_code.store(1, Ordering::SeqCst);
                    }
                });
        }
    }
    std::process::exit(exit_code.into_inner());
}
