//! # host::text
//!
//! The host-owned font and text rasterizer.
//!
//! Text rendering lives in the engine, not in userland: the paper argues that
//! consistent text is a user-agency property and that correctness here is hard
//! enough that it belongs to the trusted core. Layout libraries never rasterize
//! glyphs; they only ask the host to *measure* text (so they can place it) and
//! to *draw* it at a chosen position.
//!
//! One font, one size, loaded once. That keeps `measure_text` and the actual
//! glyph placement perfectly consistent, which is the property layout depends
//! on.

use std::collections::HashMap;

use fontdue::{Font, FontSettings};

/// The pixel height the host renders text at. A single fixed size keeps
/// measurement and rasterization consistent for the PoC.
pub const FONT_PX: f32 = 18.0;

/// A rasterized glyph: its coverage bitmap plus the metrics needed to place it
/// on the baseline.
struct Glyph {
    /// Per-pixel coverage, `width * height` bytes, row-major.
    coverage: Vec<u8>,
    width: usize,
    height: usize,
    /// Horizontal offset from the pen position to the bitmap's left edge.
    xmin: i32,
    /// Vertical offset from the baseline to the bitmap's top edge.
    ymin: i32,
    /// How far to advance the pen after drawing this glyph.
    advance: f32,
}

/// The host's font: the loaded face, derived line metrics, and a glyph cache.
pub struct FontBook {
    font: Font,
    /// Distance from the top of a line to the baseline, in pixels.
    ascent: f32,
    cache: HashMap<char, Glyph>,
}

impl FontBook {
    /// Load the embedded font and compute line metrics at [`FONT_PX`].
    ///
    /// # Errors
    /// Returns an error if the embedded font bytes fail to parse.
    pub fn load() -> Result<Self, &'static str> {
        // Embedded so the host renders identically regardless of system fonts.
        let bytes = include_bytes!("../assets/DejaVuSansMono.ttf") as &[u8];
        let font = Font::from_bytes(bytes, FontSettings::default())?;

        let metrics = font
            .horizontal_line_metrics(FONT_PX)
            .ok_or("no line metrics")?;
        Ok(Self {
            font,
            ascent: metrics.ascent,
            cache: HashMap::new(),
        })
    }

    /// Total advance width of `text` in pixels, used by layout libraries.
    pub fn measure(&mut self, text: &str) -> f32 {
        text.chars().map(|c| self.glyph(c).advance).sum()
    }

    /// Rasterize and cache a glyph, returning a reference to it.
    fn glyph(&mut self, c: char) -> &Glyph {
        self.cache.entry(c).or_insert_with(|| {
            let (metrics, coverage) = self.font.rasterize(c, FONT_PX);
            Glyph {
                coverage,
                width: metrics.width,
                height: metrics.height,
                xmin: metrics.xmin,
                ymin: metrics.ymin,
                advance: metrics.advance_width,
            }
        })
    }

    /// Draw `text` into `buffer`, with `(x, y)` at the top-left of the line.
    ///
    /// Glyph coverage is alpha-composited over existing pixels using the
    /// foreground color `rgba` (`0xRRGGBBAA`). Pixels outside the buffer are
    /// clipped.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        buffer: &mut [u32],
        buf_w: i32,
        buf_h: i32,
        text: &str,
        x: f32,
        y: f32,
        rgba: u32,
    ) {
        let (fr, fg, fb, fa) = abi::unpack_rgba(rgba as i32);
        let fa_f = fa as f32 / 255.0;
        let baseline = y + self.ascent;
        let mut pen_x = x;

        for c in text.chars() {
            // Copy the metrics we need so the cache borrow ends before we touch
            // `buffer`; avoids holding `&self` across the pixel loop.
            let (coverage, gw, gh, xmin, ymin, advance) = {
                let g = self.glyph(c);
                (
                    g.coverage.clone(),
                    g.width as i32,
                    g.height as i32,
                    g.xmin,
                    g.ymin,
                    g.advance,
                )
            };

            // Pen is on the baseline; the bitmap top sits `ymin + height` above it.
            let glyph_x = (pen_x + xmin as f32).round() as i32;
            let glyph_top = (baseline - (ymin + gh) as f32).round() as i32;

            for row in 0..gh {
                let py = glyph_top + row;
                if py < 0 || py >= buf_h {
                    continue;
                }
                for col in 0..gw {
                    let px = glyph_x + col;
                    if px < 0 || px >= buf_w {
                        continue;
                    }
                    let cov = coverage[(row * gw + col) as usize] as f32 / 255.0;
                    if cov <= 0.0 {
                        continue;
                    }
                    let a = cov * fa_f;
                    let idx = (py * buf_w + px) as usize;
                    let dst = buffer[idx];
                    let dr = ((dst >> 16) & 0xFF) as f32;
                    let dg = ((dst >> 8) & 0xFF) as f32;
                    let db = (dst & 0xFF) as f32;
                    let r = (fr as f32 * a + dr * (1.0 - a)) as u32;
                    let g = (fg as f32 * a + dg * (1.0 - a)) as u32;
                    let b = (fb as f32 * a + db * (1.0 - a)) as u32;
                    buffer[idx] = (r << 16) | (g << 8) | b;
                }
            }

            pen_x += advance;
        }
    }
}
