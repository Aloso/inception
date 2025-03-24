use core::fmt::{self, Write};

use syn::parse::{Parse, ParseStream};

use super::Pattern;

pub(crate) struct Patterns(pub(crate) Box<[Pattern]>);

impl fmt::Debug for Patterns {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, pat) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_char(' ')?;
            }
            pat.fmt(f)?;
        }
        Ok(())
    }
}

impl Parse for Patterns {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut result = Vec::<Pattern>::new();
        while !input.is_empty() {
            result.push(input.parse()?);
        }
        Ok(Patterns(result.into_boxed_slice()))
    }
}
