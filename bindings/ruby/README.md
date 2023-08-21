# css_inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
[<img alt="ruby gems" src="https://img.shields.io/gem/v/css_inline?logo=ruby&style=flat-square" height="20">](https://rubygems.org/gems/css_inline)
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
- Inlines CSS from `style` and `link` tags
- Removes `style` and `link` tags
- Resolves external stylesheets (including local files)
- Can process multiple documents in parallel
- Works on Linux, Windows, and macOS
- Supports HTML5 & CSS3

## Installation

Add this line to your application's `Gemfile`:

```
gem 'css_inline'
```

## Usage

To inline CSS in an HTML document:

```ruby
require 'css_inline'

html = "<html><head><style>h1 { color:blue; }</style></head><body><h1>Big Text</h1></body></html>"
inlined = CSSInline.inline(html)

puts inlined
# Outputs: "<html><head></head><body><h1 style=\"color:blue;\">Big Text</h1></body></html>"
```

When there is a need to inline multiple HTML documents simultaneously, `css_inline` offers the `inline_many` function.
This feature allows for concurrent processing of several inputs, significantly improving performance when dealing with a large number of documents.

```ruby
require 'css_inline'

inlined = CSSInline.inline_many(["...", "..."])
```

Under the hood, `inline_many`, spawns threads at the Rust layer to handle the parallel processing of inputs.
This results in faster execution times compared to employing parallel processing techniques at the Ruby level.

**Note**: To fully benefit from `inline_many`, you should run your application on a multicore machine.

## Configuration

For customization options use the `CSSInliner` class:

```ruby
require 'css_inline'

inliner = CSSInline::CSSInliner.new(keep_style_tags: true)
inliner.inline("...")
```

- `keep_style_tags`. Specifies whether to keep "style" tags after inlining. Default: `False`
- `keep_link_tags`. Specifies whether to keep "link" tags after inlining. Default: `False`
- `base_url`. The base URL used to resolve relative URLs. If you'd like to load stylesheets from your filesystem, use the `file://` scheme. Default: `nil`
- `load_remote_stylesheets`. Specifies whether remote stylesheets should be loaded. Default: `True`
- `extra_css`. Extra CSS to be inlined. Default: `nil`
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

```ruby
require 'css_inline'

# styles/email is relative to the current directory
inliner = CSSInline::CSSInliner.new(base_url: "file://styles/email/")
inliner.inline("...")
```

## Performance

Leveraging efficient tools from Mozilla's Servo project, this library delivers superior performance.
It consistently outperforms `premailer`, offering speed increases often exceeding **50 times**.

The table below provides a detailed comparison between `css_inline` and `premailer` when inlining CSS into an HTML document (like in the Usage section above):

|                   | Size    | `css_inline 0.10.4` | `premailer 1.21.0 with Nokogiri 1.15.2`        | Difference |
|-------------------|---------|---------------------|------------------------------------------------|------------|
| Basic usage       | 230 B   | 8.05 µs             | 419.75 µs                                      | **52.13x** |
| Realistic email 1 | 8.58 KB | 164.22 µs           | 9.75 ms                                        | **59.40x** |
| Realistic email 2 | 4.3 KB  | 106.95 µs           | Error: Cannot parse 0 calc((100% - 500px) / 2) | -          |
| GitHub Page       | 1.81 MB | 308.11 ms           | 3.08 s                                         | **9.99x**  |

Please refer to the `test/bench.rb` file to review the benchmark code.
The results displayed above were measured using stable `rustc 1.71.1` on Ruby `3.2.2`.

## Ruby support

`css_inline` supports Ruby 2.7 and 3.2.

## Further reading

If you want to know how this library was created & how it works internally, you could take a look at these articles:

- [Rust crate](https://dygalo.dev/blog/rust-for-a-pythonista-2/)
- [Python bindings](https://dygalo.dev/blog/rust-for-a-pythonista-3/)

## License

This project is licensed under the terms of the [MIT license](https://opensource.org/licenses/MIT).
