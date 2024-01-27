use crate::{append, Unformat, morpheme::Morpheme};

pub struct BlockUnformatter<const N: usize>;

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
