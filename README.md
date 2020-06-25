# css-inline

[![ci](https://github.com/Stranger6667/css-inline/workflows/ci/badge.svg)](https://github.com/Stranger6667/css-inline/actions)
[![Crates.io](https://img.shields.io/crates/v/css-inline.svg)](https://crates.io/crates/css-inline)
[![docs.rs](https://docs.rs/css-inline/badge.svg?version=0.2.0)](https://docs.rs/css-inline/0.2.0/css_inline/)

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
css-inline = "0.1"
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

### Features

`css-inline` does minimum work by default:

- No CSS transformation;
- No "style" or "link" tags removal;

It also loads external stylesheets via network or filesystem, but this behavior is configurable.

### Configuration

`css-inline` can be configured by using `InlineOptions` and `CSSInliner`:

```rust
use css_inline;

fn main() -> Result<(), css_inline::InlineError> {
    let options = css_inline::InlineOptions {
        load_remote_stylesheets: false,
        ..Default::default()
    };
    let inliner = css_inline::CSSInliner(options);
    let inlined = inliner.inline(HTML);
    // Do something with inlined HTML, e.g. send an email
    Ok(())
}
```

- `remove_style_tags`. Remove "style" tags after inlining.
- `base_url`. Base URL to resolve relative URLs
- `load_remote_stylesheets`. Whether remote stylesheets should be loaded or not
