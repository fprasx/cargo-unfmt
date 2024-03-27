use std::io::Write;

use crate::{ir::RichToken, JUNK};

pub fn line_by_line(writer: &mut impl Write, tokens: &[RichToken]) {
    for token in tokens {
        let bytes = token.as_bytes();
        writer.write_all(&bytes).unwrap();
        writer.write_all(&[b'\n']).unwrap();
    }
    writer.flush().unwrap();
}

/// Unformat into a rectangle
pub fn block(writer: &mut impl Write, mut tokens: Vec<RichToken>, width: usize) {
    let mut blocks = vec![];

    tokens.reverse();

    while !tokens.is_empty() {
        let mut block = vec![];
        let mut len = 0;

        while let Some(token) = tokens.pop() {
            // If token itself is longer than limit, end previous line, and add
            // another line with just the token
            if token.len() >= width {
                blocks.push(block);
                block = vec![token];
                break;
            }

            if len + token.len() < width {
                // Happy case, we can add token to line
                block.push(token);
                len += token.len();
            } else {
                // Token overflows line, push it back to the stream and end the
                // line
                tokens.push(token);
                break;
            }
        }

        blocks.push(block);
    }

    // Get each block as close as possible to width
    for block in blocks.iter_mut() {
        adjust_block(block, width);
    }

    for block in blocks {
        for token in block {
            writer.write_all(&token.as_bytes()).unwrap();
        }
        writer.write_all(&[b'\n']).unwrap()
    }
    writer.flush().unwrap()
}

/// Adjust a block to as close to width characters as possible
fn adjust_block(block: &mut Vec<RichToken>, width: usize) {
    let len = block.iter().map(|token| token.len()).sum::<usize>();

    let junks = block
        .iter()
        .enumerate()
        .filter_map(|(i, token)| {
            if let RichToken::Junk(_) = token {
                Some(i)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if junks.is_empty() {
        return;
    }

    let diff = width - len;
    let mut added = 0;
    for junk in &junks {
        match block.get_mut(*junk).unwrap() {
            RichToken::Junk(n) => {
                let addition = (diff - added).min(JUNK.len() - 1 - *n);
                *n += addition;
                added += addition;
            }
            _ => unreachable!("we already checked this is a junk"),
        }
    }
}
