use proc_macro2::Span;
use syn::{spanned::Spanned, visit::Visit};

#[derive(Debug, Default)]
pub struct MacroVisitor {
    spans: Vec<Span>,
}

impl MacroVisitor {
    pub fn new() -> Self {
        Self { spans: vec![] }
    }

    pub fn spans(&self) -> &[Span] {
        self.spans.as_slice()
    }
}

impl Visit<'_> for MacroVisitor {
    fn visit_macro(&mut self, i: &'_ syn::Macro) {
        self.spans.push(i.span())
    }
}
