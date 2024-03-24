use std::io::Write;

use crate::ir::RichToken;

pub fn line_by_line(writer: &mut impl Write, tokens: &[RichToken]) {
    for token in tokens {
        let bytes = token.as_bytes();
        println!(
            "{token:?} -> {}",
            String::from_utf8(bytes.clone().into_owned()).unwrap()
        );
        writer.write_all(&bytes).unwrap();
        writer.write_all(&[b'\n']).unwrap();
    }
    writer.flush().unwrap();
}

/// Unformat into a rectangle
pub fn block(writer: &mut impl Write, tokens: &[RichToken], width: usize) {
    todo!()
}
