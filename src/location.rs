use crate::lex::{self, TokenStart};

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    visit::{self, Visit},
    Stmt, StmtMacro,
};

#[derive(Debug, Default)]
pub struct Visitor {
    events: Vec<lex::Spanned<Event>>,
}

impl Visitor {
    pub fn new() -> Self {
        Self { events: vec![] }
    }

    pub fn regions(&self) -> &[lex::Spanned<Event>] {
        self.events.as_slice()
    }
}

impl Visit<'_> for Visitor {
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
            self.events.push(start);
            // Keep recursing using default visitor
            visit::visit_stmt(self, i);
            self.events.push(end);
        }
    }

    fn visit_expr(&mut self, i: &'_ syn::Expr) {
        visit::visit_expr(self, i);
    }
}

#[derive(Debug)]
pub enum Event {
    StatementStart,
    StatementEnd,
}

pub fn endpoints(t: TokenStream) -> (lex::Spanned<Event>, lex::Spanned<Event>) {
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
        lex::Spanned::new(Event::StatementStart, start_line, start_char),
        lex::Spanned::new(Event::StatementEnd, end_line, end_char),
    )
}
