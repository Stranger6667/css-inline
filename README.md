# css-inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
[<img alt="crates.io" src="https://img.shields.io/crates/v/css-inline.svg?style=flat-square&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/css-inline)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-css_inline-66c2a5?style=flat-square&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/css-inline)
[<img alt="codecov.io" src="https://img.shields.io/codecov/c/gh/Stranger6667/css-inline?logo=codecov&style=flat-square&token=tOzvV4kDY0" height="20">](https://app.codecov.io/github/Stranger6667/css-inline)
[<img alt="gitter" src="https://img.shields.io/gitter/room/Stranger6667/css-inline?style=flat-square" height="20">](https://gitter.im/Stranger6667/css-inline)

`css_inline` is a high-performance library for inlining CSS into HTML 'style' attributes.

This library is designed for scenarios such as preparing HTML emails or embedding HTML into third-party web pages.

For instance, the crate transforms HTML like this:

```html
<html>
  <head>
    <style>h1 { color:blue; }</style>
  </head>
  <body>
    <h1>Big Text</h1>
  </body>
</html>
```

into:

```html
<html>
  <head></head>
  <body>
    <h1 style="color:blue;">Big Text</h1>
  </body>
</html>
```

- Extremely fast with a minimal memory footprint
- Uses reliable components from Mozilla's Servo project
- Inlines CSS from `style` and `link` tags
- Removes `style` and `link` tags
- Resolves external stylesheets (including local files)
- Optionally caches external stylesheets
- Works on Linux, Windows, and macOS
- Supports HTML5 & CSS3
- Bindings for [Python](https://github.com/Stranger6667/css-inline/tree/master/bindings/python), [Ruby](https://github.com/Stranger6667/css-inline/tree/master/bindings/ruby), [JavaScript](https://github.com/Stranger6667/css-inline/tree/master/bindings/javascript), [Java](https://github.com/Stranger6667/css-inline/tree/master/bindings/java), [C](https://github.com/Stranger6667/css-inline/tree/master/bindings/c), [PHP](https://github.com/Stranger6667/css-inline/tree/master/bindings/php), and a [WebAssembly](https://github.com/Stranger6667/css-inline/tree/master/bindings/javascript/wasm) module to run in browsers.
- Command Line Interface

## Playground

If you'd like to try `css-inline`, you can check the WebAssembly-powered [playground](https://css-inline.org/) to see the results instantly.

## Installation

To include it in your project, add the following line to the dependencies section in your project's `Cargo.toml` file:

```toml
[dependencies]
css-inline = "0.19"
```

The Minimum Supported Rust Version is 1.80.

## Usage

```rust
const HTML: &str = r#"<html>
<head>
    <style>h1 { color:blue; }</style>
</head>
<body>
    <h1>Big Text</h1>
</body>
</html>"#;

fn main() -> css_inline::Result<()> {
    let inlined = css_inline::inline(HTML)?;
    // Do something with inlined HTML, e.g. send an email
    Ok(())
}
```

Note that `css-inline` automatically adds missing `html` and `body` tags, so the output is a valid HTML document.

Alternatively, you can inline CSS into an HTML fragment, preserving the original structure:

```rust
const FRAGMENT: &str = r#"<main>
<h1>Hello</h1>
<section>
<p>who am i</p>
</section>
</main>"#;

const CSS: &str = r#"
p {
    color: red;
}

h1 {
    color: blue;
}
"#;

fn main() -> css_inline::Result<()> {
    let inlined = css_inline::inline_fragment(FRAGMENT, CSS)?;
    Ok(())
}
```

### Configuration

`css-inline` can be configured by using `CSSInliner::options()` that implements the Builder pattern:

```rust
const HTML: &str = "...";

fn main() -> css_inline::Result<()> {
    let inliner = css_inline::CSSInliner::options()
        .load_remote_stylesheets(false)
        .build();
    let inlined = inliner.inline(HTML)?;
    // Do something with inlined HTML, e.g. send an email
    Ok(())
}
```

- `inline_style_tags`. Specifies whether to inline CSS from "style" tags. Default: `true`
- `keep_style_tags`. Specifies whether to keep "style" tags after inlining. Default: `false`
- `keep_link_tags`. Specifies whether to keep "link" tags after inlining. Default: `false`
- `keep_at_rules`. Specifies whether to keep "at-rules" (starting with `@`) after inlining. Default: `false`
- `minify_css`. Specifies whether to remove trailing semicolons and spaces between properties and values. Default: `false`
- `base_url`. The base URL used to resolve relative URLs. If you'd like to load stylesheets from your filesystem, use the `file://` scheme. Default: `None`
- `load_remote_stylesheets`. Specifies whether remote stylesheets should be loaded. Default: `true`
- `cache`. Specifies cache for external stylesheets. Default: `None`
- `extra_css`. Extra CSS to be inlined. Default: `None`
- `preallocate_node_capacity`. **Advanced**. Preallocates capacity for HTML nodes during parsing. This can improve performance when you have an estimate of the number of nodes in your HTML document. Default: `32`
- `remove_inlined_selectors`. Specifies whether to remove selectors that were successfully inlined from `<style>` blocks. Default: `false`

You can also skip CSS inlining for an HTML tag by adding the `data-css-inline="ignore"` attribute to it:

```html
<head>
  <style>h1 { color:blue; }</style>
</head>
<body>
  <!-- The tag below won't receive additional styles -->
  <h1 data-css-inline="ignore">Big Text</h1>
</body>
```

The `data-css-inline="ignore"` attribute also allows you to skip `link` and `style` tags:

```html
<head>
  <!-- Styles below are ignored -->
  <style data-css-inline="ignore">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

Alternatively, you may keep `style` from being removed by using the `data-css-inline="keep"` attribute.
This is useful if you want to keep `@media` queries for responsive emails in separate `style` tags.
Such tags will be kept in the resulting HTML even if the `keep_style_tags` option is set to `false`.

```html
<head>
  <!-- Styles below are not removed -->
  <style data-css-inline="keep">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

Another possibility is to set `keep_at_rules` option to `true`. At-rules cannot be inlined into HTML therefore they
get removed by default. This is useful if you want to keep at-rules, e.g. `@media` queries for responsive emails in
separate `style` tags but inline any styles which can be inlined.
Such tags will be kept in the resulting HTML even if the `keep_style_tags` option is explicitly set to `false`.

```html
<head>
  <!-- With keep_at_rules=true "color:blue" will get inlined into <h1> but @media will be kept in <style> -->
  <style>h1 { color: blue; } @media (max-width: 600px) { h1 { font-size: 18px; } }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

If you set the the `minify_css` option to `true`, the inlined styles will be minified by removing trailing semicolons
and spaces between properties and values.

```html
<head>
  <!-- With minify_css=true, the <h1> will have `style="color:blue;font-weight:bold"` -->
  <style>h1 { color: blue; font-weight: bold; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

If you'd like to load stylesheets from your filesystem, use the `file://` scheme:

```rust
const HTML: &str = "...";

fn main() -> css_inline::Result<()> {
    let base_url = css_inline::Url::parse("file://styles/email/").expect("Invalid URL");
    let inliner = css_inline::CSSInliner::options()
        .base_url(Some(base_url))
        .build();
    let inlined = inliner.inline(HTML);
    // Do something with inlined HTML, e.g. send an email
    Ok(())
}
```

For resolving remote stylesheets it is possible to implement a custom resolver:

```rust
#[derive(Debug, Default)]
pub struct CustomStylesheetResolver;

impl css_inline::StylesheetResolver for CustomStylesheetResolver {
    fn retrieve(&self, location: &str) -> css_inline::Result<String> {
        Err(self.unsupported("External stylesheets are not supported"))
    }
}

fn main() -> css_inline::Result<()> {
    let inliner = css_inline::CSSInliner::options()
        .resolver(std::sync::Arc::new(CustomStylesheetResolver))
        .build();
    Ok(())
}
```

You can also cache external stylesheets to avoid excessive network requests:

```rust
use std::num::NonZeroUsize;

#[cfg(feature = "stylesheet-cache")]
fn main() -> css_inline::Result<()> {
    let inliner = css_inline::CSSInliner::options()
        .cache(
            // This is an LRU cache
            css_inline::StylesheetCache::new(
                NonZeroUsize::new(5).expect("Invalid cache size")
            )
        )
        .build();
    Ok(())
}

// This block is here for testing purposes
#[cfg(not(feature = "stylesheet-cache"))]
fn main() -> css_inline::Result<()> {
    Ok(())
}
```

Caching is disabled by default.

## Performance

`css-inline` typically inlines HTML emails within hundreds of microseconds, though results may vary with input complexity.

Benchmarks for `css-inline==0.19.0`:

- Basic: **4.09 µs**, 230 bytes
- Realistic-1: **78.94 µs**, 8.58 KB
- Realistic-2: **48.56 µs**, 4.3 KB
- GitHub page: **16.78 ms**, 1.81 MB

These benchmarks, conducted using `rustc 1.91` on Ryzen 9 9950X, can be found in `css-inline/benches/inliner.rs`.

## Command Line Interface

### Installation

Install with `cargo`:

```text
cargo install css-inline
```

### Usage

The following command inlines CSS in multiple documents in parallel. The resulting files will be saved
as `inlined.email1.html` and `inlined.email2.html`:

```text
css-inline email1.html email2.html
```

For full details of the options available, you can use the `--help` flag:

```text
css-inline --help
```

## Further reading

If you're interested in learning how this library was created and how it works internally, check out these articles:

- [Rust crate](https://dygalo.dev/blog/rust-for-a-pythonista-2/)
- [Python bindings](https://dygalo.dev/blog/rust-for-a-pythonista-3/)

## Support

If you have any questions or discussions related to this library, please join our [gitter](https://gitter.im/Stranger6667/css-inline)!

## License

This project is licensed under the terms of the <a href="LICENSE">MIT license</a>.
