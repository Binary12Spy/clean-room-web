//! # abi
//!
//! The frozen boundary between the host (the "engine") and a WASM bundle.
//!
//! This crate is `no_std` so it can be shared by the native host and by the
//! `wasm32-unknown-unknown` bundles without divergence. It contains only the
//! shared *conventions* - there is no host logic and no bundle logic here.
//!
//! ## The contract, in one place
//!
//! Bundle EXPORTS (host calls these by name):
//! - `init(caps: u32)`            - once, after instantiation
//! - `on_event(tag: i32, payload: i32)` - input events
//! - `render()`                   - once per frame; calls the imports below
//!
//! Bundle IMPORTS (module `"env"`; host provides these):
//! - `push_rect(x, y, w, h: f32, rgba: i32)`
//! - `push_text(ptr, len: i32, x, y: f32, rgba: i32)`
//! - `measure_text(ptr, len: i32) -> f32`
//! - `net_fetch(ptr, len: i32) -> i32`  - only if the net capability was granted
//!
//! Imports must be declared on the bundle side with
//! `#[link(wasm_import_module = "env")]`, and the bundle must be linked with
//! `-C link-arg=--allow-undefined` so the imports resolve at instantiation.

#![no_std]

/// Capability bits passed to the bundle's `init(caps: u32)`.
///
/// A capability that is *not* set means the corresponding host import is not
/// registered at all - calling it would fail at instantiation, not silently.
pub mod caps {
    /// Networking allowed: the host registers `net_fetch`.
    pub const NET: u32 = 1 << 0;
    /// Persistent storage allowed (reserved; not used until a later milestone).
    pub const STORAGE: u32 = 1 << 1;
}

/// Event tags for `on_event(tag, payload)`.
///
/// The payload encoding depends on the tag; see [`pack_u16_pair`].
pub mod event {
    pub const MOUSE_MOVE: i32 = 0;
    pub const MOUSE_DOWN: i32 = 1;
    pub const MOUSE_UP: i32 = 2;
    pub const RESIZE: i32 = 3;
}

/// Pack two `u16` values (e.g. x/y or w/h) into a single `i32` payload.
///
/// High 16 bits = `a`, low 16 bits = `b`.
#[inline]
pub const fn pack_u16_pair(a: u16, b: u16) -> i32 {
    (((a as u32) << 16) | (b as u32)) as i32
}

/// Inverse of [`pack_u16_pair`].
#[inline]
pub const fn unpack_u16_pair(payload: i32) -> (u16, u16) {
    let v = payload as u32;
    ((v >> 16) as u16, (v & 0xFFFF) as u16)
}

/// Pack an RGBA color (8 bits per channel) into a single `i32`.
///
/// Layout: `0xRRGGBBAA`. WASM has no aggregate scalar types, so colors cross
/// the boundary as one integer rather than four separate byte arguments.
#[inline]
pub const fn pack_rgba(r: u8, g: u8, b: u8, a: u8) -> i32 {
    (((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)) as i32
}

/// Inverse of [`pack_rgba`], returning `(r, g, b, a)`.
#[inline]
pub const fn unpack_rgba(rgba: i32) -> (u8, u8, u8, u8) {
    let v = rgba as u32;
    (
        ((v >> 24) & 0xFF) as u8,
        ((v >> 16) & 0xFF) as u8,
        ((v >> 8) & 0xFF) as u8,
        (v & 0xFF) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_roundtrip() {
        let c = pack_rgba(0x12, 0x34, 0x56, 0x78);
        assert_eq!(unpack_rgba(c), (0x12, 0x34, 0x56, 0x78));
        assert_eq!(pack_rgba(0xFF, 0x00, 0x00, 0xFF) as u32, 0xFF0000FF);
    }

    #[test]
    fn u16_pair_roundtrip() {
        assert_eq!(unpack_u16_pair(pack_u16_pair(1280, 720)), (1280, 720));
        assert_eq!(unpack_u16_pair(pack_u16_pair(0, 0xFFFF)), (0, 0xFFFF));
    }
}
