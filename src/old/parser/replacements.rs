use std::iter::Peekable;

use proc_macro2::{Delimiter, Punct, Span, TokenStream, TokenTree, token_stream::IntoIter};

use crate::{
    MResult,
    macros::{
        Path,
        replacement::{Expr, Replacement, ReplacementGroup, SpecialReplacement},
    },
};

use super::path::parse_path;

pub(super) fn parse(stream: TokenStream, nesting_limit: u16) -> MResult<Box<[Replacement]>> {
    if nesting_limit == 0 {
        panic!("Recursion limit reached: {stream:?}");
    }

    let mut iter = stream.into_iter().peekable();
    let mut rep = Vec::new();

    while let Some(tt) = iter.next() {
        match tt {
            TokenTree::Group(group) => rep.push(Replacement::Group(ReplacementGroup {
                delimiter: group.delimiter().into(),
                content: parse(group.stream(), nesting_limit - 1)?,
            })),
            TokenTree::Ident(ident) => rep.push(Replacement::Ident(ident.to_string())),
            TokenTree::Punct(punct) => {
                if punct.as_char() == '$' {
                    use SpecialReplacement as S;

                    let next_span = iter.peek().map(|t| t.span()).unwrap_or(punct.span());
                    let replacement = parse_rep_after_dollar(&mut iter, punct, nesting_limit)?;
                    if let Replacement::Special(S::Else { .. } | S::ElseIf { .. }) = &replacement {
                        let Some(Replacement::Special(S::If { .. } | S::ElseIf { .. })) =
                            rep.last()
                        else {
                            bail!("`$else` is not allowed in this place!" => next_span);
                        };
                    }
                    rep.push(replacement);
                } else {
                    rep.push(Replacement::Punct(punct.into()))
                }
            }
            TokenTree::Literal(lit) => rep.push(Replacement::Literal(lit.into())),
        }
    }

    Ok(rep.into_boxed_slice())
}

fn parse_rep_after_dollar(
    iter: &mut Peekable<IntoIter>,
    dollar: Punct,
    nesting_limit: u16,
) -> MResult<Replacement> {
    let Some(next) = iter.peek() else {
        bail!("`$` needs to be followed by identifier or `{{`" => dollar.span());
    };

    match next {
        TokenTree::Punct(punct) => match punct.as_char() {
            '$' | '*' | '+' | '?' | ':' => {
                let punct = punct.clone();
                let _ = iter.next();
                Ok(Replacement::Punct(punct.into()))
            }
            char => {
                bail!("punctuation `{char}` cannot be escaped, remove the `$`" => punct.span())
            }
        },
        TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
            let special = parse_special_rep_group(group.stream(), group.span())?;
            let _ = iter.next();
            Ok(Replacement::Special(special))
        }
        TokenTree::Group(group) => {
            bail!("unexpected group delimited by {:?}", group.delimiter() => group.span_open());
        }
        TokenTree::Ident(ident) => {
            let ident = ident.clone();
            let ident_span = ident.span();
            let _ = iter.next();
            match ident.to_string().as_str() {
                "for" => {
                    Ok(Replacement::Special(parse_after_for(iter, ident_span, nesting_limit)?))
                }
                "if" => Ok(Replacement::Special(parse_after_if(iter, ident_span, nesting_limit)?)),
                "match" => bail!("matches are not yet implemented" => ident.span()),
                "else" => {
                    Ok(Replacement::Special(parse_after_else(iter, ident_span, nesting_limit)?))
                }
                _ => {
                    let path = Path(vec![ident.to_string()]);
                    Ok(Replacement::Special(SpecialReplacement::Path(path)))
                }
            }
        }
        TokenTree::Literal(lit) => {
            bail!("unexpected literal {:?}", lit => lit.span());
        }
    }
}

fn parse_special_rep_group(stream: TokenStream, span: Span) -> MResult<SpecialReplacement> {
    let mut iter = stream.into_iter().peekable();

    let Some(mut path) = parse_path(&mut iter)? else {
        bail!("expected path" => span);
    };

    if let Some(TokenTree::Group(group)) = iter.peek() {
        if group.delimiter() == Delimiter::Parenthesis {
            let args = parse_func_arguments(group.stream())?;
            let _ = iter.next();
            if path.0.len() > 1 {
                bail!("method calls are not supported" => span);
            }
            let func_segment = path.0.pop().unwrap();

            match func_segment.as_str() {
                "first" | "last" | "count" | "concat" => {
                    return Ok(SpecialReplacement::Call { func: func_segment, args });
                }
                _ => {
                    bail!("unknown function `{}`", func_segment => span);
                }
            }
        }
    }

    if let Some(rest) = iter.next() {
        bail!("unexpected token {rest}" => rest.span());
    }

    Ok(SpecialReplacement::Path(path))
}

fn parse_func_arguments(stream: TokenStream) -> MResult<Box<[Expr]>> {
    let mut iter = stream.into_iter().peekable();

    let mut args = Vec::new();
    while let Some(arg) = parse_path(&mut iter)? {
        args.push(Expr { path: arg });
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

fn parse_after_if(
    iter: &mut Peekable<IntoIter>,
    span: Span,
    recursion_limit: u16,
) -> MResult<SpecialReplacement> {
    let Some(condition) = parse_path(iter)? else {
        bail!("expected a condition after `if`" => iter.next().map(|t| t.span()).unwrap_or(span));
    };

    match iter.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
            let body = parse(group.stream(), recursion_limit - 1)?;
            Ok(SpecialReplacement::If { condition, body })
        }
        Some(tt) => bail!("unexpected token {tt}" => tt.span()),
        None => bail!("expected token `{{` after condition" => span),
    }
}

fn parse_after_else(
    iter: &mut Peekable<IntoIter>,
    span: Span,
    recursion_limit: u16,
) -> MResult<SpecialReplacement> {
    if let Some(TokenTree::Ident(ident)) = iter.peek() {
        if ident == "if" {
            let _ = iter.next();
            if let SpecialReplacement::If { condition, body } =
                parse_after_if(iter, span, recursion_limit)?
            {
                return Ok(SpecialReplacement::ElseIf { condition, body });
            } else {
                unreachable!()
            }
        } else {
            bail!("unexpected identifier `{ident}` after `$else`" => ident.span());
        }
    }

    match iter.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
            let body = parse(group.stream(), recursion_limit - 1)?;
            Ok(SpecialReplacement::Else { body })
        }
        Some(tt) => bail!("unexpected token {tt}" => tt.span()),
        None => bail!("expected token `{{` after condition" => span),
    }
}

fn parse_after_for(
    iter: &mut Peekable<IntoIter>,
    span: Span,
    recursion_limit: u16,
) -> MResult<SpecialReplacement> {
    let Some(TokenTree::Ident(ident)) = iter.peek() else {
        bail!("expected an identifier after `for`" => iter.next().map(|t| t.span()).unwrap_or(span));
    };
    let ident = ident.clone();
    let _ = iter.next();

    match iter.next() {
        Some(TokenTree::Ident(in_id)) if in_id == "in" => {
            let Some(expr) = parse_path(iter)? else {
                bail!("expected an expression after `in`" => iter.next().map(|t| t.span()).unwrap_or(span));
            };

            match iter.next() {
                Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                    let binding = ident.to_string();
                    let body = parse(group.stream(), recursion_limit - 1)?;
                    Ok(SpecialReplacement::For { binding, expr, body })
                }
                Some(tt) => bail!("unexpected token {tt}" => tt.span()),
                None => bail!("expected token `{{` after condition" => span),
            }
        }
        Some(tt) => bail!("unexpected token {tt}" => tt.span()),
        None => bail!("expected `in` keyword" => span),
    }
}
