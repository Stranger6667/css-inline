use cssparser::ToCss;
use selectors::parser::NonTSPseudoClass;
use std::fmt;

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) enum PseudoClass {
    AnyLink,
    Link,
    Visited,
    Active,
    Focus,
    Hover,
    Enabled,
    Disabled,
    Checked,
    Indeterminate,
}

impl PseudoClass {
    /// Whether any element can match it, see `Element::match_non_ts_pseudo_class`.
    /// Only links can: the rest need a user action or browser state.
    pub(crate) fn can_match(&self) -> bool {
        match self {
            PseudoClass::AnyLink | PseudoClass::Link => true,
            PseudoClass::Visited
            | PseudoClass::Active
            | PseudoClass::Focus
            | PseudoClass::Hover
            | PseudoClass::Enabled
            | PseudoClass::Disabled
            | PseudoClass::Checked
            | PseudoClass::Indeterminate => false,
        }
    }
}

impl NonTSPseudoClass for PseudoClass {
    fn is_active_or_hover(&self) -> bool {
        matches!(*self, PseudoClass::Active | PseudoClass::Hover)
    }

    fn is_user_action_state(&self) -> bool {
        matches!(
            *self,
            PseudoClass::Active | PseudoClass::Hover | PseudoClass::Focus
        )
    }
}

impl ToCss for PseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        dest.write_str(match *self {
            PseudoClass::AnyLink => ":any-link",
            PseudoClass::Link => ":link",
            PseudoClass::Visited => ":visited",
            PseudoClass::Active => ":active",
            PseudoClass::Focus => ":focus",
            PseudoClass::Hover => ":hover",
            PseudoClass::Enabled => ":enabled",
            PseudoClass::Disabled => ":disabled",
            PseudoClass::Checked => ":checked",
            PseudoClass::Indeterminate => ":indeterminate",
        })
    }
}
