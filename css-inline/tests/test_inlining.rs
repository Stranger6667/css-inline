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
        assert_eq!(
            inlined.expect_err("Should fail").to_string(),
            "Loading external URLs requires the `http` feature"
        );
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
        expected = r#"<h1 style="color:red;">Big Text</h1>
<p style="font-size:2px ;"><strong style="text-decoration:none ;">Yes!</strong></p>
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
<style type="text/css" data-css-inline="ignore">
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
fn ignore_inlining_attribute_link() {
    // When a `link` tag contains `data-css-inline="ignore"`
    let html = r#"
<html>
<head>
<link href="tests/external.css" rel="stylesheet" type="text/css" data-css-inline="ignore">
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
        body = r#"<a class="test-class" href="https://example.com">Test</a>"#,
        // Then the final style should come from the more specific selector
        expected = r#"<a class="test-class" href="https://example.com" style="padding-top: 15px;padding: 10px;padding-left: 12px;">Test</a>"#
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
        expected = r#"<h1 class="test" style="color: blue;padding: 0;padding-left: 16px"></h1>"#
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
fn important_no_rule_exists() {
    // `!important` rules should override existing inline styles
    assert_inlined!(
        style = "h1 { color: blue !important; }",
        body = r#"<h1 style="margin:0">Big Text</h1>"#,
        expected = r#"<h1 style="margin: 0;color: blue">Big Text</h1>"#
    )
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
fn href_attribute_unchanged() {
    // All HTML attributes should be serialized as is
    let html = r#"<html>
<head>
    <title>Test</title>
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
    <title>Test</title>
    
</head>
<body>
    <h1 style="color:blue;">Big Text</h1>
    <a href="https://example.org/test?a=b&c=d">Link</a>

</body></html>"#
    );
}

#[test]
fn complex_child_selector() {
    let html = r#"<html>
   <head>
      <title>Test</title>
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
      <title>Test</title>
      
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
fn existing_styles_with_merge() {
    // When there is a `style` attribute on a tag that contains a rule
    // And the `style` tag contains the same rule applicable to that tag
    // And there is a new rule in the `style` tag
    assert_inlined!(
        style = "h1 { color: red; font-size:14px; }",
        body = r#"<h1 style="color: blue">Hello world!</h1>"#,
        // Then the existing rule should be preferred
        // And the new style should be merged
        expected = r#"<h1 style="color: blue;font-size: 14px">Hello world!</h1>"#
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
    let html = html!("h1 {background-color: blue;}", "<h1>Hello world!</h1>");
    let result = inline(&html).unwrap();
    assert_eq!(result, "<html><head><title>Test</title></head><body><h1 style=\"background-color: blue;\">Hello world!</h1></body></html>")
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
fn extra_css() {
    let html = html!("h1 {background-color: blue;}", "<h1>Hello world!</h1>");
    let inliner = CSSInliner::options()
        .extra_css(Some("h1 {background-color: green;}".into()))
        .build();
    let result = inliner.inline(&html).unwrap();
    assert_eq!(
        result,
        "<html><head><title>Test</title></head><body><h1 style=\"background-color: green;\">Hello world!</h1></body></html>"
    )
}

#[test]
fn remote_file_stylesheet() {
    let html = r#"
<html>
<head>
<link href="tests/external.css" rel="stylesheet" type="text/css">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style type="text/css">
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
<link href="tests/missing.css" rel="stylesheet" type="text/css">
</head>
<body>
<h1>Big Text</h1>
</body>
</html>"#;
    let inlined = inline(html);
    #[cfg(feature = "file")]
    {
        assert_eq!(
            inlined.expect_err("Should be an error").to_string(),
            "Missing stylesheet file: tests/missing.css"
        );
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
<link href="tests/external.css" rel="stylesheet" type="text/css">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style type="text/css">
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
<link href="http://127.0.0.1:5000/external.css" rel="stylesheet" type="text/css">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style type="text/css">
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
<link href="http:" rel="stylesheet" type="text/css">
</head>
<body>
</body>
</html>"#;
    assert!(inline(html).is_err());
}

#[test]
fn remote_network_stylesheet_same_scheme() {
    let html = r#"
<html>
<head>
<link href="//127.0.0.1:5000/external.css" rel="stylesheet" type="text/css">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style type="text/css">
h2 { color: red; }
</style>
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>"#;
    let inliner = CSSInliner::options()
        .base_url(Some(Url::parse("http://127.0.0.1:5000").unwrap()))
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
<link href="external.css" rel="stylesheet" type="text/css">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style type="text/css">
h2 { color: red; }
</style>
</head>
<body>
<h1>Big Text</h1>
<h2>Smaller Text</h2>
</body>
</html>"#;
    let inliner = CSSInliner::options()
        .base_url(Some(Url::parse("http://127.0.0.1:5000").unwrap()))
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
<link href="external.css" rel="stylesheet" type="text/css">
<link rel="alternate" type="application/rss+xml" title="RSS" href="/rss.xml">
<style type="text/css">
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
    assert_eq!(String::from_utf8_lossy(&out), "<html><head><title>Test</title></head><body><h1 style=\"color: blue ;\">Big Text</h1></body></html>")
}

#[test]
fn keep_style_tags() {
    let inliner = CSSInliner::options().keep_style_tags(true).build();
    let html = r#"
<html>
<head>
<style type="text/css">
h2 { color: red; }
</style>
</head>
<body>
<h2></h2>
</body>
</html>"#;
    let inlined = inliner.inline(html).unwrap();
    assert_eq!(inlined, "<html><head>\n<style type=\"text/css\">\nh2 { color: red; }\n</style>\n</head>\n<body>\n<h2 style=\"color: red;\"></h2>\n\n</body></html>");
}

#[test]
fn keep_link_tags() {
    let inliner = CSSInliner::options()
        .base_url(Some(Url::parse("http://127.0.0.1:5000").unwrap()))
        .keep_link_tags(true)
        .build();
    let html = r#"
<html>
<head>
<link href="external.css" rel="stylesheet" type="text/css">
</head>
<body>
<h1></h1>
</body>
</html>"#;
    let inlined = inliner.inline(html);
    assert_http(
        inlined,
        "<html><head>\n<link href=\"external.css\" rel=\"stylesheet\" type=\"text/css\">\n</head>\n<body>\n<h1 style=\"color: blue;\"></h1>\n\n</body></html>",
    );
}
