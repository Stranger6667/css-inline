pub struct CSSRuleListParser;
pub(crate) struct CSSDeclarationListParser;

pub type Declaration = (String, String);
pub type QualifiedRule = (String, Vec<Declaration>);

fn exhaust(input: &mut cssparser::Parser) -> String {
    let start = input.position();
    while input.next().is_ok() {}
    input.slice_from(start).to_string()
}

/// Parser for qualified rules - a prelude + a simple {} block.
///
/// Usually these rules are a selector + list of declarations: `p { color: blue; font-size: 2px }`
impl<'i> cssparser::QualifiedRuleParser<'i> for CSSRuleListParser {
    type Prelude = String;
    type QualifiedRule = QualifiedRule;
    type Error = ();

    fn parse_prelude<'t>(
        &mut self,
        input: &mut cssparser::Parser<'i, 't>,
    ) -> Result<Self::Prelude, cssparser::ParseError<'i, Self::Error>> {
        // Proceed with parsing until the end of the prelude.
        Ok(exhaust(input))
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _: cssparser::SourceLocation,
        input: &mut cssparser::Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, cssparser::ParseError<'i, Self::Error>> {
        // Parse list of declarations
        let parser = cssparser::DeclarationListParser::new(input, CSSDeclarationListParser);
        let mut declarations = vec![];

        for item in parser {
            if let Ok(declaration) = item {
                declarations.push(declaration);
            }
        }

        Ok((prelude, declarations))
    }
}

/// Parse a declaration within {} block: `color: blue`
impl<'i> cssparser::DeclarationParser<'i> for CSSDeclarationListParser {
    type Declaration = Declaration;
    type Error = ();

    fn parse_value<'t>(
        &mut self,
        name: cssparser::CowRcStr<'i>,
        input: &mut cssparser::Parser<'i, 't>,
    ) -> Result<Self::Declaration, cssparser::ParseError<'i, Self::Error>> {
        Ok((name.to_string(), exhaust(input)))
    }
}

impl cssparser::AtRuleParser<'_> for CSSRuleListParser {
    type PreludeNoBlock = String;
    type PreludeBlock = String;
    type AtRule = QualifiedRule;
    type Error = ();
}

/// Parsing for at-rules, e.g: `@charset "utf-8";`
/// Since they are can not be inlined we use the default implementation, that rejects all at-rules.
impl cssparser::AtRuleParser<'_> for CSSDeclarationListParser {
    type PreludeNoBlock = String;
    type PreludeBlock = String;
    type AtRule = Declaration;
    type Error = ();
}

pub struct CSSParser<'i, 't> {
    input: cssparser::Parser<'i, 't>,
}

impl<'i: 't, 't> CSSParser<'i, 't> {
    pub fn new(css: &'t mut cssparser::ParserInput<'i>) -> CSSParser<'i, 't> {
        CSSParser {
            input: cssparser::Parser::new(css),
        }
    }

    pub fn parse<'a>(&'a mut self) -> cssparser::RuleListParser<'i, 't, 'a, CSSRuleListParser> {
        cssparser::RuleListParser::new_for_stylesheet(&mut self.input, CSSRuleListParser)
    }
}
