extern crate proc_macro;
use std::{collections::HashMap, sync::Mutex, time::Instant};

use errors::{MResult, MacroError};
use expand::expand_macro;
use macros::{DeriveArgs, Expand, Rules};
use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Span, TokenStream as TokenStream2, TokenTree};

#[macro_use]
mod errors;
#[macro_use]
mod helper;
mod expand;
mod macros;
mod old;

static RULES: Mutex<Option<HashMap<String, Vec<Rules>>>> = Mutex::new(None);

#[proc_macro]
pub fn rules(tokens: TokenStream) -> TokenStream {
    let start = Instant::now();
    let rules = syn::parse_macro_input!(tokens as Rules);
    eprintln!("parsing rules took {:?}", start.elapsed());

    let name = rules.macro_rule.name.clone();

    let mut rules_guard = RULES.lock().unwrap();
    let rules_map = rules_guard.get_or_insert_default();

    let entry = rules_map.entry(rules.macro_rule.name.clone()).or_default();
    let index = entry.len();
    entry.push(rules);

    let span = Span::mixed_site();
    eprintln!("rules: {} v{index}", name);

    TokenStream2::from_iter([
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
    ])
    .into()
}

#[doc(hidden)]
#[proc_macro]
pub fn expand(tokens: TokenStream) -> TokenStream {
    let Expand { name, name_span, index, input, span } = syn::parse_macro_input!(tokens as Expand);

    let mut rules_guard = RULES.lock().unwrap();
    let rules_map = rules_guard.get_or_insert_default();

    eprintln!("expand: {name} v{index}");

    let Some(macros) = rules_map.get(&name) else {
        synbail!(name_span, "unknown inception macro");
    };
    let Some(rules) = macros.get(index) else {
        synbail!(name_span, "unknown inception macro version {index} of macro {name}");
    };

    match expand_macro(&name, rules, input, span) {
        Ok(output) => output.into(),
        Err(MacroError { message, span, stream: Some(stream) }) => {
            errors::error_with(&message, span, stream).into()
        }
        Err(MacroError { message, span, stream: None }) => errors::error(&message, span).into(),
    }
}

#[proc_macro_attribute]
pub fn derive(meta: TokenStream, tokens: TokenStream) -> TokenStream {
    let tokens: proc_macro2::TokenStream = tokens.into();
    let DeriveArgs { names } = syn::parse_macro_input!(meta as DeriveArgs);

    let mut results = vec![tokens.clone()];
    for name in names {
        results.push(TokenStream2::from_iter([
            name.into(),
            t!['!'],
            TokenTree::Group(Group::new(Delimiter::Brace, tokens.clone())),
        ]));
    }

    TokenStream2::from_iter(results).into()
}

#[proc_macro_attribute]
pub fn attr(meta: TokenStream, tokens: TokenStream) -> TokenStream {
    let name = syn::parse_macro_input!(meta as syn::Ident);

    TokenStream2::from_iter([
        TokenTree::Ident(name),
        t!['!'],
        TokenTree::Group(Group::new(Delimiter::Brace, tokens.into())),
    ])
    .into()
}
