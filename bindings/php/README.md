# css_inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
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

- Uses HTML and CSS parsing components from Mozilla's Servo project
- 3-473x faster than `css-to-inline-styles` and `emogrifier` ([benchmarks](#performance))
- Inlines CSS from `style` and `link` tags
- Removes `style` and `link` tags
- Resolves external stylesheets (including local files)
- Caches external stylesheets (opt-in)
- Processes multiple documents in parallel
- Works on Linux and macOS (Windows is not supported)
- Supports HTML5 & CSS3

## Playground

Try `css-inline` in the WebAssembly [playground](https://css-inline.org/).

## Installation

`css_inline` ships as a PHP extension that you build from source.

Requirements:
- PHP 8.2 or higher
- Rust toolchain
- PHP development headers and `php-config` (for example, the `php-dev` package on Debian/Ubuntu)
- `libclang`, which `ext-php-rs` uses to generate PHP bindings
- Linux or macOS (the underlying `ext-php-rs` library does not support Windows)

Build the extension:

```shell
git clone https://github.com/Stranger6667/css-inline.git
cd css-inline/bindings/php
cargo build --release
```

Copy the compiled extension to your PHP extensions directory:

```shell
# Linux
cp target/release/libcss_inline_php.so $(php-config --extension-dir)/css_inline.so

# macOS
cp target/release/libcss_inline_php.dylib $(php-config --extension-dir)/css_inline.so
```

Enable the extension in your `php.ini` (`php --ini` shows its location):

```ini
extension=css_inline
```

Check that PHP loads it:

```shell
php -m | grep css_inline
```

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
//     <h1 style="color: blue;">Big Text</h1>
// </body>
// </html>
```

`CssInline\inline` adds missing `html`, `head` and `body` tags, so the output is a complete HTML document.

To inline CSS into an HTML fragment, pass the fragment and the CSS to `CssInline\inlineFragment`. It returns a fragment without adding `html`, `head` or `body` tags. If the input contains these tags, the output keeps only their contents. To get a full document, use `CssInline\inline`:

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

`CssInline\inlineMany` and `CssInline\inlineManyFragments` process several documents in parallel and return an array of results in input order:

```php
<?php

$results = CssInline\inlineMany([
    '<html><head><style>h1 { color:blue; }</style></head><body><h1>One</h1></body></html>',
    '<html><head><style>h1 { color:red; }</style></head><body><h1>Two</h1></body></html>',
]);

// The same CSS applies to every fragment
$fragments = CssInline\inlineManyFragments(['<h1>One</h1>', '<h1>Two</h1>'], 'h1 { color:blue; }');
```

Both functions run on a Rust thread pool, so the speedup depends on the number of CPU cores. If any input fails, the whole call throws `CssInline\InlineError`.

### Errors

Inlining functions and methods throw `CssInline\InlineError` (a subclass of `\Exception`) when inlining fails, for example when a stylesheet fails to load. The `CssInliner` constructor also throws `CssInline\InlineError` for an invalid `baseUrl`.

```php
<?php

use CssInline\InlineError;

try {
    $inlined = CssInline\inline($html);
} catch (InlineError $e) {
    echo $e->getMessage();
}
```

### Configuration

Pass options to the `CssInliner` constructor as named arguments. It has the same methods as the functions above: `inline`, `inlineFragment`, `inlineMany` and `inlineManyFragments`.

```php
<?php

use CssInline\CssInliner;

$inliner = new CssInliner(keepStyleTags: true);
$inliner->inline($html);
```

- `inlineStyleTags`. Inline CSS from `style` tags. Default: `true`
- `keepStyleTags`. Keep `style` tags after inlining. Default: `false`
- `keepLinkTags`. Keep `link` tags after inlining. Default: `false`
- `keepAtRules`. Keep at-rules (rules starting with `@`) after inlining. Default: `false`
- `minifyCss`. Remove trailing semicolons and spaces between properties and values. Default: `false`
- `baseUrl`. Base URL for resolving relative URLs. Use the `file://` scheme to load stylesheets from the filesystem. Default: `null`
- `loadRemoteStylesheets`. Load stylesheets from `link` tags, over the network or from `file://` paths. Default: `true`
- `cache`. Cache for external stylesheets, for example `new CssInline\StylesheetCache(size: 5)`. Default: `null`
- `extraCss`. Extra CSS to inline. Default: `null`
- `preallocateNodeCapacity`. **Advanced**. Number of HTML nodes to preallocate during parsing. Set it near your document's node count to avoid reallocations. Default: `32`
- `removeInlinedSelectors`. Remove selectors from `<style>` blocks once inlined. Default: `false`
- `applyWidthAttributes`. Add `width` HTML attributes from CSS `width` properties on `table`, `td`, `th` and `img`. Default: `false`
- `applyHeightAttributes`. Add `height` HTML attributes from CSS `height` properties on `table`, `td`, `th` and `img`. Default: `false`

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

The same attribute on a `link` or `style` tag makes `css_inline` skip its CSS:

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
The tag stays even when `keepStyleTags` is `false`.

```html
<head>
  <!-- Styles below are not removed -->
  <style data-css-inline="keep">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

Another option is `keepAtRules: true`. A `style` attribute cannot hold at-rules, so `css_inline` removes them by default.
With `keepAtRules: true`, it inlines regular rules and keeps at-rules, such as `@media` queries, in `style` tags.
These tags stay even when `keepStyleTags` is `false`.

```html
<head>
  <!-- With keepAtRules: true, "color: blue" goes into the <h1> style attribute and @media stays in <style> -->
  <style>h1 { color: blue; } @media (max-width: 600px) { h1 { font-size: 18px; } }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

With `minifyCss: true`, `css_inline` removes trailing semicolons and spaces between properties and values in inlined styles.

```html
<head>
  <!-- With minifyCss: true, the <h1> gets `style="color:blue;font-weight:bold"` -->
  <style>h1 { color: blue; font-weight: bold; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

To load stylesheets from the filesystem, use the `file://` scheme in `baseUrl`.
An absolute directory takes three slashes: `file:///var/www/styles/`.
`css_inline` also accepts two slashes for a path relative to the working directory of the PHP process.
Standard URL parsing would read `styles` in `file://styles/email/` as a host name. `css_inline` treats it as the relative path `styles/email/` on purpose:

```php
<?php

use CssInline\CssInliner;

// Absolute path
$inliner = new CssInliner(baseUrl: "file:///var/www/styles/email/");

// Relative to the working directory (non-standard)
$inliner = new CssInliner(baseUrl: "file://styles/email/");
$inliner->inline($html);
```

To avoid repeated network requests, cache external stylesheets. `size` sets the maximum number of cached stylesheets. A size of zero throws `\Exception`. When the cache is full, it evicts the least recently used stylesheet:

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

`css_inline` is 3-473x faster than `css-to-inline-styles` and `emogrifier` on the inputs below:

|                   | Size    | `css_inline 0.22.0` | `css-to-inline-styles 2.4.0` | `emogrifier 8.2.0`     |
|-------------------|---------|---------------------|------------------------------|------------------------|
| Simple            | 230 B   | 5.27 µs             | 27.81 µs (**5.28x**)         | 155.93 µs (**29.59x**) |
| Realistic email 1 | 8.58 KB | 88.10 µs            | 287.67 µs (**3.27x**)        | 638.54 µs (**7.25x**)  |
| Realistic email 2 | 4.30 KB | 52.90 µs            | 613.71 µs (**11.60x**)       | 2.44 ms (**46.06x**)   |
| GitHub Page       | 1.81 MB | 17.91 ms            | skipped†                     | 8.47 s (**472.92x**)‡  |

† `css-to-inline-styles` returns the GitHub page without inlining any styles.
‡ Timed outside phpbench: median of 5 runs. At 10 iterations of 100 revolutions, `composer bench` would spend about 2.4 hours on this cell, so it skips it.

The benchmark code lives in [`bindings/php/benchmarks/InlineBench.php`](https://github.com/Stranger6667/css-inline/blob/master/bindings/php/benchmarks/InlineBench.php). It reads the `simple`, `big_email_1`, `big_email_2` and `big_page` inputs from [`benchmarks/benchmarks.json`](https://github.com/Stranger6667/css-inline/blob/master/benchmarks/benchmarks.json).
To reproduce, build the extension, then run `composer install` and `composer bench` in `bindings/php` (phpbench, 10 iterations of 100 revolutions each; the table shows the median iteration).
Measured with stable `rustc 1.99` on PHP `8.5.6`, Ryzen 9 9950X.

## Further reading

These articles explain how this library was built and how it works:

- [Rust crate](https://dygalo.dev/blog/rust-for-a-pythonista-2/)
- [Python bindings](https://dygalo.dev/blog/rust-for-a-pythonista-3/)

## License

`css_inline` uses the [MIT license](https://opensource.org/licenses/MIT).
