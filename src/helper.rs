use core::fmt;

macro_rules! t {
    (usize $e:expr) => {
        ::proc_macro2::TokenTree::Literal(::proc_macro2::Literal::usize_unsuffixed($e))
    };

    (parentheses( $($e:expr),* $(,)? )) => {
        ::proc_macro2::TokenTree::Group(::proc_macro2::Group::new(
            ::proc_macro2::Delimiter::Parenthesis, ::proc_macro2::TokenStream::from_iter([ $($e),* ])
        ))
    };
    (braces( $($e:expr),* $(,)? )) => {
        ::proc_macro2::TokenTree::Group(::proc_macro2::Group::new(
            ::proc_macro2::Delimiter::Brace, ::proc_macro2::TokenStream::from_iter([ $($e),* ])
        ))
    };

    ($t:expr, $span:expr) => {
        ::proc_macro2::TokenTree::Ident(::proc_macro2::Ident::new($t, $span))
    };

    ($t:literal) => {
        ::proc_macro2::TokenTree::Punct(::proc_macro2::Punct::new($t, ::proc_macro2::Spacing::Alone))
    };
    ($t:literal joint) => {
        ::proc_macro2::TokenTree::Punct(::proc_macro2::Punct::new($t, ::proc_macro2::Spacing::Joint))
    };
}

pub(super) struct DebugToDisplay<T>(pub(super) T);

impl<T: fmt::Display> fmt::Debug for DebugToDisplay<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
