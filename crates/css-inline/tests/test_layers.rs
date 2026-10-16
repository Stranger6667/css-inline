#[macro_use]
mod utils;

use css_inline::{inline, inline_fragment, CSSInliner};

fn body(html: &str) -> &str {
    let start = html.find("<body>").expect("no body") + "<body>".len();
    let end = html.rfind("</body>").expect("no body end");
    &html[start..end]
}

#[test]
fn issue_example() {
    // Unlayered `color` wins over the layered one despite lower specificity.
    assert_inlined!(
        style =
            "@layer legacy { div > span { font-weight: bold; color: red; } } span { color: blue; }",
        body = "<div><span></span></div>",
        expected = r#"<div><span style="font-weight: bold;color: blue;"></span></div>"#
    )
}

#[test]
fn statement_defines_order() {
    // `b` is declared before `a`, so `a` wins regardless of block order.
    assert_inlined!(
        style = "@layer b, a; @layer a { p { color: red } } @layer b { p { color: blue } }",
        body = "<p></p>",
        expected = r#"<p style="color: red;"></p>"#
    )
}

#[test]
fn block_order_without_statement() {
    assert_inlined!(
        style = "@layer a { p { color: red } } @layer b { p { color: blue } }",
        body = "<p></p>",
        expected = r#"<p style="color: blue;"></p>"#
    )
}

#[test]
fn important_reverses_layer_order() {
    assert_inlined!(
        style =
            "@layer a { p { color: red !important } } @layer b { p { color: blue !important } }",
        body = "<p></p>",
        expected = r#"<p style="color: red !important;"></p>"#
    )
}

#[test]
fn layered_important_beats_unlayered_important() {
    assert_inlined!(
        style = "@layer a { p { color: red !important } } p { color: blue !important }",
        body = "<p></p>",
        expected = r#"<p style="color: red !important;"></p>"#
    )
}

#[test]
fn layered_important_beats_unlayered_normal() {
    assert_inlined!(
        style = "@layer a { p { color: red !important } } p { color: blue }",
        body = "<p></p>",
        expected = r#"<p style="color: red !important;"></p>"#
    )
}

#[test]
fn unlayered_important_beats_layered_normal() {
    assert_inlined!(
        style = "p { color: blue !important } @layer a { #x p { color: red } }",
        body = "<div id=\"x\"><p></p></div>",
        expected = r#"<div id="x"><p style="color: blue !important;"></p></div>"#
    )
}

#[test]
fn nested_and_dotted_are_the_same_layer() {
    // Both rules are in `a.b`; the later one wins.
    assert_inlined!(
        style = "@layer a { @layer b { p { color: red } } } @layer a.b { p { color: blue } }",
        body = "<p></p>",
        expected = r#"<p style="color: blue;"></p>"#
    );
    assert_inlined!(
        style = "@layer a.b { p { color: red } } @layer a { @layer b { p { color: blue } } }",
        body = "<p></p>",
        expected = r#"<p style="color: blue;"></p>"#
    )
}

#[test]
fn parent_rules_beat_sublayers() {
    assert_inlined!(
        style = "@layer a { @layer b { p { color: red } } p { color: blue } }",
        body = "<p></p>",
        expected = r#"<p style="color: blue;"></p>"#
    );
    assert_inlined!(
        style = "@layer a { p { color: blue } @layer b { #x p { color: red } } }",
        body = "<div id=\"x\"><p></p></div>",
        expected = r#"<div id="x"><p style="color: blue;"></p></div>"#
    )
}

#[test]
fn sublayers_lose_to_parent_important_reversed() {
    // Important declarations reverse the whole tree: `a.b` beats `a`.
    assert_inlined!(
        style =
            "@layer a { @layer b { p { color: red !important } } p { color: blue !important } }",
        body = "<p></p>",
        expected = r#"<p style="color: red !important;"></p>"#
    )
}

#[test]
fn sibling_layer_beats_whole_subtree() {
    assert_inlined!(
        style = "@layer a { @layer b { #x p { color: red } } } @layer c { p { color: green } }",
        body = "<div id=\"x\"><p></p></div>",
        expected = r#"<div id="x"><p style="color: green;"></p></div>"#
    )
}

#[test]
fn anonymous_layers_are_distinct() {
    assert_inlined!(
        style = "@layer { #x p { color: red } } @layer { p { color: blue } }",
        body = "<div id=\"x\"><p></p></div>",
        expected = r#"<div id="x"><p style="color: blue;"></p></div>"#
    );
    assert_inlined!(
        style = "@layer a { p { color: red } } @layer { p { color: blue } } @layer a { p { color: green } }",
        body = "<p></p>",
        expected = r#"<p style="color: blue;"></p>"#
    );
    assert_inlined!(
        style = "@layer { #x p { color: red } } p { color: green }",
        body = "<div id=\"x\"><p></p></div>",
        expected = r#"<div id="x"><p style="color: green;"></p></div>"#
    )
}

#[test]
fn reopened_layer_keeps_its_position() {
    assert_inlined!(
        style = "@layer a { p { color: red } } @layer b { p { color: blue } } @layer a { p { color: green } }",
        body = "<p></p>",
        expected = r#"<p style="color: blue;"></p>"#
    )
}

#[test]
fn unlayered_beats_higher_specificity_layered() {
    assert_inlined!(
        style = "@layer a { #x .y p { color: red; margin: 0 } } p { color: blue }",
        body = "<div id=\"x\"><div class=\"y\"><p></p></div></div>",
        expected =
            r#"<div id="x"><div class="y"><p style="margin: 0;color: blue;"></p></div></div>"#
    )
}

#[test]
fn layer_order_spans_style_tags_and_extra_css() {
    let inliner = CSSInliner::options()
        .extra_css(Some(
            "@layer b { p { color: blue } } @layer a { p { color: green } }".into(),
        ))
        .build();
    let html = html!("@layer a { p { color: red } }", "<p></p>");
    let inlined = inliner.inline(&html).unwrap();
    assert_eq!(body(&inlined), r#"<p style="color: blue;"></p>"#);

    // A statement in the first `<style>` orders layers declared later in `extra_css`.
    let html = html!(
        "@layer b, a; @layer a { p { color: red } }",
        "<style>@layer b { p { color: blue } }</style><p></p>"
    );
    let inlined = inliner.inline(&html).unwrap();
    assert_eq!(body(&inlined), r#"<p style="color: green;"></p>"#);
}

#[test]
fn fragment_mode() {
    let inlined = inline_fragment(
        "<p></p>",
        "@layer a { #x p { color: red } } p { color: blue }",
    )
    .unwrap();
    assert_eq!(inlined, r#"<p style="color: blue;"></p>"#);

    // `<style>` content comes before the `css` argument in the cascade.
    let inlined = inline_fragment(
        "<style>@layer a { p { color: red } }</style><p></p>",
        "@layer b { p { color: blue } } @layer a { p { color: green } }",
    )
    .unwrap();
    assert_eq!(inlined, r#"<p style="color: blue;"></p>"#);
}

#[test]
fn inline_style_attribute_beats_layered_rules() {
    assert_inlined!(
        style = "@layer a { p { color: red } } p { color: blue }",
        body = r#"<p style="color: green"></p>"#,
        expected = r#"<p style="color: green"></p>"#
    );
    // Important stylesheet declarations beat normal element-attached ones.
    assert_inlined!(
        style = "@layer a { p { color: red !important } }",
        body = r#"<p style="color: green"></p>"#,
        expected = r#"<p style="color: red !important"></p>"#
    )
}

#[test]
fn media_inside_layer_does_not_break_siblings() {
    assert_inlined!(
        style = "@layer a { p { color: red } @media (hover: hover) { p:hover { color: blue } } p { margin: 0 } }",
        body = "<p></p>",
        expected = r#"<p style="color: red;margin: 0;"></p>"#
    )
}

#[test]
fn invalid_layer_rules_are_ignored() {
    // `@layer a, b { }` and `@layer;` are invalid; `@layer a .b {}` has a bad name.
    assert_inlined!(
        style = "@layer a, b { p { color: red } } @layer; @layer a .b { p { color: red } } p { margin: 0 }",
        body = "<p></p>",
        expected = r#"<p style="margin: 0;"></p>"#
    )
}

#[test]
fn keep_at_rules_wraps_nested_at_rules_in_their_layer() {
    let inliner = CSSInliner::options().keep_at_rules(true).build();
    let html = html!(
        "@layer base, utilities; @layer base { h1 { color: blue } } @layer utilities { .flex { display: flex } @media (hover: hover) { .x:hover { color: red } } }",
        r#"<h1 class="flex"></h1>"#
    );
    let inlined = inliner.inline(&html).unwrap();
    assert_eq!(
        inlined,
        r#"<html><head><style>@layer base, utilities; @layer utilities { @media (hover: hover) { .x:hover { color: red } } } </style></head><body><h1 class="flex" style="color: blue;display: flex;"></h1></body></html>"#
    );
}

#[test]
fn keep_at_rules_declares_layers_whose_blocks_become_empty() {
    // `a` first appears as a block with no `@`-rules; its position still matters for `b`.
    let inliner = CSSInliner::options().keep_at_rules(true).build();
    let html = html!(
        "@layer a { h1 { color: blue } } @layer b { @media print { h1 { color: red } } } @layer a { @media print { h1 { color: green } } }",
        "<h1></h1>"
    );
    let inlined = inliner.inline(&html).unwrap();
    assert_eq!(
        inlined,
        r#"<html><head><style>@layer a; @layer b { @media print { h1 { color: red } } } @layer a { @media print { h1 { color: green } } } </style></head><body><h1 style="color: blue;"></h1></body></html>"#
    );
}

#[test]
fn keep_at_rules_drops_layers_without_at_rules() {
    let inliner = CSSInliner::options().keep_at_rules(true).build();
    let html = html!(
        "@layer a, b; @layer a { h1 { color: blue } } @layer { h1 { margin: 0 } } @media print { h1 { color: red } }",
        "<h1></h1>"
    );
    let inlined = inliner.inline(&html).unwrap();
    assert_eq!(
        inlined,
        r#"<html><head><style>@media print { h1 { color: red } } </style></head><body><h1 style="color: blue;margin: 0;"></h1></body></html>"#
    );
}

#[test]
fn keep_at_rules_nested_layers() {
    let inliner = CSSInliner::options().keep_at_rules(true).build();
    let html = html!(
        "@layer a { @layer b { @media print { h1 { color: red } } } h1 { color: blue } } @layer { @media print { h1 { color: green } } }",
        "<h1></h1>"
    );
    let inlined = inliner.inline(&html).unwrap();
    assert_eq!(
        inlined,
        r#"<html><head><style>@layer a { @layer b { @media print { h1 { color: red } } } } @layer { @media print { h1 { color: green } } } </style></head><body><h1 style="color: blue;"></h1></body></html>"#
    );
}

#[test]
fn remove_inlined_selectors_inside_layers() {
    let inliner = CSSInliner::options().remove_inlined_selectors(true).build();
    let html = r#"<html><head><style>@layer a { h1 { color: blue; } h1:hover { color: red; } } @layer b { p { margin: 0; } }</style></head><body><h1></h1><p></p></body></html>"#;
    let inlined = inliner.inline(html).unwrap();
    assert_eq!(
        inlined,
        "<html><head><style>@layer a {\nh1:hover { color: red; }\n}</style></head><body><h1 style=\"color: blue;\"></h1><p style=\"margin: 0;\"></p></body></html>"
    );
}

#[test]
fn remove_inlined_selectors_keeps_layer_order() {
    // `a` is emptied but declared first, so `b`'s hover rule must still lose to `a`'s.
    let inliner = CSSInliner::options().remove_inlined_selectors(true).build();
    let html = r#"<html><head><style>@layer a { h1 { color: blue; } } @layer b { h1:hover { color: red; } } @layer a { h1:hover { color: green; } }</style></head><body><h1></h1></body></html>"#;
    let inlined = inliner.inline(html).unwrap();
    assert_eq!(
        inlined,
        "<html><head><style>@layer a;\n@layer b {\nh1:hover { color: red; }\n}\n@layer a {\nh1:hover { color: green; }\n}</style></head><body><h1 style=\"color: blue;\"></h1></body></html>"
    );
}

#[test]
fn remove_inlined_selectors_drops_empty_layers_and_statements() {
    let inliner = CSSInliner::options().remove_inlined_selectors(true).build();
    let html = r#"<html><head><style>@layer a, b; @layer a { h1 { color: blue; } } @layer b { @layer c { h1 { margin: 0; } } }</style></head><body><h1></h1></body></html>"#;
    let inlined = inliner.inline(html).unwrap();
    assert_eq!(
        inlined,
        "<html><head></head><body><h1 style=\"color: blue;margin: 0;\"></h1></body></html>"
    );
}

#[test]
fn remove_inlined_selectors_with_at_rules_inside_layers() {
    let inliner = CSSInliner::options()
        .remove_inlined_selectors(true)
        .keep_at_rules(true)
        .build();
    let html = r#"<html><head><style>@layer a, b; @layer a { h1 { color: blue; } } @layer b { @media print { h1 { color: red; } } .x { color: green; } }</style></head><body><h1></h1></body></html>"#;
    let inlined = inliner.inline(html).unwrap();
    assert_eq!(
        inlined,
        "<html><head><style>@layer a, b; @layer b { @media print { h1 { color: red; } } } </style><style>@layer a, b;\n@layer b {\n@media print { h1 { color: red; } }\n.x { color: green; }\n}</style></head><body><h1 style=\"color: blue;\"></h1></body></html>"
    );
}

#[test]
fn tailwind_v4_excerpt() {
    let css = r#"/*! tailwindcss v4.3.3 | MIT License | https://tailwindcss.com */
@layer theme, base, components, utilities;
@layer theme {
  :root, :host {
    --spacing: 0.25rem;
    --color-blue-500: oklch(62.3% 0.214 259.815);
  }
}
@layer base {
  h1, h2, h3, h4, h5, h6 {
    font-size: inherit;
    font-weight: inherit;
  }
  b, strong {
    font-weight: bolder;
  }
  [hidden]:where(:not([hidden="until-found"])) {
    display: none !important;
  }
}
@layer utilities {
  .flex {
    display: flex;
  }
  .p-4 {
    padding: calc(var(--spacing) * 4);
  }
  .font-bold {
    font-weight: 700;
  }
  @media (hover: hover) {
    .hover\:text-blue-500:hover {
      color: var(--color-blue-500);
    }
  }
}
"#;
    let html = format!(
        r#"<html><head><style>{css}</style></head><body><h1 class="font-bold">T</h1><div class="flex p-4"><strong hidden="">S</strong></div></body></html>"#
    );
    let inlined = inline(&html).unwrap();
    assert_eq!(
        inlined,
        // `[hidden]:where(...)` is not inlined: `:where()` is unsupported.
        r#"<html style="--spacing: 0.25rem;--color-blue-500: oklch(62.3% 0.214 259.815);"><head></head><body><h1 class="font-bold" style="font-size: inherit;font-weight: 700;">T</h1><div class="flex p-4" style="display: flex;padding: calc(var(--spacing) * 4);"><strong hidden="" style="font-weight: bolder;">S</strong></div></body></html>"#
    );

    let inliner = CSSInliner::options().keep_at_rules(true).build();
    let inlined = inliner.inline(&html).unwrap();
    assert!(
        inlined.starts_with(r#"<html style="--spacing: 0.25rem;--color-blue-500: oklch(62.3% 0.214 259.815);"><head><style>@layer theme, base, components, utilities; @layer utilities { @media (hover: hover) {
    .hover\:text-blue-500:hover {
      color: var(--color-blue-500);
    }
  } } </style></head>"#),
        "{inlined}"
    );
}

#[test]
fn layers_declared_inside_conditional_rules_are_ordered() {
    // Conditions are not evaluated; `b` is declared first either way.
    assert_inlined!(
        style = "@media all { @layer b { } } @layer a { p { color: red } } @layer b { p { color: green } }",
        body = "<p></p>",
        expected = r#"<p style="color: red;"></p>"#
    );
    assert_inlined!(
        style = "@supports (display: flex) { @layer b { } } @layer a { p { color: red } } @layer b { p { color: green } }",
        body = "<p></p>",
        expected = r#"<p style="color: red;"></p>"#
    );
    // Statements, nesting and dotted names inside the condition count too.
    assert_inlined!(
        style = "@media all { @layer c, b; } @layer b { p { color: red } } @layer c { p { color: green } }",
        body = "<p></p>",
        expected = r#"<p style="color: red;"></p>"#
    );
    assert_inlined!(
        style = "@layer a { @media all { @layer c { @layer d { } } } @layer b { p { color: green } } @layer c.d { p { color: red } } }",
        body = "<p></p>",
        expected = r#"<p style="color: green;"></p>"#
    );
}

#[test]
fn rules_inside_conditional_rules_are_not_inlined() {
    assert_inlined!(
        style = "@media all { @layer b { p { margin: 0 } } p { padding: 0 } } p { color: red }",
        body = "<p></p>",
        expected = r#"<p style="color: red;"></p>"#
    )
}

#[test]
fn conditional_rules_declaring_layers_are_kept_verbatim() {
    let inliner = CSSInliner::options().keep_at_rules(true).build();
    let html = html!(
        "@media all { @layer b { p { color: blue } } } @layer a { p { color: red } } @layer b { p { color: green } }",
        "<p></p>"
    );
    assert_eq!(
        inliner.inline(&html).unwrap(),
        r#"<html><head><style>@media all { @layer b { p { color: blue } } } </style></head><body><p style="color: red;"></p></body></html>"#
    );

    let inliner = CSSInliner::options().remove_inlined_selectors(true).build();
    let html = r#"<html><head><style>@media all { @layer b { } } @layer a { h1 { color: red; } h1:hover { color: blue; } }</style></head><body><h1></h1></body></html>"#;
    assert_eq!(
        inliner.inline(html).unwrap(),
        "<html><head><style>@media all { @layer b { } }\n@layer a {\nh1:hover { color: blue; }\n}</style></head><body><h1 style=\"color: red;\"></h1></body></html>"
    );
}
