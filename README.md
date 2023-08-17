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

- Uses reliable components from Mozilla's Servo project
- Inlines CSS from `style` and `link` tags
- Removes `style` and `link` tags
- Resolves external stylesheets (including local files)
- Works on Linux, Windows, and macOS
- Supports HTML5 & CSS3

## Installation

To include it in your project, add the following line to the dependencies section in your project's `Cargo.toml` file:

```toml
[dependencies]
css-inline = "0.10"
```

The Minimum Supported Rust Version is 1.62.1.

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

fn main() -> Result<(), css_inline::InlineError> {
    let inlined = css_inline::inline(HTML)?;
    // Do something with inlined HTML, e.g. send an email
    Ok(())
}
```

### Configuration

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

- `keep_style_tags`. Specifies whether to keep "style" tags after inlining. Default: `false`
- `keep_link_tags`. Specifies whether to keep "link" tags after inlining. Default: `false`
- `base_url`. The base URL used to resolve relative URLs. If you'd like to load stylesheets from your filesystem, use the `file://` scheme. Default: `None`
- `load_remote_stylesheets`. Specifies whether remote stylesheets should be loaded. Default: `true`
- `extra_css`. Extra CSS to be inlined. Default: `None`
- `preallocate_node_capacity`. **Advanced**. Preallocates capacity for HTML nodes during parsing. This can improve performance when you have an estimate of the number of nodes in your HTML document. Default: `32`

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

If you'd like to load stylesheets from your filesystem, use the `file://` scheme:

```rust
const HTML: &str = "...";

fn main() -> Result<(), css_inline::InlineError> {
    let base_url = css_inline::Url::parse("file://styles/email/").expect("Invalid URL");
    let inliner = css_inline::CSSInliner::options()
        .base_url(Some(base_url))
        .build();
    let inlined = inliner.inline(HTML);
    // Do something with inlined HTML, e.g. send an email
    Ok(())
}
```

## Performance

`css-inline` typically inlines HTML emails within hundreds of microseconds, though results may vary with input complexity.

Benchmarks for `css-inline==0.10.4`:

- Basic: **6.72 µs**, 230 bytes
- Realistic-1: **144.70 µs**, 8.58 KB
- Realistic-2: **95.83 µs**, 4.3 KB
- GitHub page: **539.26 ms**, 1.81 MB

These benchmarks, conducted using `rustc 1.71.1`, can be found in `css-inline/benches/inliner.rs`.

## Bindings

`css-inline` is primarily a Rust library, but we also provide bindings for several other languages:

- [C](https://github.com/Stranger6667/css-inline/tree/master/bindings/c)
- [Python](https://github.com/Stranger6667/css-inline/tree/master/bindings/python)
- [Ruby](https://github.com/Stranger6667/css-inline/tree/master/bindings/ruby)
- [WebAssembly](https://github.com/Stranger6667/css-inline/tree/master/bindings/wasm)

## Command Line Interface

### Installation

Install with `cargo`:

```text
cargo install css-inline
```

### Usage

The following command inlines CSS in multiple documents in parallel. Resulting files will be saved
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
