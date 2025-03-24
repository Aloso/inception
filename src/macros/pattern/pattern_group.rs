use core::fmt::{self, Write};

use crate::macros::Delimiter;

use super::Pattern;

#[derive(Debug)]
pub(crate) struct PatternGroup {
    pub(crate) delimiter: Delimiter,
    pub(crate) content: Box<[Pattern]>,
}

impl fmt::Display for PatternGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (start, end) = match self.delimiter {
            Delimiter::Parenthesis => ('(', ')'),
            Delimiter::Bracket => ('[', ']'),
            Delimiter::Brace => ('{', ']'),
            Delimiter::None => return format_patterns(f, &self.content),
        };
        f.write_char(start)?;
        format_patterns(f, &self.content)?;
        f.write_char(end)?;
        Ok(())
    }
}

fn format_patterns(f: &mut fmt::Formatter<'_>, patterns: &[Pattern]) -> fmt::Result {
    for (i, pattern) in patterns.iter().enumerate() {
        if i > 0 {
            f.write_char(' ')?;
        }
        fmt::Display::fmt(&pattern, f)?;
    }
    Ok(())
}
