#[macro_export]
macro_rules! html {
    ($style: expr, $body: expr) => {
        format!(
            r#"<html><head><title>Test</title><style>{}</style></head><body>{}</body></html>"#,
            $style, $body
        )
    };
}

#[macro_export]
macro_rules! assert_inlined {
    (style = $style: expr, body = $body: expr, expected = $expected: expr) => {{
        let html = html!($style, $body);
        let inlined = css_inline::inline(&html).unwrap();
        assert_eq!(inlined, html!($style, $expected))
    }};
}
