use core::fmt;

use syn::{
    Token,
    ext::IdentExt,
    parse::{Parse, ParseStream},
};

pub(crate) struct Path(pub(crate) Vec<String>);

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, segment) in self.0.iter().enumerate() {
            if i > 0 {
                let _ = write!(f, ".");
            }
            let _ = write!(f, "{}", segment);
        }
        Ok(())
    }
}

impl Parse for Path {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.call(syn::Ident::parse_any)?;
        let mut path = vec![ident.to_string()];

        while input.peek(Token![.]) {
            _ = input.parse::<Token![.]>();
            let next = input.call(syn::Ident::parse_any)?;
            path.push(next.to_string());
        }

        Ok(Path(path))
    }
}
