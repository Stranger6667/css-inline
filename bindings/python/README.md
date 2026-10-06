# css_inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
[<img alt="pypi" src="https://img.shields.io/pypi/v/css_inline.svg?style=flat-square" height="20">](https://pypi.org/project/css_inline/)
[<img alt="versions" src="https://img.shields.io/pypi/pyversions/css_inline.svg?style=flat-square" height="20">](https://pypi.org/project/css_inline/)
[<img alt="license" src="https://img.shields.io/pypi/l/css_inline.svg?style=flat-square" height="20">](https://opensource.org/licenses/MIT)
[<img alt="codecov.io" src="https://img.shields.io/codecov/c/gh/Stranger6667/css-inline?logo=codecov&style=flat-square&token=tOzvV4kDY0" height="20">](https://app.codecov.io/github/Stranger6667/css-inline)
[<img alt="gitter" src="https://img.shields.io/gitter/room/Stranger6667/css-inline?style=flat-square" height="20">](https://gitter.im/Stranger6667/css-inline)

`css_inline` is a high-performance library for inlining CSS into HTML 'style' attributes.

Use it to prepare HTML emails or to embed HTML into third-party web pages.

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
<html><head>

  </head>
  <body>
    <h1 style="color: blue;">Big Text</h1>

</body></html>
```

- Uses components from Mozilla's Servo project
- 14x to 680x faster than `premailer`, the fastest Python alternative (see [Performance](#performance))
- Inlines CSS from `style` and `link` tags
- Removes `style` and `link` tags
- Resolves external stylesheets (including local files)
- Caches external stylesheets (opt-in)
- Processes multiple documents in parallel
- Works on Linux, Windows, macOS and in the browser via Pyodide
- Supports HTML5 & CSS3
- Tested on CPython 3.10, 3.11, 3.12, 3.13, 3.14 and PyPy 3.11.

## Playground

Try `css-inline` in the WebAssembly-powered [playground](https://css-inline.org/).

## Installation

Install with `pip`:

```shell
pip install css_inline
```

The PyPI name is `css-inline`; `pip` accepts `css_inline` too. Import it as `css_inline`.

PyPI ships pre-compiled wheels for:

- Linux (glibc and musl): x86_64, aarch64, armv7; glibc also i686
- macOS: x86_64, arm64
- Windows: x64, x86
- PyPy 3.11: Linux x86_64 and aarch64, macOS x86_64

On other platforms, `pip` builds the package from source, which requires Rust 1.85 or newer.

## Usage

```python
import css_inline

HTML = """<html>
<head>
    <style>h1 { color:blue; }</style>
</head>
<body>
    <h1>Big Text</h1>
</body>
</html>"""

inlined = css_inline.inline(HTML)
# <html><head>
#
# </head>
# <body>
#     <h1 style="color: blue;">Big Text</h1>
#
# </body></html>
```

`inline` adds missing `html`, `head` and `body` tags, so the output is a complete HTML document.

To inline CSS into an HTML fragment, use `inline_fragment(html, css)`. Both arguments are required; `css` holds the stylesheet to apply. The output stays a fragment: no `<html>`, `<head>` or `<body>` wrapper. Use `inline` for full documents:

```python
FRAGMENT = """<main>
<h1>Hello</h1>
<section>
<p>who am i</p>
</section>
</main>"""

CSS = """
p {
    color: red;
}

h1 {
    color: blue;
}
"""

inlined = css_inline.inline_fragment(FRAGMENT, CSS)
# <main>
# <h1 style="color: blue;">Hello</h1>
# <section>
# <p style="color: red;">who am i</p>
# </section>
# </main>
```

To inline many documents at once, use `inline_many` or `inline_many_fragments`.
Both process inputs in parallel and return a list of strings in input order.
`inline_many_fragments` pairs `htmls[i]` with `css[i]`.

**Warning**: if the lists differ in length, `inline_many_fragments` silently drops the extra items from the longer list.
`CSSInliner` has the same two methods.

```python
import css_inline

css_inline.inline_many(["<...>", "<...>"])
css_inline.inline_many_fragments(["<p>a</p>", "<p>b</p>"], ["p { color: red; }", "p { color: blue; }"])
```

`inline_many` processes inputs in parallel on a Rust thread pool ([Rayon](https://github.com/rayon-rs/rayon)), so you don't need `multiprocessing`.
The speedup scales with the number of CPU cores.

### Configuration

Pass options to the `CSSInliner` class:

```python
import css_inline

inliner = css_inline.CSSInliner(keep_style_tags=True)
inliner.inline("...")
```

- `inline_style_tags`. Inline CSS from `style` tags. Default: `True`
- `keep_style_tags`. Keep `style` tags after inlining. Default: `False`
- `keep_link_tags`. Keep `link` tags after inlining. Default: `False`
- `keep_at_rules`. Keep at-rules (starting with `@`) after inlining. Default: `False`
- `minify_css`. Remove trailing semicolons and spaces between properties and values. Default: `False`
- `base_url`. Base URL for resolving relative URLs. Use the `file://` scheme to load stylesheets from the filesystem. Default: `None`
- `load_remote_stylesheets`. Load stylesheets from `link` tags, over the network or from `file://` paths. Default: `True`
- `cache`. Cache for external stylesheets, for example `StylesheetCache(size=5)`. Default: `None`
- `extra_css`. Extra CSS to inline. Default: `None`
- `preallocate_node_capacity`. **Advanced**. Number of HTML nodes to preallocate during parsing. Set it to your expected node count to avoid reallocations. Default: `32`
- `remove_inlined_selectors`. Remove inlined selectors from `<style>` blocks. Default: `False`
- `apply_width_attributes`. Add `width` HTML attributes from CSS `width` properties on supported elements (`table`, `td`, `th`, `img`). Default: `False`
- `apply_height_attributes`. Add `height` HTML attributes from CSS `height` properties on supported elements (`table`, `td`, `th`, `img`). Default: `False`

To skip CSS inlining for an HTML tag, add the `data-css-inline="ignore"` attribute:

```html
<head>
  <style>h1 { color:blue; }</style>
</head>
<body>
  <!-- The tag below won't receive additional styles -->
  <h1 data-css-inline="ignore">Big Text</h1>
</body>
```

The same attribute on a `link` or `style` tag makes the inliner skip that stylesheet:

```html
<head>
  <!-- Styles below are ignored -->
  <style data-css-inline="ignore">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

To keep a `style` tag in the output, add `data-css-inline="keep"`.
Use it for `@media` queries in responsive emails.
The tag stays even with `keep_style_tags=False`.

```html
<head>
  <!-- Styles below are not removed -->
  <style data-css-inline="keep">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

At-rules can't go into `style` attributes, so the inliner removes them by default.
Set `keep_at_rules=True` to keep them, for example `@media` queries for responsive emails, in a `style` tag while inlining everything else.
The tag stays even with `keep_style_tags=False`.

```html
<head>
  <!-- With keep_at_rules=True, "color: blue" goes into <h1> and @media stays in <style> -->
  <style>h1 { color: blue; } @media (max-width: 600px) { h1 { font-size: 18px; } }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

`minify_css=True` removes trailing semicolons and spaces between properties and values in inlined styles.

```html
<head>
  <!-- With minify_css=True, the <h1> will have `style="color:blue;font-weight:bold"` -->
  <style>h1 { color: blue; font-weight: bold; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

To load stylesheets from the filesystem, use the `file://` scheme.
Standard URL parsing reads `file://styles/email/` as host `styles` and path `/email/`.
`css-inline` deviates from the standard here: for a `file://` URL with a host, it drops the `file://` prefix and treats `styles/email/` as a path relative to the current directory.
With either form, `<link href="main.css">` resolves against the base, so keep the trailing slash: without it, `main.css` replaces the last segment (`styles/main.css`).

```python
import css_inline

# Absolute directory: three slashes
inliner = css_inline.CSSInliner(base_url="file:///srv/app/styles/email/")
inliner.inline("...")

# Relative to the current directory: two slashes
inliner = css_inline.CSSInliner(base_url="file://styles/email/")
inliner.inline("...")
```

To avoid repeated network requests, cache external stylesheets:

```python
import css_inline

inliner = css_inline.CSSInliner(
    cache=css_inline.StylesheetCache(size=5)
)
inliner.inline("...")
```

`size` is the maximum number of stylesheets in the cache and must be greater than zero; otherwise `StylesheetCache` raises `InlineError`. When it is full, the least recently used entry is evicted. Caching is off by default.

### Errors

Inlining raises `css_inline.InlineError`, a `ValueError` subclass, when a stylesheet fails to load (missing file, network error) or to parse.
An invalid `base_url` raises a plain `ValueError`.

```python
import css_inline

try:
    css_inline.inline('<link href="missing.css" rel="stylesheet">')
except css_inline.InlineError as exc:
    print(exc)  # Missing stylesheet file: missing.css
```

## XHTML compatibility

`css-inline` outputs HTML5, so void tags stay unclosed (`<hr>`, not `<hr/>`). For XHTML output, re-serialize with `lxml`:

```python
import css_inline
from lxml import html, etree

document = "..."  # Your HTML document
inlined = css_inline.inline(document)
tree = html.fromstring(inlined)
inlined = etree.tostring(tree).decode(encoding="utf-8")
```

## Performance

`css-inline` is 14x to 680x faster than the next fastest Python inliner, `premailer`, on the inputs below:

|             | Size    | `css_inline 0.22.0` | `premailer 3.10.0`     | `toronado 0.1.0`        | `pynliner 0.8.0`        |
|-------------|---------|---------------------|------------------------|-------------------------|-------------------------|
| Basic       | 230 B   | 4.19 µs             | 90.77 µs (**21.66x**)  | 538.65 µs (**128.56x**) | 951.89 µs (**227.18x**) |
| Realistic-1 | 8.58 KB | 78.15 µs            | 1.12 ms (**14.32x**)   | 12.25 ms (**156.77x**)  | 12.92 ms (**165.31x**)  |
| Realistic-2 | 4.30 KB | 42.70 µs            | 1.47 ms (**34.38x**)   | ERROR                   | ERROR                   |
| GitHub page | 1.81 MB | 16.79 ms            | 11.42 s (**680.16x**)  | ERROR                   | ERROR                   |

`toronado` and `pynliner` fail on Realistic-2 and the GitHub page because those inputs contain CSS they do not support: `toronado` raises `ExpressionError: Pseudo-elements are not supported.`, `pynliner` raises `Exception: No match was found. We're done or something is broken`.
`inlinestyler` 0.2.5 is not in the table: it crashes with current `lxml` (`'CSSSelector' object has no attribute 'evaluate'`).

Inputs live in [`benchmarks/benchmarks.json`](https://github.com/Stranger6667/css-inline/blob/master/benchmarks/benchmarks.json): Basic is `simple`, Realistic-1 is `big_email_1`, Realistic-2 is `big_email_2`, GitHub page is `big_page`.
Benchmark code: [`benches/bench.py`](https://github.com/Stranger6667/css-inline/blob/master/bindings/python/benches/bench.py) (`pytest-benchmark`, mean time). Environment: `rustc 1.99` (stable), Python `3.14.5`, Ryzen 9 9950X.

## Comparison with other libraries

Compared to other Python inliners, `css-inline`:

- Supports more CSS features. For example, `toronado` and `pynliner` don't support pseudo-elements.
- Has fewer configuration options than `premailer`.
- Has no debug logs.
- Supports only HTML5.

## Further reading

How the library works internally:

- [Rust crate](https://dygalo.dev/blog/rust-for-a-pythonista-2/)
- [Python bindings](https://dygalo.dev/blog/rust-for-a-pythonista-3/)

## License

This project is licensed under the terms of the [MIT license](https://opensource.org/licenses/MIT).
