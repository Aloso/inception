use proc_macro::{Delimiter, Ident, Span, TokenStream, TokenTree, token_stream::IntoIter};
use std::iter::Peekable;

use crate::errors::MResult;

use super::{MacroRule, MatchRule, Rule, Vis};

mod path;
mod patterns;
mod replacements;

pub(crate) struct RuleParser {
    iter: Peekable<IntoIter>,
    last_span: Span,
}

#[allow(unused)]
impl RuleParser {
    pub(crate) fn new(tokens: TokenStream, last_span: Span) -> Self {
        RuleParser { iter: tokens.into_iter().peekable(), last_span }
    }

    pub(crate) fn is_empty(&mut self) -> bool {
        self.iter.peek().is_none()
    }

    pub(crate) fn peek(&mut self) -> Option<&TokenTree> {
        self.iter.peek()
    }

    pub(crate) fn advance(&mut self) {
        if let Some(tt) = self.iter.next() {
            self.last_span = tt.span();
        }
    }

    pub(crate) fn expect_punct(&mut self, c: char) -> MResult<Span> {
        match self.iter.peek() {
            Some(TokenTree::Punct(p)) if p.as_char() == c => {
                self.last_span = p.span();
                _ = self.iter.next();
                Ok(self.last_span)
            }
            Some(tt) => bail!("expected `{c}`" => tt.span()),
            None => bail!("expected `{c}`" => self.last_span),
        }
    }

    fn expect_ident(&mut self, expected: &str) -> Option<Span> {
        let Some(TokenTree::Ident(ident)) = self.iter.peek() else {
            return None;
        };
        if ident.to_string() == expected {
            self.last_span = ident.span();
            _ = self.iter.next();
            Some(self.last_span)
        } else {
            None
        }
    }

    fn span(&mut self) -> Span {
        match self.iter.peek() {
            Some(tt) => tt.span(),
            None => self.last_span,
        }
    }

    fn parse_ident(&mut self) -> MResult<Ident> {
        let Some(TokenTree::Ident(ident)) = self.iter.peek() else {
            bail!("expected identifier" => self.span());
        };
        let ident = ident.clone();
        self.last_span = ident.span();
        _ = self.iter.next();
        Ok(ident)
    }

    pub(crate) fn parse_rule(&mut self) -> MResult<Option<Rule>> {
        let vis = self.expect_ident("pub");

        if self.is_empty() {
            if let Some(vis_span) = vis {
                bail!("expected `match` or `macro`" => vis_span);
            }
            return Ok(None);
        };

        let rule_inner = self.parse_rule_inner(match vis {
            Some(_) => Vis::Public,
            None => Vis::Private,
        })?;
        Ok(Some(rule_inner))
    }

    fn parse_rule_inner(&mut self, vis: Vis) -> MResult<Rule> {
        match self.expect_ident("macro") {
            Some(_) => self.parse_macro_rule(vis).map(Rule::Macro),
            None => match self.expect_ident("match") {
                Some(_) => self.parse_match_rule(vis).map(Rule::Match),
                None => bail!("expected `macro` or `match`" => self.span()),
            },
        }
    }

    fn parse_macro_rule(&mut self, vis: Vis) -> MResult<MacroRule> {
        let Some(TokenTree::Ident(ident)) = self.iter.peek() else {
            bail!("expected identifier after `macro`" => self.span());
        };
        let name = ident.to_string();
        self.advance();

        let Some(TokenTree::Group(group)) = self.peek() else {
            bail!("expected parentheses" => self.span());
        };
        let Delimiter::Parenthesis = group.delimiter() else {
            bail!("expected parentheses" => self.span());
        };
        let pattern = patterns::parse(group.stream(), 1024)?;
        self.advance();

        let Some(TokenTree::Group(group)) = self.peek() else {
            bail!("expected braces" => self.span());
        };
        let Delimiter::Brace = group.delimiter() else {
            bail!("expected braces" => self.span());
        };
        let replacements = replacements::parse(group.stream(), 1024)?;
        self.advance();

        Ok(MacroRule { name, patterns: pattern, replacements })
    }

    fn parse_match_rule(&mut self, vis: Vis) -> MResult<MatchRule> {
        let Some(TokenTree::Ident(ident)) = self.iter.peek() else {
            bail!("expected match rule name" => self.span())
        };
        let name = ident.to_string();
        self.advance();
        let Some(_) = self.expect_ident("as") else { bail!("expected `as`" => self.span()) };

        _ = self.expect_punct('|');

        let mut patterns = vec![];

        loop {
            let Some(TokenTree::Group(group)) = self.peek() else {
                bail!("expected parentheses or pipe" => self.span());
            };
            let Delimiter::Parenthesis = group.delimiter() else {
                bail!("expected parentheses" => self.span());
            };
            let pattern = patterns::parse(group.stream(), 1024)?;
            self.advance();
            patterns.push(pattern);

            if let Some(TokenTree::Punct(p)) = self.peek() {
                if p.as_char() == '|' {
                    self.advance();
                    continue;
                }
            }
            break;
        }

        self.expect_punct(';')?;
        Ok(MatchRule { name, pattern_set: patterns.into_boxed_slice() })
    }
}

fn consume_punct(iter: &mut Peekable<IntoIter>, expected: char) -> Option<Span> {
    match iter.peek() {
        Some(TokenTree::Punct(p)) if p.as_char() == expected => {
            let span = p.span();
            let _ = iter.next();
            Some(span)
        }
        _ => None,
    }
}
