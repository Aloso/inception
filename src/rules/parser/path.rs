use std::iter::Peekable;

use proc_macro::{TokenTree, token_stream::IntoIter};

use crate::{MResult, rules::Path};

use super::consume_punct;

pub(super) fn parse_path(iter: &mut Peekable<IntoIter>) -> MResult<Option<Path>> {
    let Some(TokenTree::Ident(ident)) = iter.peek() else {
        return Ok(None);
    };
    let mut path = vec![ident.to_string()];
    let _ = iter.next();

    while let Some(dot) = consume_punct(iter, '.') {
        if let Some(TokenTree::Ident(ident)) = iter.next() {
            path.push(ident.to_string());
        } else {
            let span = iter.next().map(|t| t.span()).unwrap_or(dot);
            bail!("expected identifier or number after `.`" => span);
        }
    }

    Ok(Some(Path::new(path)))
}
