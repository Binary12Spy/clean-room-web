//! # ui
//!
//! The userland UI vocabulary shared by applications and layout libraries.
//!
//! This crate is the seam *inside* a bundle. An application produces a
//! [`Node`] tree describing *what* it wants on screen (a column of labels and
//! buttons), with no opinion about pixel positions. A layout library consumes
//! that tree and decides *where* everything goes, emitting draw calls to the
//! host. Because both halves are plain WASM compiled into one bundle, they talk
//! through these Rust types rather than the host ABI.
//!
//! Nothing here is known to the host. The host only ever sees `push_rect` /
//! `push_text` calls. That is what lets the layout library be swapped for a
//! completely different one (milestone M3) without the host noticing.

#![no_std]

extern crate alloc;

pub mod host;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

/// An RGBA color, `0xRRGGBBAA` per channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8, pub u8);

impl Color {
    /// Pack into the single-`i32` form the host ABI expects.
    pub fn packed(self) -> i32 {
        abi::pack_rgba(self.0, self.1, self.2, self.3)
    }
}

/// How a container arranges its children.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    /// Children stacked top to bottom.
    Vertical,
    /// Children placed left to right.
    Horizontal,
}

/// A node in the content tree.
///
/// Apps build these; layout libraries interpret them. The variants describe
/// intent, not geometry: a `Button` is "a clickable thing labeled X", not a
/// rectangle at a position.
pub enum Node {
    /// A run of text.
    Label { text: String, color: Color },
    /// A clickable region labeled with text, identified by `id` so the app can
    /// recognize which one was activated.
    Button {
        id: u32,
        text: String,
        bg: Color,
        fg: Color,
    },
    /// A container that groups children along an [`Axis`].
    Container {
        axis: Axis,
        gap: f32,
        children: Vec<Node>,
    },
}

impl Node {
    /// Convenience constructor for a vertical container.
    pub fn column(gap: f32, children: Vec<Node>) -> Node {
        Node::Container {
            axis: Axis::Vertical,
            gap,
            children,
        }
    }

    /// Convenience constructor for a horizontal container.
    pub fn row(gap: f32, children: Vec<Node>) -> Node {
        Node::Container {
            axis: Axis::Horizontal,
            gap,
            children,
        }
    }
}

/// A rectangular region a layout library reports as clickable, tagged with the
/// button `id` it belongs to. The app uses these to turn a click coordinate
/// into a button activation.
#[derive(Clone, Copy, Debug)]
pub struct HitRegion {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl HitRegion {
    /// Whether the point `(px, py)` falls inside this region.
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.w && py >= self.y && py < self.y + self.h
    }
}

/// The contract every layout library satisfies.
///
/// Given a content tree and a viewport, a layout library draws the UI (via the
/// host imports in [`host`]) and returns the clickable regions it produced.
/// Different implementations may arrange the same tree very differently; that
/// difference is the whole point of milestone M3.
pub trait Layout {
    /// Lay out and draw `root` within a `width` x `height` viewport, returning
    /// the hit regions for any buttons drawn.
    fn render(&mut self, root: &Node, width: f32, height: f32) -> Vec<HitRegion>;
}

// Keep `Box` in use for downstream bundles that store trait objects, and to
// document that layout libraries are intended to be swappable behind a pointer.
#[doc(hidden)]
pub type BoxedLayout = Box<dyn Layout>;
