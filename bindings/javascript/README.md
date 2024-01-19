# css-inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
[<img alt="npm" src="https://img.shields.io/npm/v/@css-inline/css-inline.svg?style=flat-square" height="20">](https://www.npmjs.com/package/@css-inline/css-inline)
[<img alt="codecov.io" src="https://img.shields.io/codecov/c/gh/Stranger6667/css-inline?logo=codecov&style=flat-square&token=tOzvV4kDY0" height="20">](https://app.codecov.io/github/Stranger6667/css-inline)
[<img alt="gitter" src="https://img.shields.io/gitter/room/Stranger6667/css-inline?style=flat-square" height="20">](https://gitter.im/Stranger6667/css-inline)

`css-inline` is a high-performance library for inlining CSS into HTML 'style' attributes.

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
- Works on Linux, Windows, and macOS
- Supports HTML5 & CSS3
- Tested on Node.js 18 & 20.

## Playground

If you'd like to try `css-inline`, you can check the WebAssembly-powered [playground](https://css-inline.org/) to see the results instantly.

## Installation

### Node.js

Install with `npm`:

```shell
npm i @css-inline/css-inline
```

## Usage

```typescript
import { inline } from "@css-inline/css-inline";

var inlined = inline(
  `
  <html>
    <head>
      <style>h1 { color:red }</style>
    </head>
    <body>
      <h1>Test</h1>
    </body>
  </html>
  `,
);
// Do something with the inlined HTML, e.g. send an email
```

### Configuration

- `inlineStyleTags`. Specifies whether to inline CSS from "style" tags. Default: `true`
- `keepStyleTags`. Specifies whether to keep "style" tags after inlining. Default: `false`
- `keepLinkTags`. Specifies whether to keep "link" tags after inlining. Default: `false`
- `baseUrl`. The base URL used to resolve relative URLs. If you'd like to load stylesheets from your filesystem, use the `file://` scheme. Default: `null`
- `loadRemoteStylesheets`. Specifies whether remote stylesheets should be loaded. Default: `true`
- `extraCss`. Extra CSS to be inlined. Default: `null`
- `preallocateNodeCapacity`. **Advanced**. Preallocates capacity for HTML nodes during parsing. This can improve performance when you have an estimate of the number of nodes in your HTML document. Default: `32`

You can also skip CSS inlining for an HTML tag by adding the `data-css-inline="ignore"` attribute to it:

```html
<head>
    <style>h1 { color:blue; }</style>
</head>
<body>
    <!-- The tag below won't receive additional styles -->
    <h1 data-css-inline="ignore">Big Text</h1>
</body>
</html>
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

## WebAssembly

`css-inline` also ships a WebAssembly module built with `wasm-bindgen` to run in browsers.

```html
<script src="https://unpkg.com/@css-inline/css-inline-wasm"></script>
<script>
    // Initialize the WASM module first
    cssInline.initWasm(fetch('https://unpkg.com/@css-inline/css-inline-wasm/index_bg.wasm'));

    const inlinedHtml = cssInline.inline(`<html>
  <head>
    <style>h1 { color:blue; }</style>
  </head>
  <body>
    <h1>Big Text</h1>
  </body>
</html>`);

    document.getElementById('output').src = inlinedHtml
</script>
```

**NOTE**: WASM module currently lacks support for fetching stylesheets from network or filesystem.

## Performance

`css-inline` is powered by efficient tooling from Mozilla's Servo project and significantly outperforms other JavaScript alternatives in terms of speed.
Most of the time it achieves over a **3x** speed advantage compared to the next fastest alternative.

Here is the performance comparison:

|             | Size    | `css-inline 0.13.0` | `css-inline-wasm 0.13.0` | `juice 10.0.0`       | `inline-css 4.0.2`   |
|-------------|---------|---------------------|--------------------------|----------------------|----------------------|
| Basic       | 230 B   | 15.34 µs            | 19.99 µs (**1.30x**)     | 56.70 µs (**3.69x**) | 67.95 µs (**4.42x**) |
| Realistic-1 | 8.58 KB | 333.22 µs           | 434.59 µs (**1.30x**)    | 1.07 ms (**3.23x**)  | 1.24 ms (**3.73x**)  |
| Realistic-2 | 4.3 KB  | 195.80 µs           | 261.09 µs (**1.33x**)    | 0.99 ms (**5.10x**)  | 0.76 ms (**3.90x**)  |

The "Basic" case was obtained from benchmarking the example from the Usage section.

The benchmarking code is available in the `benches/bench.ts` file. The benchmarks were conducted using the stable `rustc 1.75.0` on Node.js `v21.1.0`.

## License

This project is licensed under the terms of the [MIT license](https://opensource.org/licenses/MIT).
