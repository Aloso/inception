use std::iter::Peekable;

use proc_macro::{
    Delimiter, Ident, Punct, Spacing, Span, TokenStream, TokenTree, token_stream::IntoIter,
};

use crate::{
    MResult,
    rules::{
        AstGroup, Interspersed, Pattern, Patterns, Quantifier, Repeat, RepeatKind, SpecialPattern,
    },
};

use super::consume_punct;

pub(super) fn parse(stream: TokenStream, nesting_limit: u16) -> MResult<Patterns> {
    if nesting_limit == 0 {
        panic!("Recursion limit reached: {stream:?}");
    }

    let mut iter = stream.into_iter().peekable();
    let mut patterns = Vec::new();

    while let Some(tt) = iter.next() {
        match tt {
            TokenTree::Group(group) => patterns.push(Pattern::Group(AstGroup {
                delimiter: group.delimiter().into(),
                content: parse(group.stream(), nesting_limit - 1)?.0,
            })),
            TokenTree::Ident(ident) => patterns.push(Pattern::Ident(ident.to_string())),
            TokenTree::Punct(punct) => {
                if punct.as_char() == '$' {
                    patterns.push(parse_pattern_after_dollar(&mut iter, punct)?);
                } else {
                    patterns.push(Pattern::Punct(punct.into()))
                }
            }
            TokenTree::Literal(lit) => patterns.push(Pattern::Literal(lit.into())),
        }
    }

    Ok(Patterns(patterns.into_boxed_slice()))
}

fn parse_pattern_after_dollar(iter: &mut Peekable<IntoIter>, dollar: Punct) -> MResult<Pattern> {
    let Some(next) = iter.peek() else {
        bail!("`$` should be followed by identifier, `{{`, or escaped punctuation" => dollar.span());
    };

    match next {
        TokenTree::Punct(punct) => match punct.as_char() {
            '$' | '*' | '+' | '?' | ':' => {
                let punct = punct.clone();
                let _ = iter.next();
                Ok(Pattern::Punct(punct.into()))
            }
            char => {
                bail!("punctuation `{char}` cannot be escaped, remove the `$`" => punct.span())
            }
        },
        TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
            let mut special = parse_special_pattern_group(None, group.stream(), group.span())?;
            let _ = iter.next();
            if consume_punct(iter, '?').is_some() {
                if let Some(repeat) = &mut special.repeat {
                    // ${x + ..}?
                    repeat.quantifier = Quantifier::Star;
                }
            }
            Ok(Pattern::Special(special))
        }
        TokenTree::Group(group) => {
            bail!("unexpected group delimited by {:?}", group.delimiter() => group.span_open());
        }
        TokenTree::Ident(ident) => {
            let ident = ident.clone();
            if let "for" | "if" | "match" = ident.to_string().as_str() {
                bail!("unexpected macro keyword" => ident.span());
            }
            let _ = iter.next();

            if let Some(colon) = consume_punct(iter, ':') {
                let Some(next) = iter.peek() else {
                    bail!("`:` needs to be followed by identifier or `{{`" => colon);
                };

                match next {
                    TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                        let mut special =
                            parse_special_pattern_group(Some(ident), group.stream(), group.span())?;
                        let _ = iter.next();
                        if consume_punct(iter, '?').is_some() {
                            if let Some(repeat) = &mut special.repeat {
                                // $name:{x + ..}?
                                repeat.quantifier = Quantifier::Star;
                            }
                        }
                        Ok(Pattern::Special(special))
                    }
                    TokenTree::Ident(ty) => {
                        let ty = ty.to_string();
                        let _ = iter.next();
                        let mut special =
                            SpecialPattern { name: Some(ident.to_string()), ty, repeat: None };
                        if let Some(quantifier) = parse_quantifier(iter) {
                            special.repeat = Some(Repeat { quantifier, interspersed: None });
                        }
                        Ok(Pattern::Special(special))
                    }
                    _ => {
                        bail!("unexpected token {:?}", next.to_string() => next.span());
                    }
                }
            } else {
                let ty = ident.to_string();
                let mut special = SpecialPattern { name: None, ty, repeat: None };
                if let Some(quantifier) = parse_quantifier(iter) {
                    special.repeat = Some(Repeat { quantifier, interspersed: None });
                }
                Ok(Pattern::Special(special))
            }
        }
        TokenTree::Literal(lit) => {
            bail!("unexpected literal {:?}", lit => lit.span());
        }
    }
}

fn parse_quantifier(iter: &mut Peekable<IntoIter>) -> Option<Quantifier> {
    if let Some(TokenTree::Punct(punct)) = iter.peek() {
        let quantifier = match punct.as_char() {
            '+' => Quantifier::Plus,
            '*' => Quantifier::Star,
            '?' => Quantifier::QuestionMark,
            _ => return None,
        };
        let _ = iter.next();
        Some(quantifier)
    } else {
        None
    }
}

fn parse_special_pattern_group(
    name: Option<Ident>,
    stream: TokenStream,
    span: Span,
) -> MResult<SpecialPattern> {
    let mut iter = stream.into_iter().peekable();
    let mut leading = None;
    if let Some(TokenTree::Punct(p)) = iter.peek() {
        leading = Some(p.clone());
        let _ = iter.next();
    }
    // TODO: support double punctuation, like `||`

    let Some(TokenTree::Ident(ident)) = iter.peek() else {
        bail!("expected ident" => iter.next().map(|t| t.span()).unwrap_or(span));
    };
    let ident = ident.clone();
    if name.is_none() {
        if let "for" | "if" | "match" = ident.to_string().as_str() {
            bail!("unexpected macro keyword" => ident.span());
        }
    }
    let _ = iter.next();

    let Some(TokenTree::Punct(middle)) = iter.peek() else {
        bail!("expected punctuation" => iter.next().map(|t| t.span()).unwrap_or(span));
    };
    let middle = middle.clone();
    let _ = iter.next();

    if !consume_two_puncts(&mut iter, '.', '.') {
        bail!("expected `..`" => iter.next().map(|t| t.span()).unwrap_or(span));
    }

    let mut trailing = None;
    if let Some(TokenTree::Punct(p)) = iter.peek() {
        trailing = Some(p.clone());
        let _ = iter.next();
    };

    if let Some(leading) = &leading {
        if leading.as_char() != middle.as_char() {
            bail!("expected punctuations to be equal" => middle.span());
        }
    }
    if let Some(trailing) = &trailing {
        if trailing.as_char() != middle.as_char() {
            bail!("expected punctuations to be equal" => trailing.span());
        }
    }
    let kind = match (leading, trailing) {
        (Some(_), Some(trailing)) => {
            bail!("setting both leading and trailing punctuations is unsupported" => trailing.span());
        }
        (Some(_), None) => RepeatKind::Leading,
        (None, Some(_)) => RepeatKind::Trailing,
        (None, None) => RepeatKind::Middle,
    };

    let interspersed = Some(Interspersed { kind, punct: middle.into() });
    let repeat = Some(Repeat { quantifier: Quantifier::Plus, interspersed });
    Ok(SpecialPattern { name: name.map(|n| n.to_string()), ty: ident.to_string(), repeat })
}

fn consume_two_puncts(iter: &mut Peekable<IntoIter>, c1: char, c2: char) -> bool {
    match iter.peek() {
        Some(TokenTree::Punct(p)) if p.as_char() == c1 && p.spacing() == Spacing::Joint => {
            let _ = iter.next();
            match iter.peek() {
                Some(TokenTree::Punct(p)) if p.as_char() == c2 => {
                    let _ = iter.next();
                    true
                }
                _ => false,
            }
        }
        _ => false,
    }
}
