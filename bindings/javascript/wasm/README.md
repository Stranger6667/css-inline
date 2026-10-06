# css-inline

## @css-inline/css-inline-wasm

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
[<img alt="npm" src="https://img.shields.io/npm/v/@css-inline/css-inline-wasm.svg?style=flat-square" height="20">](https://www.npmjs.com/package/@css-inline/css-inline-wasm)
[<img alt="codecov.io" src="https://img.shields.io/codecov/c/gh/Stranger6667/css-inline?logo=codecov&style=flat-square&token=tOzvV4kDY0" height="20">](https://app.codecov.io/github/Stranger6667/css-inline)
[<img alt="gitter" src="https://img.shields.io/gitter/room/Stranger6667/css-inline?style=flat-square" height="20">](https://gitter.im/Stranger6667/css-inline)

`css-inline` inlines CSS into HTML `style` attributes. This package is its **WebAssembly** build for browsers and bundlers.

## Installation

```shell
npm i @css-inline/css-inline-wasm
```

The package ships TypeScript types (`index.d.ts`).

## Usage

With a bundler or ESM:

```javascript
import { initWasm, inline, inlineFragment } from "@css-inline/css-inline-wasm";

// `initWasm` is async and runs once: wait for it before calling `inline`
await initWasm(fetch("https://unpkg.com/@css-inline/css-inline-wasm@0.22.0/index_bg.wasm"));

const html = inline("<html><head><style>h1 { color:blue; }</style></head><body><h1>Big Text</h1></body></html>");
// <html><head></head><body><h1 style="color: blue;">Big Text</h1></body></html>

const fragment = inlineFragment("<h1>Big Text</h1>", "h1 { color: red; }");
// <h1 style="color: red;">Big Text</h1>
```

In Node.js, pass the file contents:

```javascript
import { readFile } from "node:fs/promises";
import { initWasm, inline } from "@css-inline/css-inline-wasm";

await initWasm(readFile(new URL(import.meta.resolve("@css-inline/css-inline-wasm/index_bg.wasm"))));
```

From a CDN, the script defines a global `cssInline`. Pin the version in both URLs to the release you want; unpinned URLs load the latest:

```html
<iframe id="output"></iframe>
<script src="https://unpkg.com/@css-inline/css-inline-wasm@0.22.0"></script>
<script>
  cssInline.initWasm(fetch("https://unpkg.com/@css-inline/css-inline-wasm@0.22.0/index_bg.wasm")).then(() => {
    const inlinedHtml = cssInline.inline("<html><head><style>h1 { color:blue; }</style></head><body><h1>Big Text</h1></body></html>");
    document.getElementById("output").srcdoc = inlinedHtml;
  });
</script>
```

## API

- `initWasm(moduleOrPath)`: load the `.wasm` module. Accepts a URL, `Response`, `BufferSource`, `WebAssembly.Module` or a promise of one.
- `inline(html, options?)`
- `inlineFragment(html, css, options?)`: `css` is a CSS string.
- `version()`

Options match the [Node.js package](https://github.com/Stranger6667/css-inline/tree/master/bindings/javascript#configuration) except `cache`, which WASM lacks:
`inlineStyleTags`, `keepStyleTags`, `keepLinkTags`, `keepAtRules`, `minifyCss`, `baseUrl`, `loadRemoteStylesheets`, `extraCss`, `preallocateNodeCapacity`, `removeInlinedSelectors`, `applyWidthAttributes`, `applyHeightAttributes`.

## Playground

Try `css-inline` in the WebAssembly-powered [playground](https://css-inline.org/).

## Restrictions

The WASM module cannot load stylesheets from the network or filesystem.
`inline` and `inlineFragment` throw a string (not an `Error`) for every `link` stylesheet:

- Remote URL: `Loading remote stylesheets is not supported on WASM: <url>`
- Relative `href` (with or without `baseUrl`) or `file://`: `Loading local files is not supported on WASM: <url>`

To avoid the error, either:

- pass `loadRemoteStylesheets: false` to skip every `link` tag, remote and local, or
- add `data-css-inline="ignore"` to the `link` tag.

Add the skipped CSS through `extraCss` if you need it.

Other errors are strings too, e.g. an invalid `baseUrl` throws `relative URL without a base: <value>`.

## License

Licensed under the [MIT license](https://opensource.org/licenses/MIT).
