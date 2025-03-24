use core::fmt;

use proc_macro2::TokenTree;
use syn::parse::{Parse, ParseStream, Parser};

use super::{Literal, Punct};

pub(crate) use pattern_group::PatternGroup;
pub(crate) use pattern_matcher::PatternMatcher;
pub(crate) use patterns::Patterns;
pub(crate) use repeat::{Interspersed, Quantifier, Repeat, RepeatKind};

mod pattern_group;
mod pattern_matcher;
mod patterns;
mod repeat;

pub(crate) enum Pattern {
    Group(PatternGroup),
    Ident(String),
    Punct(Punct),
    Literal(Literal),
    Matcher(PatternMatcher),
}

impl fmt::Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Group(group) => group.fmt(f),
            Self::Ident(ident) => fmt::Display::fmt(ident, f),
            Self::Punct(punct) => fmt::Display::fmt(punct, f),
            Self::Literal(lit) => fmt::Display::fmt(lit, f),
            Self::Matcher(special) => fmt::Display::fmt(special, f),
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Group(group) => fmt::Display::fmt(group, f),
            Self::Ident(ident) => fmt::Display::fmt(ident, f),
            Self::Punct(punct) => fmt::Display::fmt(punct, f),
            Self::Literal(lit) => fmt::Display::fmt(lit, f),
            Self::Matcher(special) => fmt::Display::fmt(special, f),
        }
    }
}

impl Parse for Pattern {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Token![$]) {
            _ = input.parse::<syn::Token![$]>();

            if let Ok(punct) = input.parse::<proc_macro2::Punct>() {
                let char = punct.as_char();
                if let '$' | '*' | '+' | '?' | ':' = char {
                    return Ok(Pattern::Punct(punct.into()));
                } else {
                    synerr!(punct.span(), "punctuation `{char}` cannot be escaped, remove the `$`");
                }
            }

            let matcher = input.call(PatternMatcher::parse_after_dollar)?;
            return Ok(Pattern::Matcher(matcher));
        }

        Ok(match input.parse::<TokenTree>()? {
            TokenTree::Group(group) => Pattern::Group(PatternGroup {
                delimiter: group.delimiter().into(),
                content: Patterns::parse.parse2(group.stream())?.0,
            }),
            TokenTree::Ident(ident) => Pattern::Ident(ident.to_string()),
            TokenTree::Punct(punct) => Pattern::Punct(punct.into()),
            TokenTree::Literal(lit) => Pattern::Literal(lit.into()),
        })
    }
}
