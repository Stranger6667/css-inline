pub(crate) struct CSSRuleListParser;
pub(crate) struct CSSDeclarationListParser;

pub(crate) type Name<'i> = cssparser::CowRcStr<'i>;
pub(crate) type Declaration<'i> = (Name<'i>, &'i str);
pub(crate) type QualifiedRule<'i> = (&'i str, Vec<Declaration<'i>>);

fn exhaust<'i>(input: &mut cssparser::Parser<'i, '_>) -> &'i str {
    let start = input.position();
    while input.next().is_ok() {}
    input.slice_from(start)
}

/// Parser for qualified rules - a prelude + a simple {} block.
///
/// Usually these rules are a selector + list of declarations: `p { color: blue; font-size: 2px }`
impl<'i> cssparser::QualifiedRuleParser<'i> for CSSRuleListParser {
    type Prelude = &'i str;
    type QualifiedRule = QualifiedRule<'i>;
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
        _: &cssparser::ParserState,
        input: &mut cssparser::Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, cssparser::ParseError<'i, Self::Error>> {
        // Parse list of declarations
        let parser = cssparser::DeclarationListParser::new(input, CSSDeclarationListParser);
        let declarations: Vec<_> = parser.flatten().collect();
        Ok((prelude, declarations))
    }
}

/// Parse a declaration within {} block: `color: blue`
impl<'i> cssparser::DeclarationParser<'i> for CSSDeclarationListParser {
    type Declaration = Declaration<'i>;
    type Error = ();

    fn parse_value<'t>(
        &mut self,
        name: Name<'i>,
        input: &mut cssparser::Parser<'i, 't>,
    ) -> Result<Self::Declaration, cssparser::ParseError<'i, Self::Error>> {
        Ok((name, exhaust(input)))
    }
}

impl<'i> cssparser::AtRuleParser<'i> for CSSRuleListParser {
    type Prelude = &'i str;
    type AtRule = QualifiedRule<'i>;
    type Error = ();
}

/// Parsing for at-rules, e.g: `@charset "utf-8";`
/// Since they are can not be inlined we use the default implementation, that rejects all at-rules.
impl<'i> cssparser::AtRuleParser<'i> for CSSDeclarationListParser {
    type Prelude = String;
    type AtRule = Declaration<'i>;
    type Error = ();
}
