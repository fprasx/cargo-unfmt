use anyhow::Context;
use ir::Ir;
use location::StmtVisitor;
use syn::visit::Visit;

pub mod location;

pub mod emit;
pub mod ir;
pub mod lex;

const JUNK: [&str; 15] = [
    "",
    ";",
    "3;",
    "();",
    "{;};",
    "({});",
    "{();};",
    "*&*&();",
    "((),());",
    "let _=();",
    "if true{};",
    "let _=||();",
    "loop{break};",
    "loop{break;};",
    "if let _=(){};",
];

pub fn unformat(src: &str) -> anyhow::Result<Ir> {
    let tokens = lex::lex_file(src).context("source was not valid")?;

    let mut stmts = StmtVisitor::new();
    stmts.visit_file(&syn::parse_file(src).unwrap());

    let ir = Ir::new(tokens.into_iter());
    let ir = ir.populate_junk(stmts.regions());

    Ok(ir)
}
