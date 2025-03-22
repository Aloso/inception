use proc_macro::{Literal, Span, TokenStream, TokenTree};

use crate::{MResult, rules::Expr};

use super::Match;

pub(crate) fn first(
    args: &[Expr],
    matches: &Match,
    span: Span,
    result: &mut TokenStream,
) -> MResult<()> {
    if args.len() != 1 {
        bail!("expected 1 argument, got {}", args.len() => span);
    }
    if let Some(tt) = matches.find_child(&args[0].path) {
        if tt.is_empty() {
            bail!("the argument `{:?}` is empty", &args[0].path => span);
        }
        let first = &tt[0..1];
        result.extend(first.iter().cloned());
    } else {
        bail!("the argument `{:?}` does not exist", &args[0].path => span);
    }

    Ok(())
}

pub(crate) fn last(
    args: &[Expr],
    matches: &Match,
    span: Span,
    result: &mut TokenStream,
) -> MResult<()> {
    if args.len() != 1 {
        bail!("expected 1 argument, got {}", args.len() => span);
    }
    if let Some(tt) = matches.find_child(&args[0].path) {
        if tt.is_empty() {
            bail!("the argument `{:?}` is empty", &args[0].path => span);
        }
        let last = &tt[tt.len() - 1..];
        result.extend(last.iter().cloned());
    } else {
        bail!("the argument `{:?}` does not exist", &args[0].path => span);
    }

    Ok(())
}

pub(crate) fn count(
    args: &[Expr],
    matches: &Match,
    span: Span,
    result: &mut TokenStream,
) -> MResult<()> {
    if args.len() != 1 {
        bail!("expected 1 argument, got {}", args.len() => span);
    }
    if let Some(tt) = matches.find_child(&args[0].path) {
        result.extend([TokenTree::Literal(Literal::usize_unsuffixed(tt.len()))]);
    } else {
        bail!("the argument `{:?}` does not exist", &args[0].path => span);
    }

    Ok(())
}
