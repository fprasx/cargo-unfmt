use tokenizer::{Affinity, Token, Token2};

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
    fn unformat(self, tokens: &[Token2<'a>]) -> String;
}

pub trait Nature {
    fn affinity(&self) -> Affinity;
}

fn append(buf: &mut String, last: &Token2, token: &Token2) -> usize {
    match (last.affinity(), token.affinity()) {
        (Affinity::Repel, Affinity::Repel) => {
            buf.push(' ');
            let str = token.as_str();
            buf.push_str(str);
            1 + str.len()
        }
        (Affinity::Tight, Affinity::Tight) => {
            match (last, token) {
                // let x: ::std... should not become let x:::std....
                (Token2::Colon, Token2::PathSeparator) => {
                    buf.push(' ');
                    let str = token.as_str();
                    buf.push_str(str);
                    1 + str.len()
                }
                // / and * fuse to become /*, the start of a comment
                (Token2::Slash, Token2::Star) => {
                    buf.push(' ');
                    let str = token.as_str();
                    buf.push_str(str);
                    1 + str.len()
                }
                // For some reason it doesn't like <-, so < -1 needs can't become <-1
                (Token2::LessThan, Token2::Minus) => {
                    buf.push(' ');
                    let str = token.as_str();
                    buf.push_str(str);
                    1 + str.len()
                }
                // .. and => combine to form ..=> which is parsed as an inclusive range
                (Token2::Range, Token2::FatArrow) => {
                    buf.push(' ');
                    let str = token.as_str();
                    buf.push_str(str);
                    1 + str.len()
                }
                _ => {
                    let str = token.as_str();
                    buf.push_str(str);
                    str.len()
                }
            }
        }
        _ => {
            let str = token.as_str();
            buf.push_str(str);
            str.len()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn repel_special_cases() {
        // for cases like let x: ::std::usize ...
        let mut buf = String::from(":");
        append(
            &mut buf,
            &Token2::Colon,
            &Token2::PathSeparator
        );
        assert_eq!(buf, ": ::");

        // for cases like: let x = y / *z;
        let mut buf = String::from("/");
        append(&mut buf, &Token2::Slash, &Token2::Star);
        assert_eq!(buf, "/ *");

        // for cases like: let x = x < -z;
        let mut buf = String::from("<");
        append(&mut buf, &Token2::LessThan, &Token2::Minus);
        assert_eq!(buf, "< -");

        let mut buf = String::from("..");
        append(&mut buf, &Token2::Range, &Token2::FatArrow);
        assert_eq!(buf, ".. =>");
    }
}
