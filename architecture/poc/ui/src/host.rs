//! # ui::host
//!
//! Safe wrappers over the raw host imports.
//!
//! The host exposes `push_rect`, `push_text`, and `measure_text` as untyped
//! WASM imports. Layout libraries should not scatter `unsafe` calls throughout
//! their code, so this module wraps them once, here, with the safety reasoning
//! stated in a single place.

/// The line height the host renders text at, in pixels.
///
/// The host owns the font and draws at a fixed size; layout libraries need the
/// same value to size rows that contain text. Kept in sync with the host's
/// `text::FONT_PX` line metrics by convention (one font, one size for the PoC).
pub const TEXT_LINE_PX: f32 = 22.0;

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn push_rect(x: f32, y: f32, w: f32, h: f32, rgba: i32);
    fn push_text(ptr: i32, len: i32, x: f32, y: f32, rgba: i32);
    fn measure_text(ptr: i32, len: i32) -> f32;
    #[link_name = "navigate"]
    fn host_navigate(ptr: i32, len: i32);
}

/// Draw a filled rectangle.
pub fn rect(x: f32, y: f32, w: f32, h: f32, rgba: i32) {
    // Safety: these arguments are plain scalars; the host validates nothing
    // beyond clipping to the framebuffer, so any values are sound.
    unsafe { push_rect(x, y, w, h, rgba) }
}

/// Draw `text` with its top-left at `(x, y)`.
pub fn text(s: &str, x: f32, y: f32, rgba: i32) {
    // Safety: `s` is a live `&str`, so its pointer/length describe valid bytes
    // in this module's linear memory for the duration of the call. The host
    // copies the bytes out before returning.
    unsafe { push_text(s.as_ptr() as i32, s.len() as i32, x, y, rgba) }
}

/// Measure the rendered width of `text` in pixels, as the host would draw it.
pub fn measure(s: &str) -> f32 {
    // Safety: same invariant as `text` - `s` is a valid live string slice.
    unsafe { measure_text(s.as_ptr() as i32, s.len() as i32) }
}

/// Request that the host load the document (or resource) at `target`.
///
/// The host decides whether and how to honor it; the bundle just expresses the
/// intent (e.g. a clicked link).
pub fn navigate(target: &str) {
    // Safety: `target` is a live `&str`; the host copies the bytes out before
    // returning.
    unsafe { host_navigate(target.as_ptr() as i32, target.len() as i32) }
}
