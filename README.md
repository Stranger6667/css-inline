# css-inline

[![ci](https://github.com/Stranger6667/css-inline/workflows/ci/badge.svg)](https://github.com/Stranger6667/css-inline/actions)
[![Crates.io](https://img.shields.io/crates/v/css-inline.svg)](https://crates.io/crates/css-inline)
[![docs.rs](https://docs.rs/css-inline/badge.svg?version=0.5.0)](https://docs.rs/css-inline/0.5.0/css_inline/)

A crate for inlining CSS into HTML documents. When you send HTML emails you need to use "style" attributes instead of "style" tags.

For example, this HTML:

```html
<html>
    <head>
        <title>Test</title>
        <style>h1 { color:blue; }</style>
    </head>
    <body>
        <h1>Big Text</h1>
    </body>
</html>
```

Will be turned into this:

```html
<html>
    <head><title>Test</title></head>
    <body>
        <h1 style="color:blue;">Big Text</h1>
    </body>
</html>
```

To use it in your project add the following line to your `dependencies` section in project's `Cargo.toml` file:

```toml
css-inline = "0.5"
```

## Usage

```rust
use css_inline;

const HTML: &str = r#"<html>
<head>
    <title>Test</title>
    <style>h1 { color:blue; }</style>
</head>
<body>
    <h1>Big Text</h1>
</body>
</html>"#;

fn main() -> Result<(), css_inline::InlineError> {
    let inlined = css_inline::inline(HTML)?;
    // Do something with inlined HTML, e.g. send an email
    Ok(())
}
```

### Features & Configuration

`css-inline` can be configured by using `CSSInliner::options()` that implements the Builder pattern:

```rust
use css_inline;

fn main() -> Result<(), css_inline::InlineError> {
    let inliner = css_inline::CSSInliner::options()
        .load_remote_stylesheets(false)
        .build();
    let inlined = inliner.inline(HTML);
    // Do something with inlined HTML, e.g. send an email
    Ok(())
}
```

- `inline_style_tags`. Whether to inline CSS from "style" tags. Default: `true`
- `remove_style_tags`. Remove "style" tags after inlining. Default: `false`
- `base_url`. Base URL to resolve relative URLs. Default: `None`
- `load_remote_stylesheets`. Whether remote stylesheets should be loaded or not. Default: `true`
- `extra_css`. Additional CSS to inline. Default: `None`

## Command Line Interface

`css-inline` provides a command-line interface:

```bash
$ css-inline --help

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
    --inline-style-tags
        Whether to inline CSS from "style" tags. The default value is `true`. To disable inlining
        from "style" tags use `-inline-style-tags=false`.

    --remove-style-tags
        Remove "style" tags after inlining.

    --base-url
        Used for loading external stylesheets via relative URLs.

    --load-remote-stylesheets
        Whether remote stylesheets should be loaded or not.

    --extra-css
        Additional CSS to inline.
```
