//! # layout_grid
//!
//! A fixed-grid layout library, living entirely in userland.
//!
//! Where `layout_flex` honors the tree's containers and lets each item take its
//! natural size, this library does something deliberately different: it
//! *flattens* the tree into a flat sequence of leaf items and drops them into a
//! uniform grid of cells, wrapping across a fixed number of columns. Container
//! axes and gaps are ignored; geometry comes from the grid, not the content.
//!
//! The point is that this is a fundamentally different layout paradigm reading
//! the *same* [`ui::Node`] tree. The host cannot tell which library a bundle
//! carries - swapping `layout_flex` for `layout_grid` needs no host change at
//! all. That substitution is the keystone claim of the project (M3).

#![no_std]

extern crate alloc;

use alloc::vec::Vec;

use ui::host::{self, TEXT_LINE_PX};
use ui::{Color, HitRegion, Layout, Node};

/// Number of columns in the grid.
const COLUMNS: usize = 2;
/// Outer margin around the grid, in pixels.
const MARGIN: f32 = 24.0;
/// Gap between cells, in pixels.
const CELL_GAP: f32 = 14.0;
/// Fixed cell height, in pixels.
const CELL_H: f32 = TEXT_LINE_PX + 20.0;
/// Inner padding within a cell, in pixels.
const CELL_PAD: f32 = 8.0;

/// A flattened leaf item to be placed in a grid cell.
enum Cell<'a> {
    /// Static text.
    Label { text: &'a str, color: Color },
    /// A clickable item with an id.
    Button {
        id: u32,
        text: &'a str,
        bg: Color,
        fg: Color,
    },
}

/// The fixed-grid layout engine.
#[derive(Default)]
pub struct GridLayout {
    hits: Vec<HitRegion>,
}

impl GridLayout {
    /// Create a new layout engine.
    pub fn new() -> Self {
        Self { hits: Vec::new() }
    }

    /// Flatten a node into the leaf cells it contains, depth-first. A leaf
    /// becomes a single cell; a container contributes its children in order.
    /// Container axis and gap are ignored - geometry comes from the grid, not
    /// the content. That is the defining difference from the flex layout.
    fn leaves<'a>(node: &'a Node, out: &mut Vec<Cell<'a>>) {
        match node {
            Node::Label { text, color } => out.push(Cell::Label {
                text,
                color: *color,
            }),
            Node::Button { id, text, bg, fg } => out.push(Cell::Button {
                id: *id,
                text,
                bg: *bg,
                fg: *fg,
            }),
            Node::Container { children, .. } => {
                for child in children {
                    Self::leaves(child, out);
                }
            }
        }
    }

    /// Draw one cell within the rectangle `(cx, cy, cw, CELL_H)`.
    fn draw_cell(&mut self, cell: &Cell, cx: f32, cy: f32, cw: f32) {
        match *cell {
            Cell::Label { text, color } => {
                // Cell background so the grid structure is visible.
                host::rect(cx, cy, cw, CELL_H, Color(0x26, 0x2c, 0x38, 0xff).packed());
                host::text(text, cx + CELL_PAD, cy + CELL_PAD, color.packed());
            }
            Cell::Button { id, text, bg, fg } => {
                host::rect(cx, cy, cw, CELL_H, bg.packed());
                host::text(text, cx + CELL_PAD, cy + CELL_PAD, fg.packed());
                self.hits.push(HitRegion {
                    id,
                    x: cx,
                    y: cy,
                    w: cw,
                    h: CELL_H,
                });
            }
        }
    }
}

impl Layout for GridLayout {
    fn render(&mut self, root: &Node, width: f32, height: f32) -> Vec<HitRegion> {
        self.hits.clear();

        // A distinct background so the grid layout is visually unmistakable
        // next to the flex layout.
        host::rect(
            0.0,
            0.0,
            width,
            height,
            Color(0x2b, 0x1c, 0x24, 0xFF).packed(),
        );

        // Each top-level child of the root is one grid row. This keeps columns
        // aligned: a full-width title row does not push the item rows out of
        // their columns the way a single flat stream would.
        let rows: &[Node] = match root {
            Node::Container { children, .. } => children,
            single => core::slice::from_ref(single),
        };

        let usable_w = width - MARGIN * 2.0;
        let cell_w = (usable_w - CELL_GAP * (COLUMNS as f32 - 1.0)) / COLUMNS as f32;
        let mut cy = MARGIN;

        let mut cells = Vec::new();
        for row in rows {
            cells.clear();
            Self::leaves(row, &mut cells);

            if cells.len() <= 1 {
                // A lone cell (e.g. the title) spans the full row width.
                if let Some(cell) = cells.first() {
                    self.draw_cell(cell, MARGIN, cy, usable_w);
                }
            } else {
                // Distribute this row's cells across the fixed columns.
                for (col, cell) in cells.iter().enumerate().take(COLUMNS) {
                    let cx = MARGIN + col as f32 * (cell_w + CELL_GAP);
                    self.draw_cell(cell, cx, cy, cell_w);
                }
            }
            cy += CELL_H + CELL_GAP;
        }

        core::mem::take(&mut self.hits)
    }
}
