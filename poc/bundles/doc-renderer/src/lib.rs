//! # doc-renderer
//!
//! The host's default renderer for `.wcd` documents.
//!
//! This is a WASM bundle, exactly like an application bundle - which is the
//! point: rendering a document is itself a userland concern, not something
//! baked into the engine. The host ships this renderer as its default for the
//! document MIME type, but it is structurally just another bundle behind the
//! same ABI.
//!
//! Crucially, the *document* contributes no code. The renderer reads the
//! document as bytes (via the `doc_len` / `doc_read` host imports), parses it
//! with the shared `wcd` parser, and draws it. A `.wcd` file is pure data; it
//! cannot execute. That is the zero-runtime static-rendering property document
//! mode is meant to demonstrate.

#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use lol_alloc::{AssumeSingleThreaded, FreeListAllocator};
use ui::Color;
use ui::host;
use wcd::{Block, Document};

#[global_allocator]
static ALLOCATOR: AssumeSingleThreaded<FreeListAllocator> =
    unsafe { AssumeSingleThreaded::new(FreeListAllocator::new()) };

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Document-mode host imports specific to a renderer: read the current document
// and learn how many bytes it is. (`navigate` lives in `ui::host`.)
#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn doc_len() -> i32;
    fn doc_read(ptr: i32);
}

// Layout constants, in pixels.
const MARGIN: f32 = 40.0;
const LINE_H: f32 = 26.0;
const H1_GAP: f32 = 16.0;
const H2_GAP: f32 = 10.0;
const PARA_GAP: f32 = 8.0;
const LIST_INDENT: f32 = 24.0;
const LINK_PAD: f32 = 6.0;
const CONTENT_W: f32 = 1200.0;

// Colors.
const BG: Color = Color(0xf7, 0xf4, 0xee, 0xff);
const HEADING: Color = Color(0x1a, 0x1a, 0x1a, 0xff);
const BODY: Color = Color(0x33, 0x33, 0x33, 0xff);
const BULLET: Color = Color(0x88, 0x88, 0x88, 0xff);
const LINK_BG: Color = Color(0x1e, 0x5a, 0x9c, 0xff);
const LINK_FG: Color = Color(0xff, 0xff, 0xff, 0xff);

/// A clickable link region recorded during layout, tagged with its target.
struct LinkHit {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    target: String,
}

/// Renderer state: the parsed document and the link regions from the last frame.
struct Renderer {
    doc: Document,
    links: Vec<LinkHit>,
    width: f32,
    height: f32,
}

impl Renderer {
    fn new() -> Self {
        Self {
            doc: Document::default(),
            links: Vec::new(),
            width: 1280.0,
            height: 720.0,
        }
    }

    /// Pull the current document from the host and parse it.
    fn load_document(&mut self) {
        // Safety: `doc_len` is a pure query; we then reserve exactly that many
        // bytes and let the host fill them via `doc_read`.
        let len = unsafe { doc_len() } as usize;
        let mut buf: Vec<u8> = alloc::vec![0u8; len];
        if len > 0 {
            // Safety: `buf` has `len` bytes; the host writes exactly `doc_len()`
            // bytes starting at this pointer.
            unsafe { doc_read(buf.as_mut_ptr() as i32) };
        }
        let text = String::from_utf8_lossy(&buf);
        self.doc = wcd::parse(&text).unwrap_or_default();
    }

    /// Draw the whole document, recording link hit regions.
    fn render(&mut self) {
        self.links.clear();
        host::rect(0.0, 0.0, self.width, self.height, BG.packed());

        let mut y = MARGIN;
        // Collect links first to avoid borrowing `self.doc` and `self.links`
        // simultaneously; blocks are cheap to walk by index.
        let mut new_links: Vec<LinkHit> = Vec::new();

        for block in &self.doc.blocks {
            match block {
                Block::Heading { level, text } => {
                    y += if *level == 1 { H1_GAP } else { H2_GAP };
                    host::text(text, MARGIN, y, HEADING.packed());
                    y += LINE_H + if *level == 1 { H1_GAP } else { H2_GAP };
                }
                Block::Paragraph(text) => {
                    host::text(text, MARGIN, y, BODY.packed());
                    y += LINE_H + PARA_GAP;
                }
                Block::ListItem(text) => {
                    host::text("\u{2022}", MARGIN, y, BULLET.packed());
                    host::text(text, MARGIN + LIST_INDENT, y, BODY.packed());
                    y += LINE_H;
                }
                Block::Link { text, target } => {
                    let w = host::measure(text) + LINK_PAD * 2.0;
                    let h = LINE_H;
                    host::rect(MARGIN, y, w, h, LINK_BG.packed());
                    host::text(text, MARGIN + LINK_PAD, y + 2.0, LINK_FG.packed());
                    new_links.push(LinkHit {
                        x: MARGIN,
                        y,
                        w,
                        h,
                        target: target.clone(),
                    });
                    y += LINE_H + PARA_GAP;
                }
            }
        }

        let _ = CONTENT_W; // reserved for future text wrapping
        self.links = new_links;
    }

    /// Handle a click: if it lands on a link, ask the host to navigate.
    fn click(&mut self, px: f32, py: f32) {
        for link in &self.links {
            if px >= link.x && px < link.x + link.w && py >= link.y && py < link.y + link.h {
                host::navigate(&link.target);
                return;
            }
        }
    }
}

static mut RENDERER: Option<Renderer> = None;

#[allow(static_mut_refs)]
fn renderer() -> &'static mut Renderer {
    // Safety: single-threaded WASM instance driven by the host; no re-entrancy.
    unsafe {
        if RENDERER.is_none() {
            RENDERER = Some(Renderer::new());
        }
        RENDERER.as_mut().unwrap_unchecked()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn init(_caps: u32) {
    renderer().load_document();
}

#[unsafe(no_mangle)]
pub extern "C" fn on_event(tag: i32, payload: i32) {
    let r = renderer();
    match tag {
        abi::event::RESIZE => {
            let (w, h) = abi::unpack_u16_pair(payload);
            r.width = w as f32;
            r.height = h as f32;
        }
        abi::event::MOUSE_DOWN => {
            let (x, y) = abi::unpack_u16_pair(payload);
            r.click(x as f32, y as f32);
        }
        _ => {}
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn render() {
    renderer().render();
}
