use std::fmt::Display;

use anyhow::{anyhow, Context};
use rustc_lexer::TokenKind;
use syn::visit::Visit;

use crate::{visitors::MacroVisitor, Nature};

// The default display for syn errors is extremely minimal.
pub fn display_syn_error(e: syn::Error) -> String {
    format!("error @ {:?}: {e}", e.span().start())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Affinity {
    Repel,
    Tight,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token<'a> {
    pub inner: Token2<'a>,
    pub line: usize,
    pub char: usize,
}

impl<'a> Token<'a> {
    pub fn new(inner: Token2<'a>, line: usize, char: usize) -> Self {
        Self { inner, line, char }
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Tokenizer {
    line: usize,
    char: usize,
}

macro_rules! recognize {
    ($self:ident, $source:ident, $token:literal, $tokenfn:expr) => {
        if let Some(remainder) = $source.strip_prefix($token) {
            let token = $tokenfn;
            let token = Token::new(token, $self.line, $self.char);
            $self.char += $token.len();
            return Some((token, remainder));
        }
    };
}

impl Display for Token2<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token2<'a> {
    // Complex Tokens
    Ident(&'a str),
    Lifetime(&'a str),
    Literal(&'a str),
    RawIdent(&'a str),

    // Complex Puncation Tokens
    RangeInclusive,
    VariadicArgs,
    Range,

    PathSeparator,

    ParameterArrow,
    FatArrow,

    EqualCheck,
    NotEqual,

    LessThanEq,
    GreaterThanEq,

    BooleanAnd,
    BooleanOr,

    ShiftRightAssign,
    ShiftLeftAssign,
    ShiftRight,
    ShiftLeft,

    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    XorAssign,
    AndAssign,
    OrAssign,

    // Simple Punctation Tokens
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,

    Colon,
    Semi,
    Comma,
    Dot,
    At,
    Pound,
    Tilde,
    Question,
    Dollar,
    Eq,
    Not,
    LessThan,
    GreatherThan,
    Minus,
    And,
    Or,
    Plus,
    Star,
    Slash,
    Caret,
    Percent,
}

impl<'a> Token2<'a> {
    pub fn as_str(&self) -> &'a str {
        match self {
            Token2::Ident(s) => s,
            Token2::Lifetime(s) => s,
            Token2::Literal(s) => s,
            Token2::RawIdent(s) => s,
            Token2::RangeInclusive => "..=",
            Token2::VariadicArgs => "...",
            Token2::Range => "..",
            Token2::PathSeparator => "::",
            Token2::ParameterArrow => "->",
            Token2::FatArrow => "=>",
            Token2::EqualCheck => "==",
            Token2::NotEqual => "!=",
            Token2::LessThanEq => "<=",
            Token2::GreaterThanEq => ">=",
            Token2::BooleanAnd => "&&",
            Token2::BooleanOr => "||",
            Token2::ShiftRightAssign => ">>=",
            Token2::ShiftLeftAssign => "<<=",
            Token2::ShiftRight => ">>",
            Token2::ShiftLeft => "<<",
            Token2::AddAssign => "+=",
            Token2::SubAssign => "-=",
            Token2::MulAssign => "*=",
            Token2::DivAssign => "/=",
            Token2::ModAssign => "%=",
            Token2::XorAssign => "^=",
            Token2::AndAssign => "&=",
            Token2::OrAssign => "|=",
            Token2::OpenParen => "(",
            Token2::CloseParen => ")",
            Token2::OpenBrace => "{",
            Token2::CloseBrace => "}",
            Token2::OpenBracket => "[",
            Token2::CloseBracket => "]",
            Token2::Colon => ":",
            Token2::Semi => ";",
            Token2::Comma => ",",
            Token2::Dot => ".",
            Token2::At => "@",
            Token2::Pound => "#",
            Token2::Tilde => "~",
            Token2::Question => "?",
            Token2::Dollar => "$",
            Token2::Eq => "=",
            Token2::Not => "!",
            Token2::LessThan => "<",
            Token2::GreatherThan => ">",
            Token2::Minus => "-",
            Token2::And => "&",
            Token2::Or => "|",
            Token2::Plus => "+",
            Token2::Star => "*",
            Token2::Slash => "/",
            Token2::Caret => "^",
            Token2::Percent => "%",
        }
    }
}

impl<'a> Nature for Token2<'a> {
    fn affinity(&self) -> Affinity {
        match self {
            Token2::Ident(_) | Token2::Lifetime(_) | Token2::Literal(_) | Token2::RawIdent(_) => {
                Affinity::Repel
            }
            Token2::RangeInclusive
            | Token2::VariadicArgs
            | Token2::Range
            | Token2::PathSeparator
            | Token2::ParameterArrow
            | Token2::FatArrow
            | Token2::EqualCheck
            | Token2::NotEqual
            | Token2::LessThanEq
            | Token2::GreaterThanEq
            | Token2::BooleanAnd
            | Token2::BooleanOr
            | Token2::ShiftRightAssign
            | Token2::ShiftLeftAssign
            | Token2::ShiftRight
            | Token2::ShiftLeft
            | Token2::AddAssign
            | Token2::SubAssign
            | Token2::MulAssign
            | Token2::DivAssign
            | Token2::ModAssign
            | Token2::XorAssign
            | Token2::AndAssign
            | Token2::OrAssign
            | Token2::OpenParen
            | Token2::CloseParen
            | Token2::OpenBrace
            | Token2::CloseBrace
            | Token2::OpenBracket
            | Token2::CloseBracket
            | Token2::Colon
            | Token2::Semi
            | Token2::Comma
            | Token2::Dot
            | Token2::At
            | Token2::Pound
            | Token2::Tilde
            | Token2::Question
            | Token2::Dollar
            | Token2::Eq
            | Token2::Not
            | Token2::LessThan
            | Token2::GreatherThan
            | Token2::Minus
            | Token2::And
            | Token2::Or
            | Token2::Plus
            | Token2::Star
            | Token2::Slash
            | Token2::Caret
            | Token2::Percent => Affinity::Tight,
        }
    }
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
        recognize!(self, source, "..=", Token2::RangeInclusive);
        recognize!(self, source, "...", Token2::VariadicArgs);
        recognize!(self, source, "..", Token2::Range);
        // recognize!(self, source, "a",);
        recognize!(self, source, "::", Token2::PathSeparator);
        // recognize!(self, source, "a",);
        recognize!(self, source, "->", Token2::ParameterArrow);
        recognize!(self, source, "=>", Token2::FatArrow);
        // recognize!(self, source, "a",);
        recognize!(self, source, "==", Token2::EqualCheck);
        recognize!(self, source, "!=", Token2::NotEqual);
        // recognize!(self, source, "a",);
        recognize!(self, source, "<=", Token2::LessThanEq);
        recognize!(self, source, ">=", Token2::GreaterThanEq);
        // recognize!(self, source, "a",);
        recognize!(self, source, "&&", Token2::BooleanAnd);
        recognize!(self, source, "||", Token2::BooleanOr);
        // recognize!(self, source, "a",);
        recognize!(self, source, ">>=", Token2::ShiftRightAssign);
        recognize!(self, source, "<<=", Token2::ShiftLeftAssign);
        recognize!(self, source, ">>", Token2::ShiftRight);
        recognize!(self, source, "<<", Token2::ShiftLeft);
        // recognize!(self, source, "a",);
        recognize!(self, source, "+=", Token2::AddAssign);
        recognize!(self, source, "-=", Token2::SubAssign);
        recognize!(self, source, "*=", Token2::MulAssign);
        recognize!(self, source, "/=", Token2::DivAssign);
        recognize!(self, source, "%=", Token2::ModAssign);
        recognize!(self, source, "^=", Token2::XorAssign);
        recognize!(self, source, "&=", Token2::AndAssign);
        recognize!(self, source, "|=", Token2::OrAssign);
        // recognize!(self, source, "a",);
        recognize!(self, source, "(", Token2::OpenParen);
        recognize!(self, source, ")", Token2::CloseParen);
        recognize!(self, source, "{", Token2::OpenBrace);
        recognize!(self, source, "}", Token2::CloseBrace);
        recognize!(self, source, "[", Token2::OpenBracket);
        recognize!(self, source, "]", Token2::CloseBracket);
        // recognize!(self, source, "a",);
        recognize!(self, source, ":", Token2::Colon);
        recognize!(self, source, ";", Token2::Semi);
        recognize!(self, source, ",", Token2::Comma);
        recognize!(self, source, ".", Token2::Dot);
        recognize!(self, source, "@", Token2::At);
        recognize!(self, source, "#", Token2::Pound);
        recognize!(self, source, "~", Token2::Tilde);
        recognize!(self, source, "?", Token2::Question);
        recognize!(self, source, "$", Token2::Dollar);
        recognize!(self, source, "=", Token2::Eq);
        recognize!(self, source, "!", Token2::Not);
        recognize!(self, source, "<", Token2::LessThan);
        recognize!(self, source, ">", Token2::GreatherThan);
        recognize!(self, source, "-", Token2::Minus);
        recognize!(self, source, "&", Token2::And);
        recognize!(self, source, "|", Token2::Or);
        recognize!(self, source, "+", Token2::Plus);
        recognize!(self, source, "*", Token2::Star);
        recognize!(self, source, "/", Token2::Slash);
        recognize!(self, source, "^", Token2::Caret);
        recognize!(self, source, "%", Token2::Percent);

        None
    }

    /// Returns `None` if the next character is not a character based token.
    /// Returns `Some((None, new_source))` if the next token is a comment or
    /// whitespace.
    ///
    /// **Note**: `src` must not be empty.
    pub fn recognize_token<'src>(
        &mut self,
        src: &'src str,
    ) -> Option<(Option<Token<'src>>, &'src str)> {
        debug_assert!(!src.is_empty());

        let rustc_lexer::Token { kind, len } = rustc_lexer::first_token(src);
        let (token_str, rest) = src.split_at(len);

        let token = match kind {
            TokenKind::Ident => Token2::Ident(token_str),
            TokenKind::Lifetime { .. } => Token2::Lifetime(token_str),
            TokenKind::Literal { .. } => Token2::Literal(token_str),
            TokenKind::RawIdent => Token2::RawIdent(token_str),
            TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment { .. } => {
                return Some((None, rest))
            }
            TokenKind::Unknown => {
                panic!("src must correspond to valid rust | invalid token: {token_str}")
            }
            _ => {
                return None;
            } // handled by recognize_multichar_token
        };

        let token = Token::new(token, self.line, self.char);

        let lines = token_str.split('\n').collect::<Vec<_>>();
        let count = lines.len();
        let last = lines
            .last()
            .expect("split always returns at least one string");

        self.line += count - 1;

        match count {
            1 => self.char += last.len(),
            _ => self.char = last.len() + 1,
        }

        Some((Some(token), rest))
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
        if let Some((token, rest)) = tokenizer.recognize_token(source) {
            if let Some(token) = token {
                tokens.push(token);
            }
            source = rest;
        } else if let Some((token, rest)) = tokenizer.recognize_multichar_token(source) {
            tokens.push(token); // Option<Token> implements iter
            source = rest;
        } else {
            panic!("file should be valid rust syntax but could not detect next token")
        }
    }
    Ok(tokens)
}
