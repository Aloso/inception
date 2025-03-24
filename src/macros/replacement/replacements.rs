use syn::parse::{Parse, ParseStream};

use super::{Replacement, ReplacementWithKwSpan};

pub(crate) struct Replacements(pub(crate) Box<[Replacement]>);

impl Parse for Replacements {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut result = Vec::<Replacement>::new();
        while !input.is_empty() {
            let next = input.parse::<ReplacementWithKwSpan>()?;

            match (result.last(), &next.replacement) {
                (Some(r1), r2) if r1.is_if() && r2.is_else() => {}
                (_, r2) if r2.is_else() => {
                    synerr!(next.kw_span.unwrap(), "`$else` is not allowed in this place!");
                }
                _ => {}
            }

            result.push(next.replacement);
        }

        Ok(Replacements(result.into_boxed_slice()))
    }
}
