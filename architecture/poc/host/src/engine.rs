//! # host::engine
//!
//! The WASM side of the host: loads a bundle, registers the host imports, and
//! exposes a tiny surface (`init`, `on_event`, `render`) for the window loop.
//!
//! The engine is deliberately oblivious. It does not know whether the bundle is
//! a todo app, a document renderer, or anything else. It only knows the ABI.

use anyhow::{Context, Result, anyhow};
use wasmi::{Caller, Engine as WasmiEngine, Linker, Module, Store, TypedFunc};

/// One accumulated drawing command, produced by the bundle during `render()`.
#[derive(Clone, Debug)]
pub enum DrawCmd {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rgba: u32,
    },
    // Fields are consumed once the font stack lands in M2; until then the
    // rasterizer ignores Text commands but bundles may already emit them.
    #[allow(dead_code)]
    Text {
        text: String,
        x: f32,
        y: f32,
        rgba: u32,
    },
}

/// Host state threaded through every host function via `Caller`.
#[derive(Default)]
pub struct HostState {
    /// Draw commands for the current frame. Cleared by the host before each
    /// `render()` call and drained afterward to paint the pixel buffer.
    pub draw: Vec<DrawCmd>,
}

/// Reads a UTF-8 string out of the bundle's linear memory.
///
/// Critical discipline (see PLAN.md, pitfall #1): copy the bytes out *before*
/// taking a mutable borrow of host state, so the memory borrow and the
/// `data_mut()` borrow never overlap.
fn read_string(caller: &Caller<HostState>, ptr: i32, len: i32) -> Result<String> {
    let memory = caller
        .get_export("memory")
        .and_then(|e| e.into_memory())
        .ok_or_else(|| anyhow!("bundle does not export `memory`"))?;
    let data = memory.data(caller);
    let (start, end) = (ptr as usize, ptr as usize + len as usize);
    let bytes = data
        .get(start..end)
        .ok_or_else(|| anyhow!("push_text/measure_text: out-of-bounds string {ptr}+{len}"))?;
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

/// A loaded, instantiated bundle ready to be driven.
pub struct Bundle {
    store: Store<HostState>,
    f_on_event: TypedFunc<(i32, i32), ()>,
    f_render: TypedFunc<(), ()>,
}

impl Bundle {
    /// Load a `.wasm` file, register host imports gated by `granted_caps`, and
    /// instantiate it.
    ///
    /// # Arguments
    /// * `wasm` - the raw bytes of a WASM bundle.
    /// * `granted_caps` - capability bitmask (see [`abi::caps`]); ungranted
    ///   capabilities have their host imports withheld entirely.
    ///
    /// # Returns
    /// A ready-to-drive [`Bundle`] with `init` already called.
    ///
    /// # Errors
    /// Returns an error if the module is invalid, if the bundle imports a
    /// capability that was not granted (reported as `capability denied: ...`),
    /// if a required export is missing, or if `init` traps.
    pub fn load(wasm: &[u8], granted_caps: u32) -> Result<Self> {
        let engine = WasmiEngine::default();
        let module = Module::new(&engine, wasm).context("invalid wasm module")?;
        let mut store = Store::new(&engine, HostState::default());
        let mut linker = <Linker<HostState>>::new(&engine);

        // --- Unconditional imports -------------------------------------------------
        linker.func_wrap(
            "env",
            "push_rect",
            |mut caller: Caller<HostState>, x: f32, y: f32, w: f32, h: f32, rgba: i32| {
                caller.data_mut().draw.push(DrawCmd::Rect {
                    x,
                    y,
                    w,
                    h,
                    rgba: rgba as u32,
                });
            },
        )?;

        linker.func_wrap(
            "env",
            "push_text",
            |mut caller: Caller<HostState>, ptr: i32, len: i32, x: f32, y: f32, rgba: i32| {
                // Copy-out-then-mutate: see read_string doc comment.
                let text = match read_string(&caller, ptr, len) {
                    Ok(s) => s,
                    Err(_) => return,
                };
                caller.data_mut().draw.push(DrawCmd::Text {
                    text,
                    x,
                    y,
                    rgba: rgba as u32,
                });
            },
        )?;

        linker.func_wrap(
            "env",
            "measure_text",
            |caller: Caller<HostState>, ptr: i32, len: i32| -> f32 {
                // M0 placeholder metric; real measurement arrives with the font
                // stack in M2. Kept here so the import resolves and bundles can
                // call it now.
                match read_string(&caller, ptr, len) {
                    Ok(s) => s.chars().count() as f32 * 8.0,
                    Err(_) => 0.0,
                }
            },
        )?;

        // --- Capability-gated imports ----------------------------------------------
        if granted_caps & abi::caps::NET != 0 {
            linker.func_wrap(
                "env",
                "net_fetch",
                |_caller: Caller<HostState>, _ptr: i32, _len: i32| -> i32 {
                    // M1 will give this meaning. For now: a stub request id.
                    0
                },
            )?;
        }

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| describe_link_error(e, granted_caps))?
            .start(&mut store)?;

        // `init` is called exactly once, here; only `on_event` and `render`
        // are retained for the lifetime of the bundle.
        let f_init = instance.get_typed_func::<u32, ()>(&store, "init")?;
        let f_on_event = instance.get_typed_func::<(i32, i32), ()>(&store, "on_event")?;
        let f_render = instance.get_typed_func::<(), ()>(&store, "render")?;

        f_init.call(&mut store, granted_caps)?;

        Ok(Self {
            store,
            f_on_event,
            f_render,
        })
    }

    /// Forward an input event to the bundle.
    ///
    /// # Errors
    /// Returns an error if the bundle's `on_event` traps.
    pub fn on_event(&mut self, tag: i32, payload: i32) -> Result<()> {
        self.f_on_event
            .call(&mut self.store, (tag, payload))
            .context("bundle on_event trapped")
    }

    /// Run one frame and return the draw commands the bundle produced.
    ///
    /// # Returns
    /// The draw commands emitted during this frame, in paint order.
    ///
    /// # Errors
    /// Returns an error if the bundle's `render` traps.
    pub fn render(&mut self) -> Result<&[DrawCmd]> {
        self.store.data_mut().draw.clear();
        self.f_render
            .call(&mut self.store, ())
            .context("bundle render trapped")?;
        Ok(&self.store.data().draw)
    }
}

/// Turn a wasmi instantiation error into a capability-aware message. When a
/// bundle imports something the host did not register, that is most often a
/// denied capability — say so explicitly.
fn describe_link_error(err: wasmi::Error, granted_caps: u32) -> anyhow::Error {
    let msg = err.to_string();
    if msg.contains("net_fetch") && (granted_caps & abi::caps::NET == 0) {
        anyhow!("capability denied: net_fetch required by bundle but not granted")
    } else {
        anyhow!("failed to instantiate bundle: {msg}")
    }
}
