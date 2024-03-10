// NOTE: macro input is not parsed in any way, so when we search for statements,
// idents, whatever, nothing in a macro is reported! This is great, since we can't
// alter what is inside a macro.

use proc_macro2::{LineColumn, Span};
use syn::{spanned::Spanned, visit::Visit};

#[derive(Debug, PartialEq, Eq)]
pub enum RelativePosition {
    Before,
    Within,
    After,
}

/// A region of source code deliminated by a pair of (line, column).
#[derive(Debug, PartialEq, Eq)]
pub struct Region {
    line_start: usize,
    col_start: usize,
    line_end: usize,
    col_end: usize,
}

impl From<Span> for Region {
    fn from(span: Span) -> Self {
        let LineColumn {
            line: line_start,
            column: col_start,
        } = span.start();
        let LineColumn {
            line: line_end,
            column: col_end,
        } = span.end();
        Self {
            line_start,
            // proc-macro2 spans set the column start to the column before the start
            col_start: col_start + 1,
            line_end,
            col_end,
        }
    }
}

impl Region {
    /// Check if this region contains a given (line, column) pair
    pub fn contains(&self, line: usize, col: usize) -> RelativePosition {
        if line < self.line_start {
            return RelativePosition::Before;
        }

        if line == self.line_start {
            if col < self.col_start {
                return RelativePosition::Before;
            } else {
                return RelativePosition::Within;
            }
        }

        if (self.line_start + 1..self.line_end.saturating_sub(1)).contains(&line) {
            return RelativePosition::Within;
        }

        if line == self.line_end {
            if col <= self.col_end {
                return RelativePosition::Within;
            } else {
                return RelativePosition::After;
            }
        }

        RelativePosition::After
    }
}

#[derive(Debug, Default)]
pub struct MacroVisitor {
    regions: Vec<Region>,
}

impl MacroVisitor {
    pub fn new() -> Self {
        Self { regions: vec![] }
    }

    pub fn spans(&self) -> &[Region] {
        self.regions.as_slice()
    }
}

impl Visit<'_> for MacroVisitor {
    fn visit_macro(&mut self, i: &'_ syn::Macro) {
        self.regions.push(i.span().into())
    }
}

#[derive(Debug, Default)]
pub struct StmtVisitor {
    regions: Vec<Region>,
}

impl StmtVisitor {
    pub fn new() -> Self {
        Self { regions: vec![] }
    }

    pub fn spans(&self) -> &[Region] {
        self.regions.as_slice()
    }
}

impl Visit<'_> for StmtVisitor {
    fn visit_stmt(&mut self, i: &'_ syn::Stmt) {
        self.regions.push(i.span().into())
    }
}
