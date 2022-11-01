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
  { remove_style_tags: true }
)
// Inlined HTML looks like this:
// <html>
//   <head>
//     <title>Test</title>
//   </head>
//   <body>
//     <h1 style="color:red;">Test</h1>
//   </body>
// </html>
// Do something with the inlined HTML, e.g. send an email
```

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
