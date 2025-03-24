#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Delimiter {
    Parenthesis,
    Bracket,
    Brace,
    None,
}

impl From<Delimiter> for proc_macro2::Delimiter {
    fn from(value: Delimiter) -> Self {
        match value {
            Delimiter::Parenthesis => proc_macro2::Delimiter::Parenthesis,
            Delimiter::Brace => proc_macro2::Delimiter::Brace,
            Delimiter::Bracket => proc_macro2::Delimiter::Bracket,
            Delimiter::None => proc_macro2::Delimiter::None,
        }
    }
}

impl From<proc_macro2::Delimiter> for Delimiter {
    fn from(value: proc_macro2::Delimiter) -> Self {
        match value {
            proc_macro2::Delimiter::Parenthesis => Delimiter::Parenthesis,
            proc_macro2::Delimiter::Brace => Delimiter::Brace,
            proc_macro2::Delimiter::Bracket => Delimiter::Bracket,
            proc_macro2::Delimiter::None => Delimiter::None,
        }
    }
}

impl PartialEq<proc_macro2::Delimiter> for Delimiter {
    fn eq(&self, other: &proc_macro2::Delimiter) -> bool {
        matches!(
            (self, other),
            (Delimiter::Parenthesis, proc_macro2::Delimiter::Parenthesis)
                | (Delimiter::Bracket, proc_macro2::Delimiter::Bracket)
                | (Delimiter::Brace, proc_macro2::Delimiter::Brace)
                | (Delimiter::None, proc_macro2::Delimiter::None)
        )
    }
}
