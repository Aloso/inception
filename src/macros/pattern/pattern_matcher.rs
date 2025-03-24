use core::fmt::{self, Write};

use syn::{Ident, ext::IdentExt, parse::ParseStream};

use crate::{helper::DebugToDisplay, macros::Punct};

use super::{Interspersed, Quantifier, Repeat, RepeatKind};

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
pub(crate) struct PatternMatcher {
    pub(crate) name: Option<String>,
    pub(crate) ty: String,
    pub(crate) repeat: Option<Repeat>,
}

impl PatternMatcher {
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

    pub(super) fn parse_after_dollar(input: ParseStream) -> syn::Result<PatternMatcher> {
        // TODO: use `peek2` to simplify this logic

        if input.peek(syn::Ident::peek_any) {
            let ident = input.call(syn::Ident::parse_any).unwrap();
            if ident == "for" || ident == "if" || ident == "else" || ident == "match" {
                synerr!(ident.span(), "unexpected macro keyword");
            }

            if input.peek(syn::Token![:]) {
                _ = input.parse::<syn::Token![:]>();

                if input.peek(syn::Ident::peek_any) {
                    let ty = input.call(syn::Ident::parse_any).unwrap().to_string();
                    let repeat = parse_quantifier(input)
                        .map(|quantifier| Repeat { quantifier, interspersed: None });
                    Ok(PatternMatcher { name: Some(ident.to_string()), ty, repeat })
                } else {
                    let braced;
                    syn::braced!(braced in input);
                    let mut matcher = parse_braced_group(&braced, Some(ident))?;
                    handle_question_mark(input, &mut matcher);
                    Ok(matcher)
                }
            } else {
                let ty = ident.to_string();
                let repeat = parse_quantifier(input)
                    .map(|quantifier| Repeat { quantifier, interspersed: None });
                Ok(PatternMatcher { name: None, ty, repeat })
            }
        } else {
            let braced;
            syn::braced!(braced in input);
            let mut matcher = parse_braced_group(&braced, None)?;
            handle_question_mark(input, &mut matcher);
            Ok(matcher)
        }
    }
}

impl fmt::Display for PatternMatcher {
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

impl fmt::Debug for PatternMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PatternMatcher")
            .field("name", &self.name.as_ref().map(DebugToDisplay))
            .field("ty", &DebugToDisplay(&self.ty))
            .field("repeat", &self.repeat)
            .finish()
    }
}

fn handle_question_mark(input: ParseStream, matcher: &mut PatternMatcher) {
    if input.peek(syn::Token![?]) {
        if let Some(repeat) = &mut matcher.repeat {
            // ${x + ..}?
            _ = input.parse::<syn::Token![?]>();
            repeat.quantifier = Quantifier::Star;
        }
    }
}

fn parse_quantifier(input: ParseStream) -> Option<Quantifier> {
    if input.peek(syn::Token![+]) {
        _ = input.parse::<syn::Token![+]>();
        Some(Quantifier::Plus)
    } else if input.peek(syn::Token![*]) {
        _ = input.parse::<syn::Token![*]>();
        Some(Quantifier::Star)
    } else if input.peek(syn::Token![?]) {
        _ = input.parse::<syn::Token![?]>();
        Some(Quantifier::QuestionMark)
    } else {
        None
    }
}

fn parse_braced_group(input: ParseStream, name: Option<Ident>) -> syn::Result<PatternMatcher> {
    let leading = input.parse::<proc_macro2::Punct>().ok();
    // TODO: support double punctuation, like `||`

    let ident = input.call(syn::Ident::parse_any)?;
    if name.is_none() && (ident == "for" || ident == "if" || ident == "else" || ident == "match") {
        synerr!(ident.span(), "unexpected macro keyword");
    }

    let middle = input.parse::<proc_macro2::Punct>()?;
    input.parse::<syn::Token![..]>()?;

    let trailing = input.parse::<proc_macro2::Punct>().ok();

    if let Some(leading) = &leading {
        if leading.as_char() != middle.as_char() {
            synerr!(middle.span(), "expected punctuations to be equal");
        }
    }
    if let Some(trailing) = &trailing {
        if trailing.as_char() != middle.as_char() {
            synerr!(trailing.span(), "expected punctuations to be equal");
        }
    }
    let kind = match (leading, trailing) {
        (Some(_), Some(trailing)) => {
            synerr!(
                trailing.span(),
                "setting both leading and trailing punctuations is unsupported"
            );
        }
        (Some(_), None) => RepeatKind::Leading,
        (None, Some(_)) => RepeatKind::Trailing,
        (None, None) => RepeatKind::Middle,
    };

    let interspersed = Some(Interspersed { kind, punct: middle.into() });
    let repeat = Some(Repeat { quantifier: Quantifier::Plus, interspersed });
    Ok(PatternMatcher { name: name.map(|n| n.to_string()), ty: ident.to_string(), repeat })
}
