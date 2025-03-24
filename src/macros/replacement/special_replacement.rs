use syn::{parse::ParseStream, punctuated::Punctuated};

use crate::macros::Path;

use super::{Replacement, Replacements, expr::Expr};

#[derive(Debug)]
pub(crate) enum SpecialReplacement {
    Path(Path),
    Call { func: String, args: Box<[Expr]> },
    If { condition: Path, body: Box<[Replacement]> },
    ElseIf { condition: Path, body: Box<[Replacement]> },
    Else { body: Box<[Replacement]> },
    For { binding: String, expr: Path, body: Box<[Replacement]> },
}

impl SpecialReplacement {
    pub(super) fn parse_braced(input: ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let mut path = input.parse::<Path>()?;

        if input.peek(syn::token::Paren) {
            let parenthesized;
            let paren = syn::parenthesized!(parenthesized in input);

            let args = Punctuated::<Path, syn::Token![,]>::parse_terminated(&parenthesized)?;
            let args =
                args.into_iter().map(|path| Expr { path }).collect::<Vec<_>>().into_boxed_slice();

            if path.0.len() > 1 {
                synerr!(paren.span.join(), "method calls are not supported");
            }
            let func_segment = path.0.pop().unwrap();

            match func_segment.as_str() {
                "first" | "last" | "count" | "concat" => {
                    return Ok(SpecialReplacement::Call { func: func_segment, args });
                }
                _ => synerr!(span, "unknown function `{}`", func_segment),
            }
        }

        Ok(SpecialReplacement::Path(path))
    }

    pub(super) fn parse_after_for(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        input.parse::<syn::Token![in]>()?;
        let expr = input.parse::<Path>()?;

        let replacements;
        syn::braced!(replacements in input);
        let Replacements(body) = replacements.parse()?;

        let binding = ident.to_string();
        Ok(SpecialReplacement::For { binding, expr, body })
    }

    pub(super) fn parse_after_if(input: ParseStream) -> syn::Result<Self> {
        let condition = input.parse::<Path>()?;

        let replacements;
        syn::braced!(replacements in input);
        let Replacements(body) = replacements.parse()?;

        Ok(SpecialReplacement::If { condition, body })
    }

    pub(super) fn parse_after_else(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Token![if]) {
            let SpecialReplacement::If { condition, body } = Self::parse_after_if(input)? else {
                unreachable!();
            };
            return Ok(SpecialReplacement::ElseIf { condition, body });
        }

        let replacements;
        syn::braced!(replacements in input);
        let Replacements(body) = replacements.parse()?;

        Ok(SpecialReplacement::Else { body })
    }
}
