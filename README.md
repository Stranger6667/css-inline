# css-inline

[![ci](https://github.com/Stranger6667/css-inline/workflows/ci/badge.svg)](https://github.com/Stranger6667/css-inline/actions)
[![codecov](https://codecov.io/gh/Stranger6667/css-inline/branch/master/graph/badge.svg)](https://codecov.io/gh/Stranger6667/css-inline)
[![Crates.io](https://img.shields.io/crates/v/css-inline.svg)](https://crates.io/crates/css-inline)
[![docs.rs](https://docs.rs/css-inline/badge.svg)](https://docs.rs/css-inline/)
[![gitter](https://img.shields.io/gitter/room/Stranger6667/css-inline.svg)](https://gitter.im/Stranger6667/css-inline)

A crate for inlining CSS into HTML documents. It is built with Mozilla's Servo project components. 

When you send HTML emails, you need to use "style" attributes instead of "style" tags. For example, this HTML:

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
    <head>
        <title>Test</title>
    </head>
    <body>
        <h1 style="color:blue;">Big Text</h1>
    </body>
</html>
```

To use it in your project add the following line to your `dependencies` section in the project's `Cargo.toml` file:

```toml
css-inline = "0.8"
```

Minimum Supported Rust Version is 1.60.

## Usage

```rust
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
const HTML: &str = "...";

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
- `base_url`. Base URL to resolve relative URLs. If you'd like to load stylesheets from your filesystem, use the `file://` scheme. Default: `None`
- `load_remote_stylesheets`. Whether remote stylesheets should be loaded or not. Default: `true`
- `extra_css`. Additional CSS to inline. Default: `None`

If you'd like to skip CSS inlining for an HTML tag, add `data-css-inline="ignore"` attribute to it:

```html
<head>
    <title>Test</title>
    <style>h1 { color:blue; }</style>
</head>
<body>
    <!-- The tag below won't receive additional styles -->
    <h1 data-css-inline="ignore">Big Text</h1>
</body>
</html>
```

This attribute also allows you to skip `link` and `style` tags:

```html
<head>
    <title>Test</title>
    <!-- Styles below are ignored -->
    <style data-css-inline="ignore">h1 { color:blue; }</style>
</head>
<body>
    <h1>Big Text</h1>
</body>
</html>
```

## Standards support & restrictions

`css-inline` is built on top of [kuchiki](https://crates.io/crates/kuchiki) and [cssparser](https://crates.io/crates/cssparser) and relies on their behavior for HTML / CSS parsing and serialization.
Notably:

- Only HTML 5, XHTML is not supported;
- Only CSS 3;
- Only UTF-8 for string representation. Other document encodings are not yet supported.

## Bindings

There are bindings for Python and WebAssembly in the `bindings` directory.

## Command Line Interface

`css-inline` provides a command-line interface:

```text
css-inline --help

css-inline inlines CSS into HTML documents.

USAGE:
   css-inline [OPTIONS] [PATH ...]
   command | css-inline [OPTIONS]

ARGS:
    <PATH>...
        An HTML document to process. In each specified document
        "css-inline" will look for all relevant "style" and "link"
        tags, will load CSS from them and then inline it to the
        HTML tags, according to the corresponding CSS selectors.
        When multiple documents are specified, they will be
        processed in parallel, and each inlined file will be saved
        with "inlined." prefix. E.g., for "example.html", there
        will be "inlined.example.html".

OPTIONS:
    --inline-style-tags
        Whether to inline CSS from "style" tags. The default
        value is `true`. To disable inlining from "style" tags
        use `--inline-style-tags=false`.

    --remove-style-tags
        Remove "style" tags after inlining.

    --base-url
        Used for loading external stylesheets via relative URLs.

    --load-remote-stylesheets
        Whether remote stylesheets should be loaded or not.

    --extra-css
        Additional CSS to inline.

    --output-filename-prefix
        Custom prefix for output files. Defaults to `inlined.`.
```

## Extra materials

If you want to know how this library was created & how it works internally, you could take a look at these articles:

- [Rust crate](https://dygalo.dev/blog/rust-for-a-pythonista-2/)
- [Python bindings](https://dygalo.dev/blog/rust-for-a-pythonista-3/)

## Support

If you have anything to discuss regarding this library, please, join our [gitter](https://gitter.im/Stranger6667/css-inline)!
