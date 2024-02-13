use tokenizer::{Morpheme, MorphemeKind};

pub mod visitors;

pub mod formatters;
pub mod tokenizer;

const JUNK: &[&str] = &[
    "",
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

pub trait Unformat<'a> {
    fn unformat(self, tokens: &[Morpheme<'a>]) -> String;
}

fn append(buf: &mut String, last: &Morpheme, token: &Morpheme) -> usize {
    match (last.kind, token.kind) {
        (MorphemeKind::Repel | MorphemeKind::RepelRight, MorphemeKind::Repel) => {
            buf.push_str(&format!(" {}", token.str));
            1 + token.str.len()
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
                // .. and => combine to form ..=> which is parsed as an inclusive range
                ("..", "=>") => {
                    buf.push_str(" =>");
                    3
                }
                _ => {
                    buf.push_str(token.str);
                    token.str.len()
                }
            }
        }
        _ => {
            buf.push_str(token.str);
            token.str.len()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Morpheme;

    #[test]
    fn repel_special_cases() {
        // for cases like let x: ::std::usize ...
        let mut buf = String::from(":");
        append(
            &mut buf,
            &Morpheme::tight(":", 0, 0),
            &Morpheme::tight("::", 0, 0),
        );
        assert_eq!(buf, ": ::");

        // for cases like: let x = y / *z;
        let mut buf = String::from("/");
        append(
            &mut buf,
            &Morpheme::tight("/", 0, 0),
            &Morpheme::tight("*", 0, 0),
        );
        assert_eq!(buf, "/ *");

        // for cases like: let x = x < -z;
        let mut buf = String::from("<");
        append(
            &mut buf,
            &Morpheme::tight("<", 0, 0),
            &Morpheme::tight("-", 0, 0),
        );
        assert_eq!(buf, "< -");
    }
}
