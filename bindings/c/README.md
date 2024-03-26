# css-inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
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
- Optionally caches external stylesheets
- Works on Linux, Windows, and macOS
- Supports HTML5 & CSS3

## Playground

If you'd like to try `css-inline`, you can check the WebAssembly-powered [playground](https://css-inline.org/) to see the results instantly.

## Install

The C bindings are distributed as a header (`css_inline.h`) along with a dynamic library (`libcss_inline.so`).

To download them, go to [Releases](https://github.com/Stranger6667/css-inline/releases) and get the latest archive with the _[C]_ tag.

## Usage

```c
#include "css_inline.h"
#include <stdio.h>

#define OUTPUT_SIZE 1024

int main(void) {
  CssInlinerOptions options = css_inliner_default_options();
  const char input[] =
    "<html>"
      "<head>"
        "<style>h1 {color : red}</style>"
      "</head>"
      "<body>"
        "<h1>Test</h1>"
      "</body>"
    "</ html>";
  char output[OUTPUT_SIZE];
  if (css_inline_to(&options, input, output, sizeof(output)) == CSS_RESULT_OK) {
    printf("Inlined CSS: %s\n", output);
  }

  // Alternatively, because CSS_RESULT_OK is equal to 0, you can do
  CssResult res = css_inline_to(&options, input, output, sizeof(output));
  if (!res) {
    printf("An error occurred while inlining the CSS, see the result enum type: %d", res);
  }

  return 0;
}
```

The inline function, `css_inline_to()`, doesn't allocate, so you must provide an array big enough to fit the result. If the size is not sufficient, the enum `CSS_RESULT_IO_ERROR` will be returned.

### Configuration

You can change the inline behavior by modifying the `CssInlinerOptions` struct parameter that will be passed to `css_inline_to()`:

```c
#include "css_inline.h"
#include <stdbool.h>

int main(void) {
  CssInlinerOptions options = css_inliner_default_options();
  options.load_remote_stylesheets = true;
  char input[] = "...";
  char output[256];
  if (!css_inline_to(&options, input, output, sizeof(output))) {
    // Deal with the error
  }
  return 0;
}
```

Possible configurations:

- `inline_style_tags`. Specifies whether to inline CSS from "style" tags. Default: `true`
- `keep_style_tags`. Specifies whether to keep "style" tags after inlining. Default: `false`
- `keep_link_tags`. Specifies whether to keep "link" tags after inlining. Default: `false`
- `base_url`. The base URL used to resolve relative URLs. If you'd like to load stylesheets from your filesystem, use the `file://` scheme. Default: `NULL`
- `load_remote_stylesheets`. Specifies whether remote stylesheets should be loaded. Default: `true`
- `cache`. Specifies caching options for external stylesheets. Default: `NULL`
- `extra_css`. Extra CSS to be inlined. Default: `NULL`
- `preallocate_node_capacity`. **Advanced**. Preallocates capacity for HTML nodes during parsing. This can improve performance when you have an estimate of the number of nodes in your HTML document. Default: `32`

You can also skip CSS inlining for an HTML tag by adding the `data-css-inline="ignore"` attribute to it:

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

You can also cache external stylesheets to avoid excessive network requests:

```c
int main(void) {
  // Configure cache
  StylesheetCache cache = css_inliner_stylesheet_cache(8);
  CssInlinerOptions options = css_inliner_default_options();
  options.cache = &cache;
  // ... Inline CSS
  return 0;
}
```

Caching is disabled by default.

## License

This project is licensed under the terms of the [MIT license](https://opensource.org/licenses/MIT).
