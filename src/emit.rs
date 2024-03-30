use std::io::Write;

use crate::{
    ir::{Ir, RichToken},
    JUNK,
};

pub fn line_by_line(writer: &mut impl Write, tokens: &[RichToken]) {
    for token in tokens {
        let bytes = token.as_bytes();
        writer.write_all(&bytes).unwrap();
        writer.write_all(&[b'\n']).unwrap();
    }
    writer.flush().unwrap();
}

/// Unformat into a rectangle
pub fn block(writer: &mut impl Write, ir: &Ir, width: usize) {
    let mut blocks = vec![];

    let mut tokens = ir.tokens().to_vec();
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
    // Remove leading and trailing spacers
    while let Some(RichToken::Spacer) = block.first() {
        block.remove(0);
    }
    while let Some(RichToken::Spacer) = block.last() {
        block.pop();
    }

    // Add in junk
    adjust_stmts(block, width);

    // TODO: adjust exprs
    adjust_exprs(block, width);
    // TODO: add comments to end of line
}

fn adjust_stmts(block: &mut [RichToken], width: usize) {
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

fn adjust_exprs(block: &mut Vec<RichToken>, width: usize) {
    let len = block.iter().map(|token| token.len()).sum::<usize>();

    let mut exprs = vec![];
    for (i, token) in block.iter().enumerate() {
        if let RichToken::ExprOpen { id, .. } = token {
            for (j, close) in block.iter().enumerate().skip(i + 1) {
                if let RichToken::ExprClose { id: close_id, .. } = close {
                    if id == close_id {
                        exprs.push((i, j))
                    }
                }
            }
        }
    }
    if exprs.is_empty() {
        return;
    }

    let diff = width - len;

    if diff % 2 == 0 {
        adjust_exprs_by(block, diff, &exprs)
    } else {
        // Difference is odd, and we can only add an even number of characters.
        // Leace space for an end of line comment
        adjust_exprs_by(block, diff.saturating_sub(3), &exprs);
    }
}

/// Adjust exprs to add `n` characters.
fn adjust_exprs_by(block: &mut Vec<RichToken>, n: usize, exprs: &[(usize, usize)]) {
    for expr in exprs.iter().cycle().take(n / 2) {
        let (fst, snd) = expr;
        if let RichToken::ExprOpen { reps, .. } = block.get_mut(*fst).unwrap() {
            *reps += 1;
        } else {
            unreachable!("we already checked this is an expropen")
        }
        if let RichToken::ExprClose { reps, .. } = block.get_mut(*snd).unwrap() {
            *reps += 1;
        } else {
            unreachable!(
                "we already checked this is an exprclose: {:?}",
                block.get_mut(*snd)
            )
        }

    }
}
