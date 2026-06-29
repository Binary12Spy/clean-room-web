# PoC Plan
*A build plan for the clean-room web proof of concept.*

---

## What This Plan Is

The paper argues that the web's complexity was a path taken, not the only path. This PoC
exists to make that argument visible: a running host, loading real WASM bundles, with layout
living in userland and the host entirely oblivious to it. This document is the engineering
plan for building that proof. It covers architecture decisions, known pitfalls with their
resolutions, the crate structure, the milestone sequence, and what "done" means for each one.

---

## Technology Choices

### WASM Runtime: `wasmi`

There are two serious Rust-native options: `wasmtime` and `wasmi`.

`wasmtime` is a JIT-compiling, production-grade runtime backed by Cranelift. It is the
right tool for a shipping product. It is not the right tool here: it is a heavy dependency
(significant compile time, native code generation), and more critically, it pulls the
project toward production concerns before the architecture question is even answered.

`wasmi` is a pure-Rust bytecode interpreter. No JIT, no native codegen. It is slower at
runtime (irrelevant for a PoC rendering a todo list), compiles fast, and its API is a
near-exact mirror of `wasmtime`'s: `Engine`, `Store<T>`, `Linker<T>`, `Module`, `Instance`,
`TypedFunc`. The migration path to `wasmtime` later is essentially mechanical. Crucially,
its synchronous call model (`func.call(...)`) matches exactly the host-calls-WASM and
WASM-calls-host patterns the architecture requires, with no async machinery in the way.

**Chosen: `wasmi` 0.40.x**

### Windowing and Rendering: `winit` + `softbuffer`

The host needs a window and a pixel buffer. The paper deliberately avoids GPU pipeline
complexity — the draw commands from the bundle (`push_rect`, `push_text`) are rasterized
by the host into a CPU-side pixel buffer. `softbuffer` is exactly that: a cross-platform
2D pixel buffer backed by whatever the OS provides (X11 SHM, Wayland SHM, GDI, etc.).
`winit` provides the cross-platform event loop and window creation.

`softbuffer` 0.4.8 is already in the local Cargo registry cache. It pairs with `winit`
0.30 which uses the `ApplicationHandler` trait pattern.

**Chosen: `winit` 0.30.x + `softbuffer` 0.4.8**

### Text Rendering: `fontdue`

The paper calls out text rendering as the one thing that genuinely belongs in the host,
not userland: "correctness is brutal to get right and consistent text is itself a
user-agency property." The host implements `push_text` and `measure_text`. `fontdue` is a
pure-Rust font rasterizer (no system font library dependency), simple API, zero unsafe
beyond the unavoidable. It cannot do full complex-script shaping, which is fine: the PoC
proves the architecture, not a production text stack.

**Chosen: `fontdue` (latest stable)**

### Build Orchestration: `cargo xtask`

The workspace contains crates targeting two different compilation targets:

- The host: `x86_64-unknown-linux-gnu` (native)
- The WASM bundles: `wasm32-unknown-unknown`

A single `cargo build` cannot build both. The standard Rust pattern for this is `xtask`:
a workspace member named `xtask` that is invoked as `cargo xtask build`. It is just a
Rust binary that shells out to the right `cargo build --target wasm32-unknown-unknown`
commands for the bundle crates, then builds the host. This keeps everything in one
workspace with no external build tool dependency (`make` is not reliably available in
this environment).

**Chosen: cargo xtask pattern**

---

## Workspace Structure

```
poc/
├── Cargo.toml              # workspace root
├── xtask/                  # build orchestration (native binary)
│   └── src/main.rs
│
├── abi/                    # shared types, no_std compatible
│   └── src/lib.rs          # InputEvent, DrawCommand, CapabilitySet, error types
│
├── host/                   # the "browser" (native binary)
│   └── src/main.rs
│
├── layout-flex/            # layout library #1 (wasm32 cdylib)
│   └── src/lib.rs          # flexbox-style box model, hit-testing
│
├── layout-grid/            # layout library #2 (wasm32 cdylib)
│   └── src/lib.rs          # constraint/grid-style, different internal model
│
├── app-todo-flex/          # the demo app, bundled with layout-flex (wasm32 cdylib)
│   └── src/lib.rs
│
├── app-todo-grid/          # the same app logic, bundled with layout-grid (wasm32 cdylib)
│   └── src/lib.rs
│
└── doc-renderer/           # M4: document renderer (wasm32 cdylib)
    └── src/lib.rs
```

The two `app-todo-*` crates are the M3 proof: identical host, two bundles, different layout
libraries compiled in. The host binary accepts a path argument and loads whatever `.wasm`
it is pointed at. It has no `if flex / if grid` logic anywhere. That is the entire point.

---

## The ABI (The Frozen Boundary)

This is the most load-bearing design decision in the whole PoC. Everything else depends
on getting this interface right. The `abi` crate defines it as shared types; the actual
function signatures are enforced by WASM import/export names and numeric types.

### Host → Bundle (bundle exports, host calls)

```rust
// Bundle must export these three functions.
// The host calls them by name at runtime.

// Called once after instantiation. Receives capability handles as u32 flags.
// The bundle stores which capabilities it actually has.
fn init(caps: u32);

// Called on mouse move, click, key press, window resize.
// Event is serialized into a pair of i32 values (tag + payload).
fn on_event(tag: i32, payload: i32);

// Called once per frame. Bundle calls push_rect/push_text during this call.
fn render();
```

Why such a minimal shape? Because the host must be able to call any conformant bundle
without knowing anything about what it does. The bundle exports are fixed names with fixed
signatures. The host discovers them with `get_typed_func("render")`. This works regardless
of whether the bundle is a todo list or a spreadsheet.

### Bundle → Host (host registers, bundle imports)

```
module: "env"

// Always available (unconditional capability):
push_rect(x: f32, y: f32, w: f32, h: f32, r: u8, g: u8, b: u8, a: u8)
push_text(ptr: i32, len: i32, x: f32, y: f32, r: u8, g: u8, b: u8, a: u8)
measure_text(ptr: i32, len: i32) -> f32   // returns width in pixels

// Conditionally available (only registered if capability was granted):
net_fetch(url_ptr: i32, url_len: i32) -> i32   // returns request_id
```

**f32 for color components is wrong here** — WASM's type system only has i32/i64/f32/f64.
Passing RGBA as four separate u8 arguments works but is unwieldy. Instead, pack RGBA into
a single u32 passed as i32 (bitcast). The bundle packs with `(r << 24) | (g << 16) | ...`,
the host unpacks. This is a clean convention; document it in the abi crate.

### Passing Strings Across the Boundary

WASM has no native string type. The bundle passes `(ptr: i32, len: i32)` referring to a
location in the bundle's own linear memory (its `memory` export). The host reads from that
memory using `wasmi`'s `Memory::read()`. This is a two-step operation in every host
function that receives a string:

1. Get the `Memory` export from the `Caller`
2. Copy bytes `[ptr..ptr+len]` into a temporary `Vec<u8>`
3. Only then call `caller.data_mut()` to get the draw command buffer

Steps 2 and 3 must be sequential because step 1 borrows the `Caller` and step 3 borrows
it again mutably. If both borrows are held simultaneously, the Rust borrow checker rejects
it. Copying to a temp buffer releases the first borrow before taking the second. This is a
known pattern in wasmi/wasmtime code and must be consistently applied.

### The Event Encoding

`on_event(tag: i32, payload: i32)` encodes events as two integers. The tag identifies the
event type; the payload carries the data. For the PoC:

| Tag | Event              | Payload                              |
|-----|--------------------|--------------------------------------|
| 0   | MouseMove          | packed (x: u16, y: u16) into i32     |
| 1   | MouseDown (left)   | packed (x: u16, y: u16) into i32     |
| 2   | MouseUp (left)     | packed (x: u16, y: u16) into i32     |
| 3   | Resize             | packed (w: u16, h: u16) into i32     |

This is intentionally minimal. A production ABI would use a proper enum serialized through
shared memory. For the PoC, two integers are sufficient to drive a todo list.

### Capabilities

The capability word passed to `init(caps: u32)` is a bitmask:

```
bit 0: networking allowed  (net_fetch is registered)
bit 1: storage allowed     (store_set/store_get would be registered)
```

The host sets this word at load time based on its policy for the bundle being loaded. The
bundle reads it in `init`, stores it, and can choose to surface "this feature unavailable"
in its own UI rather than panic. For the M1 demo, the host runs the same bundle twice:
once with networking granted, once denied (by not registering `net_fetch` and passing 0
for the capability bit). The bundle's `render()` output reflects the difference.

---

## Known Pitfalls and Resolutions

### 1. The Double-Borrow in Host Functions

**The pitfall:** In a `wasmi` host function, `Caller<T>` gives access to both the WASM
memory (via `get_export("memory")`) and the host state (via `data_mut()`). Both borrows
go through `Caller`. You cannot hold both at the same time.

**The resolution:** Always copy memory data into a local `Vec<u8>` first, drop the memory
borrow, then call `data_mut()`. This must be a disciplined convention applied consistently
to every host function that reads WASM memory. Document it in the abi crate's source.

### 2. The `no_std` / Allocator Problem in WASM Bundles

**The pitfall:** `wasm32-unknown-unknown` targets do not link `std` by default. Rust's
standard allocator is unavailable. Any code in the bundle that allocates (Vec, String,
Box) will fail to link without an explicit allocator.

**The resolution:** The bundle crates use `#![no_std]` with `extern crate alloc`, and
declare a global allocator using `lol_alloc` (verified working; `wee_alloc` is an
alternative). Both are small pure-Rust allocators designed for WASM. The `abi` crate must
be `no_std` compatible (only `core` and `alloc`, never `std`). The `layout-*` and `app-*`
crates all share this constraint. The panic handler is `core::arch::wasm32::unreachable()`.

**Two linker details that are not optional (verified by build):**

- Imported host functions must be declared with
  `#[link(wasm_import_module = "env")]` on the `extern "C"` block, or they land in the
  wrong import namespace and the host's `func_wrap("env", ...)` will not match them.
- The bundle crate needs `rustflags = ["-C", "link-arg=--allow-undefined"]` in its
  `.cargo/config.toml` for the `wasm32-unknown-unknown` target. Without it, `rust-lld`
  rejects the unresolved host-import symbols at link time (`undefined symbol: push_rect`).
  The imports are resolved by the host at instantiation, not at bundle link time.

### 3. The Two-Target Build Problem

**The pitfall:** A Cargo workspace builds all members for one target. The host is native;
the bundles are `wasm32-unknown-unknown`. Running `cargo build` at the workspace root
tries to build the WASM crates for the native target, which fails because their
`extern "C"` exports and WASM-specific idioms don't make sense natively.

**The resolution:** The workspace `Cargo.toml` lists only the host and xtask as members.
The WASM crates are in the same repository directory but are NOT workspace members. The
`xtask` binary drives the build: it first runs `cargo build --target wasm32-unknown-unknown`
for each bundle crate (from their own directories), then builds the host. The host
hard-codes the expected output paths for the `.wasm` files and reads them at runtime.
This is explicit, simple, and requires no special tooling.

### 4. The `winit` 0.30 Ownership Model

**The pitfall:** `winit` 0.30 uses the `ApplicationHandler` trait. The event loop owns
the thread; all host state (the `wasmi` Store, the WASM instance, the softbuffer surface)
must live inside the struct that implements `ApplicationHandler`. This struct is
constructed before the event loop starts, but the `Window` and softbuffer surface can
only be created in the `Resumed` callback (required by some platforms; be consistent).

**The resolution:** Use `Option<T>` for the fields that are created lazily in `Resumed`:

```rust
struct App {
    wasm_store: Store<HostState>,   // created up front - wasmi is fine before window
    wasm_render: TypedFunc<...>,    // obtained after module instantiation
    window: Option<Window>,         // created in Resumed
    surface: Option<Surface<...>>,  // created in Resumed, after window
}
```

WASM module loading and instantiation happen in `App::new()` before the event loop starts.
The Window and pixel surface are created in `resumed()`. `render_frame()` is called from
`window_event(RedrawRequested)`.

### 5. The M3 Proof Requires Structural Discipline

**The pitfall:** It is tempting to write a host that has any conditional logic touching
the layout library concept — even something as innocent as printing which library is
loaded. That would undermine the proof.

**The resolution:** The host binary is a single `main.rs` with zero mention of "flex" or
"grid." It accepts a path to a `.wasm` file as a CLI argument. It loads it. It calls
`init`, `on_event`, and `render`. It knows nothing else. The two bundle paths
(`app-todo-flex.wasm` and `app-todo-grid.wasm`) are passed from the command line. The M3
demo is documented as: run the host twice, point at different `.wasm` files, observe that
the host binary is identical in both cases.

### 6. Text Rendering Lives in the Host, Which Means Bundle Font Negotiation

**The pitfall:** The host owns the font and rasterizes text. But the layout library
(in WASM) needs to know text dimensions to compute layout — it calls `measure_text`. If
`measure_text` is implemented in the host using a different font or size than the one the
host uses to actually render, layout will be wrong. Font and size must be consistent.

**The resolution:** For the PoC, the host loads exactly one font (a bundled embed of a
single open-license TTF, e.g. a subset of a freely licensed monospace font). It uses one
size. `measure_text` uses the same font instance as `push_text`. The layout library
hardcodes the same size assumption. In a production system this would be solved by a
font negotiation protocol in the ABI; for the PoC one font/one size is the right call.

### 7. The Document Format Scope Creep Risk

**The pitfall:** M4 requires a document format. Without a hard constraint, it will grow.
The first instinct is to add tables, then images, then inline styles, then... and suddenly
it is HTML again.

**The resolution:** The document format is intentionally named and constrained from the
start. It is called `.wcd` (web clean document). It is a line-oriented text format:
each line begins with a sigil that declares its type. Version is declared on the first
line. Nothing else is added for the PoC beyond what is needed to prove M4: headings,
paragraphs, unordered list items, and links. The document renderer is a WASM module
(proving document rendering is also a userland concern) that uses only `push_rect` and
`push_text` — no app-mode capabilities needed.

---

## Milestone Sequence

### M0: Rectangle on Screen

**Goal:** The host loads a WASM bundle. The bundle calls `push_rect`. The host renders a
colored rectangle in a window.

**What it proves:** The host↔bundle ABI boundary functions end to end. The event loop,
WASM instantiation, and frame render path all work.

**Done when:** Running `cargo xtask run -- bundles/hello-rect.wasm` opens a window with a
colored rectangle. No crash, no missing symbol.

**Crates involved:** `xtask`, `host`, `abi`, `hello-rect` (a minimal test bundle, not a
final PoC artifact).

---

### M1: Capability Enforcement

**Goal:** The host loads a bundle that declares a `net_fetch` import. When run with the
capability granted, the bundle's `render()` displays "fetch: ok". When run with the
capability denied (host does not register the import; instantiation is rejected), the host
prints a clear error and exits cleanly.

**What it proves:** The capability model is real. A bundle cannot call what it was not
handed. The enforcement is structural (import resolution fails), not runtime (no silent
failure or ignore).

**Done when:** Two runs of the host with the same bundle, one with `--cap net`, one
without. First run: window with "fetch: ok". Second run: clean error message
`capability denied: net_fetch required by bundle but not granted`. No panic.

**Crates involved:** `host`, `abi`, `cap-test` (a minimal test bundle).

---

### M2: Layout in Userland

**Goal:** `layout-flex` is a WASM library. `app-todo-flex` bundles it. The host loads
`app-todo-flex.wasm`. A working todo list renders: text labels, a button, hit-testing
(clicking a todo marks it done).

**What it proves:** Layout — box sizing, text wrapping, hit-testing — lives entirely in
the WASM bundle. The host's only involvement is `push_rect`, `push_text`,
`measure_text`, and forwarding input events. The host has no concept of a button.

**Done when:** The todo list renders correctly, items can be checked off by clicking, the
host source has no layout logic of any kind.

**Crates involved:** `host`, `abi`, `layout-flex`, `app-todo-flex`.

---

### M3: The Keystone — Layout Swap with Oblivious Host

**Goal:** `layout-grid` is a second WASM layout library implementing a different internal
model (grid/constraint-based rather than flexbox). `app-todo-grid` bundles it. Running
the host against `app-todo-grid.wasm` produces the same application with a visually
distinct layout. The host binary is bit-for-bit identical in both invocations.

**What it proves:** The central claim of the paper. The thing that makes the current web
impossible to reimplement — the layout engine — is here just a dependency. The host
knows nothing about it. Swapping it requires no host change whatsoever.

**Done when:** `cargo xtask run -- bundles/app-todo-flex.wasm` and
`cargo xtask run -- bundles/app-todo-grid.wasm` both produce a working todo list. The
host's `main.rs` contains the string "flex" exactly zero times and the string "grid"
exactly zero times. A diff of the two runs' host binary is empty.

**Crates involved:** `host`, `abi`, `layout-flex`, `layout-grid`, `app-todo-flex`,
`app-todo-grid`.

---

### M4: Document Mode

**Goal:** A `.wcd` document file is loaded by the host's document mode. It renders
without any WASM execution on the document's part — the host's built-in document renderer
(itself a WASM module, but one the host ships as a default, not one the document
provides) parses and renders it. A link in the first document opens a second document.

**What it proves:** The document half of the architecture. Zero-runtime static rendering.
Linkability. The inspectability-by-construction property (the `.wcd` file is
human-readable plain text, not a binary blob).

**Done when:** `cargo xtask run -- docs/page-one.wcd` renders a page with a heading,
paragraphs, a list, and a link. Clicking the link loads `page-two.wcd`. Both documents
render without any capability grants.

**Crates involved:** `host`, `abi`, `doc-renderer`, and two `.wcd` document files.

---

## The `.wcd` Document Format

The format is line-oriented. Each line is either blank or begins with a sigil.

```
# wcd/1                  ← version declaration, must be first line

h1 Welcome               ← heading level 1
h2 A Subheading          ← heading level 2
p  This is a paragraph.  ← paragraph text
-  First list item        ← unordered list item
-  Second list item
link Page Two => page-two.wcd   ← link: display text => target path
```

Lines beginning with `#` (after the version line) are comments. Blank lines are ignored.
There are no inline styles, no spans, no nesting. This is intentional. The format proves
the principle; the principle is that a static document can be minimal, readable, and
renderable with zero runtime. That proof does not require tables or images.

---

## What This PoC Deliberately Omits

These are not forgotten — they are out of scope by design, and their absence does not
affect the validity of the proof.

- **Performance.** `wasmi` is an interpreter. The PoC will not benchmark well. That is
  fine. The claim is about architecture, not throughput.
- **Complex text shaping.** The host font renderer handles ASCII + basic Latin. BiDi,
  Arabic, Indic scripts, emoji: out of scope. The paper acknowledges text as genuinely
  hard; the PoC demonstrates the boundary, not a production text implementation.
- **Networking implementation.** M1 proves the capability model. The `net_fetch` call in
  M1 does not need to make a real HTTP request — it can return a hardcoded payload. The
  PoC proves the *grant/deny* mechanism, not an HTTP stack.
- **The document/app seam.** The paper flags this as the hardest open design question
  (the Limitations section, "The Seam"). M4 demonstrates both modes but does not
  implement an embedded app island within a document. That is the follow-up work.
- **Security hardening.** The capability model is structurally correct but the host is
  not audited for sandbox escapes. This is a proof, not a product.
- **Multiple concurrent bundles.** The host loads one bundle at a time. Tabs, iframes,
  and the embedding model are out of scope.

---

## Success Criteria

The PoC succeeds if and only if the following can be independently verified:

1. The host binary contains no layout logic and no reference to any layout library.
2. `app-todo-flex.wasm` and `app-todo-grid.wasm` both produce a working UI when loaded
   by the exact same host binary.
3. A bundle that declares a `net_fetch` import cannot be instantiated without the host
   explicitly granting it.
4. A `.wcd` document renders to screen without any WASM from the document itself
   executing.
5. All four of the above are demonstrated by the same host binary.

That is the whole argument of the paper, made visible and checkable.
