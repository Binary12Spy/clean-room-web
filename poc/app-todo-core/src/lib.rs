//! # app_todo_core
//!
//! The todo application, with no dependency on any layout library.
//!
//! This crate owns the application state (a list of todo items) and knows how
//! to turn that state into a [`ui::Node`] tree and how to react to button
//! activations. It does not know or care how that tree is arranged on screen.
//!
//! The same `app_todo_core` is linked into both the flex and the grid bundles.
//! Only the layout library differs between them. That is what makes the M3
//! comparison honest: identical app, identical content tree, different layout,
//! oblivious host.

#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use ui::{Color, Node};

/// One todo entry.
struct Item {
    /// Stable identifier, also used as the button id for its toggle.
    id: u32,
    label: String,
    done: bool,
}

/// The whole application state.
pub struct TodoApp {
    items: Vec<Item>,
}

impl Default for TodoApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TodoApp {
    /// Create the app pre-populated with a few items so the demo shows content
    /// immediately.
    pub fn new() -> Self {
        let seed = [
            "Write the paper",
            "Build the host",
            "Prove the layout swap",
            "Ship the document mode",
        ];
        let items = seed
            .iter()
            .enumerate()
            .map(|(i, label)| Item {
                id: i as u32,
                label: label.to_string(),
                done: i == 0,
            })
            .collect();
        Self { items }
    }

    /// Toggle the done state of the item whose toggle button has `id`.
    ///
    /// Returns `true` if an item matched (so the caller can request a redraw).
    pub fn on_button(&mut self, id: u32) -> bool {
        for item in &mut self.items {
            if item.id == id {
                item.done = !item.done;
                return true;
            }
        }
        false
    }

    /// Build the content tree for the current state.
    ///
    /// Each row is a horizontal container holding a toggle button and the item
    /// label; the rows are stacked in a column under a title.
    pub fn view(&self) -> Node {
        let title = Node::Label {
            text: "Todo".to_string(),
            color: Color(0xff, 0xff, 0xff, 0xff),
        };

        let mut rows: Vec<Node> = vec![title];
        for item in &self.items {
            let (mark, bg) = if item.done {
                ("[x]".to_string(), Color(0x2e, 0x7d, 0x32, 0xff))
            } else {
                ("[ ]".to_string(), Color(0x45, 0x52, 0x5b, 0xff))
            };

            let label_color = if item.done {
                Color(0x8a, 0x9b, 0xa8, 0xff)
            } else {
                Color(0xe6, 0xed, 0xf3, 0xff)
            };

            let row = Node::row(
                10.0,
                vec![
                    Node::Button {
                        id: item.id,
                        text: mark,
                        bg,
                        fg: Color(0xff, 0xff, 0xff, 0xff),
                    },
                    Node::Label {
                        text: item.label.clone(),
                        color: label_color,
                    },
                ],
            );
            rows.push(row);
        }

        Node::column(12.0, rows)
    }
}
