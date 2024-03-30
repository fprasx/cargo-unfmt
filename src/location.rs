use crate::lex::{self, TokenStart};

use proc_macro2::TokenTree;
use quote::ToTokens;
use syn::{
    visit::{self, Visit},
    PatPath, Stmt, StmtMacro,
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
    // fn visit_stmt(&mut self, i: &'_ syn::Stmt) {
    //     if let Stmt::Expr(_, None)
    //     | Stmt::Macro(StmtMacro {
    //         semi_token: None, ..
    //     }) = i
    //     {
    //         // These statements don't have a semicolon so we can't put junk after them.
    //         // Keep recursing using default visitor
    //         visit::visit_stmt(self, i);
    //     } else {
    //         // Output statement start/begins in DFS order
    //         let (start, end) = stmt_endpoints(i);
    //         self.events.push(start);
    //         // Keep recursing using default visitor
    //         visit::visit_stmt(self, i);
    //         self.events.push(end);
    //     }
    // }

    fn visit_expr(&mut self, i: &'_ syn::Expr) {
        match i {
            syn::Expr::Array(_)
            | syn::Expr::Assign(_)
            | syn::Expr::Await(_)
            | syn::Expr::Binary(_)
            | syn::Expr::Break(_)
            | syn::Expr::Call(_)
            | syn::Expr::Cast(_)
            | syn::Expr::Closure(_)
            | syn::Expr::Continue(_)
            | syn::Expr::Index(_)
            | syn::Expr::Infer(_)
            | syn::Expr::Macro(_)
            | syn::Expr::MethodCall(_)
            | syn::Expr::Paren(_)
            | syn::Expr::Reference(_)
            | syn::Expr::Repeat(_)
            | syn::Expr::Return(_)
            | syn::Expr::Struct(_)
            | syn::Expr::Try(_)
            | syn::Expr::Tuple(_)
            | syn::Expr::Unary(_)
            | syn::Expr::Yield(_) => {
                let (start, end) = expr_endpoints(i);
                self.events.push(start);
                visit::visit_expr(self, i);
                self.events.push(end);
            },
            syn::Expr::Lit(_)  => {
                // syn doesn't understand doc comments
                let (start, end) = expr_endpoints(i);
                if start.region.char == 1 {
                    visit::visit_expr(self, i);
                } else {
                self.events.push(start);
                visit::visit_expr(self, i);
                self.events.push(end);

                }
            },
            // A path of length one is just and identifier
            // ACTUALLY: can't do these because of struct initers, ex.
            // X { a, b } cannot be converted to X { (a), (b) }
            // syn::Expr::Path(syn::ExprPath {
            //     path: syn::Path {
            //         segments,
            //         ..
            //     },
            //     ..
            // }) if segments.len() == 1 => {
            //     let (start, end) = expr_endpoints(i);
            //     self.events.push(start);
            //     visit::visit_expr(self, i);
            //     self.events.push(end);
            // }

            syn::Expr::Path(_)
            | syn::Expr::Block(_)
            | syn::Expr::Group(_) // not sure what this is
            | syn::Expr::Range(_) // dot tokens handled weirdly
            | syn::Expr::Field(_) // also something weird going on here
            | syn::Expr::Verbatim(_)
            | syn::Expr::Match(_)
            | syn::Expr::Let(_)
            | syn::Expr::Async(_)
            | syn::Expr::Const(_)
            | syn::Expr::ForLoop(_)
            | syn::Expr::If(_)
            | syn::Expr::Loop(_)
            | syn::Expr::TryBlock(_)
            | syn::Expr::Unsafe(_)
            | syn::Expr::While(_) => {
                visit::visit_expr(self, i);
            }
            _ => panic!("new expression variant"),
        }
    }
}

#[derive(Debug)]
pub enum Event {
    StatementStart,
    StatementEnd,
    ExprOpen,
    ExprClose,
}

pub fn stmt_endpoints(stmt: &syn::Stmt) -> (lex::Spanned<Event>, lex::Spanned<Event>) {
    let tokens = stmt.to_token_stream().into_iter().collect::<Vec<_>>();

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

pub fn expr_endpoints(stmt: &syn::Expr) -> (lex::Spanned<Event>, lex::Spanned<Event>) {
    let tokens = stmt.to_token_stream().into_iter().collect::<Vec<_>>();

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
        lex::Spanned::new(Event::ExprOpen, start_line, start_char),
        lex::Spanned::new(Event::ExprClose, end_line, end_char),
    )
}
