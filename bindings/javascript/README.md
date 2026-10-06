# css-inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
[<img alt="npm" src="https://img.shields.io/npm/v/@css-inline/css-inline.svg?style=flat-square" height="20">](https://www.npmjs.com/package/@css-inline/css-inline)
[<img alt="codecov.io" src="https://img.shields.io/codecov/c/gh/Stranger6667/css-inline?logo=codecov&style=flat-square&token=tOzvV4kDY0" height="20">](https://app.codecov.io/github/Stranger6667/css-inline)
[<img alt="gitter" src="https://img.shields.io/gitter/room/Stranger6667/css-inline?style=flat-square" height="20">](https://gitter.im/Stranger6667/css-inline)

`css-inline` inlines CSS into HTML `style` attributes. Use it to prepare HTML emails or to embed HTML into third-party web pages.

It turns this HTML:

```html
<html><head><style>h1 { color:blue; }</style></head><body><h1>Big Text</h1></body></html>
```

into:

```html
<html><head></head><body><h1 style="color: blue;">Big Text</h1></body></html>
```

`css-inline` keeps whitespace text nodes from the input, so indented HTML stays indented in the output.

- Builds on Mozilla Servo components (`html5ever`, `cssparser`, `selectors`)
- Inlines CSS from `style` and `link` tags
- Removes `style` and `link` tags
- Resolves external stylesheets (including local files)
- Optionally caches external stylesheets
- Works on Linux, Windows, macOS and Android
- Supports HTML5 & CSS3
- Tested on Node.js 20 & 22

## Playground

Try `css-inline` in the WebAssembly-powered [playground](https://css-inline.org/).

## Installation

### Node.js

Install with `npm`:

```shell
npm i @css-inline/css-inline
```

The package ships TypeScript types (`index.d.ts`).

npm installs a prebuilt binary for your platform, so you don't need Rust. Prebuilt targets:
`linux-x64-gnu`, `linux-x64-musl`, `linux-arm64-gnu`, `linux-arm64-musl`, `linux-arm-gnueabihf`,
`darwin-x64`, `darwin-arm64`, `win32-x64-msvc`, `win32-arm64-msvc`, `android-arm64`, `android-arm-eabi`.

## Usage

```javascript
import { inline } from "@css-inline/css-inline";
// CommonJS: const { inline } = require("@css-inline/css-inline");

const inlined = inline(
  "<html><head><style>h1 { color:red }</style></head><body><h1>Test</h1></body></html>",
);
// <html><head></head><body><h1 style="color: red;">Test</h1></body></html>
```

`inline` adds missing `html`, `head` and `body` tags, so the output is a complete HTML document.

`inlineFragment(html, css, options?)` inlines the `css` string into an HTML fragment. It never adds `<html>`, `<head>` or `<body>`; if the input has them, it keeps only their contents. Use `inline` to keep the full document structure:

```javascript
import { inlineFragment } from "@css-inline/css-inline";

const inlined = inlineFragment(
  "<main><h1>Hello</h1><section><p>who am i</p></section></main>",
  "p { color: red; } h1 { color: blue; }",
);
// <main><h1 style="color: blue;">Hello</h1><section><p style="color: red;">who am i</p></section></main>
```

### Configuration

Pass options as the second argument to `inline` or the third to `inlineFragment`.

- `inlineStyleTags`. Inline CSS from `style` tags. Default: `true`
- `keepStyleTags`. Keep `style` tags after inlining. Default: `false`
- `keepLinkTags`. Keep `link` tags after inlining. Default: `false`
- `keepAtRules`. Keep at-rules (`@media`, `@font-face`, ...) after inlining. Default: `false`
- `minifyCss`. Remove trailing semicolons and spaces between properties and values. Default: `false`
- `baseUrl`. Base URL for resolving relative URLs. Use the `file://` scheme to load stylesheets from the filesystem; end directory URLs with `/` (`file:///path/to/styles/`), or `css-inline` resolves relative to the parent directory. Default: `null`
- `loadRemoteStylesheets`. Load stylesheets from `link` tags, over the network or from `file://` paths. `false` skips every `link` tag. Default: `true`
- `cache`. LRU cache for external stylesheets, keyed by resolved URL. `size` is the number of stylesheets to keep and must be greater than zero, e.g. `{ size: 5 }`. Default: `null`
- `extraCss`. Extra CSS to inline. Default: `null`
- `preallocateNodeCapacity`. **Advanced**. Number of HTML nodes to preallocate during parsing. Set it to your expected node count to avoid reallocations. Default: `32`
- `removeInlinedSelectors`. Remove selectors that were inlined from `<style>` blocks. Default: `false`
- `applyWidthAttributes`. Add `width` HTML attributes from CSS `width` properties on `table`, `td`, `th` and `img`. Default: `false`
- `applyHeightAttributes`. Add `height` HTML attributes from CSS `height` properties on `table`, `td`, `th` and `img`. Default: `false`

To skip inlining for a tag, add the `data-css-inline="ignore"` attribute:

```html
<html>
<head>
    <style>h1 { color:blue; }</style>
</head>
<body>
    <!-- The tag below won't receive additional styles -->
    <h1 data-css-inline="ignore">Big Text</h1>
</body>
</html>
```

The same attribute on a `link` or `style` tag excludes its styles:

```html
<head>
  <!-- Styles below are ignored -->
  <style data-css-inline="ignore">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

To keep a `style` tag in the output, add `data-css-inline="keep"`. Use it to keep `@media` queries for responsive emails in separate `style` tags.
`css-inline` keeps such tags even when `keepStyleTags` is `false`.

```html
<head>
  <!-- Styles below are not removed -->
  <style data-css-inline="keep">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

At-rules cannot be inlined into HTML, so `css-inline` removes them by default.
Set `keepAtRules: true` to keep them in `style` tags and inline the remaining styles.
`css-inline` keeps these tags even when `keepStyleTags` is `false`.

```html
<head>
  <!-- With keepAtRules: true, "color: blue" goes into <h1> and @media stays in <style> -->
  <style>h1 { color: blue; } @media (max-width: 600px) { h1 { font-size: 18px; } }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

With `minifyCss: true`, `css-inline` removes trailing semicolons and spaces between properties and values in inlined styles.

```html
<head>
  <!-- With minifyCss: true, <h1> gets style="color:blue;font-weight:bold" -->
  <style>h1 { color: blue; font-weight: bold; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

Caching is off by default. Enable it to avoid fetching the same external stylesheet on every call:

```javascript
import { inline } from "@css-inline/css-inline";

const inlined = inline(
  `
  <html>
    <head>
      <link href="http://127.0.0.1:1234/external.css" rel="stylesheet">
      <style>h1 { color:red }</style>
    </head>
    <body>
      <h1>Test</h1>
    </body>
  </html>
  `,
  { cache: { size: 5 } },
);
```

`inline` and `inlineFragment` are synchronous. `css-inline` fetches remote stylesheets with a blocking HTTP request, so the call blocks the event loop until the fetch completes.
Set `loadRemoteStylesheets: false` to skip all `link` tags.

### Errors

`inline` and `inlineFragment` throw an `Error`:

- Invalid `baseUrl`: `relative URL without a base: not a url`
- Failed fetch: `error sending request for url (http://127.0.0.1:9/a.css): http://127.0.0.1:9/a.css`
- Missing local file: `Missing stylesheet file: /path/to/missing.css`
- `cache: { size: 0 }`: `Cache size must be an integer greater than zero`

## WebAssembly

`@css-inline/css-inline-wasm` runs `css-inline` in browsers. It is built with `wasm-bindgen`.

```html
<iframe id="output"></iframe>
<script src="https://unpkg.com/@css-inline/css-inline-wasm"></script>
<script>
  // `initWasm` is async: wait for it before calling `inline`
  cssInline.initWasm(fetch('https://unpkg.com/@css-inline/css-inline-wasm/index_bg.wasm')).then(() => {
    const inlinedHtml = cssInline.inline(`<html>
  <head>
    <style>h1 { color:blue; }</style>
  </head>
  <body>
    <h1>Big Text</h1>
  </body>
</html>`);
    document.getElementById('output').srcdoc = inlinedHtml;
  });
</script>
```

**NOTE**: The WASM module cannot load stylesheets from the network or filesystem and has no `cache` option. See [its README](https://github.com/Stranger6667/css-inline/blob/master/bindings/javascript/wasm/README.md) for npm install, Node.js setup and `link` tag handling. The unpkg URLs above load the latest release; pin a version (`@css-inline/css-inline-wasm@0.22.0`) to match your installed package.

## Performance

Native `css-inline` runs 3.15x to 10.33x faster than `juice` and `inline-css` on the cases below. The WASM build runs 1.37x to 1.68x slower than native.

|             | Size    | `css-inline`| `css-inline-wasm`    | `juice`                 | `inline-css`            |
|-------------|---------|-------------|----------------------|-------------------------|-------------------------|
| Basic       | 230 B   | 8.42 µs     | 12.96 µs (**1.54x**) | 41.05 µs (**4.88x**)    | 78.16 µs (**9.28x**)    |
| Realistic-1 | 8.58 KB | 161.13 µs   | 270.56 µs (**1.68x**)| 508.13 µs (**3.15x**)   | 1.07 ms (**6.62x**)     |
| Realistic-2 | 4.30 KB | 85.15 µs    | 133.48 µs (**1.57x**)| 605.33 µs (**7.11x**)   | 839.63 µs (**9.86x**)   |
| GitHub page | 1.81 MB | 31.04 ms    | 42.61 ms (**1.37x**) | 320.68 ms (**10.33x**)  | ERROR                   |

`inline-css` fails on the GitHub page because it cannot parse its CSS: `Error: Unexpected } (line 193, char 13771)`.

Inputs come from [`benchmarks/benchmarks.json`](https://github.com/Stranger6667/css-inline/blob/master/benchmarks/benchmarks.json): "Basic" is `simple`, "Realistic-1" and "Realistic-2" are `big_email_1` and `big_email_2`, "GitHub page" is `big_page`.
Ratios are relative to `css-inline`.

[`benches/bench.ts`](https://github.com/Stranger6667/css-inline/blob/master/bindings/javascript/benches/bench.ts) runs the suite with `benny` (mean time; `css-inline 0.22.0`, `juice 11.0.3`, `inline-css 4.0.3`), built with stable `rustc 1.99` on Node.js `v24.18.0`, Ryzen 9 9950X.

## License

Licensed under the [MIT license](https://opensource.org/licenses/MIT).
