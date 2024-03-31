use std::io::Write;

use crate::{
    ir::{Ir, RichToken},
    lex::Token,
    JUNK,
};

fn block_len(block: &[RichToken]) -> usize {
    block.iter().map(|token| token.len()).sum::<usize>()
}

/// Unformat into a rectangle
pub fn block(writer: &mut impl Write, ir: &Ir, width: usize) {
    let mut blocks = vec![];

    let mut tokens = ir.tokens().to_vec();
    tokens.reverse();

    // Don't exceed this point; we can always fill with a comment
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

    // Add in exprs
    adjust_exprs(block, width);

    // If we're one away, add a space in the middle.
    let len = block_len(block);
    if len == width - 1 {
        block.insert(block.len() / 2, RichToken::Spacer);
    }

    // Add comments to end of line
    let len = block_len(block);
    if len < width && !ends_in_slash(block) {
        let comment_text_len = (width - len).saturating_sub(2);
        block.push(RichToken::EndOfLineComment(
            JUNK[comment_text_len.min(JUNK.len() - 1)],
        ));
    }
}

fn ends_in_slash(block: &[RichToken]) -> bool {
    let mut i = block.len() - 1;
    // Ignore control tokens
    while i > 0 && !matches!(block.get(i).unwrap(), RichToken::Token(_)) {
        i -= 1;
    }
    is_slash(block.get(i).unwrap())
}

fn is_slash(token: &RichToken) -> bool {
    if let RichToken::Token(token) = token {
        matches!(token.inner, Token::Slash)
    } else {
        false
    }
}

fn adjust_stmts(block: &mut [RichToken], width: usize) {
    let len = block_len(block);

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

    let diff = width.saturating_sub(len);
    adjust_stmts_by(block, diff, &junks);
}

fn adjust_stmts_by(block: &mut [RichToken], n: usize, junks: &[usize]) {
    let per_junk = n / junks.len();
    let extra = n % junks.len();

    for junk in junks {
        if let RichToken::Junk(n) = block.get_mut(*junk).unwrap() {
            *n += per_junk;
        } else {
            panic!("we already checked this is a junk")
        }
    }

    for junk in junks.iter().take(extra) {
        if let RichToken::Junk(n) = block.get_mut(*junk).unwrap() {
            *n += 1;
        } else {
            panic!("we already checked this is a junk")
        }
    }
}

fn adjust_exprs(block: &mut [RichToken], width: usize) {
    let len = block_len(block);

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

    let diff = width.saturating_sub(len);

    if diff % 2 == 0 {
        adjust_exprs_by(block, diff, &exprs)
    } else {
        // Difference is odd, and we can only add an even number of characters.
        // Leave space for an end of line comment
        adjust_exprs_by(block, diff, &exprs);
    }
}

/// Adjust exprs to add `n` characters.
fn adjust_exprs_by(block: &mut [RichToken], n: usize, exprs: &[(usize, usize)]) {
    let iters = n / 2; // each addition adds two characters, ( and )

    let per_expr = iters / exprs.len();
    let extra = iters % exprs.len();

    for (open, close) in exprs {
        if let RichToken::ExprOpen { reps, .. } = block.get_mut(*open).unwrap() {
            *reps += per_expr;
        } else {
            panic!("we already checked this is an expropen")
        }
        if let RichToken::ExprClose { reps, .. } = block.get_mut(*close).unwrap() {
            *reps += per_expr;
        } else {
            panic!("we already checked this is an exprclose",)
        }
    }

    for (open, close) in exprs.iter().take(extra) {
        if let RichToken::ExprOpen { reps, .. } = block.get_mut(*open).unwrap() {
            *reps += 1;
        } else {
            panic!("we already checked this is an expropen")
        }
        if let RichToken::ExprClose { reps, .. } = block.get_mut(*close).unwrap() {
            *reps += 1;
        } else {
            panic!("we already checked this is an exprclose",)
        }
    }
}
