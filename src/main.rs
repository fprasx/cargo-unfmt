use anyhow::Context;
use cargo_unfmt::{ir::RichToken, lex::Spanned};

fn main() -> anyhow::Result<()> {
    // let src = include_str!("../test_files/long-rust-file.rs");
    let src = "fn main() { let x: ::std::usize = 1; }";
    let tokens = cargo_unfmt::lex::lex_file(src).context("failed to tokenize")?;
    println!("{tokens:#?}");

    let rts = RichToken::new(tokens.into_iter().map(Spanned::into_inner));
    println!("{rts:#?}");
    Ok(())
}
