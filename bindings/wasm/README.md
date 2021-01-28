# css-inline

[![ci](https://github.com/Stranger6667/css-inline/workflows/ci/badge.svg)](https://github.com/Stranger6667/css-inline/actions)

A WASM package for inlining CSS into HTML documents.

```typescript
import { inline } from "css-inline";

var inlined = inline(
  "<html><head><title>Test</title><style>h1 { color:red; }</style></head><body><h1>Test</h1></body></html>",
  { remove_style_tags: true }
)
// Do something with the inlined HTML, e.g. send an email
```
