# css_inline

[![Build](https://github.com/Stranger6667/css-inline/workflows/ci/badge.svg)](https://github.com/Stranger6667/css-inline/actions)
[![PyPI](https://img.shields.io/pypi/v/css_inline.svg)](https://pypi.org/project/css_inline/)
[![Python versions](https://img.shields.io/pypi/pyversions/css_inline.svg)](https://pypi.org/project/css_inline/)
[![License](https://img.shields.io/pypi/l/css_inline.svg)](https://opensource.org/licenses/MIT)

Blazing-fast CSS inlining for Python implemented with Mozilla's Servo project components.

Features:

- Removing `style` tags after inlining;
- Resolving external stylesheets (including local files);
- Control if `style` tags should be processed;
- Out-of-document CSS to inline;
- Inlining multiple documents in parallel (via Rust-level threads)

The project supports CSS Syntax Level 3.

## Installation

To install `css_inline` via `pip` run the following command:

```
pip install css_inline
```

Pre-compiled wheels for most popular platforms are provided. If your platform is not in the support table below, you will need
a Rust compiler to build this package from source. The minimum supported Rust version is 1.60.

## Usage

To inline CSS in an HTML document:

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

For customization options use the `CSSInliner` class:

```python
import css_inline

inliner = css_inline.CSSInliner(remove_style_tags=False)
inliner.inline("...")
```

- `inline_style_tags`. Whether to inline CSS from "style" tags. Default: `True`
- `remove_style_tags`. Remove "style" tags after inlining. Default: `True`
- `base_url`. Base URL to resolve relative URLs. If you'd like to load stylesheets from your filesystem, use the `file://` scheme. Default: `None`
- `load_remote_stylesheets`. Whether remote stylesheets should be loaded or not. Default: `True`
- `extra_css`. Additional CSS to inline. Default: `None`
- `preallocate_node_capacity`. **Advanced**. Pre-allocate capacity for HTML nodes during parsing. It can improve performance when you have an estimate of the number of nodes in your HTML document. Default: `8`

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

If you'd like to load stylesheets from your filesystem, use the `file://` scheme:

```python
import css_inline

# styles/email is relative to the current directory
inliner = css_inline.CSSInliner(base_url="file://styles/email/")
inliner.inline("...")
```

## Standards support & restrictions

`css-inline` is built on top of [cssparser](https://crates.io/crates/cssparser) and relies on its behavior for CSS parsing.
Notably:

- Only HTML 5, XHTML is not supported;
- Only CSS 3;
- Only UTF-8 for string representation. Other document encodings are not yet supported.

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

- `css_inline 0.9.0` - 13.58 us
- `premailer 3.10.0` - 310.06 us (**x22.83**)
- `toronado 0.1.0` - 1.43 ms (**x105.32**)
- `inlinestyler 0.2.5` - 2.11 ms (**x155.46**)
- `pynliner 0.8.0` - 2.51 ms (**x184.97**)

Realistic email 1:

- `css_inline 0.9.0` - 266.53 us
- `premailer 3.10.0` - 2.78 ms (**x10.47**)
- `toronado 0.1.0` - 31.20 ms (**x117.09**)
- `inlinestyler 0.2.5` - 52.64 ms (**x197.52**)
- `pynliner 0.8.0` - 101.14 ms (**x379.47**)

Realistic email 2:

- `css_inline 0.9.0` - 237.23 us
- `premailer 3.10.0` - 4.07 ms (**x17.24**)
- `toronado 0.1.0` - `Error: Pseudo-elements are not supported`
- `inlinestyler 0.2.5` - 33.87 ms (**x142.81**)
- `pynliner 0.8.0` - `Error: No match was found`

You can take a look at the benchmarks' code at `benches/bench.py` file.
The results above were measured with stable `rustc 1.69.0`, `Python 3.11.0`, `Linux x86_64` on i8700K, and 32GB RAM.

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

The code in this project is licensed under [MIT license](https://opensource.org/licenses/MIT).
By contributing to `css_inline`, you agree that your contributions
will be licensed under its MIT license.
