//! CPU rasterization of draw commands into a softbuffer pixel buffer.
//!
//! softbuffer expects each pixel as a `u32` in `0x00RRGGBB` (the high byte is
//! ignored on most platforms). Our ABI colors are `0xRRGGBBAA`; we composite
//! with simple source-over alpha against whatever is already in the buffer.

use crate::engine::DrawCmd;

/// Paint all commands, in order, onto the buffer. Later commands draw over
/// earlier ones, matching the painter's-algorithm model the bundle assumes.
pub fn paint(buffer: &mut [u32], width: u32, height: u32, cmds: &[DrawCmd]) {
    // Clear to black first so a bundle that does not cover the whole window
    // produces a defined result rather than stale pixels.
    for px in buffer.iter_mut() {
        *px = 0x0000_0000;
    }

    let w = width as i32;
    let h = height as i32;

    for cmd in cmds {
        match *cmd {
            DrawCmd::Rect {
                x,
                y,
                w: rw,
                h: rh,
                rgba,
            } => {
                fill_rect(buffer, w, h, x, y, rw, rh, rgba);
            }
            // Text rendering arrives with the font stack in M2. For M0 it is a
            // no-op so bundles can already emit text commands harmlessly.
            DrawCmd::Text { .. } => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn fill_rect(buffer: &mut [u32], w: i32, h: i32, x: f32, y: f32, rw: f32, rh: f32, rgba: u32) {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let x1 = (x + rw).ceil() as i32;
    let y1 = (y + rh).ceil() as i32;

    let (sr, sg, sb, sa) = abi::unpack_rgba(rgba as i32);
    let sa_f = sa as f32 / 255.0;

    let cx0 = x0.max(0);
    let cy0 = y0.max(0);
    let cx1 = x1.min(w);
    let cy1 = y1.min(h);

    for py in cy0..cy1 {
        let row = (py * w) as usize;
        for px in cx0..cx1 {
            let idx = row + px as usize;
            let dst = buffer[idx];
            let dr = ((dst >> 16) & 0xFF) as f32;
            let dg = ((dst >> 8) & 0xFF) as f32;
            let db = (dst & 0xFF) as f32;
            let r = (sr as f32 * sa_f + dr * (1.0 - sa_f)) as u32;
            let g = (sg as f32 * sa_f + dg * (1.0 - sa_f)) as u32;
            let b = (sb as f32 * sa_f + db * (1.0 - sa_f)) as u32;
            buffer[idx] = (r << 16) | (g << 8) | b;
        }
    }
}
