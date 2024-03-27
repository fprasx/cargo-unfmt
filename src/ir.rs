use std::borrow::Cow;

use crate::{
    lex::{Spanned, Token},
    location::{Region, RelativePosition},
    JUNK,
};

#[derive(Debug, Copy, Clone)]
pub enum RichToken<'a> {
    Junk(usize),
    Space(usize),
    /// Guaranteed space that separates two tokens can't fuse
    /// ex. - and > can't fuse as they would form a ->
    Spacer,
    Token(Spanned<Token<'a>>),

    // // at end of line
    EndOfLineComment,
    // /**/
    Comment,
}

impl<'a> From<Spanned<Token<'a>>> for RichToken<'a> {
    fn from(value: Spanned<Token<'a>>) -> Self {
        RichToken::Token(value)
    }
}

impl<'a> RichToken<'a> {
    pub fn as_bytes(&self) -> Cow<[u8]> {
        match self {
            RichToken::Junk(n) => Cow::Borrowed(JUNK[*n].as_bytes()),
            RichToken::Space(n) => Cow::Owned(b" ".repeat(*n)),
            RichToken::Spacer => Cow::Borrowed(&[b' ']),
            RichToken::Token(token) => Cow::Borrowed(token.inner.as_str().as_bytes()),
            RichToken::EndOfLineComment => Cow::Borrowed("//".as_bytes()),
            RichToken::Comment => Cow::Borrowed("/**/".as_bytes()),
        }
    }

    pub fn as_str(&self) -> Cow<str> {
        match self {
            RichToken::Junk(n) => Cow::Borrowed(JUNK[*n]),
            RichToken::Space(n) => Cow::Owned(" ".repeat(*n)),
            RichToken::Spacer => Cow::Borrowed(" "),
            RichToken::Token(token) => Cow::Borrowed(token.inner.as_str()),
            RichToken::EndOfLineComment => Cow::Borrowed("//"),
            RichToken::Comment => Cow::Borrowed("/**/"),
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.as_str().len()
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
                (
                    Token::Ident(_) | Token::RawIdent(_) | Token::Literal(_) | Token::Lifetime(_),
                    Token::Ident(_) | Token::RawIdent(_) | Token::Literal(_) | Token::Lifetime(_),
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

    /// Add junk tokens where legal.
    pub fn populate_junk(&self, regions: &[Region]) -> Ir<'a> {
        let mut tokens = vec![];
        let mut src_tokens = self.tokens.iter().cloned().peekable();

        for region in regions {
            if src_tokens.peek().is_none() {
                break;
            }

            while let Some(token) = src_tokens.by_ref().next_if(|token| match token {
                RichToken::Junk(_)
                | RichToken::Space(_)
                | RichToken::Spacer
                | RichToken::EndOfLineComment
                | RichToken::Comment => true,
                RichToken::Token(inner) => match region.find(inner) {
                    RelativePosition::Before => true,
                    RelativePosition::Within => false,
                    RelativePosition::After => panic!("{inner:?}"),
                },
            }) {
                tokens.push(token);
            }

            if !matches!(tokens.last(), Some(RichToken::Junk(_))) {
                tokens.push(RichToken::Junk(0));
            }

            while let Some(token) = src_tokens.by_ref().next_if(|token| match token {
                RichToken::Junk(_)
                | RichToken::Space(_)
                | RichToken::Spacer
                | RichToken::EndOfLineComment
                | RichToken::Comment => true,
                RichToken::Token(inner) => match region.find(inner) {
                    RelativePosition::Before => panic!("{inner:?}"),
                    RelativePosition::Within => true,
                    RelativePosition::After => false,
                },
            }) {
                tokens.push(token);
            }

            if !matches!(tokens.last(), Some(RichToken::Junk(_))) {
                tokens.push(RichToken::Junk(0));
            }
        }

        tokens.extend(src_tokens);

        Self { tokens }
    }

    pub fn tokens(&self) -> &[RichToken<'a>] {
        self.tokens.as_slice()
    }
}
