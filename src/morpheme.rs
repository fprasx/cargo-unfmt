use rustc_lexer::{Token, TokenKind};
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MorphemeKind {
    Repel,
    RepelRight,
    Tight,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Morpheme<'a> {
    pub str: &'a str,
    pub kind: MorphemeKind,
}

impl<'a> Morpheme<'a> {
    pub fn new(str: &'a str, kind: MorphemeKind) -> Self {
        Self { str, kind }
    }

    #[cfg(test)]
    pub fn repel(str: &'a str) -> Self {
        Morpheme::new(str, MorphemeKind::Repel)
    }

    #[cfg(test)]
    pub fn repel_right(str: &'a str) -> Self {
        Morpheme::new(str, MorphemeKind::RepelRight)
    }

    #[cfg(test)]
    pub fn tight(str: &'a str) -> Self {
        Morpheme::new(str, MorphemeKind::Tight)
    }

    pub fn len(&self) -> usize {
        self.str.len()
    }
}

impl Display for Morpheme<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Tokens<'a> {
    tokens: Vec<Morpheme<'a>>,
}

macro_rules! recognize {
    ($source:ident, $tokens:ident, $token:literal, $tokenfn:ident) => {
        if let Some(remainder) = $source.strip_prefix($token) {
            $tokens.push(Morpheme::new($token, MorphemeKind::$tokenfn));
            $source = remainder;
            continue;
        }
    };
}

impl<'a> Tokens<'a> {
    pub fn tokens(&self) -> &[Morpheme<'a>] {
        &self.tokens
    }

    pub fn tokenize(mut source: &'a str) -> Self {
        let mut tokens = vec![];
        while !source.is_empty() {
            // Order of these matters
            recognize!(source, tokens, "..=", Tight);
            recognize!(source, tokens, "...", Tight);
            recognize!(source, tokens, "..", Tight);

            recognize!(source, tokens, "::", Tight);

            recognize!(source, tokens, "->", Tight);
            recognize!(source, tokens, "=>", Tight);

            recognize!(source, tokens, "==", Tight);
            recognize!(source, tokens, "!=", Tight);

            recognize!(source, tokens, "<=", Tight);
            recognize!(source, tokens, ">=", Tight);

            recognize!(source, tokens, "&&", Tight);
            recognize!(source, tokens, "||", Tight);

            recognize!(source, tokens, ">>=", Tight);
            recognize!(source, tokens, "<<=", Tight);
            recognize!(source, tokens, ">>", Tight);
            recognize!(source, tokens, "<<", Tight);

            recognize!(source, tokens, "+=", Tight);
            recognize!(source, tokens, "-=", Tight);
            recognize!(source, tokens, "*=", Tight);
            recognize!(source, tokens, "/=", Tight);
            recognize!(source, tokens, "%=", Tight);
            recognize!(source, tokens, "^=", Tight);
            recognize!(source, tokens, "&=", Tight);
            recognize!(source, tokens, "|=", Tight);

            let Token { kind, len } = rustc_lexer::first_token(source);
            let (str, remainder) = source.split_at(len);
            source = remainder;

            let mtype = match kind {
                TokenKind::Ident => MorphemeKind::Repel,
                TokenKind::Lifetime { .. } => MorphemeKind::RepelRight,
                TokenKind::Literal { kind, .. } => match kind {
                    rustc_lexer::LiteralKind::Int { .. } => MorphemeKind::Repel,
                    rustc_lexer::LiteralKind::Float { .. } => MorphemeKind::Repel,
                    rustc_lexer::LiteralKind::Char { .. } => MorphemeKind::Repel,
                    rustc_lexer::LiteralKind::Byte { .. } => MorphemeKind::Repel,
                    rustc_lexer::LiteralKind::Str { .. } => MorphemeKind::Repel,
                    rustc_lexer::LiteralKind::ByteStr { .. } => MorphemeKind::Repel,
                    rustc_lexer::LiteralKind::RawStr { .. } => MorphemeKind::Repel,
                    rustc_lexer::LiteralKind::RawByteStr { .. } => MorphemeKind::Repel,
                },
                TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment { .. } => {
                    continue;
                }
                TokenKind::Colon => MorphemeKind::Tight,
                _ => MorphemeKind::Tight,
            };
            tokens.push(Morpheme::new(str, mtype));
        }
        Self { tokens }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idents_separated() {
        assert_eq!(
            Tokens::tokenize("use it fn"),
            Tokens {
                tokens: vec![
                    Morpheme::repel("use"),
                    Morpheme::repel("it"),
                    Morpheme::repel("fn"),
                ],
            }
        )
    }

    #[test]
    fn lifetime_repels_ident() {
        assert_eq!(
            Tokens::tokenize("'a ident"),
            Tokens {
                tokens: vec![Morpheme::repel_right("'a"), Morpheme::repel("ident")]
            }
        )
    }
}
