# css-inline

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/Stranger6667/css-inline/build.yml?style=flat-square&labelColor=555555&logo=github" height="20">](https://github.com/Stranger6667/css-inline/actions/workflows/build.yml)
[<img alt="codecov.io" src="https://img.shields.io/codecov/c/gh/Stranger6667/css-inline?logo=codecov&style=flat-square&token=tOzvV4kDY0" height="20">](https://app.codecov.io/github/Stranger6667/css-inline)
[<img alt="gitter" src="https://img.shields.io/gitter/room/Stranger6667/css-inline?style=flat-square" height="20">](https://gitter.im/Stranger6667/css-inline)

`css-inline` inlines CSS into HTML `style` attributes. Use it to prepare HTML emails or to embed HTML into third-party web pages. See the [benchmarks](https://github.com/Stranger6667/css-inline#performance) for speed against other inliners.

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

- Builds on components from Mozilla's Servo project
- Inlines CSS from `style` and `link` tags
- Removes `style` and `link` tags
- Resolves external stylesheets, including local files
- Supports HTML5 & CSS3

## Playground

Try `css-inline` in the WebAssembly [playground](https://css-inline.org/).

## Install

The C bindings ship as a header (`css_inline.h`) and a dynamic library (`libcss_inline.so`, Linux x86_64).
Download both from the latest [Releases](https://github.com/Stranger6667/css-inline/releases) entry titled _[C] Release_.

On other platforms, build the library from source with Rust installed, then copy the library and header next to your program:

```shell
git clone https://github.com/Stranger6667/css-inline
cd css-inline/bindings/c
cargo build --release
cp target/release/libcss_inline.so css_inline.h /path/to/your/project/
```

Compile and run your program from that directory:

```shell
gcc -I. main.c libcss_inline.so -o main
LD_LIBRARY_PATH=. ./main
```

CI builds and tests the C bindings on Linux only. On macOS, Cargo names the library `libcss_inline.dylib` and the loader reads `DYLD_LIBRARY_PATH`; on Windows, it produces `css_inline.dll` (with `css_inline.dll.lib` to link against) and the loader searches `PATH`.

## Usage

The header declares two inlining functions:

```c
enum CssResult css_inline_to(const struct CssInlinerOptions *options,
                             const char *input,
                             char *output,
                             size_t output_size);

enum CssResult css_inline_fragment_to(const struct CssInlinerOptions *options,
                                      const char *input,
                                      const char *css,
                                      char *output,
                                      size_t output_size);
```

All strings are NUL-terminated UTF-8. `output` is a buffer you own, `output_size` its size in bytes. The header typedefs `CssResult`, `CssInlinerOptions` and `StylesheetCache`, so the examples use the bare names.

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
    "</html>";
  char output[OUTPUT_SIZE];
  CssResult res = css_inline_to(&options, input, output, sizeof(output));
  if (res == CSS_RESULT_OK) {
    printf("Inlined CSS: %s\n", output);
    // Inlined CSS: <html><head></head><body><h1 style="color: red;">Test</h1></body></html>
  } else {
    printf("Inlining failed with CssResult %d\n", res);
  }

  return 0;
}
```

`css_inline_to()` writes the result into your buffer and NUL-terminates it. The library cannot report the result size in advance. If the result plus the NUL byte does not fit, the call returns `CSS_RESULT_IO_ERROR`, and the buffer holds the first `output_size - 1` bytes with no NUL terminator. Do not read it as a string.

`CSS_RESULT_IO_ERROR` also covers a failed read of a local stylesheet (for example, permission denied). The library loads stylesheets before writing any output, so that failure leaves the buffer untouched. Set `output[0] = '\0'` before the call: after `CSS_RESULT_IO_ERROR`, a non-empty first byte means the buffer was too small. Retry with a larger buffer up to a cap you choose:

```c
#include "css_inline.h"
#include <stdio.h>
#include <stdlib.h>

#define MAX_OUTPUT_SIZE (16 * 1024 * 1024)

int main(void) {
  CssInlinerOptions options = css_inliner_default_options();
  const char input[] = "<html><head><style>h1 {color: red}</style></head><body><h1>Test</h1></body></html>";
  size_t size = 16;
  char *output = NULL;
  CssResult res;
  do {
    char *grown = realloc(output, size);
    if (!grown) {
      break;
    }
    output = grown;
    output[0] = '\0';
    res = css_inline_to(&options, input, output, size);
    size *= 2;
  } while (res == CSS_RESULT_IO_ERROR && output[0] != '\0' && size <= MAX_OUTPUT_SIZE);
  if (output && res == CSS_RESULT_OK) {
    printf("Inlined CSS: %s\n", output);
  }
  free(output);
  return 0;
}
```

`css-inline` adds missing `html`, `head` and `body` tags, so the output is a complete HTML document.

To inline CSS into an HTML fragment, call `css_inline_fragment_to()` with the fragment and the CSS. `css` must not be `NULL`; pass `""` when you have no CSS. `<style>` tags inside the fragment apply too. The function drops structural tags (`<html>`, `<head>`, `<body>`) and keeps their contents. Use `css_inline_to()` to keep the full document structure:

```c
#include "css_inline.h"
#include <stdio.h>

#define OUTPUT_SIZE 1024

int main(void) {
  CssInlinerOptions options = css_inliner_default_options();
  const char fragment[] =
    "<main>"
      "<h1>Hello</h1>"
      "<section>"
      "<p>who am i</p>"
      "</section>"
    "</main>";

  const char css[] =
    "p {"
      "color: red;"
    "}"
    "h1 {"
      "color: blue;"
    "}";
  char output[OUTPUT_SIZE];
  CssResult res = css_inline_fragment_to(&options, fragment, css, output, sizeof(output));
  if (res == CSS_RESULT_OK) {
    printf("Inlined CSS: %s\n", output);
    // Inlined CSS: <main><h1 style="color: blue;">Hello</h1><section><p style="color: red;">who am i</p></section></main>
  } else {
    printf("Inlining failed with CssResult %d\n", res);
  }
  return 0;
}
```

### Return values

Both functions return a `CssResult`. `CSS_RESULT_OK` is `0`; any other value is an error:

| Value | Meaning |
|-------|---------|
| `CSS_RESULT_MISSING_STYLESHEET` | A local stylesheet file is missing |
| `CSS_RESULT_REMOTE_STYLESHEET_NOT_AVAILABLE` | A remote stylesheet failed to load |
| `CSS_RESULT_IO_ERROR` | I/O failure, or the output buffer is too small |
| `CSS_RESULT_INTERNAL_SELECTOR_PARSE_ERROR` | CSS failed to parse |
| `CSS_RESULT_NULL_OPTIONS` | `options` is `NULL` |
| `CSS_RESULT_INVALID_URL` | `base_url` is not a valid URL or not UTF-8 |
| `CSS_RESULT_INVALID_EXTRA_CSS` | `extra_css` is not UTF-8 |
| `CSS_RESULT_INVALID_INPUT_STRING` | The HTML or CSS input is not UTF-8 |
| `CSS_RESULT_INVALID_CACHE_SIZE` | The stylesheet cache size is `0` |

### Configuration

Set fields on `CssInlinerOptions` before passing it to `css_inline_to()` or `css_inline_fragment_to()`:

```c
#include "css_inline.h"
#include <stdbool.h>

int main(void) {
  CssInlinerOptions options = css_inliner_default_options();
  options.load_remote_stylesheets = false;
  char input[] = "...";
  char output[256];
  if (css_inline_to(&options, input, output, sizeof(output)) != CSS_RESULT_OK) {
    // Handle the error
  }
  return 0;
}
```

Fields:

- `inline_style_tags`. Inline CSS from `style` tags. Default: `true`
- `keep_style_tags`. Keep `style` tags after inlining. Default: `false`
- `keep_link_tags`. Keep `link` tags after inlining. Default: `false`
- `keep_at_rules`. Keep at-rules (starting with `@`) after inlining. Default: `false`
- `minify_css`. Remove trailing semicolons and spaces between properties and values. Default: `false`
- `base_url` (`const char *`). Base URL for resolving relative URLs. Use the `file://` scheme to load stylesheets from your filesystem. Default: `NULL`
- `load_remote_stylesheets`. Load stylesheets from `link` tags, over the network or from `file://` paths. Default: `true`
- `cache` (`const StylesheetCache *`). Cache for external stylesheets. Default: `NULL` (no caching)
- `extra_css` (`const char *`). Extra CSS to inline. Default: `NULL`
- `preallocate_node_capacity`. **Advanced**. Number of HTML nodes to preallocate during parsing. Set it to your expected node count to avoid reallocations. Default: `32`
- `remove_inlined_selectors`. Remove selectors from `<style>` blocks once inlined. Default: `false`
- `apply_width_attributes`. Add `width` HTML attributes from CSS `width` properties on `table`, `td`, `th` and `img`. Default: `false`
- `apply_height_attributes`. Add `height` HTML attributes from CSS `height` properties on `table`, `td`, `th` and `img`. Default: `false`

With the defaults, the library loads every stylesheet that a `link` tag references, over the network or from local files. Set `load_remote_stylesheets` to `false` to block both network and file access for `link` tags.

The library reads `base_url`, `extra_css` and `cache` during each call and keeps no reference afterwards. Keep them valid until the call returns.

To skip inlining for an HTML tag, add the `data-css-inline="ignore"` attribute:

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

The same attribute on a `link` or `style` tag skips that stylesheet:

```html
<head>
  <!-- Styles below are ignored -->
  <style data-css-inline="ignore">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

To keep a `style` tag in the output, add `data-css-inline="keep"`, for example to keep `@media` queries for responsive emails.
The tag stays even with `keep_style_tags` set to `false`.

```html
<head>
  <!-- Styles below are not removed -->
  <style data-css-inline="keep">h1 { color:blue; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

HTML attributes cannot hold at-rules, so `css-inline` drops them by default.
Set `keep_at_rules` to `true` to inline the regular rules and keep the at-rules, such as `@media` queries, in a `style` tag.
The tag stays even with `keep_style_tags` set to `false`.

```html
<head>
  <!-- With keep_at_rules=true "color:blue" will get inlined into <h1> but @media will be kept in <style> -->
  <style>h1 { color: blue; } @media (max-width: 600px) { h1 { font-size: 18px; } }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

With `minify_css` set to `true`, `css-inline` removes trailing semicolons and spaces between properties and values in inlined styles.

```html
<head>
  <!-- With minify_css=true, the <h1> will have `style="color:blue;font-weight:bold"` -->
  <style>h1 { color: blue; font-weight: bold; }</style>
</head>
<body>
  <h1>Big Text</h1>
</body>
```

Each call builds a new LRU cache of up to `size` stylesheets, so nothing carries over between calls. The library already fetches each distinct `href` once per call, so the cache only saves a fetch when two different `href` values resolve to the same URL. `StylesheetCache` is a plain struct with no free function. A `size` of `0` makes the inlining call, not `css_inliner_stylesheet_cache()`, return `CSS_RESULT_INVALID_CACHE_SIZE`:

```c
#include "css_inline.h"

int main(void) {
  StylesheetCache cache = css_inliner_stylesheet_cache(8);
  CssInlinerOptions options = css_inliner_default_options();
  options.cache = &cache;
  // ... Inline CSS
  return 0;
}
```

## License

This project uses the [MIT license](https://opensource.org/licenses/MIT).
