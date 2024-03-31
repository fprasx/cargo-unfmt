use anyhow::Context;
use ir::Ir;
use location::Visitor;
use syn::visit::Visit;

mod location;

mod emit;
mod ir;
mod lex;

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

/// Unformat a source file into lines of length `width`.
///
/// ## Details
/// This process strips comments, inserts no-op statements, and wraps expressions
/// in extra parentheses to achieve the desired line length.
///
/// ## Errors
/// Returns an error if the source file is not valid Rust.
///
/// This function returns a spurious error if source has documentation comments
/// not at the start of a line, for example:
/// ```
/// let x = blah; /// bad!
/// ```
/// This is because we use syn under the hood, which does not understand doc comments.
pub fn unformat(src: &str, width: usize) -> anyhow::Result<Vec<u8>> {
    let src = remove_doc_comments(src);

    let tokens = lex::lex_file(&src).context("source was not valid")?;

    let mut stmts = Visitor::new();
    stmts.visit_file(&syn::parse_file(&src).unwrap());

    let ir = Ir::new(tokens.into_iter());
    let ir = ir.populate_events(stmts.events());

    let mut unformatted = vec![];
    crate::emit::block(&mut unformatted, &ir, width);

    Ok(unformatted)
}

/// Remove doc comments heuristically. Necessary because syn doesn't understand them
/// as comments, mand treats them as expressions.
fn remove_doc_comments(src: &str) -> String {
    let mut out = vec![];
    for line in src.lines() {
        if !line.trim_start().starts_with("///") {
            out.push(line)
        }
    }
    out.join("\n")
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
