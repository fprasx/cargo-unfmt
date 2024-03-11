use crate::Token;

enum RichToken<'a> {
    Junk(usize),
    Space(usize),
    Token(Token<'a>),
}
