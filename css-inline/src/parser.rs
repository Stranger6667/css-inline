use cssparser::ParserState;

pub(crate) struct CSSRuleListParser<'d, 'i>(&'d mut Vec<Declaration<'i>>);

impl<'d, 'i> CSSRuleListParser<'d, 'i> {
    #[inline]
    pub(crate) fn new(declarations: &'d mut Vec<Declaration<'i>>) -> CSSRuleListParser<'d, 'i> {
        CSSRuleListParser(declarations)
    }
}

pub(crate) struct CSSDeclarationListParser;

pub(crate) type Name<'i> = cssparser::CowRcStr<'i>;
pub(crate) type Declaration<'i> = (Name<'i>, &'i str);
pub(crate) type QualifiedRule<'i> = (&'i str, (usize, usize));

fn exhaust<'i>(input: &mut cssparser::Parser<'i, '_>) -> &'i str {
    let start = input.position();
    while input.next().is_ok() {}
    input.slice_from(start)
}

/// Parser for qualified rules - a prelude + a simple {} block.
///
/// Usually these rules are a selector + list of declarations: `p { color: blue; font-size: 2px }`
impl<'i> cssparser::QualifiedRuleParser<'i> for CSSRuleListParser<'_, 'i> {
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
        _: &ParserState,
        input: &mut cssparser::Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, cssparser::ParseError<'i, Self::Error>> {
        // Parse list of declarations
        let mut parser = CSSDeclarationListParser;
        let parser = cssparser::RuleBodyParser::new(input, &mut parser);
        let start = self.0.len();
        for item in parser.flatten() {
            self.0.push(item);
        }
        Ok((prelude, (start, self.0.len())))
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
        _declaration_start: &ParserState,
    ) -> Result<Self::Declaration, cssparser::ParseError<'i, Self::Error>> {
        Ok((name, exhaust(input)))
    }
}

impl<'i> cssparser::AtRuleParser<'i> for CSSRuleListParser<'_, 'i> {
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

impl<'i> cssparser::RuleBodyItemParser<'i, Declaration<'i>, ()> for CSSDeclarationListParser {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

impl<'i> cssparser::QualifiedRuleParser<'i> for CSSDeclarationListParser {
    type Prelude = String;
    type QualifiedRule = Declaration<'i>;
    type Error = ();
}

pub(crate) struct AtRuleFilteringParser<'d, 'i, 'o> {
    declarations: &'d mut Vec<Declaration<'i>>,
    at_rules: &'o mut String,
}

impl<'d, 'i, 'o> AtRuleFilteringParser<'d, 'i, 'o> {
    #[inline]
    pub(crate) fn new(
        declarations: &'d mut Vec<Declaration<'i>>,
        at_rules: &'o mut String,
    ) -> AtRuleFilteringParser<'d, 'i, 'o> {
        AtRuleFilteringParser {
            declarations,
            at_rules,
        }
    }
}

impl<'i> cssparser::QualifiedRuleParser<'i> for AtRuleFilteringParser<'_, 'i, '_> {
    type Prelude = &'i str;
    type QualifiedRule = QualifiedRule<'i>;
    type Error = ();

    fn parse_prelude<'t>(
        &mut self,
        input: &mut cssparser::Parser<'i, 't>,
    ) -> Result<Self::Prelude, cssparser::ParseError<'i, Self::Error>> {
        Ok(exhaust(input))
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _: &ParserState,
        input: &mut cssparser::Parser<'i, 't>,
    ) -> Result<Self::QualifiedRule, cssparser::ParseError<'i, Self::Error>> {
        let mut parser = CSSDeclarationListParser;
        let parser = cssparser::RuleBodyParser::new(input, &mut parser);
        let start = self.declarations.len();
        for item in parser.flatten() {
            self.declarations.push(item);
        }
        Ok((prelude, (start, self.declarations.len())))
    }
}

impl<'i> cssparser::AtRuleParser<'i> for AtRuleFilteringParser<'_, 'i, '_> {
    type Prelude = &'i str;
    type AtRule = QualifiedRule<'i>;
    type Error = ();

    fn parse_prelude<'t>(
        &mut self,
        name: cssparser::CowRcStr<'i>,
        input: &mut cssparser::Parser<'i, 't>,
    ) -> Result<Self::Prelude, cssparser::ParseError<'i, Self::Error>> {
        // TODO pushing @ feels odd, there should be a less dumb way of doing it?
        self.at_rules.push_str("@");
        self.at_rules.push_str(&name);
        Ok(exhaust(input))
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _start: &ParserState,
        input: &mut cssparser::Parser<'i, 't>,
    ) -> Result<Self::AtRule, cssparser::ParseError<'i, Self::Error>> {
        // TODO same here, pushing braces manually feels odd
        let start = self.declarations.len();
        self.at_rules.push_str(prelude);
        self.at_rules.push_str(" {");
        self.at_rules.push_str(exhaust(input));
        self.at_rules.push_str("}");
        Ok((prelude, (start, self.declarations.len())))
    }
}
