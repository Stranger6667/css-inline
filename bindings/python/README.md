# css_inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline)
[<img alt="pypi" src="https://img.shields.io/pypi/v/css_inline.svg?style=flat-square" height="20">](https://pypi.org/project/css_inline/)
[<img alt="versions" src="https://img.shields.io/pypi/pyversions/css_inline.svg?style=flat-square" height="20">](https://pypi.org/project/css_inline/)
[<img alt="license" src="https://img.shields.io/pypi/l/css_inline.svg?style=flat-square" height="20">](https://opensource.org/licenses/MIT)
[<img alt="codecov.io" src="https://img.shields.io/codecov/c/gh/Stranger6667/css-inline?logo=codecov&style=flat-square&token=tOzvV4kDY0" height="20">](https://app.codecov.io/github/Stranger6667/css-inline)
[<img alt="gitter" src="https://img.shields.io/gitter/room/Stranger6667/css-inline?style=flat-square" height="20">](https://gitter.im/Stranger6667/css-inline)

`css-inline` inlines CSS into HTML documents, using components from Mozilla's Servo project.

This process is essential for sending HTML emails as you need to use "style" attributes instead of "style" tags.

For instance, the library transforms HTML like this:

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

into:

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

- Uses reliable components from Mozilla's Servo
- 10-300x faster than alternatives
- Inlines CSS from `style` and `link` tags
- Removes `style` and `link` tags
- Resolves external stylesheets (including local files)
- Can process multiple documents in parallel
- Works on Linux, Windows, and macOS
- Supports HTML5 & CSS3

## Installation

Install with `pip`:

```
pip install css_inline
```

Pre-compiled wheels for most popular platforms are provided. If your platform is not supported, you will need
a Rust compiler to build this package from source. The minimum supported Rust version is 1.60.

## Usage

```python
import css_inline

HTML = """<html>
<head>
    <title>Test</title>
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
#    <title>Test</title>
#    <style>h1 { color:blue; }</style>
# </head>
# <body>
#     <h1 style="color:blue;">Big Text</h1>
# </body>
# </html>
```

If you want to inline many HTML documents, you can utilize `inline_many` that processes the input in parallel.

```python
import css_inline

css_inline.inline_many(["<...>", "<...>"])
```

`inline_many` will spawn threads on the Rust level; thus, you can expect it's running faster than `css_inline.inline` via Python's `multiprocessing` or `threading` modules.

### Configuration

For configuration options use the `CSSInliner` class:

```python
import css_inline

inliner = css_inline.CSSInliner(keep_style_tags=True)
inliner.inline("...")
```

- `keep_style_tags`. Specifies whether to keep "style" tags after inlining. Default: `False`
- `keep_link_tags`. Specifies whether to keep "link" tags after inlining. Default: `False`
- `base_url`. The base URL used to resolve relative URLs. If you'd like to load stylesheets from your filesystem, use the `file://` scheme. Default: `None`
- `load_remote_stylesheets`. Specifies whether remote stylesheets should be loaded. Default: `True`
- `extra_css`. Extra CSS to be inlined. Default: `None`
- `preallocate_node_capacity`. **Advanced**. Preallocates capacity for HTML nodes during parsing. This can improve performance when you have an estimate of the number of nodes in your HTML document. Default: `8`

You can also skip CSS inlining for an HTML tag by adding the `data-css-inline="ignore"` attribute to it:

```html
<head>
    <title>Test</title>
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
    <title>Test</title>
    <!-- Styles below are ignored -->
    <style data-css-inline="ignore">h1 { color:blue; }</style>
</head>
<body>
    <h1>Big Text</h1>
</body>
```

If you'd like to load stylesheets from your filesystem, use the `file://` scheme:

```python
import css_inline

# styles/email is relative to the current directory
inliner = css_inline.CSSInliner(base_url="file://styles/email/")
inliner.inline("...")
```

## Standards support & restrictions

`css-inline` is built on top of [html5ever](https://crates.io/crates/html5ever) and [cssparser](https://crates.io/crates/cssparser) and relies on their behavior for HTML & CSS parsing.

- Only HTML 5 is supported, not XHTML.
- Only CSS 3 is supported.
- Only UTF-8 encoding for string representation. Other document encodings are not yet supported.

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

Due to the usage of efficient tooling from Mozilla's Servo project (`html5ever`, `rust-cssparser` and others) this
library has excellent performance characteristics. In comparison with other Python projects, it is usually >10x faster than the nearest alternative.

For inlining CSS in the html document from the `Usage` section above there is the following breakdown in the benchmarks:

- `css_inline 0.9.0` - 9.95 us
- `premailer 3.10.0` - 210.27 us (**x21.11**)
- `toronado 0.1.0` - 1.10 ms (**x110.96**)
- `inlinestyler 0.2.5` - 1.72 ms (**x172.92**)
- `pynliner 0.8.0` - 1.95 ms (**x195.90**)

Realistic email 1:

- `css_inline 0.9.0` - 277.01 us
- `premailer 3.10.0` - 2.68 ms (**x9.69**)
- `toronado 0.1.0` - 27.03 ms (**x97.59**)
- `inlinestyler 0.2.5` - 51.74 ms (**x186.81**)
- `pynliner 0.8.0` - 86.89 ms (**x313.67**)

Realistic email 2:

- `css_inline 0.9.0` - 155.67 us
- `premailer 3.10.0` - 4.05 ms (**x26.04**)
- `toronado 0.1.0` - `Error: Pseudo-elements are not supported`
- `inlinestyler 0.2.5` - 29.51 ms (**x189.58**)
- `pynliner 0.8.0` - `Error: No match was found`

You can take a look at the benchmarks' code at `benches/bench.py` file.
The results above were measured with stable `rustc 1.70.0` on `Python 3.11.0`.

## Comparison with other libraries

Besides performance, `css-inline` differs from other Python libraries for CSS inlining.

- Generally supports more CSS features than other libraries (for example, `toronado` and `pynliner` do not support pseudo-elements);
- It has fewer configuration options and not as flexible as `premailer`;
- Works on fewer platforms than LXML-based libraries (`premailer`, `inlinestyler`, `toronado`, and optionally `pynliner`);
- Does not have debug logs yet;
- Supports only HTML 5.

## Python support

`css_inline` supports CPython 3.7, 3.8, 3.9, 3.10, 3.11 and PyPy 3.7, 3.8, 3.9.

## Extra materials

If you want to know how this library was created & how it works internally, you could take a look at these articles:

- [Rust crate](https://dygalo.dev/blog/rust-for-a-pythonista-2/)
- [Python bindings](https://dygalo.dev/blog/rust-for-a-pythonista-3/)

## License

This project is licensed under the terms of the [MIT license](https://opensource.org/licenses/MIT).
