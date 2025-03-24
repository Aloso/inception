use core::fmt;

use proc_macro2::Spacing;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Punct {
    pub(crate) char: char,
    pub(crate) spacing: Spacing,
}

impl From<proc_macro2::Punct> for Punct {
    fn from(value: proc_macro2::Punct) -> Self {
        Punct { char: value.as_char(), spacing: value.spacing() }
    }
}

impl From<Punct> for proc_macro2::Punct {
    fn from(value: Punct) -> Self {
        proc_macro2::Punct::new(value.char, value.spacing)
    }
}

impl fmt::Display for Punct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.char, f)
    }
}
