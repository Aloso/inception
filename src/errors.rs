use core::fmt;

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

pub(crate) fn error(s: &str, span: Span) -> TokenStream {
    error_range(s, span, span)
}

pub(crate) fn error_range(s: &str, start: Span, end: Span) -> TokenStream {
    let group = [respan(Literal::string(s), Span::call_site())].into_iter().collect();

    TokenStream::from_iter([
        respan(Ident::new("compile_error", start), start),
        respan(Punct::new('!', Spacing::Alone), Span::call_site()),
        respan(Group::new(Delimiter::Brace, group), end),
    ])
}

pub(crate) fn error_with(s: &str, span: Span, stream: TokenStream) -> TokenStream {
    let group = [respan(Literal::string(s), Span::call_site())].into_iter().collect();

    TokenStream::from_iter(
        [
            respan(Ident::new("compile_error", span), span),
            respan(Punct::new('!', Spacing::Alone), Span::call_site()),
            respan(Group::new(Delimiter::Brace, group), span),
        ]
        .into_iter()
        .chain(stream),
    )
}

fn respan<T: Into<TokenTree>>(t: T, span: Span) -> TokenTree {
    let mut t = t.into();
    t.set_span(span);
    t
}

pub(crate) struct MacroError {
    pub(crate) message: String,
    pub(crate) span: Span,
    pub(crate) stream: Option<TokenStream>,
}

impl fmt::Debug for MacroError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}", self.message)
    }
}

pub(crate) type MResult<T> = Result<T, MacroError>;

macro_rules! bail {
    ($message:literal $(, $e:expr)* => $span:expr) => {
        return Err($crate::errors::MacroError {
            message: format!($message $(, $e)*),
            span: $span,
            stream: None,
        })
    };
    ($message:literal $(, $e:expr)* => $span:expr; append $stream:expr) => {
        return Err($crate::errors::MacroError {
            message: format!($message $(, $e)*),
            span: $span,
            stream: Some($stream),
        })
    };
}

macro_rules! synerr {
    ($span:expr, $message:literal $(, $e:expr)*) => {
        return Err(syn::Error::new($span, format!($message $(, $e)*)))
    };
}

macro_rules! synbail {
    ($span:expr, $message:literal $(, $e:expr)*) => {
        return syn::Error::new($span, format!($message $(, $e)*)).into_compile_error().into()
    };
}
