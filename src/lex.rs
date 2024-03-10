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
pub struct Spanned<T> {
    inner: T,
    pub line: usize,
    pub char: usize,
}

impl<T> Spanned<T> {
    pub fn new(inner: T, line: usize, char: usize) -> Self {
        Self { inner, line, char }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

/// Internal type that keeps line/char counts as we lex.
#[derive(Debug, PartialEq, Eq, Default)]
struct Tokenizer {
    line: usize,
    char: usize,
}

macro_rules! recognize {
    ($self:ident, $source:ident, $token:literal, $tokenfn:expr) => {
        if let Some(remainder) = $source.strip_prefix($token) {
            let token = $tokenfn;
            let token = Spanned::new(token, $self.line, $self.char);
            $self.char += $token.len();
            return Some((token, remainder));
        }
    };
}

pub fn lex_file(mut source: &str) -> anyhow::Result<Vec<Spanned<Token<'_>>>> {
    let mut tokenizer = Tokenizer::new();

    let parsed = syn::parse_file(source)
        .map_err(|e| anyhow!(display_syn_error(e)))
        .context("not valid Rust syntax")?;

    let mut macros = MacroVisitor::new();
    macros.visit_file(&parsed);

    let mut tokens = vec![];

    while !source.is_empty() {
        if let Some((token, rest)) = tokenizer.lex_textual_token(source) {
            // Token is None if it lex_textual_token just stripped away whitespace
            if let Some(token) = token {
                tokens.push(token);
            }
            source = rest;
        } else if let Some((token, rest)) = tokenizer.lex_punctuation_token(source) {
            tokens.push(token); // Option<Token> implements iter
            source = rest;
        } else {
            panic!("file should be valid rust syntax but could not detect next token")
        }
    }
    Ok(tokens)
}

impl Tokenizer {
    fn new() -> Self {
        Self { line: 1, char: 1 }
    }

    /// Lexes simple tokens consisting of punction, such as operators.
    ///
    /// We have a custom function for doing this because rustc_lexer is extremely
    /// granular. For example, it will barse && as two binary &'s instead of as
    /// && (logical and).
    fn lex_punctuation_token<'src>(
        &mut self,
        source: &'src str,
    ) -> Option<(Spanned<Token<'src>>, &'src str)> {
        recognize!(self, source, "..=", Token::RangeInclusive);
        recognize!(self, source, "...", Token::VariadicArgs);
        recognize!(self, source, "..", Token::Range);

        recognize!(self, source, "::", Token::PathSeparator);

        recognize!(self, source, "->", Token::ParameterArrow);
        recognize!(self, source, "=>", Token::FatArrow);

        recognize!(self, source, "==", Token::EqualCheck);
        recognize!(self, source, "!=", Token::NotEqual);

        recognize!(self, source, "<=", Token::LessThanEq);
        recognize!(self, source, ">=", Token::GreaterThanEq);

        recognize!(self, source, "&&", Token::BooleanAnd);
        recognize!(self, source, "||", Token::BooleanOr);

        recognize!(self, source, ">>=", Token::ShiftRightAssign);
        recognize!(self, source, "<<=", Token::ShiftLeftAssign);
        recognize!(self, source, ">>", Token::ShiftRight);
        recognize!(self, source, "<<", Token::ShiftLeft);

        recognize!(self, source, "+=", Token::AddAssign);
        recognize!(self, source, "-=", Token::SubAssign);
        recognize!(self, source, "*=", Token::MulAssign);
        recognize!(self, source, "/=", Token::DivAssign);
        recognize!(self, source, "%=", Token::ModAssign);
        recognize!(self, source, "^=", Token::XorAssign);
        recognize!(self, source, "&=", Token::AndAssign);
        recognize!(self, source, "|=", Token::OrAssign);

        recognize!(self, source, "(", Token::OpenParen);
        recognize!(self, source, ")", Token::CloseParen);
        recognize!(self, source, "{", Token::OpenBrace);
        recognize!(self, source, "}", Token::CloseBrace);
        recognize!(self, source, "[", Token::OpenBracket);
        recognize!(self, source, "]", Token::CloseBracket);

        recognize!(self, source, ":", Token::Colon);
        recognize!(self, source, ";", Token::Semi);
        recognize!(self, source, ",", Token::Comma);
        recognize!(self, source, ".", Token::Dot);
        recognize!(self, source, "@", Token::At);
        recognize!(self, source, "#", Token::Pound);
        recognize!(self, source, "~", Token::Tilde);
        recognize!(self, source, "?", Token::Question);
        recognize!(self, source, "$", Token::Dollar);
        recognize!(self, source, "=", Token::Eq);
        recognize!(self, source, "!", Token::Not);
        recognize!(self, source, "<", Token::LessThan);
        recognize!(self, source, ">", Token::GreatherThan);
        recognize!(self, source, "-", Token::Minus);
        recognize!(self, source, "&", Token::And);
        recognize!(self, source, "|", Token::Or);
        recognize!(self, source, "+", Token::Plus);
        recognize!(self, source, "*", Token::Star);
        recognize!(self, source, "/", Token::Slash);
        recognize!(self, source, "^", Token::Caret);
        recognize!(self, source, "%", Token::Percent);

        None
    }

    /// Lexes textual tokens such as indentifiers.
    ///
    /// Returns `Some((None, src))` if the next token is a comment or whitespace.
    pub fn lex_textual_token<'src>(
        &mut self,
        src: &'src str,
    ) -> Option<(Option<Spanned<Token<'src>>>, &'src str)> {
        debug_assert!(!src.is_empty());

        let rustc_lexer::Token { kind, len } = rustc_lexer::first_token(src);
        let (token_str, rest) = src.split_at(len);

        let token = match kind {
            TokenKind::Ident => Token::Ident(token_str),
            TokenKind::Lifetime { .. } => Token::Lifetime(token_str),
            TokenKind::Literal { .. } => Token::Literal(token_str),
            TokenKind::RawIdent => Token::RawIdent(token_str),
            TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment { .. } => {
                self.advance_counts(token_str);
                return Some((None, rest));
            }
            TokenKind::Unknown => {
                panic!("src must correspond to valid rust | invalid token: {token_str}")
            }
            _ => {
                return None; // handled by recognize_multichar_token
            }
        };

        let token = Spanned::new(token, self.line, self.char);
        self.advance_counts(token_str);

        Some((Some(token), rest))
    }

    fn advance_counts(&mut self, token: &str) {
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
    }
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token<'a> {
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

impl<'a> Token<'a> {
    pub fn as_str(&self) -> &'a str {
        match self {
            Token::Ident(s) => s,
            Token::Lifetime(s) => s,
            Token::Literal(s) => s,
            Token::RawIdent(s) => s,
            Token::RangeInclusive => "..=",
            Token::VariadicArgs => "...",
            Token::Range => "..",
            Token::PathSeparator => "::",
            Token::ParameterArrow => "->",
            Token::FatArrow => "=>",
            Token::EqualCheck => "==",
            Token::NotEqual => "!=",
            Token::LessThanEq => "<=",
            Token::GreaterThanEq => ">=",
            Token::BooleanAnd => "&&",
            Token::BooleanOr => "||",
            Token::ShiftRightAssign => ">>=",
            Token::ShiftLeftAssign => "<<=",
            Token::ShiftRight => ">>",
            Token::ShiftLeft => "<<",
            Token::AddAssign => "+=",
            Token::SubAssign => "-=",
            Token::MulAssign => "*=",
            Token::DivAssign => "/=",
            Token::ModAssign => "%=",
            Token::XorAssign => "^=",
            Token::AndAssign => "&=",
            Token::OrAssign => "|=",
            Token::OpenParen => "(",
            Token::CloseParen => ")",
            Token::OpenBrace => "{",
            Token::CloseBrace => "}",
            Token::OpenBracket => "[",
            Token::CloseBracket => "]",
            Token::Colon => ":",
            Token::Semi => ";",
            Token::Comma => ",",
            Token::Dot => ".",
            Token::At => "@",
            Token::Pound => "#",
            Token::Tilde => "~",
            Token::Question => "?",
            Token::Dollar => "$",
            Token::Eq => "=",
            Token::Not => "!",
            Token::LessThan => "<",
            Token::GreatherThan => ">",
            Token::Minus => "-",
            Token::And => "&",
            Token::Or => "|",
            Token::Plus => "+",
            Token::Star => "*",
            Token::Slash => "/",
            Token::Caret => "^",
            Token::Percent => "%",
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.as_str().len()
    }
}

impl<'a> Nature for Token<'a> {
    fn affinity(&self) -> Affinity {
        match self {
            Token::Ident(_) | Token::Lifetime(_) | Token::Literal(_) | Token::RawIdent(_) => {
                Affinity::Repel
            }
            Token::RangeInclusive
            | Token::VariadicArgs
            | Token::Range
            | Token::PathSeparator
            | Token::ParameterArrow
            | Token::FatArrow
            | Token::EqualCheck
            | Token::NotEqual
            | Token::LessThanEq
            | Token::GreaterThanEq
            | Token::BooleanAnd
            | Token::BooleanOr
            | Token::ShiftRightAssign
            | Token::ShiftLeftAssign
            | Token::ShiftRight
            | Token::ShiftLeft
            | Token::AddAssign
            | Token::SubAssign
            | Token::MulAssign
            | Token::DivAssign
            | Token::ModAssign
            | Token::XorAssign
            | Token::AndAssign
            | Token::OrAssign
            | Token::OpenParen
            | Token::CloseParen
            | Token::OpenBrace
            | Token::CloseBrace
            | Token::OpenBracket
            | Token::CloseBracket
            | Token::Colon
            | Token::Semi
            | Token::Comma
            | Token::Dot
            | Token::At
            | Token::Pound
            | Token::Tilde
            | Token::Question
            | Token::Dollar
            | Token::Eq
            | Token::Not
            | Token::LessThan
            | Token::GreatherThan
            | Token::Minus
            | Token::And
            | Token::Or
            | Token::Plus
            | Token::Star
            | Token::Slash
            | Token::Caret
            | Token::Percent => Affinity::Tight,
        }
    }
}
