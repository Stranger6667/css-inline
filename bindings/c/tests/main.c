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

/**
 * @brief Makes a html-like string in @p html given a @p style and a @p body.
 * @param html where to store the result.
 * @param style the style to be used.
 * @param body the body to be used.
 * @return true if MAX_SIZE is enough to fit the generated html, or else, false
 */
static bool make_hmtl(char *html, const char *style, const char *body) {
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
  assert(make_hmtl(html, SAMPLE_STYLE, SAMPLE_BODY));
  CssInlinerOptions options = css_inliner_default_options();
  char output[MAX_SIZE];
  assert(inline_to(&options, html, output, sizeof(output)) == CSS_RESULT_OK);
  assert(strcmp(output, SAMPLE_INLINED) == 0);
}

static void test_output_size_too_smal(void) {
  char html[MAX_SIZE];
  assert(make_hmtl(html, SAMPLE_STYLE, SAMPLE_BODY));
  CssInlinerOptions options = css_inliner_default_options();
  char output[1];
  assert(inline_to(&options, html, output, sizeof(output)) ==
         CSS_RESULT_IO_ERROR);
}

static void test_missing_stylesheet(void) {
  CssInlinerOptions options = css_inliner_default_options();
  char html[] =
      "<html><head><link href=\"tests/missing.css\" rel=\"stylesheet\" "
      "type=\"text/css\"></head><body><h1>Big Text</h1></body>";
  char output[MAX_SIZE];
  assert(inline_to(&options, html, output, sizeof(output)) ==
         CSS_RESULT_MISSING_STYLESHEET);
}

static void test_invalid_base_url(void) {
  char html[MAX_SIZE];
  assert(make_hmtl(html, SAMPLE_STYLE, SAMPLE_BODY));
  CssInlinerOptions options = css_inliner_default_options();
  options.base_url = "foo";
  char output[MAX_SIZE];
  assert(inline_to(&options, html, output, sizeof(output)) ==
         CSS_RESULT_INVALID_URL);
}

static void test_file_scheme(void) {
  char html[] = "<html><head><link href=\"external.css\" rel=\"stylesheet\" "
                "type=\"text/css\"></head><body><h1>Big Text</h1></body>";
  CssInlinerOptions options = css_inliner_default_options();
  options.base_url = "file://tests/";
  char output[MAX_SIZE];
  assert(inline_to(&options, html, output, sizeof(output)) == CSS_RESULT_OK);
}

int main(void) {
  test_default_options();
  test_output_size_too_smal();
  test_missing_stylesheet();
  test_invalid_base_url();
  test_file_scheme();
  return 0;
}
