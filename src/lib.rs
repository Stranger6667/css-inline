use crate::parse::Declaration;
use kuchiki::traits::TendrilSink;
use kuchiki::{parse_html, ElementData, NodeDataRef, Selectors};
use std::io;

mod parse;

#[derive(Debug)]
struct Rule {
    selectors: kuchiki::Selectors,
    declarations: Vec<Declaration>,
}

impl Rule {
    pub fn new(selectors: &str, declarations: Vec<Declaration>) -> Result<Rule, ()> {
        Ok(Rule {
            selectors: Selectors::compile(selectors)?,
            declarations,
        })
    }
}

fn process_style_node(node: NodeDataRef<ElementData>) -> Vec<Rule> {
    let css = node.text_contents();
    let mut parse_input = cssparser::ParserInput::new(css.as_str());
    let mut parser = parse::CSSParser::new(&mut parse_input);
    parser
        .parse()
        .filter_map(|r| {
            r.map(|(selector, declarations)| Rule::new(&selector, declarations))
                .ok()
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

/// Inline CSS styles from <style> tags to matching elements in the HTML tree.
pub fn inline(html: &str) -> Result<String, io::Error> {
    let document = parse_html().one(html);
    let rules = document
        .select("style")
        .unwrap()
        .map(process_style_node)
        .flatten();

    for rule in rules {
        let matching_elements = document
            .inclusive_descendants()
            .filter_map(|node| node.into_element_ref())
            .filter(|element| rule.selectors.matches(element));
        for matching_element in matching_elements {
            let style = rule
                .declarations
                .iter()
                .map(|&(ref key, ref value)| format!("{}:{};", key, value));
            matching_element
                .attributes
                .borrow_mut()
                .insert("style", style.collect());
        }
    }

    let mut out = vec![];
    document
        .select("html")
        .unwrap()
        .next()
        .unwrap()
        .as_node()
        .serialize(&mut out)?;
    Ok(String::from_utf8_lossy(&out).to_string())
}

#[cfg(test)]
mod tests {
    use crate::*;

    const HTML: &str = r#"<html>
<head>
<title>Test</title>
<style>
h1, h2 { color:red; }
strong {
  text-decoration:none
  }
p { font-size:2px }
p.footer { font-size: 1px}
</style>
</head>
<body>
<h1>Hi!</h1>
<p><strong>Yes!</strong></p>
<p class="footer">Feetnuts</p>
</body>
</html>"#;

    #[test]
    fn test_inline() {
        let inlined = inline(HTML).unwrap();
        assert_eq!(
            inlined,
            r#"<html><head>
<title>Test</title>
<style>
h1, h2 { color:red; }
strong {
  text-decoration:none
  }
p { font-size:2px }
p.footer { font-size: 1px}
</style>
</head>
<body>
<h1 style="color:red;">Hi!</h1>
<p style="font-size:2px ;"><strong style="text-decoration:none
  ;">Yes!</strong></p>
<p class="footer" style="font-size: 1px;">Feetnuts</p>

</body></html>"#
        )
    }
}
