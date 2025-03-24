use syn::parse::{Parse, ParseStream};

#[derive(Debug)]
pub(crate) enum Vis {
    Private,
    Public,
}

impl Parse for Vis {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match input.parse::<Option<syn::Token![pub]>>()? {
            Some(_) => Ok(Vis::Public),
            None => Ok(Vis::Private),
        }
    }
}
