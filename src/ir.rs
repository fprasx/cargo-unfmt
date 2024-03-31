use std::borrow::Cow;

use crate::{
    lex::{Spanned, Token},
    location::Event,
    SafeLen, JUNK,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RichToken<'a> {
    Junk(usize),
    /// Guaranteed space that separates two tokens can't fuse
    /// ex. - and > can't fuse as they would form a ->
    Spacer,
    Token(Spanned<Token<'a>>),
    // // at end of line
    EndOfLineComment(&'static str),
    ExprOpen {
        id: usize,
        reps: usize,
    },
    ExprClose {
        id: usize,
        reps: usize,
    },
}

impl<'a> From<Spanned<Token<'a>>> for RichToken<'a> {
    fn from(value: Spanned<Token<'a>>) -> Self {
        RichToken::Token(value)
    }
}

impl<'a> RichToken<'a> {
    pub fn as_bytes(&self) -> Cow<[u8]> {
        // TODO: use as_str imple and as_bytes
        match self {
            RichToken::Junk(n) => Cow::Borrowed(JUNK[*n].as_bytes()),
            RichToken::Spacer => Cow::Borrowed(b" "),
            RichToken::Token(token) => Cow::Borrowed(token.inner.as_str().as_bytes()),
            RichToken::EndOfLineComment(c) => Cow::Owned(format!("//{c}").into_bytes()),
            RichToken::ExprOpen { reps, .. } => Cow::Owned(b"(".repeat(*reps)),
            RichToken::ExprClose { reps, .. } => Cow::Owned(b")".repeat(*reps)),
        }
    }

    pub fn as_str(&self) -> Cow<str> {
        match self {
            RichToken::Junk(n) => Cow::Borrowed(JUNK[*n]),
            RichToken::Spacer => Cow::Borrowed(" "),
            RichToken::Token(token) => Cow::Borrowed(token.inner.as_str()),
            RichToken::EndOfLineComment(c) => Cow::Owned(format!("//{c}")),
            RichToken::ExprOpen { reps, .. } => Cow::Owned("(".repeat(*reps)),
            RichToken::ExprClose { reps, .. } => Cow::Owned(")".repeat(*reps)),
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.as_str().as_ref().safe_len()
    }
}

#[derive(Debug)]
pub struct Ir<'a> {
    tokens: Vec<RichToken<'a>>,
}

impl<'a> Ir<'a> {
    pub fn new(tokens: impl Iterator<Item = Spanned<Token<'a>>>) -> Self {
        let mut rts = vec![];

        // Nothing repels semicolons, so we just start with this
        let mut last = Token::Semi;

        for token in tokens {
            match (last, token.inner) {
                // let x: ::std... should not become let x:::std....
                (Token::Colon, Token::PathSeparator) => {
                    rts.push(RichToken::Spacer);
                    rts.push(RichToken::Token(token));
                }
                // / and * fuse to become /*, the start of a comment
                (Token::Slash, Token::Star) => {
                    rts.push(RichToken::Spacer);
                    rts.push(RichToken::Token(token));
                }
                // For some reason it doesn't like <-, so < -1 needs can't become <-1
                (Token::LessThan, Token::Minus) => {
                    rts.push(RichToken::Spacer);
                    rts.push(RichToken::Token(token));
                }
                // .. and => combine to form ..=> which is parsed as an inclusive range
                (Token::Range, Token::FatArrow) => {
                    rts.push(RichToken::Spacer);
                    rts.push(RichToken::Token(token));
                }
                // < and < combine to form << (can happen in macro)
                (Token::LessThan, Token::LessThan)
                // > and => combine to form >=> (can happen in macro)
                // Same thing can happen with +, *, ?
                | (Token::GreatherThan | Token::Plus|Token::Star|Token::Question, Token::FatArrow)
                    => {
                    rts.push(RichToken::Spacer);
                    rts.push(RichToken::Token(token));
                }
                (
                    Token::Ident(_) | Token::RawIdent(_) | Token::Literal(_) | Token::Lifetime(_),
                    Token::Ident(_)
                    | Token::RawIdent(_)
                    | Token::Literal(_)
                    | Token::Lifetime(_)
                    // ident # cannot become ident# because then ident looks like
                    // a prefix to some sort of literal
                    | Token::Pound,
                ) => {
                    rts.push(RichToken::Spacer);
                    rts.push(RichToken::Token(token));
                }
                (_, _) => {
                    rts.push(RichToken::Token(token));
                }
            }

            last = token.inner;
        }

        Self { tokens: rts }
    }

    /// Add in junk/expr tokens where needed.
    pub fn populate_events(&self, mut events: &[Spanned<Event>]) -> Ir<'a> {
        let mut out = vec![];

        // Handling expr open/close
        let mut next_id = 0;
        let mut expr_starts = vec![];

        let tokens = self.tokens.iter().cloned();

        // Check for unaligned tokens. This can happen because of how syn parses
        // ranges. In this case we just ignore expropen/close and just go by
        // junk.
        for event in events {
            let mut found = false;
            for token in tokens.clone() {
                if let RichToken::Token(inner) = token {
                    found |= event.aligns_with(&inner)
                } else {
                    continue;
                }
            }
            if !found {
                return self.populate_events(
                    events
                        .iter()
                        .copied()
                        .filter(|event| {
                            matches!(event.inner, Event::StatementStart | Event::StatementEnd)
                        })
                        .collect::<Vec<_>>()
                        .as_slice(),
                );
            }
        }

        for token in tokens {
            match token {
                RichToken::Junk(_)
                | RichToken::Spacer
                | RichToken::EndOfLineComment(_)
                | RichToken::ExprOpen { .. }
                | RichToken::ExprClose { .. } => out.push(token),
                RichToken::Token(inner) => {
                    let mut befores = vec![];
                    let mut afters = vec![];
                    while events
                        .first()
                        .is_some_and(|event| event.aligns_with(&inner))
                    {
                        match events.first().expect("checked this exists").inner {
                            Event::StatementStart => {
                                befores.push(RichToken::Junk(0));
                            }
                            Event::StatementEnd => {
                                afters.push(RichToken::Junk(0));
                            }
                            Event::ExprOpen => {
                                let id = next_id;
                                befores.push(RichToken::ExprOpen { id, reps: 0 });
                                expr_starts.push(id);
                                next_id += 1;
                            }
                            Event::ExprClose => {
                                let id = expr_starts
                                    .pop()
                                    .expect("expression start was already added to stack");
                                afters.push(RichToken::ExprClose { id, reps: 0 });
                            }
                        }
                        events = &events[1..];
                    }

                    out.extend(befores);
                    out.push(token);
                    out.extend(afters);
                }
            }
        }

        Ir { tokens: out }
    }

    pub fn tokens(&self) -> &[RichToken<'a>] {
        self.tokens.as_slice()
    }
}
