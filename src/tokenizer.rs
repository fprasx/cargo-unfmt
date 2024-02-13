use anyhow::{anyhow, Context};
use rustc_lexer::TokenKind;
use std::fmt::Display;
use syn::visit::Visit;

use crate::visitors::MacroVisitor;

// The default display for syn errors is extremely minimal.
pub fn display_syn_error(e: syn::Error) -> String {
    format!("error @ {:?}: {e}", e.span().start())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Affinity {
    Repel,
    RepelRight,
    RepelLeft,
    Tight,
    // Junk
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token<'a> {
    pub str: &'a str,
    pub kind: Affinity,
    pub line: usize,
    pub char: usize,
}

impl<'a> Token<'a> {
    pub fn new(str: &'a str, kind: Affinity, line: usize, char: usize) -> Self {
        Self {
            str,
            kind,
            line,
            char,
        }
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str)
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Tokenizer {
    line: usize,
    char: usize,
}

macro_rules! recognize {
    ($self:ident, $source:ident, $token:literal, $tokenfn:ident) => {
        if let Some(remainder) = $source.strip_prefix($token) {
            let token = Token::new($token, Affinity::$tokenfn, $self.line, $self.char);
            $self.char += $token.len();
            return Some((token, remainder));
        }
    };
}

impl Tokenizer {
    pub fn new() -> Self {
        Self { line: 1, char: 1 }
    }

    /// Detects simple tokens such as operators at the beginning of source, returning
    /// the new source with the token stripped away if successful.
    fn recognize_multichar_token<'src>(
        &mut self,
        source: &'src str,
    ) -> Option<(Token<'src>, &'src str)> {
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

    /// Returns `None` if the next token is whitespace or a comment.
    ///
    /// **Note**: `src` must not be empty.
    pub fn recognize_token<'src>(&mut self, src: &'src str) -> (Option<Token<'src>>, &'src str) {
        debug_assert!(!src.is_empty());

        let rustc_lexer::Token { kind, len } = rustc_lexer::first_token(src);
        let (token, rest) = src.split_at(len);

        let mtype = match kind {
            TokenKind::Ident => Some(Affinity::Repel),
            TokenKind::Lifetime { .. } => Some(Affinity::Repel),
            TokenKind::Literal { .. } => Some(Affinity::Repel),
            TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment { .. } => None,
            TokenKind::Colon => Some(Affinity::Tight),
            _ => Some(Affinity::Tight),
        };

        let morpheme = mtype.map(|mtype| Token::new(token, mtype, self.line, self.char));

        let lines = token.split('\n').collect::<Vec<_>>();
        let count = lines.len();
        let last = lines
            .last()
            .expect("split always returns at least one string");

        self.line += count - 1;

        match count {
            1 => self.char += last.len(),
            _ => self.char = last.len() + 1,
        }

        (morpheme, rest)
    }
}

pub fn tokenize_file(mut source: &str) -> anyhow::Result<Vec<Token<'_>>> {
    let mut tokenizer = Tokenizer::new();

    let parsed = syn::parse_file(source)
        .map_err(|e| anyhow!(display_syn_error(e)))
        .context("not valid Rust syntax")?;

    let mut macros = MacroVisitor::new();
    macros.visit_file(&parsed);

    let mut tokens = vec![];

    while !source.is_empty() {
        // rustc_lexer's tokens are very granular, as in two &'s instead of
        // an &&, so we recognize multicharacter tokens like operators manually.
        if let Some((token, rest)) = tokenizer.recognize_multichar_token(source) {
            tokens.push(token);
            source = rest;
        } else {
            let (token, rest) = tokenizer.recognize_token(source);
            tokens.extend(token); // Option<Token> implements iter
            source = rest;
        };
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<'a> Token<'a> {
        #[cfg(test)]
        pub fn repel(str: &'a str, line: usize, char: usize) -> Self {
            Token::new(str, Affinity::Repel, line, char)
        }

        #[cfg(test)]
        pub fn repel_right(str: &'a str, line: usize, char: usize) -> Self {
            Token::new(str, Affinity::RepelRight, line, char)
        }

        #[cfg(test)]
        pub fn tight(str: &'a str, line: usize, char: usize) -> Self {
            Token::new(str, Affinity::Tight, line, char)
        }
    }

    #[test]
    fn idents_separated() {
        assert_eq!(
            tokenize_file("use it;").unwrap(),
            vec![
                Token::repel("use", 1, 1),
                Token::repel("it", 1, 5),
                Token::tight(";", 1, 7),
            ],
        )
    }

    #[test]
    fn lifetime_repels_ident() {
        assert_eq!(
            tokenize_file("type x = &'a ident;").unwrap(),
            vec![
                Token::repel("type", 1, 1),
                Token::repel("x", 1, 6),
                Token::tight("=", 1, 8),
                Token::tight("&", 1, 10),
                Token::repel_right("'a", 1, 11), // the important ones
                Token::repel("ident", 1, 14),    //
                Token::tight(";", 1, 19),
            ]
        )
    }
}
