# css_inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
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
- 3-25x faster than alternatives
- Inlines CSS from `style` and `link` tags
- Removes `style` and `link` tags
- Resolves external stylesheets (including local files)
- Optionally caches external stylesheets
- Can process multiple documents in parallel
- Works on Linux and macOS (Windows is not supported)
- Supports HTML5 & CSS3

## Playground

If you'd like to try `css-inline`, you can check the WebAssembly-powered [playground](https://css-inline.org/) to see the results instantly.

## Installation

`css_inline` is distributed as a PHP extension. You'll need to compile it from source:

```shell
git clone https://github.com/Stranger6667/css-inline.git
cd css-inline/bindings/php
cargo build --release
```

Then copy the compiled extension to your PHP extensions directory:

```shell
# Linux
cp target/release/libcss_inline_php.so $(php-config --extension-dir)/css_inline.so

# macOS
cp target/release/libcss_inline_php.dylib $(php-config --extension-dir)/css_inline.so
```

Enable the extension in your `php.ini`:

```ini
extension=css_inline
```

Requirements:
- PHP 8.2 or higher
- Rust toolchain (for building from source)
- Linux or macOS (Windows is not supported by the underlying `ext-php-rs` library)

## Usage

```php
<?php

$html = <<<HTML
<html>
<head>
    <style>h1 { color:blue; }</style>
</head>
<body>
    <h1>Big Text</h1>
</body>
</html>
HTML;

$inlined = CssInline\inline($html);
// HTML becomes:
// <html>
// <head></head>
// <body>
//     <h1 style="color:blue;">Big Text</h1>
// </body>
// </html>
```

Note that `css_inline` automatically adds missing `html` and `body` tags, so the output is a valid HTML document.

Alternatively, you can inline CSS into an HTML fragment, preserving the original structure:

```php
<?php

$fragment = <<<HTML
<main>
<h1>Hello</h1>
<section>
<p>who am i</p>
</section>
</main>
HTML;

$css = <<<CSS
p {
    color: red;
}
h1 {
    color: blue;
}
CSS;

$inlined = CssInline\inlineFragment($fragment, $css);
// HTML becomes:
// <main>
// <h1 style="color: blue;">Hello</h1>
// <section>
// <p style="color: red;">who am i</p>
// </section>
// </main>
```

When there is a need to inline multiple HTML documents simultaneously, `css_inline` offers `inlineMany` and `inlineManyFragments` functions.
This feature allows for concurrent processing of several inputs, significantly improving performance when dealing with a large number of documents.

```php
<?php

$results = CssInline\inlineMany([$html1, $html2, $html3]);
```

Under the hood, `inlineMany` spawns threads at the Rust layer to handle the parallel processing of inputs.

**Note**: To fully benefit from `inlineMany`, you should run your application on a multicore machine.

### Configuration

For configuration options use the `CssInliner` class:

```php
<?php

use CssInline\CssInliner;

$inliner = new CssInliner(keepStyleTags: true);
$inliner->inline($html);
```

- `inlineStyleTags`. Specifies whether to inline CSS from "style" tags. Default: `true`
- `keepStyleTags`. Specifies whether to keep "style" tags after inlining. Default: `false`
- `keepLinkTags`. Specifies whether to keep "link" tags after inlining. Default: `false`
- `keepAtRules`. Specifies whether to keep "at-rules" (starting with `@`) after inlining. Default: `false`
- `minifyCss`. Specifies whether to remove trailing semicolons and spaces between properties and values. Default: `false`
- `baseUrl`. The base URL used to resolve relative URLs. If you'd like to load stylesheets from your filesystem, use the `file://` scheme. Default: `null`
- `loadRemoteStylesheets`. Specifies whether remote stylesheets should be loaded. Default: `true`
- `cache`. Specifies caching options for external stylesheets (for example, `new StylesheetCache(size: 5)`). Default: `null`
- `extraCss`. Extra CSS to be inlined. Default: `null`
- `preallocateNodeCapacity`. **Advanced**. Preallocates capacity for HTML nodes during parsing. This can improve performance when you have an estimate of the number of nodes in your HTML document. Default: `32`
- `removeInlinedSelectors`. Specifies whether to remove selectors that were successfully inlined from `<style>` blocks. Default: `false`

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
Such tags will be kept in the resulting HTML even if the `keepStyleTags` option is set to `false`.

```html
<head>
  <!-- Styles below are not removed -->
  <style data-css-inline="keep">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

Another possibility is to set `keepAtRules` option to `true`. At-rules cannot be inlined into HTML therefore they
get removed by default. This is useful if you want to keep at-rules, e.g. `@media` queries for responsive emails in
separate `style` tags but inline any styles which can be inlined.
Such tags will be kept in the resulting HTML even if the `keepStyleTags` option is explicitly set to `false`.

```html
<head>
  <!-- With keepAtRules=true "color:blue" will get inlined into <h1> but @media will be kept in <style> -->
  <style>h1 { color: blue; } @media (max-width: 600px) { h1 { font-size: 18px; } }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

If you set the `minifyCss` option to `true`, the inlined styles will be minified by removing trailing semicolons
and spaces between properties and values.

```html
<head>
  <!-- With minifyCss=true, the <h1> will have `style="color:blue;font-weight:bold"` -->
  <style>h1 { color: blue; font-weight: bold; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

If you'd like to load stylesheets from your filesystem, use the `file://` scheme:

```php
<?php

use CssInline\CssInliner;

// styles/email is relative to the current directory
$inliner = new CssInliner(baseUrl: "file://styles/email/");
$inliner->inline($html);
```

You can also cache external stylesheets to avoid excessive network requests:

```php
<?php

use CssInline\CssInliner;
use CssInline\StylesheetCache;

$inliner = new CssInliner(
    cache: new StylesheetCache(size: 5)
);
$inliner->inline($html);
```

Caching is disabled by default.

## Performance

`css_inline` is powered by efficient tooling from Mozilla's Servo project and significantly outperforms other PHP alternatives in terms of speed.

Here is the performance comparison:

|                   | Size    | `css_inline 0.19.0` | `css-to-inline-styles 2.3.0` | `emogrifier 7.3.0`     |
|-------------------|---------|---------------------|------------------------------|------------------------|
| Simple            | 230 B   | 5.69 µs             | 26.22 µs (**4.61x**)         | 134.37 µs (**23.61x**) |
| Realistic email 1 | 8.58 KB | 94.07 µs            | 288.20 µs (**3.06x**)        | 588.00 µs (**6.25x**)  |
| Realistic email 2 | 4.3 KB  | 58.15 µs            | 585.24 µs (**10.07x**)       | 2.24 ms (**38.58x**)   |
| GitHub Page†      | 1.81 MB | 37.72 ms            | ERROR                        | ERROR                  |

† The GitHub page benchmark contains complex modern CSS that neither `css-to-inline-styles` nor `emogrifier` can process.

Please refer to the `benchmarks/InlineBench.php` file to review the benchmark code.
The results displayed above were measured using stable `rustc 1.91` on PHP `8.4.14`.

## Further reading

If you want to know how this library was created & how it works internally, you could take a look at these articles:

- [Rust crate](https://dygalo.dev/blog/rust-for-a-pythonista-2/)
- [Python bindings](https://dygalo.dev/blog/rust-for-a-pythonista-3/)

## License

This project is licensed under the terms of the [MIT license](https://opensource.org/licenses/MIT).
