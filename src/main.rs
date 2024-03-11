use anyhow::Context;

fn main() -> anyhow::Result<()> {
    use cargo_unfmt::visitors::StmtVisitor;
    use syn::visit::Visit;
    let src = include_str!("../test_files/short-rust-file.rs");
    let file = syn::parse_file(src).unwrap();
    let tokens = cargo_unfmt::lex::lex_file(src).context("faile to tokenize")?;
    let mut idents = StmtVisitor::new();
    idents.visit_file(&file);
    println!("{idents:#?}");
    println!("{tokens:#?}");
    Ok(())
}

