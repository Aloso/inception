use core::fmt::{self, Write};

use crate::{helper::DebugToDisplay, macros::Punct};

#[derive(Debug)]
pub(crate) struct Repeat {
    pub(crate) quantifier: Quantifier,
    pub(crate) interspersed: Option<Interspersed>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Quantifier {
    Star,
    Plus,
    QuestionMark,
}

impl fmt::Display for Quantifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(match self {
            Quantifier::Star => '*',
            Quantifier::Plus => '+',
            Quantifier::QuestionMark => '?',
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Interspersed {
    pub(crate) kind: RepeatKind,
    pub(crate) punct: Punct,
}

impl fmt::Debug for Interspersed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interspersed")
            .field("kind", &self.kind)
            .field("punct", &DebugToDisplay(&self.punct))
            .finish()
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum RepeatKind {
    Leading,
    Trailing,
    Middle,
}
