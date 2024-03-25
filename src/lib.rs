use lex::{Affinity, Token};

pub mod location;

pub mod emit;
pub mod ir;
pub mod lex;

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
    fn unformat(self, tokens: &[Token<'a>]) -> String;
}

pub trait Nature {
    fn affinity(&self) -> Affinity;
}
