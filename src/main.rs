use css_inline::{CSSInliner, InlineOptions};
use rayon::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const HELP_MESSAGE: &str = concat!(
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
        all relevant "style" and "link" tags, will load CSS from them and then will inline it
        to the HTML tags, according to the relevant CSS selectors.
        When multiple documents are specified, they will be processed in parallel and each inlined
        file will be saved with "inlined." prefix. E.g. for "example.html", there will be
        "inlined.example.html".

OPTIONS:
    --remove-style-tags
        Remove "style" tags after inlining.

    --base-url
        Used for loading external stylesheets via relative URLs.

    --load-remote-stylesheets
        Whether remote stylesheets should be loaded or not."#
);

struct Args {
    help: bool,
    version: bool,
    remove_style_tags: bool,
    base_url: Option<String>,
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
    let args = Args {
        help: args.contains(["-h", "--help"]),
        version: args.contains(["-v", "--version"]),
        remove_style_tags: args.contains("--remove-style-tags"),
        base_url: args.opt_value_from_str("--base-url")?,
        load_remote_stylesheets: args.contains("--load-remote-stylesheets"),
        files: args.free()?,
    };

    if args.help {
        println!("{}", HELP_MESSAGE)
    } else if args.version {
        println!("css-inline {}", VERSION)
    } else {
        let options = InlineOptions {
            remove_style_tags: args.remove_style_tags,
            base_url: parse_url(args.base_url)?,
            load_remote_stylesheets: args.load_remote_stylesheets,
        };
        let inliner = CSSInliner::new(options);
        if args.files.is_empty() {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            inliner.inline_to(buffer.as_str().trim(), &mut io::stdout())?;
        } else {
            let results: Vec<_> = args
                .files
                .par_iter()
                .map(|filename| {
                    File::open(filename)
                        .and_then(read_file)
                        .and_then(|contents| {
                            let mut new_filename =
                                String::with_capacity(filename.len().saturating_add(8));
                            new_filename.push_str("inlined.");
                            new_filename.push_str(filename);
                            File::create(new_filename).and_then(|file| Ok((file, contents)))
                        })
                        .and_then(|(mut file, contents)| {
                            Ok((filename, inliner.inline_to(contents.as_str(), &mut file)))
                        })
                        .map_err(|error| (filename, error))
                })
                .collect();
            for result in results {
                match result {
                    Ok((filename, result)) => match result {
                        Ok(_) => println!("{}: SUCCESS", filename),
                        Err(error) => println!("{}: FAILURE ({})", filename, error),
                    },
                    Err((filename, error)) => println!("{}: FAILURE ({})", filename, error),
                }
            }
        }
    }
    Ok(())
}

fn read_file(mut file: File) -> io::Result<String> {
    let mut contents = String::new();
    file.read_to_string(&mut contents).and(Ok(contents))
}
