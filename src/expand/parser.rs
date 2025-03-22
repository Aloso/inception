use std::iter::Peekable;

use proc_macro::{Group, Ident, Span, TokenStream, TokenTree, token_stream::IntoIter};

use crate::errors::MResult;

pub(crate) fn parse_derive_arguments(stream: TokenStream) -> MResult<Box<[Ident]>> {
    let mut iter = stream.into_iter().peekable();

    let mut args = Vec::new();
    while let Some(arg) = parse_derive_argument(&mut iter) {
        args.push(arg);
        if let Some(TokenTree::Punct(p)) = iter.peek() {
            if p.as_char() == ',' {
                let _ = iter.next();
                continue;
            } else {
                bail!("unexpected punctuation `{p}`" => p.span());
            }
        }
        break;
    }

    if let Some(rest) = iter.next() {
        bail!("unexpected token `{rest}` in function argument list" => rest.span());
    }

    Ok(args.into_boxed_slice())
}

fn parse_derive_argument(iter: &mut Peekable<IntoIter>) -> Option<Ident> {
    let Some(TokenTree::Ident(ident)) = iter.next() else {
        return None;
    };
    Some(ident)
}

pub(crate) fn parse_single_argument(stream: TokenStream, span: Span) -> MResult<Ident> {
    let mut iter = stream.into_iter().peekable();

    let Some(TokenTree::Ident(ident)) = iter.next() else {
        bail!("expected macro identifier" => span);
    };

    if let Some(rest) = iter.next() {
        bail!("unexpected token `{rest}`" => rest.span());
    }

    Ok(ident)
}

pub(crate) fn parse_expand(stream: TokenStream, span: Span) -> MResult<(Ident, usize, Group)> {
    let mut iter = stream.into_iter().peekable();

    let Some(TokenTree::Ident(name)) = iter.peek() else {
        bail!("expected macro identifier" => span);
    };
    let name = name.clone();
    let _ = iter.next();

    let Some(TokenTree::Literal(lit)) = iter.peek() else {
        bail!("expected literal" => span);
    };
    let Ok(index) = lit.to_string().parse() else {
        bail!("invalid index {lit}" => span);
    };
    let _ = iter.next();

    match iter.peek() {
        Some(TokenTree::Group(group)) => {
            let group = group.clone();
            let _ = iter.next();
            if let Some(rest) = iter.next() {
                bail!("unexpected token `{rest}`" => rest.span());
            }

            Ok((name, index, group))
        }
        Some(tt) => bail!("unexpected token `{tt}`" => tt.span()),
        None => bail!("expected `(`, `{{` or `[`" => span),
    }
}
