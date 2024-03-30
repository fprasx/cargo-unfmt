use anyhow::Context;
use ir::Ir;
use location::Visitor;
use syn::visit::Visit;

pub mod location;

pub mod emit;
pub mod ir;
pub mod lex;

const JUNK: [&str; 81] = [
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
    "*&*&();((),());",
    "((),());((),());",
    "((),());let _=();",
    "let _=();let _=();",
    "let _=();if true{};",
    "if true{};if true{};",
    "if true{};let _=||();",
    "let _=||();let _=||();",
    "let _=||();loop{break};",
    "loop{break};loop{break};",
    "loop{break};loop{break;};",
    "loop{break;};loop{break;};",
    "loop{break;};if let _=(){};",
    "if let _=(){};if let _=(){};",
    "if let _=(){};*&*&();((),());",
    "*&*&();((),());*&*&();((),());",
    "*&*&();((),());((),());((),());",
    "((),());((),());((),());((),());",
    "((),());((),());((),());let _=();",
    "((),());let _=();((),());let _=();",
    "((),());let _=();let _=();let _=();",
    "let _=();let _=();let _=();let _=();",
    "let _=();let _=();let _=();if true{};",
    "let _=();if true{};let _=();if true{};",
    "let _=();if true{};if true{};if true{};",
    "if true{};if true{};if true{};if true{};",
    "if true{};if true{};if true{};let _=||();",
    "if true{};let _=||();if true{};let _=||();",
    "if true{};let _=||();let _=||();let _=||();",
    "let _=||();let _=||();let _=||();let _=||();",
    "let _=||();let _=||();let _=||();loop{break};",
    "let _=||();loop{break};let _=||();loop{break};",
    "let _=||();loop{break};loop{break};loop{break};",
    "loop{break};loop{break};loop{break};loop{break};",
    "loop{break};loop{break};loop{break};loop{break;};",
    "loop{break};loop{break;};loop{break};loop{break;};",
    "loop{break};loop{break;};loop{break;};loop{break;};",
    "loop{break;};loop{break;};loop{break;};loop{break;};",
    "loop{break;};loop{break;};loop{break;};if let _=(){};",
    "loop{break;};if let _=(){};loop{break;};if let _=(){};",
    "loop{break;};if let _=(){};if let _=(){};if let _=(){};",
    "if let _=(){};if let _=(){};if let _=(){};if let _=(){};",
    "if let _=(){};if let _=(){};if let _=(){};*&*&();((),());",
    "if let _=(){};*&*&();((),());if let _=(){};*&*&();((),());",
    "if let _=(){};*&*&();((),());*&*&();((),());*&*&();((),());",
    "*&*&();((),());*&*&();((),());*&*&();((),());*&*&();((),());",
    "*&*&();((),());*&*&();((),());*&*&();((),());((),());((),());",
    "*&*&();((),());((),());((),());*&*&();((),());((),());((),());",
    "*&*&();((),());((),());((),());((),());((),());((),());((),());",
    "((),());((),());((),());((),());((),());((),());((),());((),());",
    "((),());((),());((),());((),());((),());((),());((),());let _=();",
    "((),());((),());((),());let _=();((),());((),());((),());let _=();",
    "((),());((),());((),());let _=();((),());let _=();((),());let _=();",
    "((),());let _=();((),());let _=();((),());let _=();((),());let _=();",
    "((),());let _=();((),());let _=();((),());let _=();let _=();let _=();",
    "((),());let _=();let _=();let _=();((),());let _=();let _=();let _=();",
    "((),());let _=();let _=();let _=();let _=();let _=();let _=();let _=();",
    "let _=();let _=();let _=();let _=();let _=();let _=();let _=();let _=();",
    "let _=();let _=();let _=();let _=();let _=();let _=();let _=();if true{};",
    "let _=();let _=();let _=();if true{};let _=();let _=();let _=();if true{};",
    "let _=();let _=();let _=();if true{};let _=();if true{};let _=();if true{};",
    "let _=();if true{};let _=();if true{};let _=();if true{};let _=();if true{};",
    "let _=();if true{};let _=();if true{};let _=();if true{};if true{};if true{};",
    "let _=();if true{};if true{};if true{};let _=();if true{};if true{};if true{};",
    "let _=();if true{};if true{};if true{};if true{};if true{};if true{};if true{};",
    "if true{};if true{};if true{};if true{};if true{};if true{};if true{};if true{};",
];

pub fn unformat(src: &str) -> anyhow::Result<Ir> {
    let tokens = lex::lex_file(src).context("source was not valid")?;

    let mut stmts = Visitor::new();
    stmts.visit_file(&syn::parse_file(src).unwrap());

    let ir = Ir::new(tokens.into_iter());
    let ir = ir.populate_events(stmts.regions());

    Ok(ir)
}

trait SafeLen {
    /// Returns the displayed length of a string.
    fn safe_len(&self) -> usize;
}

impl SafeLen for &str {
    fn safe_len(&self) -> usize {
        self.chars().count()
    }
}
