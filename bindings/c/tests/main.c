#include "css_inline.h"
#include <assert.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>

#define MAX_SIZE 2048
#define SAMPLE_STYLE                                                           \
  "h1, h2 { color:red; } strong { text-decoration:none } p { font-size:2px } " \
  "p.footer { font-size: 1px}"
#define SAMPLE_BODY                                                            \
  "<h1>Big Text</h1><p><strong>Yes!</strong></p><p class=\"footer\">Foot "     \
  "notes</p>"
#define SAMPLE_INLINED                                                         \
  "<html><head></head><body><h1 style=\"color: red;\">Big Text</h1><p "        \
  "style=\"font-size: 2px;\"><strong style=\"text-decoration: "                \
  "none;\">Yes!</strong></p><p class=\"footer\" style=\"font-size: "           \
  "1px;\">Foot notes</p></body></html>"
#define SAMPLE_FRAGMENT                                                        \
  "<main>"                                                                     \
  "<h1>Hello</h1>"                                                             \
  "<section>"                                                                  \
  "<p>who am i</p>"                                                            \
  "</section>"                                                                 \
  "</main>"
#define SAMPLE_FRAGMENT_STYLE                                                  \
  "p { color: red; } h1 { color: blue; }"
#define SAMPLE_INLINED_FRAGMENT                                                \
  "<main><h1 style=\"color: blue;\">Hello</h1><section><p style=\"color: red;\">who am i</p></section></main>"

/**
 * @brief Makes a html-like string in @p html given a @p style and a @p body.
 * @param html where to store the result.
 * @param style the style to be used.
 * @param body the body to be used.
 * @return true if MAX_SIZE is enough to fit the generated html, or else, false
 */
static bool make_html(char *html, const char *style, const char *body) {
  int res =
      // sprintf is very dangerous so I am using the safe snprintf instead
      snprintf(html, MAX_SIZE,
               "<html><head><style>%s</style></head><body>%s</body></html>",
               style, body);
  if (res >= 0 && res < MAX_SIZE) {
    return true;
  }
  return false;
}

static void test_default_options(void) {
  char html[MAX_SIZE];
  assert(make_html(html, SAMPLE_STYLE, SAMPLE_BODY));
  CssInlinerOptions options = css_inliner_default_options();
  char output[MAX_SIZE];
  assert(css_inline_to(&options, html, output, sizeof(output)) ==
         CSS_RESULT_OK);
  assert(strcmp(output, SAMPLE_INLINED) == 0);
}

static void test_output_size_too_small(void) {
  char html[MAX_SIZE];
  assert(make_html(html, SAMPLE_STYLE, SAMPLE_BODY));
  CssInlinerOptions options = css_inliner_default_options();
  char output[1];
  assert(css_inline_to(&options, html, output, sizeof(output)) ==
         CSS_RESULT_IO_ERROR);
}

static void test_missing_stylesheet(void) {
  CssInlinerOptions options = css_inliner_default_options();
  char html[] =
      "<html><head><link href=\"tests/missing.css\" rel=\"stylesheet\" "
      "type=\"text/css\"></head><body><h1>Big Text</h1></body>";
  char output[MAX_SIZE];
  assert(css_inline_to(&options, html, output, sizeof(output)) ==
         CSS_RESULT_MISSING_STYLESHEET);
}

static void test_invalid_base_url(void) {
  char html[MAX_SIZE];
  assert(make_html(html, SAMPLE_STYLE, SAMPLE_BODY));
  CssInlinerOptions options = css_inliner_default_options();
  options.base_url = "foo";
  char output[MAX_SIZE];
  assert(css_inline_to(&options, html, output, sizeof(output)) ==
         CSS_RESULT_INVALID_URL);
}

static void test_file_scheme(void) {
  char html[] = "<html><head><link href=\"external.css\" rel=\"stylesheet\" "
                "type=\"text/css\"></head><body><h1>Big Text</h1></body>";
  CssInlinerOptions options = css_inliner_default_options();
  options.base_url = "file://tests/";
  char output[MAX_SIZE];
  assert(css_inline_to(&options, html, output, sizeof(output)) ==
         CSS_RESULT_OK);
}

static void test_cache_valid(void) {
    char html[MAX_SIZE];
    assert(make_html(html, SAMPLE_STYLE, SAMPLE_BODY));

    StylesheetCache cache = css_inliner_stylesheet_cache(8);
    CssInlinerOptions options = css_inliner_default_options();
    options.cache = &cache;

    char first_output[MAX_SIZE];
    char second_output[MAX_SIZE];

    assert(css_inline_to(&options, html, first_output, sizeof(first_output)) == CSS_RESULT_OK);
    assert(strcmp(first_output, SAMPLE_INLINED) == 0);
}

static void test_cache_invalid(void) {
    char html[MAX_SIZE];
    assert(make_html(html, SAMPLE_STYLE, SAMPLE_BODY));

    StylesheetCache cache = css_inliner_stylesheet_cache(0);
    CssInlinerOptions options = css_inliner_default_options();
    options.cache = &cache;

    char first_output[MAX_SIZE];
    char second_output[MAX_SIZE];

    assert(css_inline_to(&options, html, first_output, sizeof(first_output)) == CSS_RESULT_INVALID_CACHE_SIZE);
}

static void test_inline_fragment(void) {
  CssInlinerOptions options = css_inliner_default_options();
  char output[MAX_SIZE];
  assert(css_inline_fragment_to(&options, SAMPLE_FRAGMENT, SAMPLE_FRAGMENT_STYLE, output, sizeof(output)) ==
         CSS_RESULT_OK);
  assert(strcmp(output, SAMPLE_INLINED_FRAGMENT) == 0);
}

int main(void) {
  test_default_options();
  test_output_size_too_small();
  test_missing_stylesheet();
  test_invalid_base_url();
  test_file_scheme();
  test_cache_valid();
  test_cache_invalid();
  test_inline_fragment();
  return 0;
}
