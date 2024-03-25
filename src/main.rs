use anyhow::Context;
use cargo_unfmt::{ir::Ir, location::StmtVisitor};
use syn::visit::Visit;

fn main() -> anyhow::Result<()> {
    let src = include_str!("../test_files/short-rust-file.rs");
    let tokens = cargo_unfmt::lex::lex_file(src).context("failed to tokenize")?;
    println!("{tokens:?}");

    let ir = Ir::new(tokens.into_iter());

    let mut stmts = StmtVisitor::new();
    stmts.visit_file(&syn::parse_file(src).unwrap());
    println!("{stmts:?}");

    let with_junk = ir.populate_junk(stmts.regions());
    println!("{with_junk:?}");
    Ok(())
}
