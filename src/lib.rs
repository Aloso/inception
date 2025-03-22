extern crate proc_macro;
use std::{collections::HashMap, sync::Mutex, time::Instant};

use errors::{MResult, MacroError};
use expand::{expand_macro, parse_derive_arguments, parse_expand, parse_single_argument};
use proc_macro::{Delimiter, Group, Span, TokenStream, TokenTree};
use rules::{Rule, RuleParser, Rules};

#[macro_use]
mod errors;
#[macro_use]
mod helper;
mod expand;
mod rules;

static RULES: Mutex<Option<HashMap<String, Vec<Rules>>>> = Mutex::new(None);

#[proc_macro]
pub fn rules(tokens: TokenStream) -> TokenStream {
    handle(|| {
        let parsed_rules = parse_rules(tokens)?;

        let mut macro_rule = None;
        let mut matches = HashMap::new();

        for rule in parsed_rules {
            match rule {
                Rule::Match(match_rule) => {
                    if matches.contains_key(&match_rule.name) {
                        bail!("duplicate matcher name {}", match_rule.name => Span::call_site());
                    }
                    matches.insert(match_rule.name, match_rule.pattern_set);
                }
                Rule::Macro(rule) => {
                    if macro_rule.is_some() {
                        bail!("inception rules contain multiple macros" => Span::call_site());
                    }
                    macro_rule = Some(rule);
                }
            }
        }
        let Some(macro_rule) = macro_rule else {
            bail!("does not declare a `pub macro`" => Span::call_site());
        };
        let rules = Rules { macro_rule, matches };
        let name = rules.macro_rule.name.clone();

        let mut rules_guard = RULES.lock().unwrap();
        let rules_map = rules_guard.get_or_insert_default();

        let entry = rules_map.entry(rules.macro_rule.name.clone()).or_default();
        let index = entry.len();
        entry.push(rules);

        let span = Span::mixed_site();
        eprintln!("rules: {} v{index}", name);

        let output = TokenStream::from_iter([
            t!["macro_rules", span],
            t!['!'],
            t![&name, span],
            t![braces(
                t![parentheses(
                    t!['$'],
                    t![parentheses(t!['$'], t!["t", span], t![':'], t!["tt", span])],
                    t!['*']
                )],
                t!['=' joint],
                t!['>'],
                t![braces(
                    t![':' joint],
                    t![':'],
                    t!["inception", span],
                    t![':' joint],
                    t![':'],
                    t!["expand", span],
                    t!['!'],
                    t![braces(
                        t![&name, span],
                        t![usize index],
                        t![braces(t!['$'], t![parentheses(t!['$' joint], t!["t", span])], t!['*'])],
                    )],
                )],
                t![';'],
            )],
        ]);
        Ok(output)
    })
}

#[doc(hidden)]
#[proc_macro]
pub fn expand(tokens: TokenStream) -> TokenStream {
    handle(|| {
        let (name, index, group) = parse_expand(tokens, Span::call_site())?;
        let name_str = name.to_string();

        let mut rules_guard = RULES.lock().unwrap();
        let rules_map = rules_guard.get_or_insert_default();

        eprintln!("expand: {name_str} v{index}");

        let Some(macros) = rules_map.get(&name_str) else {
            bail!("unknown inception macro" => name.span());
        };
        let Some(rules) = macros.get(index) else {
            bail!("unknown inception macro version {index} of macro {name_str}" => name.span());
        };

        expand_macro(&name_str, rules, group.stream(), group.span())
    })
}

#[proc_macro_attribute]
pub fn derive(meta: TokenStream, tokens: TokenStream) -> TokenStream {
    handle(|| {
        let names = parse_derive_arguments(meta)?;

        let mut results = vec![tokens.clone()];
        for name in names {
            results.push(TokenStream::from_iter([
                TokenTree::Ident(name),
                t!['!'],
                TokenTree::Group(Group::new(Delimiter::Brace, tokens.clone())),
            ]));
        }

        Ok(TokenStream::from_iter(results))
    })
}

#[proc_macro_attribute]
pub fn attr(meta: TokenStream, tokens: TokenStream) -> TokenStream {
    handle(|| {
        let name = parse_single_argument(meta, Span::call_site())?;
        Ok(TokenStream::from_iter([
            TokenTree::Ident(name),
            t!['!'],
            TokenTree::Group(Group::new(Delimiter::Brace, tokens.clone())),
        ]))
    })
}

fn parse_rules(tokens: TokenStream) -> MResult<Vec<Rule>> {
    let start = Instant::now();
    let mut parser = RuleParser::new(tokens, Span::call_site());
    let mut rules = Vec::new();
    while let Some(rule) = parser.parse_rule()? {
        rules.push(rule);
    }

    if !parser.is_empty() {
        bail!("these leftover tokens couldn't be parsed" => parser.peek().unwrap().span());
    }

    eprintln!("parsing rules took {:?}", start.elapsed());

    Ok(rules)
}

fn handle<F: FnOnce() -> Result<TokenStream, MacroError>>(closure: F) -> TokenStream {
    match closure() {
        Ok(output) => output,
        Err(MacroError { message, span, stream: Some(stream) }) => {
            errors::error_with(&message, span, stream)
        }
        Err(MacroError { message, span, stream: None }) => errors::error(&message, span),
    }
}
