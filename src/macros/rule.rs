use std::collections::HashMap;

use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};

use super::{MacroRule, MatchRule, Vis, pattern::Patterns};

#[derive(Debug)]
pub(crate) enum Rule {
    Match(MatchRule),
    Macro(MacroRule),
}

impl Parse for Rule {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        _ = Vis::parse(input)?;
        if input.peek(syn::Token![macro]) {
            input.parse::<MacroRule>().map(Rule::Macro)
        } else {
            input.parse::<MatchRule>().map(Rule::Match)
        }
    }
}

#[derive(Debug)]
pub(crate) struct Rules {
    pub(crate) macro_rule: MacroRule,
    pub(crate) matches: HashMap<String, Box<[Patterns]>>,
}

impl Parse for Rules {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut parsed_rules = Vec::new();
        while !input.is_empty() {
            parsed_rules.push(input.parse::<Rule>()?);
        }

        let mut macro_rule = None;
        let mut matches = HashMap::new();

        for rule in parsed_rules {
            match rule {
                Rule::Match(match_rule) => {
                    if matches.contains_key(&match_rule.name) {
                        synerr!(Span::call_site(), "duplicate matcher name {}", match_rule.name);
                    }
                    matches.insert(match_rule.name, match_rule.pattern_set);
                }
                Rule::Macro(rule) => {
                    if macro_rule.is_some() {
                        synerr!(Span::call_site(), "inception rules contain multiple macros");
                    }
                    macro_rule = Some(rule);
                }
            }
        }
        let Some(macro_rule) = macro_rule else {
            synerr!(Span::call_site(), "does not declare a `pub macro`");
        };
        Ok(Rules { macro_rule, matches })
    }
}
