//! M0 test bundle.
//!
//! Proves the host<->bundle ABI boundary end to end: the host instantiates this
//! module, calls `init`/`render`, and this bundle calls back into the host's
//! `push_rect` import to draw. No layout library, no app logic - just the wire.

#![no_std]

extern crate alloc;

use lol_alloc::{AssumeSingleThreaded, FreeListAllocator};

// Single-threaded WASM: this allocator assumption is sound for our host, which
// drives the instance from one thread.
#[global_allocator]
static ALLOCATOR: AssumeSingleThreaded<FreeListAllocator> =
    unsafe { AssumeSingleThreaded::new(FreeListAllocator::new()) };

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    fn push_rect(x: f32, y: f32, w: f32, h: f32, rgba: i32);
}

#[unsafe(no_mangle)]
pub extern "C" fn init(_caps: u32) {}

#[unsafe(no_mangle)]
pub extern "C" fn on_event(_tag: i32, _payload: i32) {}

#[unsafe(no_mangle)]
pub extern "C" fn render() {
    // A teal background panel and an orange rectangle on top of it, so M0
    // visibly demonstrates ordering (later draws paint over earlier ones).
    unsafe {
        push_rect(0.0, 0.0, 1280.0, 720.0, abi::pack_rgba(0x10, 0x3a, 0x40, 0xFF));
        push_rect(80.0, 80.0, 240.0, 140.0, abi::pack_rgba(0xff, 0x8c, 0x32, 0xFF));
    }
}
