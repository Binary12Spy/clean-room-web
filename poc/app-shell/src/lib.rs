//! # app_shell
//!
//! The glue that turns the todo application plus a layout library into a WASM
//! bundle the host can load.
//!
//! It provides everything that is identical across bundles: the allocator, the
//! panic handler, the persistent application state, event decoding, and the
//! three ABI exports (`init`, `on_event`, `render`). The one thing that differs
//! between bundles - which layout library is used - is supplied by the bundle
//! through the [`bundle!`] macro.
//!
//! Keeping this shared means the flex bundle and the grid bundle differ only in
//! a single type name. That is what makes the M3 comparison a fair test.

#![no_std]

extern crate alloc;

use app_todo_core::TodoApp;
use ui::{HitRegion, Layout};

pub use alloc::vec::Vec;
// Re-exported so the macro can name them from the bundle crate without the
// bundle needing its own dependencies on these crates.
pub use {abi, app_todo_core, lol_alloc, ui};

/// The mutable runtime state of a running bundle.
///
/// Holds the app, the chosen layout engine, the viewport, the last cursor
/// position, and the hit regions produced by the most recent frame.
pub struct State<L: Layout> {
    app: TodoApp,
    layout: L,
    width: f32,
    height: f32,
    cursor: (f32, f32),
    hits: Vec<HitRegion>,
}

impl<L: Layout> State<L> {
    /// Create the initial state with the given layout engine.
    pub fn new(layout: L) -> Self {
        Self {
            app: TodoApp::new(),
            layout,
            width: 1280.0,
            height: 720.0,
            cursor: (0.0, 0.0),
            hits: Vec::new(),
        }
    }

    /// Handle one decoded input event. Returns nothing; the host drives redraws.
    pub fn on_event(&mut self, tag: i32, payload: i32) {
        match tag {
            abi::event::RESIZE => {
                let (w, h) = abi::unpack_u16_pair(payload);
                self.width = w as f32;
                self.height = h as f32;
            }
            abi::event::MOUSE_MOVE => {
                let (x, y) = abi::unpack_u16_pair(payload);
                self.cursor = (x as f32, y as f32);
            }
            abi::event::MOUSE_DOWN => {
                let (x, y) = abi::unpack_u16_pair(payload);
                let (px, py) = (x as f32, y as f32);
                // Activate the first hit region under the cursor.
                if let Some(hit) = self.hits.iter().find(|h| h.contains(px, py)) {
                    self.app.on_button(hit.id);
                }
            }
            _ => {}
        }
    }

    /// Render one frame: build the content tree, lay it out, and remember the
    /// hit regions for the next click.
    pub fn render(&mut self) {
        let tree = self.app.view();
        self.hits = self.layout.render(&tree, self.width, self.height);
    }
}

/// Generate a complete WASM bundle around a chosen layout engine.
///
/// The bundle crate calls this once at module scope, passing an expression that
/// constructs its layout engine. The macro emits the allocator, panic handler,
/// global state, and the three ABI exports.
///
/// # Example
/// ```ignore
/// app_shell::bundle!(layout_flex::FlexLayout, layout_flex::FlexLayout::new());
/// ```
#[macro_export]
macro_rules! bundle {
    ($layout_ty:ty, $layout_init:expr) => {
        #[global_allocator]
        static ALLOCATOR: $crate::lol_alloc::AssumeSingleThreaded<
            $crate::lol_alloc::FreeListAllocator,
        > = unsafe {
            $crate::lol_alloc::AssumeSingleThreaded::new(
                $crate::lol_alloc::FreeListAllocator::new(),
            )
        };

        #[panic_handler]
        fn panic(_: &core::panic::PanicInfo) -> ! {
            core::arch::wasm32::unreachable()
        }

        // Single-threaded WASM instance: the host drives it from one thread, so
        // a plain mutable static behind an accessor is sound. Wrapped to keep
        // all access to the `unsafe` in one place.
        static mut STATE: Option<$crate::State<$layout_ty>> = None;

        #[allow(static_mut_refs)]
        fn state() -> &'static mut $crate::State<$layout_ty> {
            // Safety: the host never calls bundle exports re-entrantly or from
            // multiple threads, so there is never an aliasing live borrow.
            unsafe {
                if STATE.is_none() {
                    STATE = Some($crate::State::new($layout_init));
                }
                STATE.as_mut().unwrap_unchecked()
            }
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn init(_caps: u32) {
            let _ = state();
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn on_event(tag: i32, payload: i32) {
            state().on_event(tag, payload);
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn render() {
            state().render();
        }
    };
}
