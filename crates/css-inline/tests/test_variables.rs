#[macro_use]
mod utils;

use css_inline::CSSInliner;
use test_case::test_case;

fn resolve(html: &str) -> String {
    CSSInliner::options()
        .resolve_css_variables(true)
        .build()
        .inline(html)
        .expect("Inlining failed")
}

macro_rules! assert_resolved {
    (style = $style: expr, body = $body: expr, expected = $expected: expr) => {{
        assert_eq!(resolve(&html!($style, $body)), html!($expected))
    }};
}

#[test]
fn option_off_keeps_variables() {
    // The default output is unchanged: `--c` lands on `<html>` and `var()` stays.
    let html = html!(":root{--c:red} p{color:var(--c)}", "<p>x</p>");
    assert_eq!(
        css_inline::inline(&html).unwrap(),
        r#"<html style="--c: red;"><head></head><body><p style="color: var(--c);">x</p></body></html>"#
    );
}

#[test]
fn root_variable() {
    assert_resolved!(
        style = ":root{--c:rgb(255,0,0)} p{color:var(--c)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: rgb(255,0,0);">x</p>"#
    )
}

#[test]
fn fallback_for_undefined() {
    assert_resolved!(
        style = "p{color:var(--x, rgb(0,128,0))}",
        body = "<p>x</p>",
        expected = r#"<p style="color: rgb(0,128,0);">x</p>"#
    )
}

#[test]
fn inherited_and_overridden() {
    assert_resolved!(
        style = ":root{--c:rgb(255,0,0)} .box{--c:rgb(0,0,255)} p{color:var(--c)}",
        body = r#"<div class="box"><p>x</p></div><p>y</p>"#,
        expected = r#"<div class="box"><p style="color: rgb(0,0,255);">x</p></div><p style="color: rgb(255,0,0);">y</p>"#
    )
}

#[test]
fn nested_reference() {
    assert_resolved!(
        style = ":root{--a:var(--b); --b:rgb(0,128,0)} p{color:var(--a)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: rgb(0,128,0);">x</p>"#
    )
}

#[test]
fn chain_of_references() {
    assert_resolved!(
        style = ":root{--a:var(--b);--b:var(--c);--c:var(--d);--d:red} p{color:var(--a)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: red;">x</p>"#
    )
}

#[test]
fn calc_argument() {
    assert_resolved!(
        style = ":root{--s:4px} p{padding-left:calc(var(--s) * 4)}",
        body = "<p>x</p>",
        expected = r#"<p style="padding-left: calc(4px * 4);">x</p>"#
    )
}

#[test]
fn cycle_uses_fallback() {
    assert_resolved!(
        style = ":root{--a:var(--b);--b:var(--a)} p{color:var(--a, rgb(255,0,0))}",
        body = "<p>x</p>",
        expected = r#"<p style="color: rgb(255,0,0);">x</p>"#
    )
}

#[test]
fn cycle_without_fallback_drops_declaration() {
    assert_resolved!(
        style = ":root{--a:var(--b);--b:var(--a)} p{color:var(--a); margin: 0}",
        body = "<p>x</p>",
        expected = r#"<p style="margin: 0;">x</p>"#
    )
}

#[test]
fn self_reference_is_a_cycle() {
    assert_resolved!(
        style = ":root{--a:var(--a)} p{color:var(--a, blue)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: blue;">x</p>"#
    )
}

#[test]
fn cycle_member_with_fallback_is_invalid() {
    // Every custom property in a cycle is invalid, even the ones with a fallback.
    assert_resolved!(
        style = ":root{--a:var(--b, red);--b:var(--a)} p{color:var(--a, blue)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: blue;">x</p>"#
    )
}

#[test]
fn unused_fallback_does_not_form_a_cycle() {
    // `--c` is defined, so the fallback referencing `--b` is not used and creates no cycle.
    assert_resolved!(
        style = ":root{--a:var(--c, var(--b));--b:var(--a);--c:red} p{color:var(--a, blue)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: red;">x</p>"#
    )
}

#[test]
fn used_fallback_forms_a_cycle() {
    assert_resolved!(
        style = ":root{--a:var(--nope, var(--b));--b:var(--a)} p{color:var(--a, blue)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: blue;">x</p>"#
    )
}

#[test]
fn whole_chain_is_the_cycle() {
    assert_resolved!(
        style = ":root{--a:var(--b);--b:var(--c);--c:var(--a, red)} p{color:var(--a, blue); letter-spacing:var(--c, 7px)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: blue;letter-spacing: 7px;">x</p>"#
    )
}

#[test]
fn cycle_stops_resolution_of_its_members() {
    // `--c1` is a self-cycle, so its fallback is never evaluated and `--c3` is not pulled into
    // the cycle: it resolves to its own fallback.
    assert_resolved!(
        style = ":root{--c1:var(--c1, var(--c3, red));--c3:var(--c2, var(--c1, blue))} p{color:var(--c3); background:var(--c1, green)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: blue;background: green;">x</p>"#
    )
}

#[test]
fn important_inline_definition_beats_later_duplicate() {
    assert_resolved!(
        style = "p{color:var(--c)}",
        body = r#"<p style="--c: red !important; --c: blue">x</p>"#,
        expected = r#"<p style="color: red">x</p>"#
    );
    assert_resolved!(
        style = "p{--c:red !important; color:var(--c)}",
        body = r#"<p style="--c: var(--c, green); --c: blue">x</p>"#,
        expected = r#"<p style="color: red">x</p>"#
    )
}

#[test_case(r#"<p style="--c: red !IMPORTANT; --c: blue">x</p>"#; "uppercase")]
#[test_case(r#"<p style="--c: red ! important; --c: blue">x</p>"#; "space after bang")]
fn important_inline_definition_spelling(body: &str) {
    assert_resolved!(
        style = "p{color:var(--c)}",
        body = body,
        expected = r#"<p style="color: red">x</p>"#
    );
}

#[test]
fn inline_style_defines() {
    assert_resolved!(
        style = "",
        body = r#"<p style="--c: rgb(128,0,128); color: var(--c)">x</p>"#,
        expected = r#"<p style="color: rgb(128,0,128)">x</p>"#
    )
}

#[test]
fn inline_style_uses_root_variable() {
    assert_resolved!(
        style = ":root{--c:rgb(128,0,128)}",
        body = r#"<p style="color: var(--c)">x</p>"#,
        expected = r#"<p style="color: rgb(128,0,128)">x</p>"#
    )
}

#[test]
fn inline_definition_wins_over_stylesheet() {
    assert_resolved!(
        style = "p{--c:red; color:var(--c)}",
        body = r#"<p style="--c: blue">x</p>"#,
        expected = r#"<p style="color: blue">x</p>"#
    )
}

#[test]
fn important_stylesheet_definition_wins_over_inline() {
    assert_resolved!(
        style = "p{--c:red !important; color:var(--c)}",
        body = r#"<p style="--c: blue">x</p>"#,
        expected = r#"<p style="color: red">x</p>"#
    )
}

#[test]
fn inline_definition_visible_to_children() {
    assert_resolved!(
        style = "span{color:var(--c)}",
        body = r#"<p style="--c: blue"><span>x</span></p>"#,
        expected = r#"<p style=""><span style="color: blue;">x</span></p>"#
    )
}

#[test]
fn computed_in_defining_element() {
    // `--x` is computed on `.box`, where `--y` is red; `.inner` redefining `--y` does not change it.
    assert_resolved!(
        style = ".box{--x:var(--y); --y:rgb(255,0,0)} .inner{--y:rgb(0,0,255)} p{color:var(--x)}",
        body = r#"<div class="box"><div class="inner"><p>x</p></div></div>"#,
        expected = r#"<div class="box"><div class="inner"><p style="color: rgb(255,0,0);">x</p></div></div>"#
    )
}

#[test]
fn definitions_cascade_by_specificity() {
    assert_resolved!(
        style = ":root{--c:rgb(255,0,0)} html{--c:rgb(0,128,0)} p{color:var(--c)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: rgb(255,0,0);">x</p>"#
    )
}

#[test]
fn important_definition() {
    assert_resolved!(
        style = ":root{--c:rgb(255,0,0) !important} :root{--c:rgb(0,0,255)} p{color:var(--c)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: rgb(255,0,0);">x</p>"#
    )
}

#[test]
fn undefined_without_fallback_drops_declaration() {
    let html = html!("body{color:rgb(0,128,0)} p{color:var(--nope)}", "<p>x</p>");
    assert_eq!(
        resolve(&html),
        r#"<html><head></head><body style="color: rgb(0,128,0);"><p>x</p></body></html>"#
    );
}

#[test]
fn invalid_definition_shadows_inherited_value() {
    // `--c` on `.box` is invalid at computed-value time and `<p>` inherits that, not red.
    assert_resolved!(
        style = ":root{--c:red} .box{--c:var(--nope)} p{color:var(--c, blue)}",
        body = r#"<div class="box"><p>x</p></div>"#,
        expected = r#"<div class="box"><p style="color: blue;">x</p></div>"#
    )
}

#[test]
fn tailwind_excerpt() {
    assert_resolved!(
        style = ":root,:host{--spacing:0.25rem;--color-red-500:rgb(239,68,68)} .p-4{padding-left:calc(var(--spacing) * 4)} .text-red-500{color:var(--color-red-500)}",
        body = r#"<p class="p-4 text-red-500">x</p>"#,
        expected = r#"<p class="p-4 text-red-500" style="padding-left: calc(0.25rem * 4);color: rgb(239,68,68);">x</p>"#
    )
}

#[test]
fn tailwind_theme_layer() {
    let html = html!(
        r#"@layer theme, base, utilities;
@layer theme { :root, :host { --spacing: 0.25rem; --color-red-500: oklch(63.7% 0.237 25.331); --font-sans: Arial, sans-serif; --default-font-family: var(--font-sans); } }
@layer base { html, :host { line-height: 1.5; font-family: var(--default-font-family, Arial); } }
@layer utilities { .p-4 { padding: calc(var(--spacing) * 4); } .text-red-500 { color: var(--color-red-500); } }"#,
        r#"<div class="p-4 text-red-500">x</div>"#
    );
    assert_eq!(
        resolve(&html),
        r#"<html style="line-height: 1.5;font-family: Arial, sans-serif;"><head></head><body><div class="p-4 text-red-500" style="padding: calc(0.25rem * 4);color: oklch(63.7% 0.237 25.331);">x</div></body></html>"#
    );
}

#[test]
fn shorthand_with_partial_variable() {
    assert_resolved!(
        style = ":root{--w:3px} p{border:var(--w) solid rgb(255,0,0)}",
        body = "<p>x</p>",
        expected = r#"<p style="border: 3px solid rgb(255,0,0);">x</p>"#
    )
}

#[test]
fn media_definitions_are_not_visible() {
    // `@media` rules are not inlined, so definitions inside them do not take part.
    assert_resolved!(
        style = ":root{--c:rgb(255,0,0)} @media (min-width:1px){:root{--c:rgb(0,0,255)}} p{color:var(--c)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: rgb(255,0,0);">x</p>"#
    )
}

#[test]
fn names_are_case_sensitive() {
    assert_resolved!(
        style = ":root{--C:red} p{color:var(--c, blue)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: blue;">x</p>"#
    )
}

#[test]
fn function_name_is_case_insensitive() {
    assert_resolved!(
        style = ":root{--c:red} p{color:VAR(--c); background:Var(--c)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: red;background: red;">x</p>"#
    )
}

#[test]
fn whitespace_and_comments_inside_var() {
    assert_resolved!(
        style = ":root{--c:red} p{color:var( --c ); background:var(/*a*/--c/*b*/,/*c*/blue/*d*/); margin:var( --m , 1px )}",
        body = "<p>x</p>",
        expected = r#"<p style="color: red;background: red;margin: 1px;">x</p>"#
    )
}

#[test]
fn important_usage() {
    assert_resolved!(
        style = ":root{--c:red} p{color:var(--c) !important}",
        body = "<p>x</p>",
        expected = r#"<p style="color: red !important;">x</p>"#
    )
}

#[test]
fn important_is_not_part_of_the_value() {
    assert_resolved!(
        style = ":root{--c:red!important} p{color:var(--c)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: red;">x</p>"#
    )
}

#[test]
fn nested_fallbacks() {
    assert_resolved!(
        style = ":root{--y:green} p{color:var(--x, var(--y, blue)); background:var(--x, var(--z, blue))}",
        body = "<p>x</p>",
        expected = r#"<p style="color: green;background: blue;">x</p>"#
    )
}

#[test]
fn fallback_with_commas() {
    assert_resolved!(
        style = "p{color:var(--x, rgb(1, 2, 3)); font-family:var(--f, Arial, sans-serif)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: rgb(1, 2, 3);font-family: Arial, sans-serif;">x</p>"#
    )
}

#[test]
fn empty_fallback() {
    // `var(--x,)` is an empty fallback, which is not a valid color.
    assert_resolved!(
        style = ":root{--e:;} p{color:var(--x,); background:var(--e); margin:0}",
        body = "<p>x</p>",
        expected = r#"<p style="margin: 0;">x</p>"#
    )
}

#[test]
fn multiple_references() {
    assert_resolved!(
        style = ":root{--a:1px;--b:2px} p{margin:var(--a) var(--b) var(--a) var(--b)}",
        body = "<p>x</p>",
        expected = r#"<p style="margin: 1px 2px 1px 2px;">x</p>"#
    )
}

#[test]
fn strings_are_not_substituted() {
    assert_resolved!(
        style = ":root{--c:red} p{content:\"var(--c)\"; color:var(--c)}",
        body = "<p>x</p>",
        expected = r#"<p style="content: 'var(--c)';color: red;">x</p>"#
    )
}

#[test]
fn strings_and_comments_next_to_references() {
    assert_resolved!(
        style = ":root{--c:red} p{content:\"x(\" attr(y) var(--c); color:/*(*/var(--c)}",
        body = "<p>x</p>",
        expected = r#"<p style="content: 'x(' attr(y) red;color: /*(*/red;">x</p>"#
    )
}

#[test]
fn siblings_do_not_share_definitions() {
    assert_resolved!(
        style = ".a{--c:red} p{color:var(--c, blue)}",
        body = r#"<div class="a"><p>x</p></div><div><p>y</p></div>"#,
        expected = r#"<div class="a"><p style="color: red;">x</p></div><div><p style="color: blue;">y</p></div>"#
    )
}

#[test]
fn ignored_element_keeps_attribute_but_defines_for_children() {
    assert_resolved!(
        style = "p{color:var(--c)}",
        body = r#"<div data-css-inline="ignore" style="--c: blue; color: var(--c)"><p>x</p></div>"#,
        expected = r#"<div data-css-inline="ignore" style="--c: blue; color: var(--c)"><p style="color: blue;">x</p></div>"#
    )
}

#[test]
fn no_variables_in_document() {
    assert_resolved!(
        style = "p{color:red}",
        body = r#"<p style="margin: 0">x</p>"#,
        expected = r#"<p style="color: red;margin: 0">x</p>"#
    )
}

#[test]
fn fragment() {
    let inlined = CSSInliner::options()
        .resolve_css_variables(true)
        .build()
        .inline_fragment(
            r#"<div class="a"><p>x</p></div>"#,
            ".a{--c:red} p{color:var(--c)}",
        )
        .unwrap();
    assert_eq!(
        inlined,
        r#"<div class="a"><p style="color: red;">x</p></div>"#
    );
}

#[test]
fn keep_style_tags_leaves_block_untouched() {
    let html = html!(":root{--c:red} p{color:var(--c)}", "<p>x</p>");
    let inlined = CSSInliner::options()
        .resolve_css_variables(true)
        .keep_style_tags(true)
        .build()
        .inline(&html)
        .unwrap();
    assert_eq!(
        inlined,
        r#"<html><head><style>:root{--c:red} p{color:var(--c)}</style></head><body><p style="color: red;">x</p></body></html>"#
    );
}

#[test]
fn minified() {
    let html = html!(
        ":root{--c:red} p{color:var(--c); margin:var(--m, 1px)}",
        r#"<p>x</p><p style="--c: blue; padding: var(--p, 2px)">y</p>"#
    );
    let inlined = CSSInliner::options()
        .resolve_css_variables(true)
        .minify_css(true)
        .build()
        .inline(&html)
        .unwrap();
    assert_eq!(
        inlined,
        r#"<html><head></head><body><p style="color:red;margin:1px">x</p><p style="color:blue;margin:1px;padding:2px">y</p></body></html>"#
    );
}

#[test]
fn debug_output() {
    let inliner = CSSInliner::options().resolve_css_variables(true).build();
    assert!(format!("{inliner:?}").contains("resolve_css_variables: true"));
}

#[test]
fn later_definition_in_element_wins() {
    // A later declaration of the same custom property in one `style` attribute replaces the
    // earlier one, including when only the later one is `!important`.
    assert_resolved!(
        style = "p{color:var(--c)}",
        body = r#"<p style="--c: red; --c: blue">x</p>"#,
        expected = r#"<p style="color: blue">x</p>"#
    );
    assert_resolved!(
        style = "p{color:var(--c)}",
        body = r#"<p style="--c: red; --c: blue !important">x</p>"#,
        expected = r#"<p style="color: blue">x</p>"#
    )
}

#[test]
fn escaped_names() {
    // Declaration names are parsed with their escapes decoded; a `var()` spelled with escapes
    // refers to the same property.
    assert_resolved!(
        style = r":root{--my\ var: red; --my\20 other: blue} p{color: var(--my\ var); background-color: var(--my\20 other); border-top-color: var(--my\20other)}",
        body = "<p>x</p>",
        expected = r#"<p style="color: red;background-color: blue;border-top-color: blue;">x</p>"#
    )
}

#[test]
fn malformed_references_are_dropped() {
    // Chromium rejects these declarations when parsing the stylesheet, so there an earlier
    // declaration of the same property would still apply; here they are dropped after the
    // cascade.
    assert_resolved!(
        style = ":root{--c: red; --: green} p{color: var(); background-color: var(foo); letter-spacing: var(--c junk); margin-top: var( , 3px); padding-left: var(--); line-height: 2}",
        body = "<p>x</p>",
        expected = r#"<p style="line-height: 2;">x</p>"#
    )
}

#[test]
fn undefined_nested_fallback_drops_declaration() {
    assert_resolved!(
        style = "p{color: var(--x, var(--y)); letter-spacing: var(--x, var(--y, var(--z)))}",
        body = "<p>x</p>",
        expected = "<p>x</p>"
    )
}

#[test]
fn unclosed_parentheses_end_with_the_value() {
    assert_resolved!(
        style = ":root{--c: red}",
        body = r#"<p style="color: var(--c">x</p><p style="color: var(--x, red">y</p>"#,
        expected = r#"<p style="color: red">x</p><p style="color: red">y</p>"#
    )
}

#[test]
fn escapes_outside_strings() {
    assert_resolved!(
        style = r":root{--f: Arial} p{font-family: Helvetica\ Neue, var(--f)} span{font-family: var(--g, Helvetica\ Neue)}",
        body = "<p>x<span>y</span></p>",
        expected = r#"<p style="font-family: Helvetica\ Neue, Arial;">x<span style="font-family: Helvetica\ Neue;">y</span></p>"#
    )
}

#[test]
fn escapes_inside_strings() {
    assert_resolved!(
        style = r#":root{--f: Arial; --q: "\201C" "\201D"} p{font-family: "Segoe\ UI", var(--f); quotes: var(--q, "\2018" "\2019")} span{quotes: var(--nope, "\2018" "\2019")}"#,
        body = "<p>x<span>y</span></p>",
        expected = r#"<p style="font-family: 'Segoe\ UI', Arial;quotes: '\201C' '\201D';">x<span style="quotes: '\2018' '\2019';">y</span></p>"#
    )
}

#[test]
fn string_in_fallback() {
    assert_resolved!(
        style = r#"p{font-family: var(--f, "Helvetica Neue", Arial)}"#,
        body = "<p>x</p>",
        expected = r#"<p style="font-family: 'Helvetica Neue', Arial;">x</p>"#
    )
}

#[test]
fn unterminated_string_runs_to_the_end() {
    assert_resolved!(
        style = ":root{--f: Arial}",
        body = r#"<p style="font-family: var(--f), 'Open Sans">x</p>"#,
        expected = r#"<p style="font-family: Arial, 'Open Sans">x</p>"#
    )
}

#[test]
fn newline_ends_a_string() {
    // The string ends at the newline and `'; color: var(--c)` opens another one, as in the CSS
    // tokenizer, so there is no `var()` to substitute and the declaration stays as it is.
    assert_resolved!(
        style = ":root{--c: red}",
        body = "<p style=\"quotes: 'a\nb'; color: var(--c)\">x</p>",
        expected = "<p style=\"quotes: 'a\nb'; color: var(--c)\">x</p>"
    )
}

/// `k` nested `var(--u, ...)` fallbacks around `inner`.
fn nest_fallbacks(k: usize, inner: &str) -> String {
    format!("{}{inner}{}", "var(--u, ".repeat(k), ")".repeat(k))
}

#[test]
fn fallback_nesting_limit() {
    // Fallbacks nested deeper than 128 levels are invalid; Chromium has no such limit.
    assert_resolved!(
        style = format!("p{{color: {}}}", nest_fallbacks(128, "red")),
        body = "<p>x</p>",
        expected = r#"<p style="color: red;">x</p>"#
    );
    assert_resolved!(
        style = format!("p{{color: {}}}", nest_fallbacks(129, "red")),
        body = "<p>x</p>",
        expected = "<p>x</p>"
    );
    assert_resolved!(
        style = format!(
            ":root{{--x: {}}} p{{color: var(--x, blue)}}",
            nest_fallbacks(128, "red")
        ),
        body = "<p>x</p>",
        expected = r#"<p style="color: red;">x</p>"#
    );
    assert_resolved!(
        style = format!(
            ":root{{--x: {}}} p{{color: var(--x, blue)}}",
            nest_fallbacks(129, "red")
        ),
        body = "<p>x</p>",
        expected = r#"<p style="color: blue;">x</p>"#
    );
    assert_resolved!(
        style = format!(
            ":root{{--x: {}}} p{{color: var(--x, blue)}}",
            nest_fallbacks(30_000, "red")
        ),
        body = "<p>x</p>",
        expected = r#"<p style="color: blue;">x</p>"#
    )
}

/// `n` definitions, each referring to the next, the last one being `red`.
fn chain(n: usize) -> String {
    let mut css: String = (1..n)
        .map(|i| format!("--v{i}: var(--v{}); ", i + 1))
        .collect();
    css.push_str(&format!("--v{n}: red; "));
    css
}

#[test]
fn chain_depth_limit() {
    // Resolving through more than 128 referencing definitions is invalid; Chromium has no such
    // limit.
    assert_resolved!(
        style = "",
        body = format!(r#"<p style="{}color: var(--v1, blue)">x</p>"#, chain(129)),
        expected = r#"<p style="color: red">x</p>"#
    );
    assert_resolved!(
        style = "",
        body = format!(r#"<p style="{}color: var(--v1, blue)">x</p>"#, chain(130)),
        expected = r#"<p style="color: blue">x</p>"#
    )
}

#[test]
fn substitution_size_limit() {
    // Each level doubles the value; past 2 MiB it is invalid, as in Chromium.
    let mut definitions = format!("--a0: {}", ["Arial"; 150].join(", "));
    let mut expected = ["Arial"; 150].join(", ");
    for level in 1..=12 {
        definitions.push_str(&format!(
            "; --a{level}: var(--a{}), var(--a{})",
            level - 1,
            level - 1
        ));
        if level <= 10 {
            expected = format!("{expected}, {expected}");
        }
    }
    assert_resolved!(
        style = format!(":root{{{definitions}}} p{{color: var(--a12, red); font-family: var(--a12)}} span{{font-family: var(--a10)}}"),
        body = "<p>x<span>y</span></p>",
        expected = format!(r#"<p style="color: red;">x<span style="font-family: {expected};">y</span></p>"#)
    )
}

#[test]
fn ignored_subtree() {
    // Ignored elements keep their attributes as they are, while their definitions still apply
    // to the descendants that are inlined.
    assert_resolved!(
        style = ".wrap{--c: red} p{color: var(--c)} span{color: var(--c, green)}",
        body = r#"<div class="wrap" data-css-inline="ignore" style="--c: blue; color: var(--c)"><p>x</p><span data-css-inline="ignore" style="color: var(--c)">y</span></div>"#,
        expected = r#"<div class="wrap" style="--c: blue; color: var(--c)" data-css-inline="ignore"><p style="color: blue;">x</p><span data-css-inline="ignore" style="color: var(--c)">y</span></div>"#
    )
}

#[test]
fn design_tokens_with_component_overrides() {
    assert_resolved!(
        style = r#":root{--brand: #0057b8; --on-brand: #fff; --space: 8px; --radius: 4px; --surface: #f6f6f6; --text: #222}
.card{--surface: #fff; --card-padding: calc(var(--space) * 2); background: var(--surface); color: var(--text); padding: var(--card-padding); border-radius: var(--radius)}
.card.dark{--surface: #1e1e1e; --text: #eee; --brand: #6ea8ff}
.btn{background: var(--brand); color: var(--on-brand); padding: var(--space) calc(var(--space) * 2); border: var(--border-width, 1px) solid var(--brand)}
.btn.ghost{background: transparent; color: var(--brand)}
.muted{color: var(--text-muted, var(--text))}"#,
        body = r##"<div class="card"><p class="muted">a</p><a class="btn" href="#">b</a></div><div class="card dark" style="--space: 12px"><p>c</p><a class="btn ghost" href="#">d</a></div>"##,
        expected = r##"<div class="card" style="background: #fff;color: #222;padding: calc(8px * 2);border-radius: 4px;"><p class="muted" style="color: #222;">a</p><a class="btn" href="#" style="background: #0057b8;color: #fff;padding: 8px calc(8px * 2);border: 1px solid #0057b8;">b</a></div><div class="card dark" style="background: #1e1e1e;color: #eee;padding: calc(12px * 2);border-radius: 4px"><p>c</p><a class="btn ghost" href="#" style="padding: 12px calc(12px * 2);border: 1px solid #6ea8ff;background: transparent;color: #6ea8ff;">d</a></div>"##
    )
}

#[test]
fn fragment_with_definitions_in_attributes() {
    let inlined = CSSInliner::options()
        .resolve_css_variables(true)
        .build()
        .inline_fragment(
            r#"<div class="card" style="--accent: #c00"><p class="title">x</p><p style="color: var(--accent)">y</p><span>z</span></div>"#,
            ".card{--text: #333; color: var(--text)} .title{color: var(--accent, var(--text)); letter-spacing: var(--space, 0)}",
        )
        .unwrap();
    assert_eq!(
        inlined,
        r#"<div class="card" style="color: #333"><p class="title" style="color: #c00;letter-spacing: 0;">x</p><p style="color: #c00">y</p><span>z</span></div>"#
    );
}

#[test]
fn tailwind_v4_output() {
    // Tailwind v4 output without its `@media` utilities, which are not inlined.
    let html = html!(
        r#"/*! tailwindcss v4.3.3 | MIT License | https://tailwindcss.com */
@layer theme, base, components, utilities;
@layer theme {
  :root, :host {
    --font-sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue",
      "Noto Sans", Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji",
      "Segoe UI Symbol", "Noto Color Emoji";
    --font-mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono",
      "Courier New", monospace;
    --color-red-500: oklch(63.7% 0.237 25.331);
    --color-blue-500: oklch(62.3% 0.214 259.815);
    --spacing: 0.25rem;
    --default-font-family: var(--font-sans);
    --default-mono-font-family: var(--font-mono);
  }
}
@layer base {
  *, ::after, ::before, ::backdrop, ::file-selector-button {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
    border: 0 solid;
  }
  html, :host {
    line-height: 1.5;
    -webkit-text-size-adjust: 100%;
    tab-size: 4;
    font-family: var(--default-font-family, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", "Noto Sans", Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji");
    font-feature-settings: var(--default-font-feature-settings, normal);
    font-variation-settings: var(--default-font-variation-settings, normal);
    -webkit-tap-highlight-color: transparent;
  }
  hr {
    height: 0;
    color: inherit;
    border-top-width: 1px;
  }
  abbr:where([title]) {
    -webkit-text-decoration: underline dotted;
    text-decoration: underline dotted;
  }
  h1, h2, h3, h4, h5, h6 {
    font-size: inherit;
    font-weight: inherit;
  }
  a {
    color: inherit;
    -webkit-text-decoration: inherit;
    text-decoration: inherit;
  }
  b, strong {
    font-weight: bolder;
  }
  code, kbd, samp, pre {
    font-family: var(--default-mono-font-family, ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace);
    font-feature-settings: var(--default-mono-font-feature-settings, normal);
    font-variation-settings: var(--default-mono-font-variation-settings, normal);
    font-size: 1em;
  }
  small {
    font-size: 80%;
  }
  sub, sup {
    font-size: 75%;
    line-height: 0;
    position: relative;
    vertical-align: baseline;
  }
  sub {
    bottom: -0.25em;
  }
  sup {
    top: -0.5em;
  }
  table {
    text-indent: 0;
    border-color: inherit;
    border-collapse: collapse;
  }
  :-moz-focusring:where(:not(iframe)) {
    outline: auto;
  }
  progress {
    vertical-align: baseline;
  }
  summary {
    display: list-item;
  }
  ol, ul, menu {
    list-style: none;
  }
  img, svg, video, canvas, audio, iframe, embed, object {
    display: block;
    vertical-align: middle;
  }
  img, video {
    max-width: 100%;
    height: auto;
  }
  button, input, select, optgroup, textarea, ::file-selector-button {
    font: inherit;
    font-feature-settings: inherit;
    font-variation-settings: inherit;
    letter-spacing: inherit;
    color: inherit;
    border-radius: 0;
    background-color: transparent;
    opacity: 1;
  }
  :where(select:is([multiple], [size])) optgroup {
    font-weight: bolder;
  }
  :where(select:is([multiple], [size])) optgroup option {
    padding-inline-start: 20px;
  }
  ::file-selector-button {
    margin-inline-end: 4px;
  }
  ::placeholder {
    opacity: 1;
  }
  @supports (not (-webkit-appearance: -apple-pay-button))  or (contain-intrinsic-size: 1px) {
    ::placeholder {
      color: currentcolor;
      @supports (color: color-mix(in lab, red, red)) {
        color: color-mix(in oklab, currentcolor 50%, transparent);
      }
    }
  }
  textarea {
    resize: vertical;
  }
  ::-webkit-search-decoration {
    -webkit-appearance: none;
  }
  ::-webkit-date-and-time-value {
    min-height: 1lh;
    text-align: inherit;
  }
  ::-webkit-datetime-edit {
    display: inline-flex;
  }
  ::-webkit-datetime-edit-fields-wrapper {
    padding: 0;
  }
  ::-webkit-datetime-edit, ::-webkit-datetime-edit-year-field, ::-webkit-datetime-edit-month-field, ::-webkit-datetime-edit-day-field, ::-webkit-datetime-edit-hour-field, ::-webkit-datetime-edit-minute-field, ::-webkit-datetime-edit-second-field, ::-webkit-datetime-edit-millisecond-field, ::-webkit-datetime-edit-meridiem-field {
    padding-block: 0;
  }
  ::-webkit-calendar-picker-indicator {
    line-height: 1;
  }
  :-moz-ui-invalid {
    box-shadow: none;
  }
  button, input:where([type="button"], [type="reset"], [type="submit"]), ::file-selector-button {
    appearance: button;
  }
  ::-webkit-inner-spin-button, ::-webkit-outer-spin-button {
    height: auto;
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
  .text-red-500 {
    color: var(--color-red-500);
  }
}
"#,
        r#"<div class="flex p-4 text-red-500"><h1>Hello</h1><p>Use <code>npm i</code> to install. <a href="https://example.com">Docs</a> <strong>now</strong></p><hr><table><tr><td>Cell</td></tr></table></div>"#
    );
    assert_eq!(
        resolve(&html),
        r#"<html style="box-sizing: border-box;margin: 0;padding: 0;border: 0 solid;line-height: 1.5;-webkit-text-size-adjust: 100%;tab-size: 4;font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue',
      'Noto Sans', Arial, sans-serif, 'Apple Color Emoji', 'Segoe UI Emoji',
      'Segoe UI Symbol', 'Noto Color Emoji';font-feature-settings: normal;font-variation-settings: normal;-webkit-tap-highlight-color: transparent;"><head style="box-sizing: border-box;margin: 0;padding: 0;border: 0 solid;"></head><body style="box-sizing: border-box;margin: 0;padding: 0;border: 0 solid;"><div class="flex p-4 text-red-500" style="box-sizing: border-box;margin: 0;border: 0 solid;padding: calc(0.25rem * 4);display: flex;color: oklch(63.7% 0.237 25.331);"><h1 style="box-sizing: border-box;margin: 0;padding: 0;border: 0 solid;font-size: inherit;font-weight: inherit;">Hello</h1><p style="box-sizing: border-box;margin: 0;padding: 0;border: 0 solid;">Use <code style="box-sizing: border-box;margin: 0;padding: 0;border: 0 solid;font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono',
      'Courier New', monospace;font-feature-settings: normal;font-variation-settings: normal;font-size: 1em;">npm i</code> to install. <a href="https://example.com" style="box-sizing: border-box;margin: 0;padding: 0;border: 0 solid;color: inherit;-webkit-text-decoration: inherit;text-decoration: inherit;">Docs</a> <strong style="box-sizing: border-box;margin: 0;padding: 0;border: 0 solid;font-weight: bolder;">now</strong></p><hr style="box-sizing: border-box;margin: 0;padding: 0;border: 0 solid;height: 0;color: inherit;border-top-width: 1px;"><table style="box-sizing: border-box;margin: 0;padding: 0;border: 0 solid;text-indent: 0;border-color: inherit;border-collapse: collapse;"><tbody style="box-sizing: border-box;margin: 0;padding: 0;border: 0 solid;"><tr style="box-sizing: border-box;margin: 0;padding: 0;border: 0 solid;"><td style="box-sizing: border-box;margin: 0;padding: 0;border: 0 solid;">Cell</td></tr></tbody></table></div></body></html>"#
    );
}

// One value repeated under different definitions must resolve per scope.
#[test_case(
    "p{color:var(--c)}",
    r#"<div style="--c: red"><p>x</p></div><div style="--c: blue"><p>y</p></div>"#,
    r#"<div style=""><p style="color: red;">x</p></div><div style=""><p style="color: blue;">y</p></div>"#;
    "sibling scopes"
)]
#[test_case(
    "p{color:var(--c)}",
    r#"<div style="--c: red"><p>x</p><div style="--c: blue"><p>y</p></div><p>z</p></div>"#,
    r#"<div style=""><p style="color: red;">x</p><div style=""><p style="color: blue;">y</p></div><p style="color: red;">z</p></div>"#;
    "redefined deeper"
)]
#[test_case(
    "p{color:var(--c)}",
    r#"<div style="--c: red !important; --c: blue"><p>x</p></div><div style="--c: red !important"><p style="--c: blue">y</p></div>"#,
    r#"<div style=""><p style="color: red;">x</p></div><div style=""><p style="color: blue">y</p></div>"#;
    "same definitions split across elements"
)]
#[test_case(
    "p{color:var(--c, green)}",
    r#"<p>x</p><div style="--c: blue"><p>y</p></div><p>z</p>"#,
    r#"<p style="color: green;">x</p><div style=""><p style="color: blue;">y</p></div><p style="color: green;">z</p>"#;
    "fallback"
)]
#[test_case(
    "p{color:var(--c)}",
    r#"<p>x</p><div style="--c: blue"><p>y</p></div><p>z</p>"#,
    r#"<p>x</p><div style=""><p style="color: blue;">y</p></div><p>z</p>"#;
    "invalid then valid"
)]
#[test_case(
    ".x{--b:var(--a)} p{color:var(--b)}",
    r#"<div class="x" style="--a: red"><p>x</p></div><div class="x" style="--a: blue"><p>y</p></div>"#,
    r#"<div class="x" style=""><p style="color: red;">x</p></div><div class="x" style=""><p style="color: blue;">y</p></div>"#;
    "nested reference"
)]
#[test_case(
    "",
    r#"<div style="--c: red"><p style="color: var(--c)">x</p></div><div style="--c: blue"><p style="color: var(--c)">y</p></div>"#,
    r#"<div style=""><p style="color: red">x</p></div><div style=""><p style="color: blue">y</p></div>"#;
    "inline style"
)]
fn same_value_in_different_scopes(style: &str, body: &str, expected: &str) {
    assert_resolved!(style = style, body = body, expected = expected)
}

#[test_case(r#"<p style="color: v&#97;r(--c)">x</p>"#, r#"<p style="color: red">x</p>"#; "escaped function name")]
#[test_case(r#"<p style="color: var&#40;--c)">x</p>"#, r#"<p style="color: red">x</p>"#; "escaped parenthesis")]
fn var_from_character_reference(body: &str, expected: &str) {
    assert_resolved!(style = ":root{--c:red}", body = body, expected = expected)
}

// Without any `var()` in the input, definitions are still dropped from the output.
#[test_case("p{--a:1;color:red}", "<p>x</p>", r#"<p style="color: red;">x</p>"#; "stylesheet definition")]
#[test_case("p{--a:1}", "<p>x</p>", "<p>x</p>"; "only definitions")]
#[test_case("p{color:red}", r#"<p style="--a: 1; margin: 0">x</p>"#, r#"<p style="color: red;margin: 0">x</p>"#; "inline definition")]
#[test_case("", r#"<p style="--a: 1; margin: 0">x</p>"#, r#"<p style="margin: 0">x</p>"#; "inline definition without rules")]
fn definitions_without_var(style: &str, body: &str, expected: &str) {
    assert_resolved!(style = style, body = body, expected = expected)
}
