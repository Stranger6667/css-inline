use css_inline::inline;

macro_rules! html {
    ($style: expr, $body: expr) => {
        format!(
            r#"<html><head><title>Test</title><style>{}</style></head><body>{}</body></html>"#,
            $style, $body
        )
    };
}

macro_rules! assert_inlined {
    (style = $style: expr, body = $body: expr, expected = $expected: expr) => {{
        let html = html!($style, $body);
        let inlined = inline(&html).unwrap();
        assert_eq!(inlined, html!($style, $expected))
    }};
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
    assert!(valid, inlined);
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
