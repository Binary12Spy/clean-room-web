//! # layout_flex
//!
//! A flexbox-style layout library, living entirely in userland.
//!
//! It interprets a [`ui::Node`] tree as nested flow containers: a vertical
//! container stacks its children top to bottom; a horizontal one places them
//! left to right; each child takes its natural size and successive children are
//! separated by the container's `gap`. Text is measured through the host so
//! labels and buttons are sized to fit.
//!
//! This is one of two layout libraries in the PoC. The other, `layout-grid`,
//! arranges the *same* tree on a fixed grid. The host cannot tell them apart;
//! swapping one for the other is the keystone demonstration (M3).

#![no_std]

extern crate alloc;

use alloc::vec::Vec;

use ui::host::{self, TEXT_LINE_PX};
use ui::{Axis, Color, HitRegion, Layout, Node};

/// Padding inside a button, in pixels, on each axis.
const BUTTON_PAD_X: f32 = 12.0;
const BUTTON_PAD_Y: f32 = 6.0;

/// Outer margin around the whole UI.
const MARGIN: f32 = 24.0;

/// The flexbox-style layout engine.
#[derive(Default)]
pub struct FlexLayout {
    hits: Vec<HitRegion>,
}

impl FlexLayout {
    /// Create a new layout engine.
    pub fn new() -> Self {
        Self { hits: Vec::new() }
    }

    /// Natural size of a node: how much space it wants given its content.
    fn measure(&self, node: &Node) -> (f32, f32) {
        match node {
            Node::Label { text, .. } => (host::measure(text), TEXT_LINE_PX),
            Node::Button { text, .. } => (
                host::measure(text) + BUTTON_PAD_X * 2.0,
                TEXT_LINE_PX + BUTTON_PAD_Y * 2.0,
            ),
            Node::Container {
                axis,
                gap,
                children,
            } => self.measure_container(*axis, *gap, children),
        }
    }

    /// Natural size of a container: sum along the main axis, max on the cross
    /// axis, with `gap` between children.
    fn measure_container(&self, axis: Axis, gap: f32, children: &[Node]) -> (f32, f32) {
        let mut main = 0.0_f32;
        let mut cross = 0.0_f32;
        for (i, child) in children.iter().enumerate() {
            let (cw, ch) = self.measure(child);
            let (child_main, child_cross) = match axis {
                Axis::Vertical => (ch, cw),
                Axis::Horizontal => (cw, ch),
            };
            main += child_main;
            if i + 1 < children.len() {
                main += gap;
            }
            cross = cross.max(child_cross);
        }
        match axis {
            Axis::Vertical => (cross, main),
            Axis::Horizontal => (main, cross),
        }
    }

    /// Draw `node` with its top-left at `(x, y)`, recording any hit regions.
    fn place(&mut self, node: &Node, x: f32, y: f32) {
        match node {
            Node::Label { text, color } => {
                host::text(text, x, y, color.packed());
            }
            Node::Button { id, text, bg, fg } => {
                let w = host::measure(text) + BUTTON_PAD_X * 2.0;
                let h = TEXT_LINE_PX + BUTTON_PAD_Y * 2.0;
                host::rect(x, y, w, h, bg.packed());
                host::text(text, x + BUTTON_PAD_X, y + BUTTON_PAD_Y, fg.packed());
                self.hits.push(HitRegion {
                    id: *id,
                    x,
                    y,
                    w,
                    h,
                });
            }
            Node::Container {
                axis,
                gap,
                children,
            } => {
                let mut cursor_x = x;
                let mut cursor_y = y;
                for child in children {
                    self.place(child, cursor_x, cursor_y);
                    let (cw, ch) = self.measure(child);
                    match axis {
                        Axis::Vertical => cursor_y += ch + gap,
                        Axis::Horizontal => cursor_x += cw + gap,
                    }
                }
            }
        }
    }
}

impl Layout for FlexLayout {
    fn render(&mut self, root: &Node, width: f32, height: f32) -> Vec<HitRegion> {
        self.hits.clear();

        // Background fill so the flex layout has a recognizable, distinct look
        // from other layout libraries.
        host::rect(
            0.0,
            0.0,
            width,
            height,
            Color(0x1c, 0x24, 0x2b, 0xFF).packed(),
        );

        self.place(root, MARGIN, MARGIN);
        core::mem::take(&mut self.hits)
    }
}
