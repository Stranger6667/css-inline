# css_inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
[<img alt="ruby gems" src="https://img.shields.io/gem/v/css_inline?logo=ruby&style=flat-square" height="20">](https://rubygems.org/gems/css_inline)
[<img alt="codecov.io" src="https://img.shields.io/codecov/c/gh/Stranger6667/css-inline?logo=codecov&style=flat-square&token=tOzvV4kDY0" height="20">](https://app.codecov.io/github/Stranger6667/css-inline)
[<img alt="gitter" src="https://img.shields.io/gitter/room/Stranger6667/css-inline?style=flat-square" height="20">](https://gitter.im/Stranger6667/css-inline)

`css_inline` moves CSS from `<style>` and `<link>` tags into HTML `style` attributes.
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
<html>
  <head></head>
  <body>
    <h1 style="color: blue;">Big Text</h1>
  </body>
</html>
```

- Parses and matches CSS with components from Mozilla's Servo project
- Inlines CSS from `style` and `link` tags
- Removes `style` and `link` tags
- Resolves external stylesheets (including local files)
- Caches external stylesheets (opt-in)
- Can process multiple documents in parallel
- Works on Linux, Windows, and macOS
- Supports HTML5 & CSS3
- Tested on Ruby 3.2, 3.3, 3.4, and 4.0

## Playground

Try `css-inline` in the WebAssembly-powered [playground](https://css-inline.org/).

## Installation

Add this line to your application's `Gemfile` and run `bundle install`:

```ruby
gem 'css_inline'
```

Or install it directly:

```sh
gem install css_inline
```

RubyGems ships precompiled native gems for Ruby 3.2 to 4.0 on Linux (x86_64, aarch64, glibc and musl), macOS (x86_64, arm64) and Windows (x64 UCRT).
On other platforms, `gem install` builds the extension from source and needs a Rust toolchain.

## Usage

Inline CSS in an HTML document:

```ruby
require 'css_inline'

html = "<html><head><style>h1 { color:blue; }</style></head><body><h1>Big Text</h1></body></html>"
inlined = CSSInline.inline(html)

puts inlined
# <html><head></head><body><h1 style="color: blue;">Big Text</h1></body></html>
```

`css-inline` adds missing `html`, `head` and `body` tags, so the output is a valid HTML document.

To inline CSS into an HTML fragment, use `inline_fragment`. It strips structural tags (`<html>`, `<head>`, `<body>`) from the output and keeps their contents. Use `CSSInline.inline` to keep the full document structure:

```ruby
require 'css_inline'

fragment = <<~HTML
  <main>
  <h1>Hello</h1>
  <section>
  <p>who am i</p>
  </section>
  </main>
HTML

css = <<~CSS
  p {
      color: red;
  }

  h1 {
      color: blue;
  }
CSS

inlined = CSSInline.inline_fragment(fragment, css)

puts inlined
# Output:
# <main>
# <h1 style="color: blue;">Hello</h1>
# <section>
# <p style="color: red;">who am i</p>
# </section>
# </main>
```

To inline many documents in parallel, use `inline_many` and `inline_many_fragments`:

```ruby
require 'css_inline'

inlined = CSSInline.inline_many(["...", "..."])
inlined = CSSInline.inline_many_fragments(["<h1>...</h1>", "<p>...</p>"], ["h1 { color: blue; }", "p { color: red; }"])
```

`inline_many_fragments` pairs HTML and CSS by index.

**Warning**: pass arrays of equal length. If they differ, `inline_many_fragments` drops the extra items of the longer array without an error.

If one input fails, the whole call raises and returns no results.

Both functions process inputs on a Rust thread pool ([rayon](https://github.com/rayon-rs/rayon)), one thread per CPU core.
Ruby threads can't run this CPU-bound work in parallel because of the Global VM Lock, so `inline_many` gains over a Ruby loop only on a multicore machine.

## Configuration

Pass options as keyword arguments to `CSSInline::CSSInliner.new`:

```ruby
require 'css_inline'

inliner = CSSInline::CSSInliner.new(keep_style_tags: true)
inliner.inline("...")
```

A `CSSInliner` instance has the same four methods as the module: `inline`, `inline_fragment`, `inline_many` and `inline_many_fragments`.
Reuse one instance to share its stylesheet cache across calls.
The module functions accept the same keyword arguments, for example `CSSInline.inline(html, keep_style_tags: true)`.

- `inline_style_tags`. Inline CSS from `style` tags. Default: `true`
- `keep_style_tags`. Keep `style` tags after inlining. Default: `false`
- `keep_link_tags`. Keep `link` tags after inlining. Default: `false`
- `keep_at_rules`. Keep at-rules (starting with `@`) after inlining. Default: `false`
- `minify_css`. Remove trailing semicolons and spaces between properties and values. Default: `false`
- `base_url`. Base URL for resolving relative URLs. Use the `file://` scheme to load stylesheets from the filesystem. Default: `nil`
- `load_remote_stylesheets`. Load stylesheets from `link` tags, over the network or from `file://` paths. Default: `true`
- `cache`. Cache for external stylesheets, for example `CSSInline::StylesheetCache.new(size: 5)`. Default: `nil`
- `extra_css`. Extra CSS to inline. Default: `nil`
- `preallocate_node_capacity`. **Advanced**. Number of HTML nodes to preallocate during parsing. Set it to your expected node count to avoid reallocations. Default: `32`
- `remove_inlined_selectors`. Remove inlined selectors from `<style>` blocks. Default: `false`
- `apply_width_attributes`. Add `width` HTML attributes from CSS `width` properties on `table`, `td`, `th` and `img`. Default: `false`
- `apply_height_attributes`. Add `height` HTML attributes from CSS `height` properties on `table`, `td`, `th` and `img`. Default: `false`

To skip CSS inlining for an HTML tag, add the `data-css-inline="ignore"` attribute to it:

```html
<head>
  <style>h1 { color:blue; }</style>
</head>
<body>
  <!-- The tag below won't receive additional styles -->
  <h1 data-css-inline="ignore">Big Text</h1>
</body>
```

The same attribute on a `link` or `style` tag skips that stylesheet:

```html
<head>
  <!-- Styles below are ignored -->
  <style data-css-inline="ignore">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

To keep a `style` tag in the output, add the `data-css-inline="keep"` attribute.
Use it to keep `@media` queries for responsive emails in separate `style` tags.
`css-inline` keeps such tags even when `keep_style_tags` is `false`.

```html
<head>
  <!-- Styles below are not removed -->
  <style data-css-inline="keep">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

Another way is the `keep_at_rules` option. At-rules can't go into `style` attributes, so `css-inline` removes them by default.
With `keep_at_rules: true`, it inlines regular rules and keeps at-rules, such as `@media` queries, in `style` tags.
It keeps these tags even when `keep_style_tags` is `false`.

```html
<head>
  <!-- With keep_at_rules: true, "color:blue" will get inlined into <h1> but @media will be kept in <style> -->
  <style>h1 { color: blue; } @media (max-width: 600px) { h1 { font-size: 18px; } }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

With `minify_css: true`, `css-inline` removes trailing semicolons and spaces between properties and values in inlined styles.

```html
<head>
  <!-- With minify_css: true, the <h1> will have `style="color:blue;font-weight:bold"` -->
  <style>h1 { color: blue; font-weight: bold; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

To load stylesheets from the filesystem, use the `file://` scheme.
`css-inline` treats a `file://` URL without a third slash, such as `file://styles/email/`, as a path relative to the current working directory.
Standard URL parsers read `styles` there as a host, so this form is specific to `css-inline`.
For an absolute path, use three slashes: `file:///path/to/styles/`.

```ruby
require 'css_inline'

# Loads stylesheets from ./styles/email/
inliner = CSSInline::CSSInliner.new(base_url: "file://styles/email/")
inliner.inline("...")
```

To avoid repeated network requests, cache external stylesheets:

```ruby
require 'css_inline'

inliner = CSSInline::CSSInliner.new(
    cache: CSSInline::StylesheetCache.new(size: 5)
)
inliner.inline("...")
```

The cache covers every `link` stylesheet, `file://` included. `size` is the number of stylesheets to keep (default: `8`). When the cache is full, it evicts the least recently used stylesheet.
Caching is off by default.

## Errors

All functions raise `ArgumentError` when they can't load a stylesheet (missing file, network error), can't parse one, or get an invalid `base_url`:

```ruby
require 'css_inline'

begin
  CSSInline.inline('<link rel="stylesheet" href="missing.css"><h1>Hi</h1>')
rescue ArgumentError => e
  puts e.message
  # Missing stylesheet file: missing.css
end
```

## Performance

`css_inline` runs 8.1x to 354x faster than `roadie` and 57x to 123x faster than `premailer` on these documents:

|                   | Size    | `css_inline 0.22.0` | `roadie 5.2.1`          | `premailer 1.27.0`      |
|-------------------|---------|---------------------|-------------------------|-------------------------|
| Basic usage       | 230 B   | 5.84 µs             | 168.88 µs (**28.92x**)  | 331.07 µs (**56.69x**)  |
| Realistic email 1 | 8.58 KB | 87.66 µs            | 713.78 µs (**8.14x**)   | 6.95 ms (**79.28x**)    |
| Realistic email 2 | 4.30 KB | 50.45 µs            | 1.98 ms (**39.25x**)    | ERROR                   |
| GitHub Page       | 1.81 MB | 19.01 ms            | 6.73 s (**354.02x**)    | 2.34 s (**123.09x**)    |

`premailer` fails on "Realistic email 2" with `ArgumentError: Cannot parse 0 calc((100% - 500px) / 2)`.

Inputs come from [`benchmarks/benchmarks.json`](https://github.com/Stranger6667/css-inline/blob/master/benchmarks/benchmarks.json): Basic usage is `simple`, Realistic email 1 and 2 are `big_email_1` and `big_email_2`, GitHub Page is `big_page`.
The benchmark code is in [`test/bench.rb`](https://github.com/Stranger6667/css-inline/blob/master/bindings/ruby/test/bench.rb) and uses [`benchmark-ips`](https://github.com/evanphx/benchmark-ips) (5 s per case; `roadie` completes one run on the GitHub page in that time).
We measured these results with stable `rustc 1.99` on Ruby `3.4.8`, Ryzen 9 9950X.

## Further reading

- [How the Rust crate works](https://dygalo.dev/blog/rust-for-a-pythonista-2/)

## License

[MIT license](https://opensource.org/licenses/MIT).
