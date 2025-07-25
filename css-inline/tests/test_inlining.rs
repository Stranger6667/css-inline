use std::{error::Error, sync::Arc};

#[macro_use]
mod utils;

use css_inline::{inline, CSSInliner, InlineOptions, Url};
use test_case::test_case;

#[cfg(not(feature = "file"))]
fn assert_file_error(inlined: Result<String, css_inline::InlineError>) {
    assert_eq!(
        inlined.expect_err("Should fail").to_string(),
        "Loading local files requires the `file` feature"
    );
}

#[allow(unused_variables)]
fn assert_file(inlined: Result<String, css_inline::InlineError>, expected: &str) {
    #[cfg(feature = "file")]
    {
        assert!(inlined.expect("Inlining failed").ends_with(expected));
    }
    #[cfg(not(feature = "file"))]
    {
        assert_file_error(inlined)
    }
}

#[allow(unused_variables)]
fn assert_http(inlined: Result<String, css_inline::InlineError>, expected: &str) {
    #[cfg(feature = "http")]
    {
        assert!(inlined.expect("Inlining failed").ends_with(expected));
    }
    #[cfg(not(feature = "http"))]
    {
        let error = inlined.expect_err("Should fail");
        assert_eq!(
            error.to_string(),
            "Loading external URLs requires the `http` feature"
        );
        assert!(error.source().is_some());
    }
}

#[test]
fn no_existing_style() {
    // When no "style" attributes exist
    assert_inlined!(
        style = r#"h1, h2 { color:red; }
strong { text-decoration:none }
p { font-size:2px }
p.footer { font-size: 1px}"#,
        body = r#"<h1>Big Text</h1>
<p><strong>Yes!</strong></p>
<p class="footer">Foot notes</p>"#,
        // Then all styles should be added to new "style" attributes
        expected = r#"<h1 style="color: red;">Big Text</h1>
<p style="font-size: 2px;"><strong style="text-decoration: none;">Yes!</strong></p>
<p class="footer" style="font-size: 1px;">Foot notes</p>"#
    )
}

#[test]
fn ignore_inlining_attribute_tag() {
    // When an HTML tag contains `data-css-inline="ignore"`
    assert_inlined!(
        style = "h1 { color:blue; }",
        body = r#"<h1 data-css-inline="ignore">Big Text</h1>"#,
        // Then it should be skipped
        expected = r#"<h1 data-css-inline="ignore">Big Text</h1>"#
    )
}

#[test]
fn ignore_inlining_attribute_style() {
    // When a `style` tag contains `data-css-inline="ignore"`
    let html = r#"
<html>
<head>
<style data-css-inline="ignore">
h1 { color: blue; }
</style>
</head>
<body>
<h1>Big Text</h1>
</body>
</html>"#;
    let result = inline(html).unwrap();
    // Then it should be skipped
    assert!(result.ends_with(
        r#"<body>
<h1>Big Text</h1>

</body></html>"#
    ))
}

#[test]
fn keep_attribute_style() {
    // When a `style` tag contains `data-css-inline="keep"`
    let html = r#"
<html>
<head>
<style data-css-inline="keep">
h1 { color: blue; }
</style>
</head>
<body>
<h1>Big Text</h1>
</body>
</html>"#;
    let result = inline(html).unwrap();
    // Then it should be kept as is even if the configuration implies removing style tags
    assert_eq!(
        result,
        r#"<html><head>
<style data-css-inline="keep">
h1 { color: blue; }
</style>
</head>
<body>
<h1 style="color: blue;">Big Text</h1>

</body></html>"#
    );
}

#[test]
fn ignore_inlining_attribute_link() {
    // When a `link` tag contains `data-css-inline="ignore"`
    let html = r#"
<html>
<head>
<link href="tests/external.css" rel="stylesheet" data-css-inline="ignore">
</head>
<body>
<h1>Big Text</h1>
</body>
</html>"#;
    let result = inline(html).unwrap();
    // Then it should be skipped
    assert!(result.ends_with(
        r#"<body>
<h1>Big Text</h1>

</body></html>"#
    ))
}

#[test]
fn specificity_same_selector() {
    assert_inlined!(
        style = r#"
.test-class {
    padding-top: 15px;
    padding: 10px;
    padding-left: 12px;
}"#,
        body = r#"<a class="test-class">Test</a>"#,
        // Then the final style should come from the more specific selector
        expected = r#"<a class="test-class" style="padding-top: 15px;padding: 10px;padding-left: 12px;">Test</a>"#
    )
}

#[test]
fn specificity_different_selectors() {
    assert_inlined!(
        style = r#"
.test { padding-left: 16px; }
h1 { padding: 0; }"#,
        body = r#"<h1 class="test"></h1>"#,
        expected = r#"<h1 class="test" style="padding: 0;padding-left: 16px;"></h1>"#
    )
}

#[test]
fn specificity_different_selectors_existing_style() {
    assert_inlined!(
        style = r#"
.test { padding-left: 16px; }
h1 { padding: 0; }"#,
        body = r#"<h1 class="test" style="color: blue;"></h1>"#,
        expected = r#"<h1 class="test" style="padding: 0;padding-left: 16px;color: blue"></h1>"#
    )
}

#[test]
fn specificity_merge_with_existing_style() {
    assert_inlined!(
        style = ".test { padding: 0; }",
        body = r#"<h1 class="test" style="padding-left: 16px"></h1>"#,
        expected = r#"<h1 class="test" style="padding: 0;padding-left: 16px"></h1>"#
    )
}

#[test]
fn overlap_styles() {
    // When two selectors match the same element
    assert_inlined!(
        style = r#"
.test-class {
    color: #ffffff;
}
a {
    color: #17bebb;
}"#,
        body = r#"<a class="test-class" href="https://example.com">Test</a>"#,
        // Then the final style should come from the more specific selector
        expected =
            r#"<a class="test-class" href="https://example.com" style="color: #ffffff;">Test</a>"#
    )
}

#[test]
fn simple_merge() {
    // When "style" attributes exist and collides with values defined in "style" tag
    let style = "h1 { color:red; }";
    let html = html!(style, r#"<h1 style="font-size: 1px">Big Text</h1>"#);
    let inlined = inline(&html).unwrap();
    // Then new styles should be merged with the existing ones
    let option_1 = html!(r#"<h1 style="font-size: 1px;color: red">Big Text</h1>"#);
    let option_2 = html!(r#"<h1 style="color: red;font-size: 1px">Big Text</h1>"#);
    let valid = (inlined == option_1) || (inlined == option_2);
    assert!(valid, "{}", inlined);
}

#[test]
fn overloaded_styles() {
    // When there is a style, applied to an ID
    assert_inlined!(
        style = "h1 { color: red; } #test { color: blue; }",
        body = r#"<h1 id="test">Hello world!</h1>"#,
        // Then it should be preferred over a more generic style
        expected = r#"<h1 id="test" style="color: blue;">Hello world!</h1>"#
    )
}

#[test]
fn important() {
    // `!important` rules should override existing inline styles
    assert_inlined!(
        style = "h1 { color: blue !important; }",
        body = r#"<h1 style="color: red;">Big Text</h1>"#,
        expected = r#"<h1 style="color: blue">Big Text</h1>"#
    )
}

#[test]
fn important_with_space_at_the_end() {
    assert_inlined!(
        style = "h1 { color: blue !important  ; }",
        body = r#"<h1 style="color: red;">Big Text</h1>"#,
        expected = r#"<h1 style="color: blue">Big Text</h1>"#
    )
}

#[test]
fn important_no_rule_exists() {
    // `!important` rules should override existing inline styles
    assert_inlined!(
        style = "h1 { color: blue !important; }",
        body = r#"<h1 style="margin:0">Big Text</h1>"#,
        expected = r#"<h1 style="color: blue;margin: 0">Big Text</h1>"#
    )
}

#[test]
fn important_multiple_rules() {
    // `!important` rules should override other rules with the same specificity.
    assert_inlined!(
        style = ".blue { color: blue !important; } .reset { color: unset }",
        body = r#"<h1 class="blue reset">Big Text</h1>"#,
        expected = r#"<h1 class="blue reset" style="color: blue !important;">Big Text</h1>"#
    );
    // check in both directions
    assert_inlined!(
        style = ".reset { color: unset } .blue { color: blue !important; }",
        body = r#"<h1 class="blue reset">Big Text</h1>"#,
        expected = r#"<h1 class="blue reset" style="color: blue !important;">Big Text</h1>"#
    );
}

#[test]
fn important_more_specific() {
    // `!important` rules should override other important rules with less specificity.
    assert_inlined!(
        style = "h1 { color: unset !important } #title { color: blue !important; }",
        body = r#"<h1 id="title">Big Text</h1>"#,
        expected = r#"<h1 id="title" style="color: blue !important;">Big Text</h1>"#
    );
}

#[test]
fn font_family_quoted() {
    // When property value contains double quotes
    assert_inlined!(
        style = r#"h1 { font-family: "Open Sans", sans-serif; }"#,
        body = r#"<h1>Hello world!</h1>"#,
        // Then it should be replaced with single quotes
        expected = r#"<h1 style="font-family: 'Open Sans', sans-serif;">Hello world!</h1>"#
    )
}

#[test]
fn font_family_quoted_with_existing_inline_style() {
    // When property value contains double quotes
    assert_inlined!(
        style = r#"h1 { font-family: "Open Sans", sans-serif; }"#,
        body = r#"<h1 style="whitespace: nowrap">Hello world!</h1>"#,
        // Then it should be replaced with single quotes
        expected = r#"<h1 style="font-family: 'Open Sans', sans-serif;whitespace: nowrap">Hello world!</h1>"#
    )
}

#[test]
fn font_family_quoted_with_inline_style_override() {
    // When property value contains double quotes
    assert_inlined!(
        style = r#"h1 { font-family: "Open Sans", sans-serif !important; }"#,
        body = r#"<h1 style="font-family: Helvetica; whitespace: nowrap">Hello world!</h1>"#,
        // Then it should be replaced with single quotes
        expected = r#"<h1 style="font-family: 'Open Sans', sans-serif;whitespace: nowrap">Hello world!</h1>"#
    )
}

#[test]
fn other_property_quoted() {
    // When property value contains double quotes
    assert_inlined!(
        style = r#"h1 { --bs-font-sant-serif: system-ui,-applie-system,"helvetica neue"; }"#,
        body = r#"<h1>Hello world!</h1>"#,
        // Then it should be replaced with single quotes
        expected = r#"<h1 style="--bs-font-sant-serif: system-ui,-applie-system,'helvetica neue';">Hello world!</h1>"#
    )
}

#[test]
fn href_attribute_unchanged() {
    // All HTML attributes should be serialized as is
    let html = r#"<html>
<head>
    <style>h1 { color:blue; }</style>
</head>
<body>
    <h1>Big Text</h1>
    <a href="https://example.org/test?a=b&c=d">Link</a>
</body>
</html>"#;
    let inlined = inline(html).unwrap();
    assert_eq!(
        inlined,
        r#"<html><head>
    
</head>
<body>
    <h1 style="color: blue;">Big Text</h1>
    <a href="https://example.org/test?a=b&amp;c=d">Link</a>

</body></html>"#
    );
}

#[test]
fn complex_child_selector() {
    let html = r#"<html>
   <head>
      <style>.parent {
         overflow: hidden;
         box-shadow: 0 4px 10px 0px rgba(0, 0, 0, 0.1);
         }
         .parent > table > tbody > tr > td,
         .parent > table > tbody > tr > td > div {
         border-radius: 3px;
         }
      </style>
   </head>
   <body>
      <div class="parent">
         <table>
            <tbody>
               <tr>
                  <td>
                     <div>
                        Test
                     </div>
                  </td>
               </tr>
            </tbody>
         </table>
      </div></body></html>"#;
    let inlined = inline(html).unwrap();
    assert_eq!(
        inlined,
        r#"<html><head>
      
   </head>
   <body>
      <div class="parent" style="overflow: hidden;box-shadow: 0 4px 10px 0px rgba(0, 0, 0, 0.1);">
         <table>
            <tbody>
               <tr>
                  <td style="border-radius: 3px;">
                     <div style="border-radius: 3px;">
                        Test
                     </div>
                  </td>
               </tr>
            </tbody>
         </table>
      </div></body></html>"#
    );
}

#[test]
fn existing_styles() {
    // When there is a `style` attribute on a tag that contains a rule
    // And the `style` tag contains the same rule applicable to that tag
    assert_inlined!(
        style = "h1 { color: red; }",
        body = r#"<h1 style="color: blue">Hello world!</h1>"#,
        // Then the existing rule should be preferred
        expected = r#"<h1 style="color: blue">Hello world!</h1>"#
    )
}

#[test]
fn existing_styles_multiple_tags() {
    // When there are `style` attribute on tags that contains rules
    // And the `style` tag contains the same rule applicable to those tags
    assert_inlined!(
        style = "h1 { color: red; }",
        body =
            r#"<h1 style="color: blue">Hello world!</h1><h1 style="color: blue">Hello world!</h1>"#,
        // Then the existing rule should be preferred
        expected =
            r#"<h1 style="color: blue">Hello world!</h1><h1 style="color: blue">Hello world!</h1>"#
    )
}

#[test]
fn existing_styles_with_merge() {
    // When there is a `style` attribute on a tag that contains a rule
    // And the `style` tag contains the same rule applicable to that tag
    // And there is a new rule in the `style` tag
    assert_inlined!(
        style = "h1 { color: red; font-size:14px; }",
        body = r#"<h1 style="color: blue">Hello world!</h1>"#,
        // Then the existing rule should be preferred
        // And the new style should be merged
        expected = r#"<h1 style="font-size: 14px;color: blue">Hello world!</h1>"#
    )
}

#[test]
fn existing_styles_with_merge_multiple_tags() {
    // When there are non-empty `style` attributes on tags
    // And the `style` tag contains the same rule applicable to those tags
    // And there is a new rule in the `style` tag
    assert_inlined!(
        style = "h1 { color: red; font-size:14px; }",
        body =
            r#"<h1 style="color: blue">Hello world!</h1><h1 style="color: blue">Hello world!</h1>"#,
        // Then the existing rule should be preferred
        // And the new style should be merged
        expected = r#"<h1 style="font-size: 14px;color: blue">Hello world!</h1><h1 style="font-size: 14px;color: blue">Hello world!</h1>"#
    )
}

#[test]
fn remove_multiple_style_tags_without_inlining() {
    let html = r#"
<html>
<head>
<style>
h1 {
    text-decoration: none;
}
</style>
<style>
.test-class {
        color: #ffffff;
}
a {
        color: #17bebb;
}
</style>
</head>
<body>
<a class="test-class" href="https://example.com">Test</a>
<h1>Test</h1>
</body>
</html>
    "#;
    let inliner = CSSInliner::options()
        .keep_style_tags(false)
        .inline_style_tags(false)
        .build();
    let result = inliner.inline(html).unwrap();
    assert_eq!(
        result,
        r#"<html><head>


</head>
<body>
<a class="test-class" href="https://example.com">Test</a>
<h1>Test</h1>


    </body></html>"#
    )
}

#[test]
fn do_not_process_style_tag() {
    let html = html!("@media (max-width: 767px) { padding: 0;} h1 {background-color: blue;}", "<h1>Hello world!</h1>");
    let options = InlineOptions {
        inline_style_tags: false,
        keep_style_tags: true,
        ..Default::default()
    };
    let inliner = CSSInliner::new(options);
    let result = inliner.inline(&html).unwrap();
    assert_eq!(
        result,
        "<html><head><style>@media (max-width: 767px) { padding: 0;} h1 {background-color: blue;}</style></head><body><h1>Hello world!</h1></body></html>"
    )
}

#[test]
fn do_not_process_and_remove_style_tag() {
    let html = html!("@media (max-width: 767px) { padding: 0;} h1 {background-color: blue;}", "<h1>Hello world!</h1>");
    let options = InlineOptions {
        keep_style_tags: false,
        inline_style_tags: false,
        ..Default::default()
    };
    let inliner = CSSInliner::new(options);
    let result = inliner.inline(&html).unwrap();
    assert_eq!(
        result,
        "<html><head></head><body><h1>Hello world!</h1></body></html>"
    )
}


#[test]
fn do_not_process_and_remove_style_tag_but_keep_at_rules() {
    let html = html!("@media (max-width: 767px) { padding: 0;} h1 {background-color: blue;}", "<h1>Hello world!</h1>");
    let options = InlineOptions {
        keep_style_tags: false,
        inline_style_tags: false,
        keep_at_rules: true,
        ..Default::default()
    };
    let inliner = CSSInliner::new(options);
    let result = inliner.inline(&html).unwrap();
    assert_eq!(
        result,
        "<html><head><style>@media (max-width: 767px) { padding: 0;} </style></head><body><h1>Hello world!</h1></body></html>"
    )
}

#[test]
fn empty_style() {
    // When the style tag is empty
    assert_inlined!(
        style = "",
        body = r#"<h1>Hello world!</h1>"#,
        // Then the body should remain the same
        expected = r#"<h1>Hello world!</h1>"#
    )
}

#[test]
fn media_query_ignore() {
    // When the style value includes @media query
    assert_inlined!(
        style = r#"@media screen and (max-width: 992px) {
  body {
    background-color: blue;
  }
}"#,
        body = "<h1>Hello world!</h1>",
        expected = "<h1>Hello world!</h1>"
    )
}

#[test_case("@wrong { color: --- }", "Invalid @ rule: wrong")]
#[test_case("ttt { 123 }", "Unexpected token: CurlyBracketBlock")]
#[test_case("----", "End of input")]
fn invalid_rule(style: &str, expected: &str) {
    let html = html!(
        "h1 {background-color: blue;}",
        format!(r#"<h1 style="{}">Hello world!</h1>"#, style)
    );
    let result = inline(&html);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), expected);
}

#[test]
fn remove_style_tag() {
    let html = html!(
        "@media (max-width: 600px) { h1 { font-size: 18px; } }\nh1 {background-color: blue;}",
        "<h1>Hello world!</h1>"
    );
    let result = inline(&html).unwrap();
    assert_eq!(result, "<html><head></head><body><h1 style=\"background-color: blue;\">Hello world!</h1></body></html>")
}

#[test]
fn remove_multiple_style_tags() {
    let html = r#"
<html>
<head>
<style>
h1 {
    text-decoration: none;
}
@media (max-width: 600px) { h1 { font-size: 18px; } }
</style>
<style>
.test-class {
        color: #ffffff;
}
a {
        color: #17bebb;
}
</style>
</head>
<body>
<a class="test-class" href="https://example.com">Test</a>
<h1>Test</h1>
</body>
</html>
    "#;
    let result = inline(html).unwrap();
    assert_eq!(
        result,
        r#"<html><head>


</head>
<body>
<a class="test-class" href="https://example.com" style="color: #ffffff;">Test</a>
<h1 style="text-decoration: none;">Test</h1>


    </body></html>"#
    )
}

#[test]
fn keep_multiple_at_rules() {
    let html = r#"
<html>
<head>
<style>
@media (max-width: 600px) { h1 { font-size: 18px; } }
@media (max-width: 400px) { h1 { font-size: 12px; } }
</style>
<style>
@media (max-width: 200px) { h1 { font-size: 8px; } }
</style>
</head>
<body>
<h1>Test</h1>
</body>
</html>
    "#;

        let options = InlineOptions {
        keep_at_rules: true,
        ..Default::default()
    };
    let inliner = CSSInliner::new(options);
    let result = inliner.inline(html).unwrap();
    assert_eq!(
        result,
        r#"<html><head><style>@media (max-width: 600px) { h1 { font-size: 18px; } } @media (max-width: 400px) { h1 { font-size: 12px; } } @media (max-width: 200px) { h1 { font-size: 8px; } } </style>


</head>
<body>
<h1>Test</h1>


    </body></html>"#
    )
}

#[test]
fn extra_css() {
    let html = html!("h1 {background-color: blue;}", "<h1>Hello world!</h1>");
    let options = InlineOptions {
        inline_style_tags: false,
        extra_css: Some("h1 {background-color: green;}".into()),
        ..Default::default()
    };
    let inliner = CSSInliner::new(options);
    let result = inliner.inline(&html).unwrap();
    assert_eq!(
        result,
        "<html><head></head><body><h1 style=\"background-color: green;\">Hello world!</h1></body></html>"
    )
}

#[test]
fn remote_file_stylesheet() {
    let html = r#"
<html>
<head>
<link href="tests/external.css" rel="stylesheet">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style>
h2 { color: red; }
</style>
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>"#;
    let inlined = inline(html);
    assert_file(
        inlined,
        r#"<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>"#,
    );
}

#[test]
fn missing_stylesheet() {
    let html = r#"
<html>
<head>
<link href="tests/missing.css" rel="stylesheet">
</head>
<body>
<h1>Big Text</h1>
</body>
</html>"#;
    let inlined = inline(html);
    #[cfg(feature = "file")]
    {
        let error = inlined.expect_err("Should be an error");
        assert_eq!(
            error.to_string(),
            "Missing stylesheet file: tests/missing.css"
        );
        assert!(error.source().is_none());
    }
    #[cfg(not(feature = "file"))]
    {
        assert_file_error(inlined);
    }
}

#[test]
fn remote_file_stylesheet_disable() {
    let html = r#"
<html>
<head>
<link href="tests/external.css" rel="stylesheet">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style>
h2 { color: red; }
</style>
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>"#;
    let inlined = inline(html);
    assert_file(
        inlined,
        r#"<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>"#,
    );
}

#[test]
fn remote_network_stylesheet() {
    let html = r#"
<html>
<head>
<link href="http://127.0.0.1:1234/external.css" rel="stylesheet">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style>
h2 { color: red; }
</style>
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>"#;
    let inlined = inline(html);
    assert_http(
        inlined,
        r#"<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>"#,
    );
}

#[test]
fn remote_network_stylesheet_invalid_url() {
    let html = r#"
<html>
<head>
<link href="http:" rel="stylesheet">
</head>
<body>
</body>
</html>"#;
    let error = inline(html).expect_err("Should fail");
    #[cfg(feature = "http")]
    let expected = "builder error: http:";
    #[cfg(not(feature = "http"))]
    let expected = "Loading external URLs requires the `http` feature";
    assert_eq!(error.to_string(), expected);
    assert!(error.source().is_some());
}

#[test]
fn remote_network_stylesheet_same_scheme() {
    let html = r#"
<html>
<head>
<link href="//127.0.0.1:1234/external.css" rel="stylesheet">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style>
h2 { color: red; }
</style>
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>"#;
    let inliner = CSSInliner::options()
        .base_url(Some(Url::parse("http://127.0.0.1:1234").unwrap()))
        .build();
    let inlined = inliner.inline(html);
    assert_http(
        inlined,
        r#"<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>"#,
    );
}

#[test]
fn remote_network_relative_stylesheet() {
    let html = r#"
<html>
<head>
<link href="external.css" rel="stylesheet">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style>
h2 { color: red; }
</style>
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>"#;
    let inliner = CSSInliner::options()
        .base_url(Some(Url::parse("http://127.0.0.1:1234").unwrap()))
        .build();
    let inlined = inliner.inline(html);
    assert_http(
        inlined,
        r#"<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>"#,
    );
}

#[test]
fn file_scheme() {
    let html = r#"
<html>
<head>
<link href="external.css" rel="stylesheet">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style>
h2 { color: red; }
</style>
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>"#;
    let options = InlineOptions {
        base_url: Some(Url::parse("file://tests/").unwrap()),
        ..Default::default()
    };
    let inliner = CSSInliner::new(options);
    let inlined = inliner.inline(html);
    assert_file(
        inlined,
        r#"<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>"#,
    );
}

#[test]
fn customize_inliner() {
    let options = InlineOptions {
        load_remote_stylesheets: false,
        ..Default::default()
    }
    .preallocate_node_capacity(25);
    assert!(!options.load_remote_stylesheets);
    assert!(!options.keep_style_tags);
    assert_eq!(options.base_url, None);
    assert_eq!(options.preallocate_node_capacity, 25);
}

#[test]
fn use_builder() {
    let url = Url::parse("https://api.example.com").unwrap();
    let _ = CSSInliner::options()
        .keep_style_tags(true)
        .base_url(Some(url))
        .load_remote_stylesheets(false)
        .extra_css(Some("h1 {color: green}".into()))
        .build();
}

#[test]
fn inline_to() {
    let html = html!("h1 { color: blue }", r#"<h1>Big Text</h1>"#);
    let mut out = Vec::new();
    css_inline::inline_to(&html, &mut out).unwrap();
    assert_eq!(
        String::from_utf8_lossy(&out),
        "<html><head></head><body><h1 style=\"color: blue;\">Big Text</h1></body></html>"
    )
}

#[test]
fn keep_style_tags() {
    let inliner = CSSInliner::options().keep_style_tags(true).build();
    let html = r#"
<html>
<head>
<style>
@media (max-width: 600px) { h1 { font-size: 18px; } }
h2 { color: red; }
</style>
</head>
<body>
<h2></h2>
</body>
</html>"#;
    let inlined = inliner.inline(html).unwrap();
    assert_eq!(inlined, "<html><head>\n<style>\n@media (max-width: 600px) { h1 { font-size: 18px; } }\nh2 { color: red; }\n</style>\n</head>\n<body>\n<h2 style=\"color: red;\"></h2>\n\n</body></html>");
}

#[test]
fn keep_link_tags() {
    let inliner = CSSInliner::options()
        .base_url(Some(Url::parse("http://127.0.0.1:1234").unwrap()))
        .keep_link_tags(true)
        .build();
    let html = r#"
<html>
<head>
<link href="external.css" rel="stylesheet">
</head>
<body>
<h1></h1>
</body>
</html>"#;
    let inlined = inliner.inline(html);
    assert_http(
        inlined,
        "<html><head>\n<link href=\"external.css\" rel=\"stylesheet\">\n</head>\n<body>\n<h1 style=\"color: blue;\"></h1>\n\n</body></html>",
    );
}

#[test]
fn keep_at_rules() {
    let inliner = CSSInliner::options().keep_at_rules(true).build();
    let html = r#"
<html>
<head>
<style>
h1 { color: blue; }
@media (max-width: 600px) { h1 { font-size: 18px; } }
p { margin: 10px; }
</style>
</head>
<body>
<h1>Hello</h1><p>World</p>
</body>
</html>"#;
    let inlined = inliner.inline(html).unwrap();
    let expected = "<html><head><style>@media (max-width: 600px) { h1 { font-size: 18px; } } </style>\n\n</head>\n<body>\n<h1 style=\"color: blue;\">Hello</h1><p style=\"margin: 10px;\">World</p>\n\n</body></html>";
    assert_eq!(inlined, expected);
}

#[test]
fn nth_child_selector() {
    let html = r#"
<html>
<head>
<style>tbody tr:nth-child(odd) td {background-color:grey;}</style>
</head>
<body>
<table>
   <tbody>
      <tr>
         <td>Test</td>
         <td>Test</td>
      </tr>
      <tr>
         <td>Test</td>
         <td>Test</td>
      </tr>
      <tr>
         <td>Test</td>
         <td>Test</td>
      </tr>
      <tr>
         <td>Test</td>
         <td>Test</td>
      </tr>
   </tbody>
</table>
</body>
</html>"#;
    let inlined = inline(html).expect("Failed to inline");
    assert_eq!(
        inlined,
        r#"<html><head>

</head>
<body>
<table>
   <tbody>
      <tr>
         <td style="background-color: grey;">Test</td>
         <td style="background-color: grey;">Test</td>
      </tr>
      <tr>
         <td>Test</td>
         <td>Test</td>
      </tr>
      <tr>
         <td style="background-color: grey;">Test</td>
         <td style="background-color: grey;">Test</td>
      </tr>
      <tr>
         <td>Test</td>
         <td>Test</td>
      </tr>
   </tbody>
</table>

</body></html>"#
    );
}

#[test]
#[cfg(feature = "stylesheet-cache")]
fn test_cache() {
    use std::{
        num::NonZeroUsize,
        sync::{Arc, Mutex},
    };

    let html = r#"
<html>
<head>
<link href="http://127.0.0.1:1234/external.css" rel="stylesheet">
<style>
h2 { color: red; }
</style>
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>"#;

    #[derive(Debug, Default)]
    pub struct CustomStylesheetResolver {
        hits: Arc<Mutex<usize>>,
    }

    impl css_inline::StylesheetResolver for CustomStylesheetResolver {
        fn retrieve(&self, _: &str) -> css_inline::Result<String> {
            let mut hits = self.hits.lock().expect("Lock is poisoned");
            *hits += 1;
            Ok("h1 { color: blue; }".to_string())
        }
    }

    let hits = Arc::new(Mutex::new(0));

    let inliner = CSSInliner::options()
        .resolver(Arc::new(CustomStylesheetResolver { hits: hits.clone() }))
        .cache(css_inline::StylesheetCache::new(
            NonZeroUsize::new(3).unwrap(),
        ))
        .build();
    for _ in 0..5 {
        let inlined = inliner.inline(html);
        let expected = r#"<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>"#;
        assert!(inlined.expect("Inlining failed").ends_with(expected));
    }

    let hits = hits.lock().expect("Lock is poisoned");
    assert_eq!(*hits, 1);
}

#[test]
#[cfg(feature = "stylesheet-cache")]
fn test_disable_cache() {
    use std::num::NonZeroUsize;

    let inliner = CSSInliner::options()
        .cache(css_inline::StylesheetCache::new(
            NonZeroUsize::new(3).unwrap(),
        ))
        .cache(None)
        .build();
    let debug = format!("{:?}", inliner);
    assert_eq!(debug, "CSSInliner { options: InlineOptions { inline_style_tags: true, keep_style_tags: false, keep_link_tags: false, base_url: None, load_remote_stylesheets: true, cache: None, extra_css: None, preallocate_node_capacity: 32, .. } }");
}

#[test]
fn test_resolver_without_implementation() {
    let html = r#"
<html>
<head>
<link href="http://127.0.0.1:1234/external.css" rel="stylesheet">
</head>
</html>"#;

    #[derive(Debug, Default)]
    pub struct CustomStylesheetResolver;

    impl css_inline::StylesheetResolver for CustomStylesheetResolver {}

    let inliner = CSSInliner::options()
        .resolver(Arc::new(CustomStylesheetResolver))
        .build();

    let error = inliner.inline(html).expect_err("Should fail");
    #[cfg(feature = "http")]
    {
        assert_eq!(
            error.to_string(),
            "Loading external URLs is not supported: http://127.0.0.1:1234/external.css"
        );
    }
    #[cfg(not(feature = "http"))]
    {
        assert_eq!(
            error.to_string(),
            "Loading external URLs requires the `http` feature"
        );
    }
    assert!(error.source().is_some());
}

const FRAGMENT: &str = r#"<main>
<h1>Hello</h1>
<section>
<p>who am i</p>
</section>
</main>"#;
const CSS: &str = r#"
p {
    color: red;
}

h1 {
    color: blue;
}
"#;
const EXPECTED_INLINED_FRAGMENT: &str = "<main>\n<h1 style=\"color: blue;\">Hello</h1>\n<section>\n<p style=\"color: red;\">who am i</p>\n</section>\n</main>";

#[test]
fn inline_fragment() {
    let inlined = css_inline::inline_fragment(FRAGMENT, CSS).unwrap();
    assert_eq!(inlined, EXPECTED_INLINED_FRAGMENT);
}

#[test]
fn inline_fragment_to() {
    let mut out = Vec::new();
    css_inline::inline_fragment_to(FRAGMENT, CSS, &mut out).unwrap();
    assert_eq!(String::from_utf8_lossy(&out), EXPECTED_INLINED_FRAGMENT)
}

#[test]
fn inline_fragment_empty() {
    let inlined = css_inline::inline_fragment("", "").unwrap();
    assert_eq!(inlined, "");
}
