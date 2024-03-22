use std::borrow::Cow;

use crate::{Token, JUNK};

#[derive(Debug)]
pub enum RichToken<'a> {
    Junk(usize),
    Space(usize),
    /// Guaranteed space that separates two tokens can't fuse
    /// ex. - and > can't fuse as they would form a ->
    Spacer,
    Token(Token<'a>),
}

impl<'a> From<Token<'a>> for RichToken<'a> {
    fn from(value: Token<'a>) -> Self {
        RichToken::Token(value)
    }
}

impl<'a> RichToken<'a> {
    pub fn new(tokens: impl Iterator<Item = Token<'a>>) -> Vec<RichToken<'a>> {
        let mut rts = vec![];

        // Nothing repels semicolons, so we just start with this
        let mut last = Token::Semi;

        for token in tokens {
            match (last, token) {
                // let x: ::std... should not become let x:::std....
                (Token::Colon, Token::PathSeparator) => {
                    rts.push(RichToken::Spacer);
                    rts.push(RichToken::Token(Token::PathSeparator));
                }
                // / and * fuse to become /*, the start of a comment
                (Token::Slash, Token::Star) => {
                    rts.push(RichToken::Spacer);
                    rts.push(RichToken::Token(Token::Star));
                }
                // For some reason it doesn't like <-, so < -1 needs can't become <-1
                (Token::LessThan, Token::Minus) => {
                    rts.push(RichToken::Spacer);
                    rts.push(RichToken::Token(Token::Minus));
                }
                // .. and => combine to form ..=> which is parsed as an inclusive range
                (Token::Range, Token::FatArrow) => {
                    rts.push(RichToken::Spacer);
                    rts.push(RichToken::Token(Token::FatArrow));
                }
                (_, Token::Semi) => {
                    rts.push(RichToken::Junk(0));
                    rts.push(RichToken::Token(Token::Semi));
                }
                (_, _) => {
                    rts.push(RichToken::Token(token));
                }
            }

            last = token;
        }

        rts
    }

    pub fn as_bytes(&self) -> Cow<[u8]> {
        match self {
            RichToken::Junk(n) => Cow::Borrowed(JUNK[*n].as_bytes()),
            RichToken::Space(n) => Cow::Owned(std::iter::repeat(b' ').take(*n).collect()),
            RichToken::Spacer => Cow::Borrowed(&[b' ']),
            RichToken::Token(token) => Cow::Borrowed(token.as_str().as_bytes()),
        }
    }
}
