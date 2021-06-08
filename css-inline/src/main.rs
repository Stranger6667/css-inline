use css_inline::{CSSInliner, InlineOptions};
use rayon::prelude::*;
use std::{
    borrow::Cow,
    error::Error,
    ffi::OsString,
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

const VERSION_MESSAGE: &[u8] = concat!("css-inline ", env!("CARGO_PKG_VERSION"), "\n").as_bytes();
const HELP_MESSAGE: &[u8] = concat!(
    "css-inline ",
    env!("CARGO_PKG_VERSION"),
    r#"
Dmitry Dygalo <dadygalo@gmail.com>

css-inline inlines CSS into HTML documents.

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

    --remove-style-tags
        Remove "style" tags after inlining.

    --base-url
        Used for loading external stylesheets via relative URLs.

    --load-remote-stylesheets
        Whether remote stylesheets should be loaded or not.

    --extra-css
        Additional CSS to inline.
"#
)
.as_bytes();

struct Args {
    inline_style_tags: bool,
    remove_style_tags: bool,
    base_url: Option<String>,
    extra_css: Option<String>,
    load_remote_stylesheets: bool,
    files: Vec<String>,
}

fn parse_url(url: Option<String>) -> Result<Option<url::Url>, url::ParseError> {
    Ok(if let Some(url) = url {
        Some(url::Url::parse(url.as_str())?)
    } else {
        None
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = pico_args::Arguments::from_env();
    if args.contains(["-h", "--help"]) {
        io::stdout().write_all(HELP_MESSAGE)?;
    } else if args.contains(["-v", "--version"]) {
        io::stdout().write_all(VERSION_MESSAGE)?;
    } else {
        let args = Args {
            inline_style_tags: args
                .opt_value_from_str("--inline-style-tags")?
                .unwrap_or(true),
            remove_style_tags: args.contains("--remove-style-tags"),
            base_url: args.opt_value_from_str("--base-url")?,
            extra_css: args.opt_value_from_str("--extra-css")?,
            load_remote_stylesheets: args.contains("--load-remote-stylesheets"),
            files: args.free()?,
        };
        let options = InlineOptions {
            inline_style_tags: args.inline_style_tags,
            remove_style_tags: args.remove_style_tags,
            base_url: parse_url(args.base_url)?,
            load_remote_stylesheets: args.load_remote_stylesheets,
            extra_css: args.extra_css.as_deref().map(Cow::Borrowed),
        };
        let inliner = CSSInliner::new(options);
        if args.files.is_empty() {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            inliner.inline_to(buffer.as_str().trim(), &mut io::stdout())?;
        } else {
            args.files
                .par_iter()
                .map(|file_path| {
                    File::open(file_path)
                        .and_then(read_file)
                        .and_then(|contents| {
                            let path = Path::new(file_path);
                            let mut new_filename = OsString::from("inlined.");
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
                        Ok(_) => println!("{}: SUCCESS", filename),
                        Err(error) => println!("{}: FAILURE ({})", filename, error),
                    },
                    Err((filename, error)) => println!("{}: FAILURE ({})", filename, error),
                });
        }
    }
    Ok(())
}

fn read_file(mut file: File) -> io::Result<String> {
    let mut contents = String::with_capacity(1024);
    file.read_to_string(&mut contents).and(Ok(contents))
}
