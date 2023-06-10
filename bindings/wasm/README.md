# css-inline

[![ci](https://github.com/Stranger6667/css-inline/workflows/ci/badge.svg)](https://github.com/Stranger6667/css-inline/actions)
[![npm version](https://badge.fury.io/js/css-inline.svg)](https://badge.fury.io/js/css-inline)

Blazing-fast WASM package for inlining CSS into HTML documents.

Features:

- Removing ``style`` tags after inlining;
- Control if ``style`` tags should be processed;
- Out-of-document CSS to inline;

The project supports CSS Syntax Level 3 implemented with Mozilla's Servo project components.

## Usage

```typescript
import { inline } from "css-inline";

var inlined = inline(
  `
  <html>
    <head>
      <title>Test</title>
      <style>h1 { color:red; }</style>
    </head>
    <body>
      <h1>Test</h1>
    </body>
  </html>
  `,
  { remove_style_tags: false }
)
// Inlined HTML looks like this:
// <html>
//   <head>
//     <title>Test</title>
//     <style>h1 { color:red; }</style>
//   </head>
//   <body>
//     <h1 style="color:red;">Test</h1>
//   </body>
// </html>
// Do something with the inlined HTML, e.g. send an email
```

- `inline_style_tags`. Whether to inline CSS from "style" tags. Default: `true`
- `remove_style_tags`. Remove "style" tags after inlining. Default: `true`
- `base_url`. Base URL to resolve relative URLs. If you'd like to load stylesheets from your filesystem, use the `file://` scheme. Default: `null`
- `load_remote_stylesheets`. Whether remote stylesheets should be loaded or not. Default: `true`
- `extra_css`. Additional CSS to inline. Default: `null`
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

## Standards support & restrictions

`css-inline` is built on top of [cssparser](https://crates.io/crates/cssparser) and relies on its behavior for CSS parsing.
Notably:

- Only HTML 5, XHTML is not supported;
- Only CSS 3;
- Only UTF-8 for string representation. Other document encodings are not yet supported.
