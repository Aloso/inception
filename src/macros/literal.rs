use core::fmt;

#[derive(Debug)]
pub(crate) struct Literal(pub(crate) String);

impl From<proc_macro2::Literal> for Literal {
    fn from(value: proc_macro2::Literal) -> Self {
        Literal(value.to_string())
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
