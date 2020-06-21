pub struct CSSRuleListParser;
struct CSSDeclarationListParser;

pub type Declaration = (String, String);
pub type QualifiedRule = (String, Vec<Declaration>);

fn exhaust(input: &mut cssparser::Parser) -> String {
    let start = input.position();
    while input.next().is_ok() {}
    input.slice_from(start).to_string()
}

impl<'i> cssparser::QualifiedRuleParser<'i> for CSSRuleListParser {
    type Prelude = String;
    type QualifiedRule = QualifiedRule;
    type Error = ();

    fn parse_prelude<'t>(
        &mut self,
        input: &mut cssparser::Parser<'i, 't>,
    ) -> Result<Self::Prelude, cssparser::ParseError<'i, Self::Error>> {
        let _ = input;
        Ok(exhaust(input))
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _: cssparser::SourceLocation,
        input: &mut cssparser::Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, cssparser::ParseError<'i, Self::Error>> {
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
