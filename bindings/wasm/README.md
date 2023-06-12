# css-inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline)
[<img alt="npm" src="https://img.shields.io/npm/v/css-inline?style=flat-square" height="20">](https://github.com/Stranger6667/css-inline/actions?query=branch%3Amaster)
[<img alt="codecov.io" src="https://img.shields.io/codecov/c/gh/Stranger6667/css-inline?logo=codecov&style=flat-square&token=tOzvV4kDY0" height="20">](https://app.codecov.io/github/Stranger6667/css-inline)
[<img alt="gitter" src="https://img.shields.io/gitter/room/Stranger6667/css-inline?style=flat-square" height="20">](https://gitter.im/Stranger6667/css-inline)

`css-inline` is a library that inlines CSS into HTML documents, built using components from Mozilla's Servo project.

This process is essential for sending HTML emails as you need to use "style" attributes instead of "style" tags.

For instance, the library transforms HTML like this:

```html
<html>
    <head>
        <title>Test</title>
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
    <head>
        <title>Test</title>
    </head>
    <body>
        <h1 style="color:blue;">Big Text</h1>
    </body>
</html>
```

- Removing ``style`` tags after inlining;
- Out-of-document CSS to inline;

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
  { keep_style_tags: true }
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

- `keep_style_tags`. Specifies whether to keep "style" tags after inlining. Default: `false`
- `base_url`. The base URL used to resolve relative URLs. If you'd like to load stylesheets from your filesystem, use the `file://` scheme. Default: `null`
- `load_remote_stylesheets`. Specifies whether remote stylesheets should be loaded. Default: `true`
- `extra_css`. Extra CSS to be inlined. Default: `null`
- `preallocate_node_capacity`. **Advanced**. Preallocates capacity for HTML nodes during parsing. This can improve performance when you have an estimate of the number of nodes in your HTML document. Default: `8`

You can also skip CSS inlining for an HTML tag by adding the `data-css-inline="ignore"` attribute to it:

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

The `data-css-inline="ignore"` attribute also allows you to skip `link` and `style` tags:

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

`css-inline` is built on top of [html5ever](https://crates.io/crates/html5ever) and [cssparser](https://crates.io/crates/cssparser) and relies on their behavior for HTML & CSS parsing.

- Only HTML 5 is supported, not XHTML.
- Only CSS 3 is supported.
- Only UTF-8 encoding for string representation. Other document encodings are not yet supported.

## License

This project is licensed under the terms of the [MIT license](https://opensource.org/licenses/MIT).
