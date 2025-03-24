use crate::macros::Delimiter;

use super::Replacement;

#[derive(Debug)]
pub(crate) struct ReplacementGroup {
    pub(crate) delimiter: Delimiter,
    pub(crate) content: Box<[Replacement]>,
}
