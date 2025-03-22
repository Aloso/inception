mod functions;
mod matching;
mod parser;

use std::{str::FromStr, time::Instant};

use matching::{Match, match_patterns};
pub(crate) use parser::{parse_derive_arguments, parse_expand, parse_single_argument};
use proc_macro::{Group, Ident, Span, TokenStream, TokenTree};

use crate::{
    Rules,
    errors::MResult,
    rules::{Literal, Replacement, SpecialReplacement},
};

pub(crate) fn expand_macro(
    name: &str,
    rules: &Rules,
    tokens: TokenStream,
    span: Span,
) -> MResult<TokenStream> {
    if rules.macro_rule.name == name {
        return expand_macro_rule(rules, tokens, span);
    }
    bail!("no rule with the name {name} found!" => span);
}

pub(crate) fn expand_macro_rule(
    rules: &Rules,
    tokens: TokenStream,
    span: Span,
) -> MResult<TokenStream> {
    eprintln!("expand: {:#?}\n", rules);

    let matches_start = Instant::now();
    let matches = match_patterns(rules, &rules.macro_rule.patterns.0, tokens)?;
    eprintln!("pattern matching took {:?}", matches_start.elapsed());
    eprintln!("matches: {matches:#?}\n");

    let mut result = TokenStream::new();
    replace_stream(&matches, &rules.macro_rule.replacements, &mut result, span)?;

    Ok(result)
}

fn replace_stream(
    matches: &Match,
    replacements: &[Replacement],
    result: &mut TokenStream,
    span: Span,
) -> MResult<()> {
    let mut last_if_matched = None;

    for replacement in replacements {
        match replacement {
            Replacement::Group(ast_group) => {
                let mut inner = TokenStream::new();
                // We can't use correct span :(
                replace_stream(matches, &ast_group.content, &mut inner, span)?;
                let group = Group::new(ast_group.delimiter.into(), inner);
                result.extend([TokenTree::Group(group)]);
            }
            Replacement::Ident(ident) => {
                // We can't use correct span :(
                result.extend([TokenTree::Ident(Ident::new(ident, span))]);
            }
            &Replacement::Punct(punct) => {
                result.extend([TokenTree::Punct(punct.into())]);
            }
            Replacement::Literal(Literal(text)) => {
                let literal = proc_macro::Literal::from_str(text).unwrap();
                result.extend([TokenTree::Literal(literal)]);
            }
            Replacement::Special(special) => match special {
                SpecialReplacement::Path(path) => {
                    if let Some(tt) = matches.find_child(path) {
                        result.extend(tt.iter().cloned());
                    } else {
                        bail!("match `{special:?}` does not exist" => span);
                    }
                }
                SpecialReplacement::Call { func, args } => match func.as_str() {
                    "first" => functions::first(args, matches, span, result)?,
                    "last" => functions::last(args, matches, span, result)?,
                    "count" => functions::count(args, matches, span, result)?,
                    s => bail!("unknown function `{s}`" => span),
                },
                SpecialReplacement::If { condition, body } => {
                    if matches.find_child(condition).is_some() {
                        replace_stream(matches, body, result, span)?;
                        last_if_matched = Some(true);
                    } else {
                        last_if_matched = Some(false);
                    }
                    continue;
                }
                SpecialReplacement::ElseIf { condition, body } => {
                    if let Some(matched) = last_if_matched {
                        if matched {
                            continue;
                        }
                        if matches.find_child(condition).is_some() {
                            replace_stream(matches, body, result, span)?;
                            last_if_matched = Some(true);
                        } else {
                            last_if_matched = Some(false);
                        }
                    } else {
                        bail!("unexpected `else if`" => span);
                    }
                    continue;
                }
                SpecialReplacement::Else { body } => {
                    if let Some(matched) = last_if_matched {
                        last_if_matched = None;
                        if matched {
                            continue;
                        }
                        replace_stream(matches, body, result, span)?;
                        last_if_matched = Some(true);
                    } else {
                        bail!("unexpected `else`" => span);
                    }
                    continue;
                }
                SpecialReplacement::For { binding, expr, body } => {
                    if let Some(exprs) = matches.find_child(expr) {
                        let mut nested_matches = matches.clone();
                        for e in &exprs {
                            nested_matches.replace_child(binding, e.clone());
                            replace_stream(&nested_matches, body, result, span)?;
                        }
                        last_if_matched = Some(!exprs.is_empty());
                    } else {
                        last_if_matched = Some(false);
                    }
                    continue;
                }
            },
        }
        last_if_matched = None;
    }

    Ok(())
}
