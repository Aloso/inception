macro_rules! t {
    (usize $e:expr) => {
        ::proc_macro::TokenTree::Literal(::proc_macro::Literal::usize_unsuffixed($e))
    };

    (parentheses( $($e:expr),* $(,)? )) => {
        ::proc_macro::TokenTree::Group(::proc_macro::Group::new(
            ::proc_macro::Delimiter::Parenthesis, ::proc_macro::TokenStream::from_iter([ $($e),* ])
        ))
    };
    (braces( $($e:expr),* $(,)? )) => {
        ::proc_macro::TokenTree::Group(::proc_macro::Group::new(
            ::proc_macro::Delimiter::Brace, ::proc_macro::TokenStream::from_iter([ $($e),* ])
        ))
    };

    ($t:expr, $span:expr) => {
        ::proc_macro::TokenTree::Ident(::proc_macro::Ident::new($t, $span))
    };

    ($t:literal) => {
        ::proc_macro::TokenTree::Punct(::proc_macro::Punct::new($t, ::proc_macro::Spacing::Alone))
    };
    ($t:literal joint) => {
        ::proc_macro::TokenTree::Punct(::proc_macro::Punct::new($t, ::proc_macro::Spacing::Joint))
    };
}
