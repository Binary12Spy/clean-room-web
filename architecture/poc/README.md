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
| **M2** | Layout (boxes, text, hit-testing) lives in a WASM lib | ⬜ next |
| **M3** | Second layout lib swaps in, host unchanged (keystone) | ⬜ |
| **M4** | Document mode: static render + links | ⬜ |

## Layout

```
poc/
├── abi/        shared host<->bundle contract (no_std)
├── host/       the engine: window, pixel surface, wasmi (native)
├── xtask/      build orchestration for the two-target workspace
└── bundles/    WASM bundles (standalone crates, target wasm32-unknown-unknown)
    └── hello-rect/   M0 test bundle
```

The `host`, `abi`, and `xtask` crates form one cargo workspace (native target).
The `bundles/*` crates are **deliberately excluded** from the workspace: they
compile to `wasm32-unknown-unknown` and are built by `xtask`, which redirects
their artifacts into the shared `target/` dir.

## Building and running

Everything runs inside the reproducible dev shell defined by the repo-root
`flake.nix` (pinned Rust 1.96 + wasm target + `wasm-tools`):

```sh
nix develop           # from the repo root; drops into the dev shell
cd architecture/poc

cargo xtask build                 # build all bundles + the host
cargo xtask run -- target/wasm32-unknown-unknown/debug/hello_rect.wasm
```

A window opens showing a teal panel with an orange rectangle — drawn entirely
by the bundle calling the host's `push_rect` import.

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
