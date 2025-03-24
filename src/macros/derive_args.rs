use syn::{
    Ident,
    parse::{Parse, ParseStream},
};

pub(crate) struct DeriveArgs {
    pub(crate) names: Vec<Ident>,
}

impl Parse for DeriveArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut names = Vec::new();
        while !input.is_empty() {
            names.push(input.parse::<syn::Ident>()?);
            if !input.peek(syn::Token![,]) {
                break;
            }
            _ = input.parse::<syn::Token![,]>();
        }

        Ok(DeriveArgs { names })
    }
}
