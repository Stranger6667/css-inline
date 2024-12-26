# css_inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
[<img alt="pypi" src="https://img.shields.io/pypi/v/css_inline.svg?style=flat-square" height="20">](https://pypi.org/project/css_inline/)
[<img alt="versions" src="https://img.shields.io/pypi/pyversions/css_inline.svg?style=flat-square" height="20">](https://pypi.org/project/css_inline/)
[<img alt="license" src="https://img.shields.io/pypi/l/css_inline.svg?style=flat-square" height="20">](https://opensource.org/licenses/MIT)
[<img alt="codecov.io" src="https://img.shields.io/codecov/c/gh/Stranger6667/css-inline?logo=codecov&style=flat-square&token=tOzvV4kDY0" height="20">](https://app.codecov.io/github/Stranger6667/css-inline)
[<img alt="gitter" src="https://img.shields.io/gitter/room/Stranger6667/css-inline?style=flat-square" height="20">](https://gitter.im/Stranger6667/css-inline)

`css_inline` is a high-performance library for inlining CSS into HTML 'style' attributes.

This library is designed for scenarios such as preparing HTML emails or embedding HTML into third-party web pages.

For instance, the library transforms HTML like this:

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
- 10-400x faster than alternatives
- Inlines CSS from `style` and `link` tags
- Removes `style` and `link` tags
- Resolves external stylesheets (including local files)
- Optionally caches external stylesheets
- Can process multiple documents in parallel
- Works on Linux, Windows, macOS and in the browser via PyOdide
- Supports HTML5 & CSS3
- Tested on CPython 3.7, 3.8, 3.9, 3.10, 3.11, 3.12 and PyPy 3.7, 3.8, 3.9, 3.10.

## Playground

If you'd like to try `css-inline`, you can check the WebAssembly-powered [playground](https://css-inline.org/) to see the results instantly.

## Installation

Install with `pip`:

```shell
pip install css_inline
```

Pre-compiled wheels are available for most popular platforms.
If not available for your platform, a Rust compiler will be needed to build this package from source. Rust version 1.65 or higher is required.

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
# HTML becomes this:
#
# <html>
# <head>
#    <style>h1 { color:blue; }</style>
# </head>
# <body>
#     <h1 style="color:blue;">Big Text</h1>
# </body>
# </html>
```

Note that `css-inline` automatically adds missing `html` and `body` tags, so the output is a valid HTML document.

Alternatively, you can inline CSS into an HTML fragment, preserving the original structure:

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
# HTML becomes this:
# <main>
# <h1 style="color: blue;">Hello</h1>
# <section>
# <p style="color: red;">who am i</p>
# </section>
# </main>
```

When there is a need to inline multiple HTML documents simultaneously, `css_inline` offers `inline_many` and `inline_many_fragments` functions.
This feature allows for concurrent processing of several inputs, significantly improving performance when dealing with a large number of documents.

```python
import css_inline

css_inline.inline_many(["<...>", "<...>"])
```

Under the hood, `inline_many`, spawns threads at the Rust layer to handle the parallel processing of inputs.
This results in faster execution times compared to employing parallel processing techniques at the Python level.

**Note**: To fully benefit from `inline_many`, you should run your application on a multicore machine.

### Configuration

For configuration options use the `CSSInliner` class:

```python
import css_inline

inliner = css_inline.CSSInliner(keep_style_tags=True)
inliner.inline("...")
```

- `inline_style_tags`. Specifies whether to inline CSS from "style" tags. Default: `True`
- `keep_style_tags`. Specifies whether to keep "style" tags after inlining. Default: `False`
- `keep_link_tags`. Specifies whether to keep "link" tags after inlining. Default: `False`
- `base_url`. The base URL used to resolve relative URLs. If you'd like to load stylesheets from your filesystem, use the `file://` scheme. Default: `None`
- `load_remote_stylesheets`. Specifies whether remote stylesheets should be loaded. Default: `True`
- `cache`. Specifies caching options for external stylesheets (for example, `StylesheetCache(size=5)`). Default: `None`
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

Alternatively, you may keep `style` from being removed by using the `data-css-inline="keep"` attribute.
This is useful if you want to keep `@media` queries for responsive emails in separate `style` tags:

```html
<head>
  <!-- Styles below are not removed -->
  <style data-css-inline="keep">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

Such tags will be kept in the resulting HTML even if the `keep_style_tags` option is set to `false`.

If you'd like to load stylesheets from your filesystem, use the `file://` scheme:

```python
import css_inline

# styles/email is relative to the current directory
inliner = css_inline.CSSInliner(base_url="file://styles/email/")
inliner.inline("...")
```

You can also cache external stylesheets to avoid excessive network requests:

```python
import css_inline

inliner = css_inline.CSSInliner(
    cache=css_inline.StylesheetCache(size=5)
)
inliner.inline("...")
```

Caching is disabled by default.

## XHTML compatibility

If you'd like to work around some XHTML compatibility issues like closing empty tags (`<hr>` vs. `<hr/>`), you can use the following snippet that involves `lxml`:

```python
import css_inline
from lxml import html, etree

document = "..."  # Your HTML document
inlined = css_inline.inline(document)
tree = html.fromstring(inlined)
inlined = etree.tostring(tree).decode(encoding="utf-8")
```

## Performance

`css-inline` is powered by efficient tooling from Mozilla's Servo project and significantly outperforms other Python alternatives in terms of speed.
Most of the time it achieves over a **10x** speed advantage compared to the next fastest alternative.

Here is the performance comparison:

|             | Size    | `css_inline 0.14.1` | `premailer 3.10.0`     | `toronado 0.1.0`        | `inlinestyler 0.2.5`   | `pynliner 0.8.0`       |
|-------------|---------|---------------------|------------------------|-------------------------|------------------------|------------------------|
| Basic       | 230 B   | 6.54 µs             | 127.62 µs (**19.51x**) | 657.50 µs (**100.52x**) | 1.02 ms (**157.01x**)  | 1.17ms (**179.64x**)   |
| Realistic-1 | 8.58 KB | 134.54 µs           | 1.40 ms (**10.42x**)   | 15.81 ms (**117.54x**)  | 26.37 ms (**196.04x**) | 52.77 ms (**392.29x**) |
| Realistic-2 | 4.3 KB  | 82.37 µs            | 2.78 ms (**33.80x**)   | ERROR                   | 17.71 ms (**215.01x**) | ERROR                  |
| GitHub page | 1.81 MB | 223.85 ms           | 25.04 s (**111.90x**)  | ERROR                   | ERROR                  | ERROR                  |

The "Basic" case was obtained by benchmarking the example from the Usage section.
Note that the `toronado`, `inlinestyler`, and `pynliner` libraries both encountered errors when used to inline CSS in the last scenario.

The benchmarking code is available in the `benches/bench.py` file. The benchmarks were conducted using the stable `rustc 1.78`, Python `3.11.7` on M1 Max.

## Comparison with other libraries

Besides performance, `css-inline` differs from other Python libraries for CSS inlining.

- Generally supports more CSS features than other libraries (for example, `toronado` and `pynliner` do not support pseudo-elements);
- It has fewer configuration options and is not as flexible as `premailer`;
- Works on fewer platforms than LXML-based libraries (`premailer`, `inlinestyler`, `toronado`, and optionally `pynliner`);
- Does not have debug logs yet;
- Supports only HTML 5.

## Further reading

If you want to know how this library was created & how it works internally, you could take a look at these articles:

- [Rust crate](https://dygalo.dev/blog/rust-for-a-pythonista-2/)
- [Python bindings](https://dygalo.dev/blog/rust-for-a-pythonista-3/)

## License

This project is licensed under the terms of the [MIT license](https://opensource.org/licenses/MIT).
