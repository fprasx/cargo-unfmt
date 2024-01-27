use std::fmt::Display;

use rustc_lexer::{Token, TokenKind};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Morpheme<'a> {
    Repel(&'a str),
    RepelRight(&'a str),
    RepelColon(&'a str),
    Tight(&'a str),
}

impl Morpheme<'_> {
    fn len(&self) -> usize {
        let str = match self {
            Morpheme::Repel(s) => s,
            Morpheme::Tight(s) => s,
            Morpheme::RepelRight(s) => s,
            Morpheme::RepelColon(s) => s,
        };
        str.len()
    }
}

impl Display for Morpheme<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Morpheme::Repel(s) => s,
            Morpheme::Tight(s) => s,
            Morpheme::RepelRight(s) => s,
            Morpheme::RepelColon(s) => s,
        };
        write!(f, "{str}")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Tokens<'a> {
    tokens: Vec<Morpheme<'a>>,
}

macro_rules! recognize {
    ($source:ident, $tokens:ident, $token:literal, $tokenfn:ident) => {
        if let Some(remainder) = $source.strip_prefix($token) {
            $tokens.push(Morpheme::$tokenfn($token));
            $source = remainder;
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

            recognize!(source, tokens, "::", RepelColon);

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
                TokenKind::Ident => Morpheme::Repel,
                TokenKind::Lifetime { .. } => Morpheme::RepelRight,
                TokenKind::Literal { kind, .. } => match kind {
                    rustc_lexer::LiteralKind::Int { .. } => Morpheme::Repel,
                    rustc_lexer::LiteralKind::Float { .. } => Morpheme::Repel,
                    rustc_lexer::LiteralKind::Char { .. } => Morpheme::Repel,
                    rustc_lexer::LiteralKind::Byte { .. } => Morpheme::Repel,
                    rustc_lexer::LiteralKind::Str { .. } => Morpheme::Repel,
                    rustc_lexer::LiteralKind::ByteStr { .. } => Morpheme::Repel,
                    rustc_lexer::LiteralKind::RawStr { .. } => Morpheme::Repel,
                    rustc_lexer::LiteralKind::RawByteStr { .. } => Morpheme::Repel,
                },
                TokenKind::Whitespace | TokenKind::LineComment | TokenKind::BlockComment { .. } => {
                    continue;
                }
                TokenKind::Colon => Morpheme::RepelColon,
                _ => Morpheme::Tight,
            };
            tokens.push(mtype(str));
        }
        Self { tokens }
    }

    pub fn format(&self) -> String {
        let mut buf = String::new();
        let mut tokens = self.tokens.iter();
        let Some(mut last) = tokens.next() else {
            return buf;
        };
        buf.push_str(&last.to_string());

        for token in tokens {
            match (last, token) {
                (Morpheme::Repel(_) | Morpheme::RepelRight(_), Morpheme::Repel(t)) => {
                    buf.push_str(&format!(" {t}"))
                }
                (Morpheme::RepelColon(_), Morpheme::RepelColon(t)) => {
                    buf.push_str(&format!(" {t}"))
                }
                _ => buf.push_str(&token.to_string()),
            }
            last = token;
        }

        buf
    }
}

pub trait Unformat<'a> {
    fn unformat(self, tokens: &[Morpheme<'a>]) -> String;
}

pub struct BlockUnformatter<const N: usize>;

fn append(buf: &mut String, last: &Morpheme, token: &Morpheme) -> usize {
    match (last, token) {
        (Morpheme::Repel(_) | Morpheme::RepelRight(_), Morpheme::Repel(t)) => {
            buf.push_str(&format!(" {t}"));
            1 + token.len()
        }
        (Morpheme::RepelColon(_), Morpheme::RepelColon(t)) => {
            buf.push_str(&format!(" {t}"));
            1 + token.len()
        }
        _ => {
            buf.push_str(&token.to_string());
            token.len()
        }
    }
}

impl<'a, const N: usize> Unformat<'a> for BlockUnformatter<N> {
    fn unformat(self, tokens: &[Morpheme<'a>]) -> String {
        let mut tokens = tokens.iter();
        let Some(mut last) = tokens.next() else {
            return String::new();
        };

        let mut char = last.len();
        let mut buf = last.to_string();

        for token in tokens {
            if char > N {
                char = 0;
                buf.push('\n');
                char += append(&mut buf, last, token);
            } else if char + token.len() > N {
                if char + token.len() - N < N - char {
                    append(&mut buf, last, token);
                    buf.push('\n');
                    char = 0;
                } else {
                    buf.push('\n');
                    char = 0;
                    char += append(&mut buf, last, token);
                }
            } else {
                char += append(&mut buf, last, token);
            }

            last = token;
        }

        buf
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
                    Morpheme::Repel("use"),
                    Morpheme::Repel("it"),
                    Morpheme::Repel("fn"),
                ],
            }
        )
    }

    #[test]
    fn lifetime_repels_ident() {
        assert_eq!(
            Tokens::tokenize("'a ident"),
            Tokens {
                tokens: vec![Morpheme::RepelRight("'a"), Morpheme::Repel("ident")]
            }
        )
    }

    /// This situation can happen for something like:
    /// ```rust
    /// let x: ::std::usize = 1;
    /// ```
    /// We don't want to emit `:::`
    #[test]
    fn colons_repel() {
        assert_eq!(
            Tokens::tokenize(":::"),
            Tokens {
                tokens: vec![Morpheme::RepelColon("::"), Morpheme::RepelColon(":")]
            }
        )
    }
}
