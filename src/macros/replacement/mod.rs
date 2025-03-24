use core::fmt;

use proc_macro2::{Delimiter, Span, TokenTree};
use syn::parse::{Parse, ParseStream, Parser};

pub(crate) use expr::Expr;
pub(crate) use replacement_group::ReplacementGroup;
pub(crate) use replacements::Replacements;
pub(crate) use special_replacement::SpecialReplacement;

use super::{Literal, Path, Punct};

mod expr;
mod replacement_group;
mod replacements;
mod special_replacement;

pub(crate) enum Replacement {
    Group(ReplacementGroup),
    Ident(String),
    Punct(Punct),
    Literal(Literal),
    Special(SpecialReplacement),
}

impl Replacement {
    fn is_if(&self) -> bool {
        use SpecialReplacement as SR;
        matches!(self, Replacement::Special(SR::If { .. } | SR::ElseIf { .. }))
    }

    fn is_else(&self) -> bool {
        use SpecialReplacement as SR;
        matches!(self, Replacement::Special(SR::Else { .. } | SR::ElseIf { .. }))
    }
}

impl fmt::Debug for Replacement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Group(group) => group.fmt(f),
            Self::Ident(ident) => fmt::Display::fmt(ident, f),
            Self::Punct(punct) => fmt::Display::fmt(punct, f),
            Self::Literal(lit) => fmt::Display::fmt(lit, f),
            Self::Special(special) => special.fmt(f),
        }
    }
}

struct ReplacementWithKwSpan {
    replacement: Replacement,
    kw_span: Option<Span>,
}

impl Parse for ReplacementWithKwSpan {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Token![$]) {
            _ = input.parse::<syn::Token![$]>();
            return ReplacementWithKwSpan::parse_after_dollar(input);
        }

        let tt = input.parse::<TokenTree>()?;
        let replacement = match tt {
            TokenTree::Group(group) => {
                let delimiter = group.delimiter().into();
                let Replacements(content) = syn::parse2(group.stream())?;
                Replacement::Group(ReplacementGroup { delimiter, content })
            }
            TokenTree::Ident(ident) => Replacement::Ident(ident.to_string()),
            TokenTree::Punct(punct) => Replacement::Punct(punct.into()),
            TokenTree::Literal(lit) => Replacement::Literal(lit.into()),
        };
        Ok(ReplacementWithKwSpan { replacement, kw_span: None })
    }
}

impl ReplacementWithKwSpan {
    fn parse_after_dollar(input: ParseStream) -> syn::Result<Self> {
        let next = input.parse::<TokenTree>()?;

        let (special, kw_span) = match next {
            TokenTree::Punct(punct) => match punct.as_char() {
                '$' | '*' | '+' | '?' | ':' => {
                    let replacement = Replacement::Punct(punct.into());
                    return Ok(ReplacementWithKwSpan { replacement, kw_span: None });
                }
                char => {
                    synerr!(punct.span(), "punctuation `{char}` cannot be escaped, remove the `$`");
                }
            },
            TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                (SpecialReplacement::parse_braced.parse2(group.stream())?, None)
            }
            TokenTree::Group(group) => {
                synerr!(group.span_open(), "unexpected group delimited by {:?}", group.delimiter());
            }
            TokenTree::Ident(ident) => {
                if ident == "for" {
                    (SpecialReplacement::parse_after_for(input)?, Some(ident.span()))
                } else if ident == "if" {
                    (SpecialReplacement::parse_after_if(input)?, Some(ident.span()))
                } else if ident == "else" {
                    (SpecialReplacement::parse_after_else(input)?, Some(ident.span()))
                } else if ident == "match" {
                    synerr!(ident.span(), "matches are not yet implemented");
                } else {
                    (SpecialReplacement::Path(Path(vec![ident.to_string()])), None)
                }
            }
            TokenTree::Literal(lit) => synerr!(lit.span(), "unexpected literal {:?}", lit),
        };

        Ok(ReplacementWithKwSpan { replacement: Replacement::Special(special), kw_span })
    }
}
