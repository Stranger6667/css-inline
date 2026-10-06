# css-inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
[<img alt="crates.io" src="https://img.shields.io/crates/v/css-inline.svg?style=flat-square&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/css-inline)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-css_inline-66c2a5?style=flat-square&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/css-inline)
[<img alt="codecov.io" src="https://img.shields.io/codecov/c/gh/Stranger6667/css-inline?logo=codecov&style=flat-square&token=tOzvV4kDY0" height="20">](https://app.codecov.io/github/Stranger6667/css-inline)
[<img alt="gitter" src="https://img.shields.io/gitter/room/Stranger6667/css-inline?style=flat-square" height="20">](https://gitter.im/Stranger6667/css-inline)

`css_inline` inlines CSS into HTML `style` attributes. Use it to prepare HTML emails or to embed HTML into third-party web pages.

It turns this HTML:

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
    <h1 style="color: blue;">Big Text</h1>
  </body>
</html>
```

- Inlines a 4-9 KB email in under 100 µs (see [Performance](#performance))
- Parses with `html5ever` and `cssparser` from Mozilla's Servo project
- Inlines CSS from `style` and `link` tags
- Removes `style` and `link` tags
- Resolves external stylesheets, including local files
- Caches external stylesheets (optional)
- Works on Linux, Windows, and macOS
- Parses HTML5 and CSS3
- Bindings for [Python](https://github.com/Stranger6667/css-inline/tree/master/bindings/python), [Ruby](https://github.com/Stranger6667/css-inline/tree/master/bindings/ruby), [JavaScript](https://github.com/Stranger6667/css-inline/tree/master/bindings/javascript), [Java](https://github.com/Stranger6667/css-inline/tree/master/bindings/java), [C](https://github.com/Stranger6667/css-inline/tree/master/bindings/c), [PHP](https://github.com/Stranger6667/css-inline/tree/master/bindings/php), and a [WebAssembly](https://github.com/Stranger6667/css-inline/tree/master/bindings/javascript/wasm) module to run in browsers.
- [Elixir bindings](https://github.com/knocklabs/css_inline) maintained by [Knock](https://github.com/knocklabs)
- [Command Line Interface](#command-line-interface)

## Playground

Try `css-inline` in the WebAssembly-powered [playground](https://css-inline.org/).

## Installation

Add `css-inline` to the dependencies in your `Cargo.toml`:

```toml
[dependencies]
css-inline = "0.22"
```

Minimum Supported Rust Version: 1.85.

Cargo features:

| Feature | Default | Enables |
|---------|---------|---------|
| `http` | yes | Loading remote stylesheets over `http(s)` (pulls in `reqwest`) |
| `file` | yes | Loading stylesheets from the local filesystem |
| `stylesheet-cache` | yes | `StylesheetCache` and the `cache` option (pulls in `lru`) |
| `cli` | yes | Parallel processing in the `css-inline` binary (pulls in `rayon`) |

The `cli` feature pulls `rayon` into library builds. For library-only use, list the features you need:

```toml
[dependencies]
css-inline = { version = "0.22", default-features = false, features = ["http", "file", "stylesheet-cache"] }
```

Without `http` and `file`, `css-inline` returns an error for any `link` stylesheet it cannot load.

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
    assert_eq!(
        inlined,
        r#"<html><head>
    
</head>
<body>
    <h1 style="color: blue;">Big Text</h1>

</body></html>"#
    );
    Ok(())
}
```

`inline` adds missing `html` and `body` tags, so the output is a complete HTML document.

To inline CSS into an HTML fragment, call `inline_fragment` with the fragment and the CSS. The output stays a fragment: `inline_fragment` does not add `<html>`, `<head>` or `<body>`, and if the input contains them, it keeps only their contents. Use `inline` for full documents:

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
    assert_eq!(
        inlined,
        r#"<main>
<h1 style="color: blue;">Hello</h1>
<section>
<p style="color: red;">who am i</p>
</section>
</main>"#
    );
    Ok(())
}
```

`inline` and `inline_fragment` return `css_inline::Result<String>`, with `css_inline::InlineError` as the error type:

- `MissingStyleSheet`: a local stylesheet file does not exist
- `Network`: fetching a remote stylesheet failed (`http` feature)
- `IO`: reading a stylesheet failed, or writing to the target of `inline_to` / `inline_fragment_to` (which write into any `std::io::Write`) failed
- `ParseError`: a CSS syntax error that `css-inline` cannot skip

A stylesheet that fails to load aborts inlining with an error. Inside a `style` tag or stylesheet, `css-inline` skips rules with invalid or unsupported selectors and inlines the rest.

### Configuration

Configure `css-inline` with the builder that `CSSInliner::options()` returns:

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

`CSSInliner` is `Send + Sync`: build it once and reuse it across calls and threads.

- `inline_style_tags`. Inline CSS from `style` tags. With `false`, `css-inline` ignores their CSS and still removes the tags unless `keep_style_tags` is `true`. Default: `true`
- `keep_style_tags`. Keep `style` tags after inlining. Default: `false`
- `keep_link_tags`. Keep `link` tags after inlining. Default: `false`
- `keep_at_rules`. Keep at-rules (starting with `@`) after inlining. Default: `false`
- `minify_css`. Remove trailing semicolons and spaces between properties and values. Default: `false`
- `base_url`. Base URL for resolving relative URLs. Use the `file://` scheme to load stylesheets from the filesystem. Default: `None`
- `load_remote_stylesheets`. Load stylesheets from `link` tags, over the network or from `file://` paths. Default: `true`
- `cache`. Cache for external stylesheets. Default: `None`
- `extra_css`. Extra CSS to inline. Default: `None`
- `preallocate_node_capacity`. **Advanced**. Number of HTML nodes to preallocate during parsing. Set it to your expected node count to avoid reallocations. Default: `32`
- `remove_inlined_selectors`. Remove inlined selectors from `<style>` blocks and keep the blocks for rules that could not be inlined (e.g. `p:hover`), even when `keep_style_tags` is `false`. Default: `false`
- `apply_width_attributes`. Add `width` HTML attributes from CSS `width` properties on `table`, `td`, `th` and `img`. Default: `false`
- `apply_height_attributes`. Add `height` HTML attributes from CSS `height` properties on `table`, `td`, `th` and `img`. Default: `false`

Add `data-css-inline="ignore"` to a tag to skip inlining styles into it:

```html
<head>
  <style>h1 { color:blue; }</style>
</head>
<body>
  <!-- The tag below won't receive additional styles -->
  <h1 data-css-inline="ignore">Big Text</h1>
</body>
```

On a `link` or `style` tag, the same attribute excludes its styles from inlining:

```html
<head>
  <!-- Styles below are ignored -->
  <style data-css-inline="ignore">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

Add `data-css-inline="keep"` to a `style` tag to keep it in the output, for example to preserve `@media` queries for responsive emails.
The tag stays even when `keep_style_tags` is `false`.

```html
<head>
  <!-- Styles below are not removed -->
  <style data-css-inline="keep">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

A `style` attribute cannot hold at-rules, so `css-inline` drops them by default.
Set `keep_at_rules` to `true` to inline regular rules and keep at-rules such as `@media` in a `style` tag.
That tag stays even when `keep_style_tags` is `false`.

```html
<head>
  <!-- With keep_at_rules=true "color:blue" will get inlined into <h1> but @media will be kept in <style> -->
  <style>h1 { color: blue; } @media (max-width: 600px) { h1 { font-size: 18px; } }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

Set `minify_css` to `true` to drop trailing semicolons and spaces between properties and values in inlined styles:

```html
<head>
  <!-- With minify_css=true, the <h1> will have `style="color:blue;font-weight:bold"` -->
  <style>h1 { color: blue; font-weight: bold; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

To load stylesheets from the filesystem, set `base_url` with the `file://` scheme.
`css-inline` joins relative `link` hrefs onto it, so end the directory with `/`: with `file://styles/email/`, `href="main.css"` resolves to `styles/email/main.css`; without the slash, to `styles/main.css`.
A standard URL parser reads `styles` in `file://styles/email/` as a host. `css-inline` instead treats `styles/email/` as a path relative to the current directory. Use `file:///abs/path/` for an absolute path.

```rust
const HTML: &str = "...";

fn main() -> css_inline::Result<()> {
    let base_url = css_inline::Url::parse("file://styles/email/").expect("Invalid URL");
    let inliner = css_inline::CSSInliner::options()
        .base_url(Some(base_url))
        .build();
    let inlined = inliner.inline(HTML)?;
    // Do something with inlined HTML, e.g. send an email
    Ok(())
}
```

To control how `css-inline` fetches external stylesheets, implement `StylesheetResolver`. Override `retrieve`, or only `retrieve_from_url` / `retrieve_from_path`. The trait's default `unsupported` method builds an `InlineError::IO` with `std::io::ErrorKind::Unsupported` for locations your resolver rejects:

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

To fetch each external stylesheet once, enable the cache (requires the `stylesheet-cache` feature, on by default):

```rust
use std::num::NonZeroUsize;

#[cfg(feature = "stylesheet-cache")]
fn main() -> css_inline::Result<()> {
    let inliner = css_inline::CSSInliner::options()
        .cache(
            // LRU cache keyed by resolved URL; holds up to 5 stylesheets
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

The `cache` option defaults to `None`, so `css-inline` fetches stylesheets on every call until you set it.

## Performance

The table shows absolute timings of the Rust crate. The [Python](https://github.com/Stranger6667/css-inline/tree/master/bindings/python#performance), [Ruby](https://github.com/Stranger6667/css-inline/tree/master/bindings/ruby#performance) and [JavaScript](https://github.com/Stranger6667/css-inline/tree/master/bindings/javascript#performance) READMEs compare `css-inline` with other inliners.

Benchmarks for `css-inline==0.22.0`:

| Case | Bench ID | Time | Size |
|------|----------|------|------|
| Basic | `simple` | **4.10 µs** | 230 bytes |
| Realistic-1 | `big_email_1` | **74.41 µs** | 8.58 KB |
| Realistic-2 | `big_email_2` | **42.13 µs** | 4.30 KB |
| GitHub page | `big_page` | **16.51 ms** | 1.81 MB |

Criterion mean time, measured with `rustc 1.99` on a Ryzen 9 9950X.

The harness is [`css-inline/benches/inliner.rs`](https://github.com/Stranger6667/css-inline/blob/master/css-inline/benches/inliner.rs) (Criterion); inputs are in [`benchmarks/benchmarks.json`](https://github.com/Stranger6667/css-inline/blob/master/benchmarks/benchmarks.json). Run it with `cargo bench` from the `css-inline/` directory.

## Command Line Interface

### Installation

Install with `cargo`:

```text
cargo install css-inline
```

### Usage

This command inlines CSS in two documents in parallel and writes `inlined.email1.html` and `inlined.email2.html` next to the input files:

```text
css-inline email1.html email2.html
```

Piped input goes to stdout: `cat email.html | css-inline > out.html`.

Key flags:

- `--base-url`: base URL for relative stylesheet links
- `--extra-css-file <PATH>`: inline extra CSS from a file (repeatable)
- `--output-filename-prefix`: output prefix instead of `inlined.`

List all options with `--help`:

```text
css-inline --help
```

## Further reading

Articles on how this library was built and how it works:

- [Rust crate](https://dygalo.dev/blog/rust-for-a-pythonista-2/)
- [Python bindings](https://dygalo.dev/blog/rust-for-a-pythonista-3/)

## Support

Ask questions in the [gitter](https://gitter.im/Stranger6667/css-inline) chat.

## License

This project is licensed under the terms of the <a href="LICENSE">MIT license</a>.
