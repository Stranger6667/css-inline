# css-inline

[![ci](https://github.com/Stranger6667/css-inline/workflows/ci/badge.svg)](https://github.com/Stranger6667/css-inline/actions)

Blazing-fast WASM package for inlining CSS into HTML documents.

Features:

- Removing ``style`` tags after inlining;
- Resolving external stylesheets (including local files);
- Control if ``style`` tags should be processed;
- Additional CSS to inline;
- Inlining multiple documents in parallel (via Rust-level threads)

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
