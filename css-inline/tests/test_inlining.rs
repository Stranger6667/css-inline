#[macro_use]
mod utils;
use css_inline::{inline, CSSInliner, InlineOptions, Url};

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
    let option_1 = html!(
        style,
        r#"<h1 style="font-size: 1px;color:red;">Big Text</h1>"#
    );
    let option_2 = html!(
        style,
        r#"<h1 style="color:red;font-size: 1px;">Big Text</h1>"#
    );
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
fn existing_styles() {
    // When there is a `style` attribute on a tag that contains a rule
    // And the `style` tag contains the same rule applicable to that tag
    assert_inlined!(
        style = "h1 { color: red; }",
        body = r#"<h1 style="color: blue">Hello world!</h1>"#,
        // Then the existing rule should be preferred
        expected = r#"<h1 style="color: blue;">Hello world!</h1>"#
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
        expected = r#"<h1 style="color: blue;font-size:14px;">Hello world!</h1>"#
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

#[test]
fn invalid_rule() {
    let html = html!(
        "h1 {background-color: blue;}",
        r#"<h1 style="@wrong { color: ---}">Hello world!</h1>"#
    );
    let result = inline(&html);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Invalid @ rule: wrong")
}

#[test]
fn remove_style_tag() {
    let html = html!("h1 {background-color: blue;}", "<h1>Hello world!</h1>");
    let inliner = CSSInliner::compact();
    let result = inliner.inline(&html).unwrap();
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
    let inliner = CSSInliner::compact();
    let result = inliner.inline(html).unwrap();
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
        .remove_style_tags(true)
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
    let html = html!("h1 {background-color: blue;}", "<h1>Hello world!</h1>");
    let options = InlineOptions {
        inline_style_tags: false,
        ..Default::default()
    };
    let inliner = CSSInliner::new(options);
    let result = inliner.inline(&html).unwrap();
    assert_eq!(
        result,
        "<html><head><title>Test</title><style>h1 {background-color: blue;}</style></head><body><h1>Hello world!</h1></body></html>"
    )
}

#[test]
fn do_not_process_style_tag_and_remove() {
    let html = html!("h1 {background-color: blue;}", "<h1>Hello world!</h1>");
    let options = InlineOptions {
        remove_style_tags: true,
        inline_style_tags: false,
        ..Default::default()
    };
    let inliner = CSSInliner::new(options);
    let result = inliner.inline(&html).unwrap();
    assert_eq!(
        result,
        "<html><head><title>Test</title></head><body><h1>Hello world!</h1></body></html>"
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
        "<html><head><title>Test</title><style>h1 {background-color: blue;}</style></head><body><h1 style=\"background-color: green;\">Hello world!</h1></body></html>"
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
    let result = inline(html).unwrap();
    assert!(result.ends_with(
        r#"<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>"#
    ))
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
    let result = inline(html).unwrap();
    assert!(result.ends_with(
        r#"<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>"#
    ))
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
    let result = inline(html).unwrap();
    assert!(result.ends_with(
        r#"<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>"#
    ))
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
    let options = InlineOptions {
        base_url: Some(Url::parse("http://127.0.0.1:5000").unwrap()),
        ..Default::default()
    };
    let inliner = CSSInliner::new(options);
    let result = inliner.inline(html).unwrap();
    assert!(result.ends_with(
        r#"<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>"#
    ))
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
    let options = InlineOptions {
        base_url: Some(Url::parse("http://127.0.0.1:5000").unwrap()),
        ..Default::default()
    };
    let inliner = CSSInliner::new(options);
    let result = inliner.inline(html).unwrap();
    assert!(result.ends_with(
        r#"<body>
<h1 style="color: blue;">Big Text</h1>
<h2 style="color: red;">Smaller Text</h2>

</body></html>"#
    ))
}

#[test]
fn customize_inliner() {
    let options = InlineOptions {
        load_remote_stylesheets: false,
        ..Default::default()
    };
    assert!(!options.load_remote_stylesheets);
    assert!(!options.remove_style_tags);
    assert_eq!(options.base_url, None);
}

#[test]
fn use_builder() {
    let url = Url::parse("https://api.example.com").unwrap();
    let _ = CSSInliner::options()
        .inline_style_tags(false)
        .remove_style_tags(false)
        .base_url(Some(url))
        .load_remote_stylesheets(false)
        .extra_css(Some("h1 {color: green}".into()))
        .build();
}
