use std::{collections::HashMap, fmt};

use proc_macro2::{Delimiter, Group, Spacing, Span, TokenStream, TokenTree};

use crate::{
    Rules,
    errors::MResult,
    macros::{
        Literal, Path,
        pattern::{Pattern, PatternMatcher, Patterns, Quantifier},
    },
};

#[derive(Default, Clone)]
pub(super) struct Match {
    pub(super) tts: Vec<TokenTree>,
    pub(super) children: HashMap<String, Match>,
}

impl Match {
    fn new(tts: Vec<TokenTree>) -> Self {
        Self { tts, children: HashMap::default() }
    }

    fn add_child(&mut self, key: String, tt: TokenTree) {
        let existing = self.children.entry(key).or_default();
        existing.tts.push(tt);
    }

    fn nest(&mut self, key: &str, mat: Match) {
        if let Some(value) = self.children.get_mut(key) {
            value.tts.extend(mat.tts);
            for (inner_key, mat) in mat.children {
                value.nest(&inner_key, mat);
            }
        } else {
            _ = self.children.insert(key.to_string(), mat);
        }
    }

    pub(super) fn replace_child(&mut self, key: &str, tt: TokenTree) {
        if let Some(value) = self.children.get_mut(key) {
            value.tts.clear();
            value.tts.push(tt);
        } else {
            _ = self.children.insert(key.to_string(), Match::new(vec![tt]));
        }
    }

    pub(super) fn find_child(&self, path: &Path) -> Option<Vec<TokenTree>> {
        // TODO: Avoid expensive clones
        let mut results = vec![(self.tts.clone(), &self.children)];

        for segment in &path.0 {
            results = results
                .into_iter()
                .flat_map(|(_, children)| {
                    children.get(segment.as_str()).map(|mat| (mat.tts.clone(), &mat.children))
                })
                .collect();
        }

        Some(results.into_iter().flat_map(|(result, _)| result).collect())
    }
}

impl fmt::Debug for Match {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct DisplayTokens<'a>(&'a Vec<TokenTree>);
        impl fmt::Debug for DisplayTokens<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.iter().map(DisplayToken)).finish()
            }
        }

        struct DisplayToken<'a>(&'a TokenTree);
        impl fmt::Debug for DisplayToken<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(self.0, f)
            }
        }

        f.debug_struct("Match")
            .field("tts", &DisplayTokens(&self.tts))
            .field("children", &self.children)
            .finish()
    }
}

pub(crate) enum Matched {
    Success { offset: usize },
    Failed { error: PatternError },
}

pub(crate) struct PatternError {
    pattern: String,
    last_offset: usize,
    span: Span,
    reasons: Option<Vec<PatternError>>,
}

impl PatternError {
    fn new(pattern: String, span: Span, last_offset: usize) -> Self {
        Self { pattern, span, last_offset, reasons: None }
    }
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn format(error: &PatternError, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
            write!(f, "unmatched pattern `{}`", error.pattern)?;
            if let Some(reasons) = &error.reasons {
                write!(f, ", reasons:")?;
                for reason in reasons {
                    write!(f, "\n    {:indent$}- ", "")?;
                    format(reason, f, indent + 4)?;
                }
            }
            Ok(())
        }

        format(self, f, 0)
    }
}

pub(crate) fn match_patterns(
    rules: &Rules,
    patterns: &[Pattern],
    stream: TokenStream,
) -> MResult<Match> {
    let tts = stream.clone().into_iter().collect::<Vec<_>>();
    let mut result = Match::new(tts);

    match match_patterns_impl(rules, patterns, 0, &mut result)? {
        Matched::Success { offset } if offset < result.tts.len() => {
            bail!("unexpected token" => result.tts[offset].span());
        }
        Matched::Success { .. } => {}
        Matched::Failed { error, .. } => {
            return Err(crate::MacroError {
                message: error.to_string(),
                span: error.span,
                stream: None,
            });
        }
    }
    groupify(&mut result.tts);

    Ok(result)
}

pub(crate) fn match_patterns_impl(
    rules: &Rules,
    patterns: &[Pattern],
    offset: usize,
    result: &mut Match,
) -> MResult<Matched> {
    let mut pattern_idx = 0;
    let mut offset = offset;

    while pattern_idx < patterns.len() {
        let pat = &patterns[pattern_idx];

        if offset >= result.tts.len() {
            if let Pattern::Matcher(PatternMatcher { repeat: Some(repeat), .. }) = pat {
                if let Quantifier::QuestionMark | Quantifier::Star = repeat.quantifier {
                    pattern_idx += 1;
                    continue;
                }
            }

            let span = if offset == 0 { Span::call_site() } else { result.tts[offset - 1].span() };
            return Ok(Matched::Failed { error: PatternError::new(pat.to_string(), span, offset) });
        }
        match match_pattern(rules, pat, offset, result)? {
            Matched::Success { offset: new_offset } => {
                offset = new_offset;
            }
            failed @ Matched::Failed { .. } => {
                return Ok(failed);
            }
        }
        pattern_idx += 1;
    }

    Ok(Matched::Success { offset })
}

pub(crate) fn match_pattern(
    rules: &Rules,
    pat: &Pattern,
    mut offset: usize,
    result: &mut Match,
) -> MResult<Matched> {
    match (pat, &result.tts[offset]) {
        (Pattern::Group(ast_group), TokenTree::Group(group))
            if ast_group.delimiter == group.delimiter() =>
        {
            let inner_result = match_patterns(rules, &ast_group.content, group.stream())?;
            result.children.extend(inner_result.children);
            Ok(Matched::Success { offset: offset + 1 })
        }
        (Pattern::Ident(ident), TokenTree::Ident(tt_ident)) if ident == &tt_ident.to_string() => {
            Ok(Matched::Success { offset: offset + 1 })
        }
        (Pattern::Punct(punct), TokenTree::Punct(tt_punct)) if punct.char == tt_punct.as_char() => {
            Ok(Matched::Success { offset: offset + 1 })
        }
        (Pattern::Literal(Literal(lit)), TokenTree::Literal(tt_lit))
            if lit == &tt_lit.to_string() =>
        {
            Ok(Matched::Success { offset: offset + 1 })
        }

        (Pattern::Matcher(special), tt) => {
            let offset_no_leading_punct = offset;
            if let Some(leading) = &special.leading_punct() {
                match tt {
                    TokenTree::Punct(punct) if punct.as_char() == leading.char => {
                        offset += 1;
                    }
                    _ => {}
                }
            }

            match special_match(rules, special, offset, result)? {
                Matched::Success { offset: offset_new } => offset = offset_new,
                failed @ Matched::Failed { .. } => {
                    if let Some(Quantifier::Star | Quantifier::QuestionMark) = special.quantifier()
                    {
                        // repeating 0 times is allowed
                        return Ok(Matched::Success { offset: offset_no_leading_punct });
                    } else {
                        return Ok(failed);
                    }
                }
            }

            if let Some(repeat) = &special.repeat {
                loop {
                    let offset_no_trailing_punct = offset;
                    if let Some(interspersed) = &repeat.interspersed {
                        if let Some(TokenTree::Punct(punct)) = result.tts.get(offset) {
                            if punct.as_char() == interspersed.punct.char {
                                offset += 1;
                            }
                        }
                    }

                    match special_match(rules, special, offset, result)? {
                        Matched::Success { offset: offset_new } => offset = offset_new,
                        Matched::Failed { .. } => {
                            if special.trailing_punct().is_some() {
                                return Ok(Matched::Success { offset });
                            } else {
                                return Ok(Matched::Success { offset: offset_no_trailing_punct });
                            }
                        }
                    };
                }
            }

            Ok(Matched::Success { offset })
        }

        _ => Ok(Matched::Failed {
            error: PatternError::new(pat.to_string(), result.tts[offset].span(), offset),
        }),
    }
}

fn special_match(
    rules: &Rules,
    pat: &PatternMatcher,
    offset: usize,
    result: &mut Match,
) -> MResult<Matched> {
    let ty = pat.ty.as_str();
    let name = pat.get_name();
    let span = result.tts[offset].span();

    if let Some(pattern_set) = rules.matches.get(ty) {
        let mut next_result = Match::new(result.tts[offset..].to_vec());
        let mut errors = Vec::new();

        for Patterns(patterns) in pattern_set {
            let increment = match match_patterns_impl(rules, patterns, 0, &mut next_result)? {
                Matched::Success { offset } => offset,
                Matched::Failed { error } => {
                    errors.push(error);
                    continue;
                }
            };
            next_result.tts.drain(increment..);
            groupify(&mut next_result.tts);
            result.nest(name, next_result);
            return Ok(Matched::Success { offset: offset + increment });
        }

        let &PatternError { last_offset, span, .. } =
            errors.iter().max_by_key(|e| e.last_offset).expect("errors can't be empty here");
        errors.retain(|e| e.last_offset == last_offset);

        return Ok(Matched::Failed {
            error: PatternError {
                pattern: name.to_string(),
                last_offset,
                span,
                reasons: Some(errors),
            },
        });
    }

    let (tt, increment) = match ty {
        "tt" => match result.tts.get(offset) {
            Some(tt) => (tt.clone(), 1),
            None => {
                return Ok(Matched::Failed {
                    error: PatternError::new(pat.to_string(), span, offset),
                });
            }
        },
        "literal" => match result.tts.get(offset) {
            Some(TokenTree::Literal(lit)) => (lit.clone().into(), 1),
            _ => {
                return Ok(Matched::Failed {
                    error: PatternError::new(pat.to_string(), span, offset),
                });
            }
        },
        "ident" => match result.tts.get(offset) {
            Some(TokenTree::Ident(ident)) => (ident.clone().into(), 1),
            _ => {
                return Ok(Matched::Failed {
                    error: PatternError::new(pat.to_string(), span, offset),
                });
            }
        },
        "lifetime" => match (result.tts.get(offset), result.tts.get(offset + 1)) {
            (Some(tt1 @ TokenTree::Punct(p)), Some(tt2 @ TokenTree::Ident(_)))
                if p.as_char() == '\'' && p.spacing() == Spacing::Joint =>
            {
                let stream = TokenStream::from_iter([tt1.clone(), tt2.clone()]);
                (Group::new(Delimiter::None, stream).into(), 2)
            }
            _ => {
                return Ok(Matched::Failed {
                    error: PatternError::new(pat.to_string(), span, offset),
                });
            }
        },
        _ => {
            let span = result.tts.get(offset).unwrap_or(result.tts.last().unwrap()).span();
            bail!("unknown matcher `{ty}`" => span)
        }
    };

    result.add_child(name.to_owned(), tt);

    Ok(Matched::Success { offset: offset + increment })
}

fn groupify(vec: &mut Vec<TokenTree>) {
    let trees = std::mem::take(vec);
    *vec = vec![TokenTree::Group(Group::new(Delimiter::None, TokenStream::from_iter(trees)))];
}
