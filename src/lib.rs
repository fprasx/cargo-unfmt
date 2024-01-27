use std::fmt::Display;

use rustc_lexer::{Token, TokenKind};

const JUNK: &[&str] = &[
    ";",
    "{}",
    "();",
    "{;};",
    "({});",
    "{();};",
    "*&*&();",
    "((),());",
    "let _=();",
    "if(true){}",
    "let _=||();",
    "loop{break;}",
    "if let _=(){}",
    "while(false){}",
];

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MorphemeKind {
    Repel,
    RepelRight,
    Tight,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Morpheme<'a> {
    str: &'a str,
    kind: MorphemeKind,
}

impl<'a> Morpheme<'a> {
    fn new(str: &'a str, kind: MorphemeKind) -> Self {
        Self { str, kind }
    }

    #[cfg(test)]
    fn repel(str: &'a str) -> Self {
        Morpheme::new(str, MorphemeKind::Repel)
    }

    #[cfg(test)]
    fn repel_right(str: &'a str) -> Self {
        Morpheme::new(str, MorphemeKind::RepelRight)
    }

    #[cfg(test)]
    fn tight(str: &'a str) -> Self {
        Morpheme::new(str, MorphemeKind::Tight)
    }

    fn len(&self) -> usize {
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

pub trait Unformat<'a> {
    fn unformat(self, tokens: &[Morpheme<'a>]) -> String;
}

pub struct BlockUnformatter<const N: usize>;

fn append(buf: &mut String, last: &Morpheme, token: &Morpheme) -> usize {
    match (last.kind, token.kind) {
        (MorphemeKind::Repel | MorphemeKind::RepelRight, MorphemeKind::Repel) => {
            buf.push_str(&format!(" {}", token.str));
            1 + token.len()
        }
        (MorphemeKind::Tight, MorphemeKind::Tight) => {
            match (last.str, token.str) {
                // let x: ::std... should not become let x:::std....
                (":", "::") => {
                    buf.push_str(" ::");
                    3
                }
                // / and * fuse to become /*, the start of a comment
                ("/", "*") => {
                    buf.push_str(" *");
                    2
                }
                // For some reason it doesn't like <-, so < -1 needs can't become <-1
                ("<", "-") => {
                    buf.push_str(" -");
                    2
                }
                _ => {
                    buf.push_str(&token.to_string());
                    token.len()
                }
            }
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

    #[test]
    fn repel_special_cases() {
        // for cases like let x: ::std::usize ...
        assert_eq!(
            BlockUnformatter::<80>.unformat(Tokens::tokenize(": ::").tokens()),
            ": ::",
        );
        // for cases like: let x = y / *z;
        assert_eq!(
            BlockUnformatter::<80>.unformat(Tokens::tokenize("/ *").tokens()),
            "/ *",
        );
        // for cases like: let x = x < -z;
        assert_eq!(
            BlockUnformatter::<80>.unformat(Tokens::tokenize("< -").tokens()),
            "< -",
        );
    }

}
