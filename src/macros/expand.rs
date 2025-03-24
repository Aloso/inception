use proc_macro2::{Span, TokenStream};
use syn::parse::{Parse, ParseStream};

pub(crate) struct Expand {
    pub(crate) name: String,
    pub(crate) name_span: Span,
    pub(crate) index: usize,
    pub(crate) input: TokenStream,
    pub(crate) span: Span,
}

impl Parse for Expand {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = syn::Ident::parse(input)?;
        let name = ident.to_string();

        let index_lit = syn::LitInt::parse(input)?;
        if index_lit.suffix() != "" {
            synerr!(index_lit.span(), "unexpected integer suffix");
        }
        let Ok(index) = index_lit.base10_parse::<usize>() else {
            synerr!(index_lit.span(), "invalid usize");
        };

        let content;
        let brace = syn::braced!(content in input);
        let group = TokenStream::parse(&content)?;

        Ok(Expand { name, name_span: ident.span(), index, input: group, span: brace.span.join() })
    }
}
