use crate::{append, tokenizer::Token2, Unformat};

pub struct BlockUnformatter<const N: usize>;

impl<'a, const N: usize> Unformat<'a> for BlockUnformatter<N> {
    fn unformat(self, tokens: &[Token2<'a>]) -> String {
        let mut tokens = tokens.iter();
        let Some(mut last) = tokens.next() else {
            return String::new();
        };

        let mut char = last.as_str().len();
        let mut buf = String::from(last.as_str());

        for token in tokens {
            if char > N {
                char = 0;
                buf.push('\n');
                char += append(&mut buf, last, token);
            } else if char + token.as_str().len() > N {
                if char + token.as_str().len() - N < N - char {
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
