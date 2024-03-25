use std::borrow::Cow;

use crate::{lex::Spanned, Token, JUNK};

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
            RichToken::Space(n) => Cow::Owned(std::iter::repeat(b' ').take(*n).collect()),
            RichToken::Spacer => Cow::Borrowed(&[b' ']),
            RichToken::Token(token) => Cow::Borrowed(token.inner.as_str().as_bytes()),
            RichToken::EndOfLineComment => Cow::Borrowed("//".as_bytes()),
            RichToken::Comment => Cow::Borrowed("/**/".as_bytes()),
        }
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
                (_, Token::Semi) => {
                    rts.push(RichToken::Junk(0));
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
    // pub fn populate_junk(&self, mut stmts: &[Region]) -> Ir {
    //     let mut prev_token_in_statement = false;
    //     let mut out = vec![];
    //     for token in self.tokens.clone() {
    //         if let Some(region) = stmts.first() {
    //             if region.contains(token)
    //             stmts = &stmts[1..];
    //         } else {
    //             out.push(token)
    //         }
    //     }
    //
    //     Self { tokens: out }
    // }

    pub fn tokens(&self) -> &[RichToken<'a>] {
        self.tokens.as_slice()
    }
}
