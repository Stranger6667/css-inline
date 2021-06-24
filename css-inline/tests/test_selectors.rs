#[macro_use]
mod utils;

// Most of the following tests are ported to Rust from https://github.com/rennat/pynliner
#[test]
fn identical_element() {
    assert_inlined!(
        style = r#"
        .text-right {
            text-align: right;
        }
        .box {
            border: 1px solid #000;
        }
        "#,
        body = r#"<div class="box"><p>Hello World</p><p class="text-right">Hello World on right</p><p class="text-right">Hello World on right</p></div>"#,
        expected = r#"<div class="box" style="border: 1px solid #000;"><p>Hello World</p><p class="text-right" style="text-align: right;">Hello World on right</p><p class="text-right" style="text-align: right;">Hello World on right</p></div>"#
    )
}

#[test]
fn is_or_prefixed_by() {
    assert_inlined!(
        style = r#"[data-type|="thing"] {color: red;}"#,
        body = r#"<span data-type="thing">1</span>"#,
        expected = r#"<span data-type="thing" style="color: red;">1</span>"#
    );
    assert_inlined!(
        style = r#"[data-type|="thing"] {color: red;}"#,
        body = r#"<span data-type="thing-1">1</span>"#,
        expected = r#"<span data-type="thing-1" style="color: red;">1</span>"#
    )
}

#[test]
fn contains() {
    assert_inlined!(
        style = r#"[data-type*="i"] {color: red;}"#,
        body = r#"<span data-type="thing">1</span>"#,
        expected = r#"<span data-type="thing" style="color: red;">1</span>"#
    )
}

#[test]
fn ends_with() {
    assert_inlined!(
        style = r#"[data-type$="ng"] {color: red;}"#,
        body = r#"<span data-type="thing">1</span>"#,
        expected = r#"<span data-type="thing" style="color: red;">1</span>"#
    )
}
#[test]
fn starts_with() {
    assert_inlined!(
        style = r#"[data-type^="th"] {color: red;}"#,
        body = r#"<span data-type="thing">1</span>"#,
        expected = r#"<span data-type="thing" style="color: red;">1</span>"#
    )
}
#[test]
fn one_of() {
    assert_inlined!(
        style = r#"[data-type~="thing1"] {color: red;}"#,
        body = r#"<span data-type="thing1 thing2">1</span>"#,
        expected = r#"<span data-type="thing1 thing2" style="color: red;">1</span>"#
    );
    assert_inlined!(
        style = r#"[data-type~="thing2"] {color: red;}"#,
        body = r#"<span data-type="thing1 thing2">1</span>"#,
        expected = r#"<span data-type="thing1 thing2" style="color: red;">1</span>"#
    )
}
#[test]
fn equals() {
    assert_inlined!(
        style = r#"[data-type="thing"] {color: red;}"#,
        body = r#"<span data-type="thing">1</span>"#,
        expected = r#"<span data-type="thing" style="color: red;">1</span>"#
    );
    assert_inlined!(
        style = r#"[data-type = "thing"] {color: red;}"#,
        body = r#"<span data-type="thing">1</span>"#,
        expected = r#"<span data-type="thing" style="color: red;">1</span>"#
    )
}
#[test]
fn exists() {
    assert_inlined!(
        style = r#"[data-type] {color: red;}"#,
        body = r#"<span data-type="thing">1</span>"#,
        expected = r#"<span data-type="thing" style="color: red;">1</span>"#
    )
}
#[test]
fn specificity() {
    assert_inlined!(
        style = r#"div,a,b,c,d,e,f,g,h,i,j { color: red; } .foo { color: blue; }"#,
        body = r#"<div class="foo"></div>"#,
        expected = r#"<div class="foo" style="color: blue;"></div>"#
    )
}

#[test]
fn first_child_descendant_selector_complex_dom() {
    assert_inlined!(
        style = r#"h1 :first-child { color: red; }"#,
        body = r#"<h1><div><span>Hello World!</span></div><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#,
        expected = r#"<h1><div style="color: red;"><span style="color: red;">Hello World!</span></div><p>foo</p><div class="barclass"><span style="color: red;">baz</span>bar</div></h1>"#
    )
}
#[test]
fn last_child_descendant_selector() {
    assert_inlined!(
        style = r#"h1 :last-child { color: red; }"#,
        body = r#"<h1><div><span>Hello World!</span></div></h1>"#,
        expected = r#"<h1><div style="color: red;"><span style="color: red;">Hello World!</span></div></h1>"#
    )
}
#[test]
fn first_child_descendant_selector() {
    assert_inlined!(
        style = r#"h1 :first-child { color: red; }"#,
        body = r#"<h1><div><span>Hello World!</span></div></h1>"#,
        expected = r#"<h1><div style="color: red;"><span style="color: red;">Hello World!</span></div></h1>"#
    )
}
#[test]
fn child_with_first_child_and_unmatched_class_selector_complex_dom() {
    assert_inlined!(
        style = r#"h1 > .hello:first-child { color: green; }"#,
        body = r#"<h1><span>Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#,
        expected = r#"<h1><span>Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#
    )
}
#[test]
fn child_with_first_child_and_class_selector_complex_dom() {
    assert_inlined!(
        style = r#"h1 > .hello:first-child { color: green; }"#,
        body = r#"<h1><span class="hello">Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#,
        expected = r#"<h1><span class="hello" style="color: green;">Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#
    )
}
#[test]
fn nested_child_with_first_child_override_selector_complex_dom() {
    assert_inlined!(
        style = r#"div > div > * { color: green; } div > div > :first-child { color: red; }"#,
        body = r#"<div><div><span>Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></div></div>"#,
        expected = r#"<div><div><span style="color: red;">Hello World!</span><p style="color: green;">foo</p><div class="barclass" style="color: green;"><span style="color: red;">baz</span>bar</div></div></div>"#
    )
}
#[test]
fn child_with_first_and_last_child_override_selector() {
    assert_inlined!(
        style = r#"p > * { color: green; } p > :first-child:last-child { color: red; }"#,
        body = r#"<p><span>Hello World!</span></p>"#,
        expected = r#"<p><span style="color: red;">Hello World!</span></p>"#
    )
}
#[test]
fn id_el_child_with_first_child_override_selector_complex_dom() {
    assert_inlined!(
        style = r#"#abc > * { color: green; } #abc > :first-child { color: red; }"#,
        body = r#"<div id="abc"><span class="cde">Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></div>"#,
        expected = r#"<div id="abc"><span class="cde" style="color: red;">Hello World!</span><p style="color: green;">foo</p><div class="barclass" style="color: green;"><span>baz</span>bar</div></div>"#
    )
}
#[test]
fn child_with_first_child_override_selector_complex_dom() {
    assert_inlined!(
        style = r#"div > * { color: green; } div > :first-child { color: red; }"#,
        body = r#"<div><span>Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></div>"#,
        expected = r#"<div><span style="color: red;">Hello World!</span><p style="color: green;">foo</p><div class="barclass" style="color: green;"><span style="color: red;">baz</span>bar</div></div>"#
    )
}
#[test]
fn child_follow_by_last_child_selector_complex_dom() {
    assert_inlined!(
        style = r#"h1 > :last-child { color: red; }"#,
        body = r#"<h1><span>Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#,
        expected = r#"<h1><span>Hello World!</span><p>foo</p><div class="barclass" style="color: red;"><span>baz</span>bar</div></h1>"#
    )
}
#[test]
fn parent_pseudo_selector() {
    assert_inlined!(
        style = r#"span:last-child span { color: red; }"#,
        body = r#"<h1><span><span>Hello World!</span></span></h1>"#,
        expected = r#"<h1><span><span style="color: red;">Hello World!</span></span></h1>"#
    );
    assert_inlined!(
        style = r#"span:last-child > span { color: red; }"#,
        body = r#"<h1><span><span>Hello World!</span></span></h1>"#,
        expected = r#"<h1><span><span style="color: red;">Hello World!</span></span></h1>"#
    );
    assert_inlined!(
        style = r#"span:last-child > span { color: red; }"#,
        body = r#"<h1><span><span>Hello World!</span></span><span>nope</span></h1>"#,
        expected = r#"<h1><span><span>Hello World!</span></span><span>nope</span></h1>"#
    )
}
#[test]
fn multiple_pseudo_selectors() {
    assert_inlined!(
        style = r#"span:first-child:last-child { color: red; }"#,
        body = r#"<h1><span>Hello World!</span></h1>"#,
        expected = r#"<h1><span style="color: red;">Hello World!</span></h1>"#
    );
    assert_inlined!(
        style = r#"span:first-child:last-child { color: red; }"#,
        body = r#"<h1><span>Hello World!</span><span>again!</span></h1>"#,
        expected = r#"<h1><span>Hello World!</span><span>again!</span></h1>"#
    )
}
#[test]
fn last_child_selector() {
    assert_inlined!(
        style = r#"h1 > :last-child { color: red; }"#,
        body = r#"<h1><span>Hello World!</span></h1>"#,
        expected = r#"<h1><span style="color: red;">Hello World!</span></h1>"#
    )
}
#[test]
fn child_follow_by_first_child_selector_complex_dom() {
    assert_inlined!(
        style = r#"h1 > :first-child { color: red; }"#,
        body = r#"<h1><span>Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#,
        expected = r#"<h1><span style="color: red;">Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#
    )
}
#[test]
fn child_follow_by_first_child_selector_with_comments() {
    assert_inlined!(
        style = r#"h1 > :first-child { color: red; }"#,
        body = r#"<h1> <!-- enough said --><span>Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#,
        expected = r#"<h1> <!-- enough said --><span style="color: red;">Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#
    )
}
#[test]
fn child_follow_by_first_child_selector_with_white_spaces() {
    assert_inlined!(
        style = r#"h1 > :first-child { color: red; }"#,
        body = r#"<h1> <span>Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#,
        expected = r#"<h1> <span style="color: red;">Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#
    )
}

#[test]
fn child_follow_by_adjacent_selector_complex_dom() {
    assert_inlined!(
        style = r#"h1 > span + p { color: red; }"#,
        body = r#"<h1><span>Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#,
        expected = r#"<h1><span>Hello World!</span><p style="color: red;">foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#
    )
}

#[test]
fn unknown_pseudo_selector() {
    assert_inlined!(
        style = r#"h1 > span:css4-selector { color: red; }"#,
        body = r#"<h1><span>Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#,
        expected = r#"<h1><span>Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#
    )
}
#[test]
fn adjacent_selector() {
    assert_inlined!(
        style = r#"h1 + h2 { color: red; }"#,
        body = r#"<h1>Hello World!</h1><h2>How are you?</h2>"#,
        expected = r#"<h1>Hello World!</h1><h2 style="color: red;">How are you?</h2>"#
    )
}
#[test]
fn child_all_selector_complex_dom() {
    assert_inlined!(
        style = r#"h1 > * { color: red; }"#,
        body = r#"<h1><span>Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#,
        expected = r#"<h1><span style="color: red;">Hello World!</span><p style="color: red;">foo</p><div class="barclass" style="color: red;"><span>baz</span>bar</div></h1>"#
    )
}
#[test]
fn child_selector_complex_dom() {
    assert_inlined!(
        style = r#"h1 > span { color: red; }"#,
        body = r#"<h1><span>Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#,
        expected = r#"<h1><span style="color: red;">Hello World!</span><p>foo</p><div class="barclass"><span>baz</span>bar</div></h1>"#
    )
}
#[test]
fn nested_child_selector() {
    assert_inlined!(
        style = r#"div > h1 > span { color: red; }""#,
        body = r#"<div><h1><span>Hello World!</span></h1></div>"#,
        expected = r#"<div><h1><span style="color: red;">Hello World!</span></h1></div>"#
    )
}
#[test]
fn child_selector() {
    assert_inlined!(
        style = r#"h1 > span { color: red; }"#,
        body = r#"<h1><span>Hello World!</span></h1>"#,
        expected = r#"<h1><span style="color: red;">Hello World!</span></h1>"#
    )
}
#[test]
fn descendant_selector() {
    assert_inlined!(
        style = r#"h1 span { color: red; }"#,
        body = r#"<h1><span>Hello World!</span></h1>"#,
        expected = r#"<h1><span style="color: red;">Hello World!</span></h1>"#
    )
}
#[test]
fn combination_selector() {
    assert_inlined!(
        style = r#"h1#a.b { color: red; }"#,
        body = r#"<h1 id="a" class="b">Hello World!</h1>"#,
        expected = r#"<h1 class="b" id="a" style="color: red;">Hello World!</h1>"#
    )
}
#[test]
fn conflicting_multiple_class_selector() {
    assert_inlined!(
        style = r#"h1.a.b { color: red; }"#,
        body = r#"<h1 class="a b">Hello World!</h1><h1 class="a">I should not be changed</h1>"#,
        expected = r#"<h1 class="a b" style="color: red;">Hello World!</h1><h1 class="a">I should not be changed</h1>"#
    )
}
#[test]
fn multiple_class_selector() {
    assert_inlined!(
        style = r#"h1.a.b { color: red; }"#,
        body = r#"<h1 class="a b">Hello World!</h1>"#,
        expected = r#"<h1 class="a b" style="color: red;">Hello World!</h1>"#
    )
}
#[test]
fn missing_link_descendant_selector() {
    assert_inlined!(
        style = r#"#a b i { color: red }"#,
        body = r#"<div id="a"><i>x</i></div>"#,
        expected = r#"<div id="a"><i>x</i></div>"#
    )
}

#[test]
fn comma_specificity() {
    assert_inlined!(
        style = r#"i, i { color: red; } i { color: blue; }"#,
        body = r#"<i>howdy</i>"#,
        expected = r#"<i style="color: blue;">howdy</i>"#
    )
}

#[test]
fn overwrite_comma() {
    assert_inlined!(
        style = r#"h1,h2,h3 {color: #000;}"#,
        body = r#"<h1 style="color: #fff">Foo</h1><h3 style="color: #fff">Foo</h3>"#,
        expected = r#"<h1 style="color: #fff;">Foo</h1><h3 style="color: #fff;">Foo</h3>"#
    )
}
