use anyhow::{Context, anyhow};
use rustc_lexer::{Token, TokenKind};
use std::fmt::Display;

/// The default display for syn errors is extremely minimal.
pub fn display_syn_error(e: syn::Error) -> String {
    format!("error @ {:?}: {e}", e.span().start())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MorphemeKind {
    Repel,
    RepelRight,
    RepelLeft,
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
}

impl Display for Morpheme<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str)
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Tokenizer<'a> {
    tokens: Vec<Morpheme<'a>>,
}

macro_rules! recognize {
    ($self:ident, $source:ident, $token:literal, $tokenfn:ident) => {
        if let Some(remainder) = $source.strip_prefix($token) {
            $self
                .tokens
                .push(Morpheme::new($token, MorphemeKind::$tokenfn));
            return Some(remainder);
        }
    };
}

impl<'a> Tokenizer<'a> {
    pub fn new() -> Self {
        Self { tokens: vec![] }
    }

    pub fn tokens(&self) -> &[Morpheme<'a>] {
        &self.tokens
    }

    /// Detects simple tokens such as operators at the beginning of source, returning
    /// the new source with the token stripped away if successful.
    fn recognize_multichar_token(&mut self, source: &'a str) -> Option<&'a str> {
        // Order of these matters
        recognize!(self, source, "..=", Tight);
        recognize!(self, source, "...", Tight);
        recognize!(self, source, "..", Tight);

        recognize!(self, source, "::", Tight);

        recognize!(self, source, "->", Tight);
        recognize!(self, source, "=>", Tight);

        recognize!(self, source, "==", Tight);
        recognize!(self, source, "!=", Tight);

        recognize!(self, source, "<=", Tight);
        recognize!(self, source, ">=", Tight);

        recognize!(self, source, "&&", Tight);
        recognize!(self, source, "||", Tight);

        recognize!(self, source, ">>=", Tight);
        recognize!(self, source, "<<=", Tight);
        recognize!(self, source, ">>", Tight);
        recognize!(self, source, "<<", Tight);

        recognize!(self, source, "+=", Tight);
        recognize!(self, source, "-=", Tight);
        recognize!(self, source, "*=", Tight);
        recognize!(self, source, "/=", Tight);
        recognize!(self, source, "%=", Tight);
        recognize!(self, source, "^=", Tight);
        recognize!(self, source, "&=", Tight);
        recognize!(self, source, "|=", Tight);

        None
    }

    pub fn tokenize_file(mut source: &'a str) -> anyhow::Result<Vec<Morpheme>> {
        let mut tokenizer = Self::new();
        let parsed = syn::parse_file(source).map_err(|e| anyhow!(display_syn_error(e))).context("not valid Rust syntax")?;

        while !source.is_empty() {
            // rustc_lexer's tokens are very granular, as in two &'s instead of
            // an &&, so we recognize multicharacter tokens like operators manually.
            if let Some(remainder) = tokenizer.recognize_multichar_token(source) {
                source = remainder;
            }

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
            tokenizer.tokens.push(Morpheme::new(str, mtype));
        }
        Ok(tokenizer.tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idents_separated() {
        assert_eq!(
            Tokenizer::tokenize_file("use it;").unwrap(),
            vec![
                Morpheme::repel("use"),
                Morpheme::repel("it"),
                Morpheme::tight(";"),
            ],
        )
    }

    #[test]
    fn lifetime_repels_ident() {
        assert_eq!(
            Tokenizer::tokenize_file("type x = &'a ident;").unwrap(),
            vec![
                Morpheme::repel("type"),
                Morpheme::repel("x"),
                Morpheme::tight("="),
                Morpheme::tight("&"),
                Morpheme::repel_right("'a"), // the important ones
                Morpheme::repel("ident"),    //
                Morpheme::tight(";"),
            ]
        )
    }
}
