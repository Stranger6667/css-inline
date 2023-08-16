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
- Can process multiple documents in parallel
- Works on Linux, Windows, and macOS
- Supports HTML5 & CSS3

## Installation

Install with `pip`:

```
pip install css_inline
```

Pre-compiled wheels are available for most popular platforms.
If not available for your platform, a Rust compiler will be needed to build this package from source. Rust version 1.62.1 or higher is required.

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

When there is a need to inline multiple HTML documents simultaneously, `css_inline` offers the `inline_many` function.
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

- `keep_style_tags`. Specifies whether to keep "style" tags after inlining. Default: `False`
- `keep_link_tags`. Specifies whether to keep "link" tags after inlining. Default: `False`
- `base_url`. The base URL used to resolve relative URLs. If you'd like to load stylesheets from your filesystem, use the `file://` scheme. Default: `None`
- `load_remote_stylesheets`. Specifies whether remote stylesheets should be loaded. Default: `True`
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

```python
import css_inline

# styles/email is relative to the current directory
inliner = css_inline.CSSInliner(base_url="file://styles/email/")
inliner.inline("...")
```

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

`css-inline`  is powered by efficient tooling from Mozilla's Servo project and significantly outperforms other Python alternatives in terms of speed.
Most of the time it achieves over a **10x** speed advantage compared to the next fastest alternative.

Here is the performance comparison:

|             | `css_inline 0.10.4` | `premailer 3.10.0`      | `toronado 0.1.0`        | `inlinestyler 0.2.5`    | `pynliner 0.8.0`        |
|-------------|---------------------|-------------------------|-------------------------|-------------------------|-------------------------|
| Basic       | 7.08 µs             | 128.33 µs  (**18.12x**) | 673.62 µs  (**95.10x**) | 1.05 ms  (**149.38x**)  | 1.21 ms  (**172.14x**)  |
| Realistic-1 | 151.83 µs           | 1.41 ms  (**9.33x**)    | 16.62 ms  (**109.50x**) | 27.53 ms  (**181.32x**) | 52.46 ms  (**345.57x**) |
| Realistic-2 | 108.66 µs           | 2.73 ms  (**25.15x**)   | ERROR                   | 18.13 ms  (**166.85x**) | ERROR                   |

The above data was obtained from benchmarking the inlining of CSS in HTML, as described in the Usage section.
Note that the `toronado` and `pynliner` libraries both encountered errors when used to inline CSS in the last scenario.

The benchmarking code is available in the `benches/bench.py` file. The benchmarks were conducted using the stable `rustc 1.71.1` on Python `3.11.4`.

## Comparison with other libraries

Besides performance, `css-inline` differs from other Python libraries for CSS inlining.

- Generally supports more CSS features than other libraries (for example, `toronado` and `pynliner` do not support pseudo-elements);
- It has fewer configuration options and not as flexible as `premailer`;
- Works on fewer platforms than LXML-based libraries (`premailer`, `inlinestyler`, `toronado`, and optionally `pynliner`);
- Does not have debug logs yet;
- Supports only HTML 5.

## Python support

`css_inline` supports CPython 3.7, 3.8, 3.9, 3.10, 3.11 and PyPy 3.7, 3.8, 3.9.

## Further reading

If you want to know how this library was created & how it works internally, you could take a look at these articles:

- [Rust crate](https://dygalo.dev/blog/rust-for-a-pythonista-2/)
- [Python bindings](https://dygalo.dev/blog/rust-for-a-pythonista-3/)

## License

This project is licensed under the terms of the [MIT license](https://opensource.org/licenses/MIT).
