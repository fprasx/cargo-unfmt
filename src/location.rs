use crate::lex::{self, TokenStart};

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    visit::{self, Visit},
    Stmt, StmtMacro,
};

#[derive(Debug, Default)]
pub struct StmtVisitor {
    regions: Vec<lex::Spanned<StatementPos>>,
}

impl StmtVisitor {
    pub fn new() -> Self {
        Self { regions: vec![] }
    }

    pub fn regions(&self) -> &[lex::Spanned<StatementPos>] {
        self.regions.as_slice()
    }
}

impl Visit<'_> for StmtVisitor {
    fn visit_stmt(&mut self, i: &'_ syn::Stmt) {
        if let Stmt::Expr(_, None)
        | Stmt::Macro(StmtMacro {
            semi_token: None, ..
        }) = i
        {
            // These statements don't have a semicolon so we can't put junk after them.
            // Keep recursing using default visitor
            visit::visit_stmt(self, i);
        } else {
            // Output statement start/begins in DFS order
            let (start, end) = endpoints(i.to_token_stream());
            self.regions.push(start);
            // Keep recursing using default visitor
            visit::visit_stmt(self, i);
            self.regions.push(end);
        }
    }
}

#[derive(Debug)]
pub enum StatementPos {
    Before,
    After,
}

pub fn endpoints(t: TokenStream) -> (lex::Spanned<StatementPos>, lex::Spanned<StatementPos>) {
    let tokens = t.into_iter().collect::<Vec<_>>();

    let TokenStart {
        line: start_line,
        char: start_char,
    } = TokenStart::from(match tokens.first().unwrap() {
        TokenTree::Group(group) => group.span_open(),
        TokenTree::Ident(ident) => ident.span(),
        TokenTree::Punct(punct) => punct.span(),
        TokenTree::Literal(literal) => literal.span(),
    });

    let TokenStart {
        line: end_line,
        char: end_char,
    } = TokenStart::from(match tokens.last().unwrap() {
        TokenTree::Group(group) => group.span_close(),
        TokenTree::Ident(ident) => ident.span(),
        TokenTree::Punct(punct) => punct.span(),
        TokenTree::Literal(literal) => literal.span(),
    });

    (
        lex::Spanned::new(StatementPos::Before, start_line, start_char),
        lex::Spanned::new(StatementPos::After, end_line, end_char),
    )
}
