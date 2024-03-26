#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("`css-inline` CLI is only available with the `cli` feature");
    std::process::exit(1);
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use css_inline::{CSSInliner, DefaultStylesheetResolver, InlineOptions};
    use rayon::prelude::*;
    use std::{
        borrow::Cow,
        ffi::OsString,
        fmt::{Display, Write as FmtWrite},
        fs::{read_to_string, File},
        io::{self, Read, Write},
        path::Path,
        sync::{
            atomic::{AtomicI32, Ordering},
            Arc,
        },
    };

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

    --base-url
        Used for loading external stylesheets via relative URLs.

    --load-remote-stylesheets
        Whether remote stylesheets should be loaded or not.

    --cache-size
        Set the cache size for remote stylesheets.

    --extra-css
        Additional CSS to inline.

    --output-filename-prefix
        Custom prefix for output files. Defaults to `inlined.`.
"#
    )
    .as_bytes();

    struct Args {
        inline_style_tags: bool,
        keep_style_tags: bool,
        keep_link_tags: bool,
        base_url: Option<String>,
        extra_css: Option<String>,
        output_filename_prefix: Option<OsString>,
        load_remote_stylesheets: bool,
        #[cfg(feature = "stylesheet-cache")]
        cache_size: Option<usize>,
        files: Vec<String>,
    }

    fn parse_url(url: Option<String>) -> Result<Option<url::Url>, url::ParseError> {
        Ok(if let Some(url) = url {
            Some(url::Url::parse(url.as_str())?)
        } else {
            None
        })
    }

    fn format_error(filename: Option<&str>, error: impl Display) {
        let mut buffer = String::with_capacity(128);
        if let Some(filename) = filename {
            writeln!(buffer, "Filename: {}", filename).expect("Failed to write to buffer");
        }
        buffer.push_str("Status: ERROR\n");
        writeln!(buffer, "Details: {}", error).expect("Failed to write to buffer");
        eprintln!("{}", buffer.trim());
    }

    let mut args = pico_args::Arguments::from_env();
    let exit_code = AtomicI32::new(0);
    if args.contains(["-h", "--help"]) {
        io::stdout().write_all(HELP_MESSAGE)?;
    } else if args.contains(["-v", "--version"]) {
        io::stdout().write_all(VERSION_MESSAGE)?;
    } else {
        let args = Args {
            inline_style_tags: args
                .opt_value_from_str("--inline-style-tags")?
                .unwrap_or(true),
            keep_style_tags: args.contains("--keep-style-tags"),
            keep_link_tags: args.contains("--keep-link-tags"),
            base_url: args.opt_value_from_str("--base-url")?,
            extra_css: args.opt_value_from_str("--extra-css")?,
            output_filename_prefix: args.opt_value_from_str("--output-filename-prefix")?,
            load_remote_stylesheets: args.contains("--load-remote-stylesheets"),
            #[cfg(feature = "stylesheet-cache")]
            cache_size: args.opt_value_from_str("--cache-size")?,
            files: args.free()?,
        };
        let base_url = match parse_url(args.base_url) {
            Ok(base_url) => base_url,
            Err(error) => {
                format_error(None, error);
                std::process::exit(1);
            }
        };
        let options = InlineOptions {
            inline_style_tags: args.inline_style_tags,
            keep_style_tags: args.keep_style_tags,
            keep_link_tags: args.keep_link_tags,
            base_url,
            load_remote_stylesheets: args.load_remote_stylesheets,
            #[cfg(feature = "stylesheet-cache")]
            cache: {
                if let Some(size) = args.cache_size {
                    if size == 0 {
                        eprintln!("ERROR: Cache size must be an integer greater than zero");
                        std::process::exit(1);
                    }

                    std::num::NonZeroUsize::new(size)
                        .map(css_inline::StylesheetCache::new)
                        .map(std::sync::Mutex::new)
                } else {
                    None
                }
            },
            extra_css: args.extra_css.as_deref().map(Cow::Borrowed),
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
                        Ok(_) => println!("{filename}: SUCCESS"),
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
