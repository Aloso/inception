mod parser;

use core::fmt;
use std::{collections::HashMap, fmt::Write};

pub(crate) use parser::RuleParser;
use proc_macro::Spacing;

pub(crate) struct Path {
    pub(crate) path: Vec<String>,
}

impl Path {
    pub(crate) fn new(path: Vec<String>) -> Self {
        Path { path }
    }
}

impl fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, segment) in self.path.iter().enumerate() {
            if i > 0 {
                let _ = write!(f, ".");
            }
            let _ = write!(f, "{}", segment);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Punct {
    pub(crate) char: char,
    pub(crate) spacing: Spacing,
}

impl From<proc_macro::Punct> for Punct {
    fn from(value: proc_macro::Punct) -> Self {
        Punct { char: value.as_char(), spacing: value.spacing() }
    }
}

impl From<Punct> for proc_macro::Punct {
    fn from(value: Punct) -> Self {
        proc_macro::Punct::new(value.char, value.spacing)
    }
}

impl fmt::Display for Punct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.char, f)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Delimiter {
    Parenthesis,
    Bracket,
    Brace,
    None,
}

impl From<Delimiter> for proc_macro::Delimiter {
    fn from(value: Delimiter) -> Self {
        match value {
            Delimiter::Parenthesis => proc_macro::Delimiter::Parenthesis,
            Delimiter::Brace => proc_macro::Delimiter::Brace,
            Delimiter::Bracket => proc_macro::Delimiter::Bracket,
            Delimiter::None => proc_macro::Delimiter::None,
        }
    }
}

impl From<proc_macro::Delimiter> for Delimiter {
    fn from(value: proc_macro::Delimiter) -> Self {
        match value {
            proc_macro::Delimiter::Parenthesis => Delimiter::Parenthesis,
            proc_macro::Delimiter::Brace => Delimiter::Brace,
            proc_macro::Delimiter::Bracket => Delimiter::Bracket,
            proc_macro::Delimiter::None => Delimiter::None,
        }
    }
}

impl PartialEq<proc_macro::Delimiter> for Delimiter {
    fn eq(&self, other: &proc_macro::Delimiter) -> bool {
        matches!(
            (self, other),
            (Delimiter::Parenthesis, proc_macro::Delimiter::Parenthesis)
                | (Delimiter::Bracket, proc_macro::Delimiter::Bracket)
                | (Delimiter::Brace, proc_macro::Delimiter::Brace)
                | (Delimiter::None, proc_macro::Delimiter::None)
        )
    }
}

#[derive(Debug)]
pub(crate) struct Literal(pub(crate) String);

impl From<proc_macro::Literal> for Literal {
    fn from(value: proc_macro::Literal) -> Self {
        Literal(value.to_string())
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Debug)]
pub(crate) enum Rule {
    Match(MatchRule),
    Macro(MacroRule),
}

#[derive(Debug)]
pub(crate) struct Rules {
    pub(crate) macro_rule: MacroRule,
    pub(crate) matches: HashMap<String, Box<[Patterns]>>,
}

pub(crate) struct Patterns(pub(crate) Box<[Pattern]>);

impl fmt::Debug for Patterns {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, pat) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_char(' ')?;
            }
            pat.fmt(f)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) enum Vis {
    Private,
    Public,
}

#[derive(Debug)]
pub(crate) struct MatchRule {
    // pub(crate) vis: Vis,
    pub(crate) name: String,
    pub(crate) pattern_set: Box<[Patterns]>,
}

#[derive(Debug)]
pub(crate) struct MacroRule {
    // pub(crate) vis: Vis,
    pub(crate) name: String,
    pub(crate) patterns: Patterns,
    pub(crate) replacements: Box<[Replacement]>,
}

pub(crate) enum Pattern {
    Group(AstGroup<Self>),
    Ident(String),
    Punct(Punct),
    Literal(Literal),
    Special(SpecialPattern),
}

impl fmt::Debug for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Group(group) => group.fmt(f),
            Self::Ident(ident) => fmt::Display::fmt(ident, f),
            Self::Punct(punct) => fmt::Display::fmt(punct, f),
            Self::Literal(lit) => fmt::Display::fmt(lit, f),
            Self::Special(special) => fmt::Display::fmt(special, f),
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Group(group) => fmt::Display::fmt(group, f),
            Self::Ident(ident) => fmt::Display::fmt(ident, f),
            Self::Punct(punct) => fmt::Display::fmt(punct, f),
            Self::Literal(lit) => fmt::Display::fmt(lit, f),
            Self::Special(special) => fmt::Display::fmt(special, f),
        }
    }
}

#[derive(Debug)]
pub(crate) struct AstGroup<C> {
    pub(crate) delimiter: Delimiter,
    pub(crate) content: Box<[C]>,
}

impl fmt::Display for AstGroup<Pattern> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (start, end) = match self.delimiter {
            Delimiter::Parenthesis => ('(', ')'),
            Delimiter::Bracket => ('[', ']'),
            Delimiter::Brace => ('{', ']'),
            Delimiter::None => return format_patterns(f, &self.content),
        };
        f.write_char(start)?;
        format_patterns(f, &self.content)?;
        f.write_char(end)?;
        Ok(())
    }
}

fn format_patterns(f: &mut fmt::Formatter<'_>, patterns: &[Pattern]) -> fmt::Result {
    for (i, pattern) in patterns.iter().enumerate() {
        if i > 0 {
            f.write_char(' ')?;
        }
        fmt::Display::fmt(&pattern, f)?;
    }
    Ok(())
}

pub(crate) enum Replacement {
    Group(AstGroup<Self>),
    Ident(String),
    Punct(Punct),
    Literal(Literal),
    Special(SpecialReplacement),
}

impl fmt::Debug for Replacement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Group(group) => group.fmt(f),
            Self::Ident(ident) => fmt::Display::fmt(ident, f),
            Self::Punct(punct) => fmt::Display::fmt(punct, f),
            Self::Literal(lit) => fmt::Display::fmt(lit, f),
            Self::Special(special) => special.fmt(f),
        }
    }
}

/// One of:
///
/// - `$foo:bar`
/// - `$foo:bar*` (possible quantifiers: `*`, `+`, `?`)
/// - repetitions with interspersed punctuation (at least one)
///   - middle: `$foo:{bar | ..}`
///   - leading: `$foo:{| bar | ..}`
///   - trailing: `$foo:{bar | .. |}`
/// - optional repetitions with interspersed punctuation: `$foo:{bar | ..}?`
/// - shorthands: `$bar`, `$bar*`, `${...}`, `${...}?`
/// - escape: `$$`, `$+`, `$*`, `$?`, `$:`
pub(crate) struct SpecialPattern {
    pub(crate) name: Option<String>,
    pub(crate) ty: String,
    pub(crate) repeat: Option<Repeat>,
}

impl SpecialPattern {
    pub(crate) fn get_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.ty)
    }

    pub(crate) fn quantifier(&self) -> Option<Quantifier> {
        self.repeat.as_ref().map(|r| r.quantifier)
    }

    pub(crate) fn leading_punct(&self) -> Option<Punct> {
        self.repeat
            .as_ref()
            .and_then(|r| r.interspersed)
            .and_then(|i| if let RepeatKind::Leading = i.kind { Some(i.punct) } else { None })
    }

    pub(crate) fn trailing_punct(&self) -> Option<Punct> {
        self.repeat
            .as_ref()
            .and_then(|r| r.interspersed)
            .and_then(|i| if let RepeatKind::Trailing = i.kind { Some(i.punct) } else { None })
    }
}

impl fmt::Display for SpecialPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => {
                write!(f, "${name}:")?;
            }
            None => f.write_char('$')?,
        };
        match self.repeat {
            Some(Repeat { quantifier, interspersed: Some(Interspersed { kind, punct }) }) => {
                f.write_char('{')?;
                if kind == RepeatKind::Leading {
                    write!(f, "{} ", punct.char)?;
                }
                write!(f, "{} {} ..", self.ty, punct.char)?;
                if kind == RepeatKind::Trailing {
                    write!(f, " {}", punct.char)?;
                }
                f.write_char('}')?;
                if quantifier == Quantifier::Star {
                    f.write_char('?')?;
                }
                Ok(())
            }
            Some(Repeat { quantifier, interspersed: None }) => {
                f.write_str(&self.ty)?;
                fmt::Display::fmt(&quantifier, f)
            }
            None => f.write_str(&self.ty),
        }
    }
}

impl fmt::Debug for SpecialPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpecialPattern")
            .field("name", &self.name.as_ref().map(Display))
            .field("ty", &Display(&self.ty))
            .field("repeat", &self.repeat)
            .finish()
    }
}

#[derive(Debug)]
pub(crate) struct Repeat {
    pub(crate) quantifier: Quantifier,
    pub(crate) interspersed: Option<Interspersed>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Quantifier {
    Star,
    Plus,
    QuestionMark,
}

impl fmt::Display for Quantifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(match self {
            Quantifier::Star => '*',
            Quantifier::Plus => '+',
            Quantifier::QuestionMark => '?',
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Interspersed {
    pub(crate) kind: RepeatKind,
    pub(crate) punct: Punct,
}

impl fmt::Debug for Interspersed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Interspersed")
            .field("kind", &self.kind)
            .field("punct", &Display(&self.punct))
            .finish()
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum RepeatKind {
    Leading,
    Trailing,
    Middle,
}

#[derive(Debug)]
pub(crate) enum SpecialReplacement {
    Path(Path),
    Call { func: String, args: Box<[Expr]> },
    If { condition: Path, body: Box<[Replacement]> },
    ElseIf { condition: Path, body: Box<[Replacement]> },
    Else { body: Box<[Replacement]> },
    For { binding: String, expr: Path, body: Box<[Replacement]> },
}

#[derive(Debug)]
pub(crate) struct Expr {
    pub(crate) path: Path,
}

struct Display<T>(T);

impl<T: fmt::Display> fmt::Debug for Display<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
