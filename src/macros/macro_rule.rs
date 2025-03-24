use syn::parse::{Parse, ParseStream};

use super::{
    pattern::Patterns,
    replacement::{Replacement, Replacements},
};

#[derive(Debug)]
pub(crate) struct MacroRule {
    // pub(crate) vis: Vis,
    pub(crate) name: String,
    pub(crate) patterns: Patterns,
    pub(crate) replacements: Box<[Replacement]>,
}

impl Parse for MacroRule {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![macro]>()?;
        let ident = input.parse::<syn::Ident>()?;
        let name = ident.to_string();

        let pattern_input;
        _ = syn::parenthesized!(pattern_input in input);
        let patterns = pattern_input.parse()?;

        let replacement_input;
        _ = syn::braced!(replacement_input in input);
        let Replacements(replacements) = replacement_input.parse()?;

        Ok(MacroRule { name, patterns, replacements })
    }
}
