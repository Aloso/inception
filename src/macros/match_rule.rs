use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
};

use super::pattern::Patterns;

#[derive(Debug)]
pub(crate) struct MatchRule {
    // pub(crate) vis: Vis,
    pub(crate) name: String,
    pub(crate) pattern_set: Box<[Patterns]>,
}

impl Parse for MatchRule {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![match]>()?;
        let ident = input.call(syn::Ident::parse_any)?;
        let name = ident.to_string();
        input.parse::<syn::Token![as]>()?;

        _ = input.parse::<syn::Token![|]>();

        let mut patterns = vec![];
        loop {
            let group;
            syn::parenthesized!(group in input);
            patterns.push(group.parse::<Patterns>()?);
            if input.parse::<syn::Token![|]>().is_err() {
                break;
            }
        }

        input.parse::<syn::Token![;]>()?;
        Ok(MatchRule { name, pattern_set: patterns.into_boxed_slice() })
    }
}
