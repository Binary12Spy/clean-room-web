# Clean-Room Web — Proof of Concept

A running demonstration of the architecture argued for in
[`../paper/what-the-web-could-have-been.md`](../paper/what-the-web-could-have-been.md):
a host (the "engine") that is **oblivious** to layout, with layout living in
userland as a swappable WASM library.

See [`PLAN.md`](./PLAN.md) for the full design, milestone ladder, and the
pitfalls worked through before implementation.

## Status

| Milestone | What it proves | State |
|-----------|----------------|-------|
| **M0** | Host loads a WASM bundle; bundle draws via the ABI | ✅ done |
| **M1** | Capability grant/deny is structural | ✅ mechanism in place |
| **M2** | Layout (boxes, text, hit-testing) lives in a WASM lib | ✅ done |
| **M3** | Second layout lib swaps in, host unchanged (keystone) | ✅ done |
| **M4** | Document mode: static render + links | ✅ done |

## Layout

```
poc/
├── abi/             shared host<->bundle contract (no_std)
├── host/            the engine: window, pixel surface, wasmi, font (native)
├── xtask/           build orchestration for the two-target workspace
├── ui/              userland UI vocabulary: Node tree, Layout trait, host wrappers
├── layout-flex/     flexbox-style layout library (wasm)
├── layout-grid/     fixed-grid layout library (wasm)
├── app-todo-core/   todo app logic, layout-agnostic (wasm)
├── app-shell/       bundle glue: allocator, ABI exports, event decode (wasm)
├── wcd/             parser for the .wcd document format (no_std, shared)
├── docs/            example .wcd documents (page-one, page-two)
└── bundles/         WASM cdylib bundles (standalone crates, wasm32-unknown-unknown)
    ├── hello-rect/       M0 test bundle
    ├── app-todo-flex/    todo app + flex layout (one-line choice)
    ├── app-todo-grid/    todo app + grid layout (same app, different layout)
    └── doc-renderer/     the host's default .wcd document renderer
```

The `app-todo-flex` and `app-todo-grid` bundles differ by exactly one line (the
layout type they name). Same app, same content tree, same host binary - only the
layout library changes. That is the M3 keystone.

The `host`, `abi`, and `xtask` crates form one cargo workspace (native target).
The `bundles/*` crates are **deliberately excluded** from the workspace: they
compile to `wasm32-unknown-unknown` and are built by `xtask`, which redirects
their artifacts into the shared `target/` dir.

## Building and running

Everything runs inside the reproducible dev shell defined by the repo-root
`flake.nix` (pinned Rust 1.96 + wasm target + `wasm-tools`):

```sh
nix develop           # from the repo root; drops into the dev shell
cd poc

cargo xtask build                 # build all bundles + the host

# M0: a rectangle drawn by the bundle through the ABI
cargo xtask run -- target/wasm32-unknown-unknown/debug/hello_rect.wasm

# M2/M3: the same todo app under two different layout libraries
cargo xtask run -- target/wasm32-unknown-unknown/debug/app_todo_flex.wasm
cargo xtask run -- target/wasm32-unknown-unknown/debug/app_todo_grid.wasm
```

Clicking a todo's toggle button checks it off (hit-testing happens in the layout
library, not the host). For a display-free check of layout output, use
`--dump-frame` (optionally with `--click X,Y`) to print the draw commands one
frame produces.

### Document mode (M4)

Point the host at a `.wcd` file instead of a `.wasm`:

```sh
cargo xtask run -- docs/page-one.wcd
```

The host recognizes the document extension, loads its own default renderer
(`doc-renderer.wasm` - itself just a bundle, proving document rendering is also
a userland concern), and feeds it the document as pure data. The document
executes nothing; it is human-readable plain text. Clicking the link at the
bottom navigates to `page-two.wcd`, which links back. This is the static,
zero-runtime, linkable document half of the architecture.

### Capabilities (M1)

Capabilities are enforced structurally: a denied capability means the host
never registers that import, so a bundle that needs it fails to instantiate
with a clear message rather than silently misbehaving.

```sh
# grant networking:
cargo xtask run -- --cap-net <bundle.wasm>
```

### The oblivious-host check (M3 keystone)

```sh
cargo xtask check-oblivious
```

Asserts that `host/src` contains no reference to any layout library by name.
This is the machine-checkable form of the paper's central claim.

## Without Nix

A `rust-toolchain.toml` pins the same toolchain for `rustup` users. You will
also need the X11/Wayland and `libxkbcommon` runtime libraries that `winit` and
`softbuffer` dlopen; the flake lists them under `runtimeLibs`.
