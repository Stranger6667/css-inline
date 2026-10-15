#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("`css-inline` CLI is only available with the `cli` feature");
    std::process::exit(1);
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use core::fmt;
    use css_inline::{CSSInliner, DefaultStylesheetResolver, InlineOptions};
    use std::{
        borrow::Cow,
        error::Error,
        ffi::OsString,
        fmt::Write as FmtWrite,
        fs::{self, read_to_string, File},
        io::{self, Read, Write},
        num::NonZeroUsize,
        path::PathBuf,
        str::FromStr,
        sync::{
            atomic::{AtomicI32, AtomicUsize, Ordering},
            Arc,
        },
        thread,
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
        files: Vec<PathBuf>,
        inline_style_tags: bool,
        keep_style_tags: bool,
        keep_link_tags: bool,
        keep_at_rules: bool,
        base_url: Option<String>,
        extra_css: Option<String>,
        extra_css_files: Vec<PathBuf>,
        output_filename_prefix: Option<OsString>,
        load_remote_stylesheets: bool,
        #[cfg(feature = "stylesheet-cache")]
        cache_size: Option<usize>,
        minify_css: bool,
        remove_inlined_selectors: bool,
        apply_width_attributes: bool,
        apply_height_attributes: bool,
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
                minify_css: false,
                remove_inlined_selectors: false,
                apply_width_attributes: false,
                apply_height_attributes: false,
            }
        }
    }

    impl From<lexopt::Error> for ParseError {
        fn from(error: lexopt::Error) -> Self {
            Self {
                message: format!("Error parsing arguments: {error}"),
            }
        }
    }

    fn parse_value<T>(value: OsString, flag: &str) -> Result<T, ParseError>
    where
        T: FromStr,
        T::Err: fmt::Display,
    {
        let value = lexopt::ValueExt::string(value)?;
        value.parse::<T>().map_err(|e| ParseError {
            message: format!("Failed to parse value '{value}' for flag '{flag}': {e}"),
        })
    }

    fn parse_args() -> Result<ParsedArgs, ParseError> {
        use lexopt::prelude::*;

        let mut parser = lexopt::Parser::from_env();
        let mut parsed = ParsedArgs::default();
        while let Some(arg) = parser.next()? {
            match arg {
                Short('h') | Long("help") => parsed.help = true,
                Short('v' | 'V') | Long("version") => parsed.version = true,
                Long("keep-style-tags") => parsed.keep_style_tags = true,
                Long("keep-link-tags") => parsed.keep_link_tags = true,
                Long("keep-at-rules") => parsed.keep_at_rules = true,
                Long("minify-css") => parsed.minify_css = true,
                Long("remove-inlined-selectors") => parsed.remove_inlined_selectors = true,
                Long("apply-width-attributes") => parsed.apply_width_attributes = true,
                Long("apply-height-attributes") => parsed.apply_height_attributes = true,
                Long("inline-style-tags") => {
                    parsed.inline_style_tags = parse_value(parser.value()?, "inline-style-tags")?;
                }
                Long("load-remote-stylesheets") => {
                    parsed.load_remote_stylesheets = match parser.optional_value() {
                        Some(value) => parse_value(value, "load-remote-stylesheets")?,
                        None => true,
                    };
                }
                Long("base-url") => parsed.base_url = Some(parser.value()?.string()?),
                Long("extra-css") => parsed.extra_css = Some(parser.value()?.string()?),
                Long("extra-css-file") => parsed.extra_css_files.push(parser.value()?.into()),
                Long("output-filename-prefix") => {
                    parsed.output_filename_prefix = Some(parser.value()?);
                }
                #[cfg(feature = "stylesheet-cache")]
                Long("cache-size") => {
                    parsed.cache_size = Some(parse_value(parser.value()?, "cache-size")?);
                }
                Value(path) => parsed.files.push(path.into()),
                _ => return Err(arg.unexpected().into()),
            }
        }
        Ok(parsed)
    }

    fn combine_extra_css(
        extra_css: Option<String>,
        extra_css_files: Vec<PathBuf>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let mut buffer = extra_css.unwrap_or_default();

        if !buffer.is_empty() {
            buffer.push('\n');
        }

        for path in extra_css_files {
            let mut file = File::open(&path)
                .map_err(|e| format!("Failed to read CSS file '{}': {e}", path.display()))?;
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

    --minify-css
        Minify CSS by removing trailing semicolons and spaces between properties and values.

    --remove-inlined-selectors
        Remove selectors that were successfully inlined from inline <style> blocks.

    --apply-width-attributes
        Apply width HTML attributes from CSS width properties on supported elements
        (table, td, th, img). Useful for email compatibility with clients like Outlook.

    --apply-height-attributes
        Apply height HTML attributes from CSS height properties on supported elements
        (table, td, th, img). Useful for email compatibility with clients like Outlook.

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

    let args = parse_args().unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });

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
            minify_css: args.minify_css,
            base_url,
            load_remote_stylesheets: args.load_remote_stylesheets,
            #[cfg(feature = "stylesheet-cache")]
            cache,
            extra_css: extra_css.as_deref().map(Cow::Borrowed),
            preallocate_node_capacity: 32,
            resolver: Arc::new(DefaultStylesheetResolver),
            remove_inlined_selectors: args.remove_inlined_selectors,
            apply_width_attributes: args.apply_width_attributes,
            apply_height_attributes: args.apply_height_attributes,
        };
        let inliner = CSSInliner::new(options);
        if args.files.is_empty() {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            if let Err(error) = inliner.inline_to(&buffer, &mut io::stdout()) {
                format_error(None, error);
                exit_code.store(1, Ordering::SeqCst);
            }
        } else {
            let process = |file_path: &PathBuf| {
                let inline = || -> Result<(), Box<dyn Error>> {
                    let contents = read_to_string(file_path)?;
                    // Inline into memory first, so a failure leaves no output file behind
                    let inlined = inliner.inline(&contents)?;
                    let mut new_filename = args
                        .output_filename_prefix
                        .clone()
                        .unwrap_or_else(|| OsString::from("inlined."));
                    new_filename.push(
                        file_path
                            .file_name()
                            .expect("It is already read, therefore it is a file"),
                    );
                    fs::write(file_path.with_file_name(new_filename), inlined)?;
                    Ok(())
                };
                match inline() {
                    Ok(()) => println!("{}: SUCCESS", file_path.display()),
                    Err(error) => {
                        format_error(Some(&file_path.to_string_lossy()), error);
                        exit_code.store(1, Ordering::SeqCst);
                    }
                }
            };
            let next = AtomicUsize::new(0);
            let workers = thread::available_parallelism()
                .map_or(1, NonZeroUsize::get)
                .min(args.files.len());
            thread::scope(|scope| {
                for _ in 0..workers {
                    scope.spawn(|| {
                        while let Some(file_path) =
                            args.files.get(next.fetch_add(1, Ordering::Relaxed))
                        {
                            process(file_path);
                        }
                    });
                }
            });
        }
    }
    std::process::exit(exit_code.into_inner());
}
