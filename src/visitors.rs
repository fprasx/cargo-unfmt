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
    /// Check if this region contains a token starting at a given (line, column) pair.
    pub fn contains<T>(&self, spanned: &crate::lex::Spanned<T>) -> RelativePosition {
        let line = spanned.line;
        let col = spanned.char;
        // On line entirely before region
        if line < self.line_start {
            return RelativePosition::Before;
        }

        // On first line of region
        if line == self.line_start {
            // We know the column start of a region is on a token boundary, so
            // if the column of the token precedes the start of the boundary, that
            // token is not in the boundary (at all)
            if col < self.col_start {
                return RelativePosition::Before;
            }

            return RelativePosition::Within;
        }

        // Line is within region
        if (self.line_start + 1..self.line_end.saturating_sub(1)).contains(&line) {
            return RelativePosition::Within;
        }

        // On last line of region
        if line == self.line_end {
            // The column end of a region refers to the last column of the last
            // token in in the region, so if the start of the token is before
            // this, it is fully contained in the region.
            if col <= self.col_end {
                return RelativePosition::Within;
            }

            return RelativePosition::After;
        }

        // On line entirely after region
        assert!(line >= self.line_end);
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
