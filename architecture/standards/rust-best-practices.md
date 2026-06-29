# Rust Best Practices

This guide is the authoritative Rust implementation guide for TunnelWatch.

Use it for architecture, correctness, safety, performance, and maintainability decisions.
Use `docs/style/rust-comment-style.md` for comment and documentation formatting rules.

## 1) Craftsmanship Standard

- Ship production-grade code, not "just passing" code.
- Prefer obvious and maintainable implementations over clever ones.
- Handle edge cases and error paths intentionally.
- Leave touched code better than you found it (tests, names, docs, reliability).
- If a request pushes toward a fragile approach, call it out and propose a safer one.

## 2) Core Rust Conventions

- Use Rust 2024 idioms and workspace conventions.
- Keep code `rustfmt`-formatted and `clippy`-clean.
- Favor immutability; use `mut` only when needed.
- Keep module boundaries clear; do not bypass crate APIs casually.
- Use `snake_case` for modules/files/functions/variables, `CamelCase` for types/traits, and `SCREAMING_SNAKE_CASE` for constants/statics.

## 3) Error Handling and Reliability

- Use `Result<T, E>` for recoverable failures.
- Use `panic!` only for unrecoverable logic violations.
- Fail with actionable context (what failed, where, and relevant identifiers).
- Do not swallow errors; propagate explicitly.
- Validate inputs at boundaries and keep operational failures diagnosable.

## 4) Dependencies (Strict)

Every dependency must earn its place.

Before adding one, document:

- The exact problem it solves.
- Why stdlib/workspace code is insufficient.
- Runtime and build cost (size, memory, startup, attack surface).
- Maintenance/security quality and transitive dependency impact.

Rules:

- Prefer existing workspace crates when practical.
- Prefer small, focused crates over heavy frameworks.
- Do not add a dependency for trivial functionality that can be safely implemented in-house.
- Run `cargo make deny-check` after dependency changes.

### Workspace dependency declarations

**Internal (repo-local) crates** — always declared in `[workspace.dependencies]`
and referenced as `crate-name.workspace = true` in member `Cargo.toml` files.
This is the only place a path is written; all members share it.

**External (crates.io) crates** — declared inline in each member `Cargo.toml`
with an explicit version. Do **not** hoist external crates into
`[workspace.dependencies]` unless the same crate is used by many members
*and* requires a coordinated version or feature set across all of them.

Rationale: keeping external versions inline makes per-crate feature sets
explicit, avoids an artificial superset feature set, and keeps
`[workspace.dependencies]` as a clear map of the internal crate graph rather
than a catch-all dependency registry.

## 5) Performance (Default-On)

- Prefer doing less work before trying to do work faster.
- Be explicit about allocations, copies, and intermediate objects.
- Treat startup and cold-path performance as user-visible.
- Avoid abstraction layers that add cost without measurable value.
- For hot paths, reason about complexity and memory behavior.
- If performance may change, include a validation method (benchmark/timing/profile note).

## 6) Testing Expectations

- Unit tests: colocate with code in `#[cfg(test)] mod unit_tests`.
- Integration tests: crate-level `tests/`.
- Cover success, failure, and edge cases.
- Add regression tests for bug fixes.
- Keep tests deterministic and isolated.

## 7) Logging and Observability

- Use project logging crates/macros for production logging.
- Do not use `println!` for production observability.
- Include enough context in logs/errors for diagnosis.

## 8) Unsafe Code

- Avoid `unsafe` unless absolutely necessary.
- Every `unsafe` block must include a clear invariant/safety justification comment.
- Keep `unsafe` scopes minimal and reviewable.

## 9) Generated Code

- Keep generated code in designated generated locations.
- Do not manually edit generated artifacts.

## 10) TODO and Incomplete Work

- Use `todo!("describe what remains")` for intentionally unfinished branches.
- Do not leave ambiguous placeholders without intent/context.

## 11) Avoid / Banned Patterns

### Avoid

- Over-abstracting simple control flow.
- Hiding important behavior behind implicit side effects.
- Long functions that mix unrelated responsibilities.
- Silent fallback behavior that masks real failures.

### Banned

- `unwrap()`/`expect()` in production paths without explicit, justified invariant context.
- `println!` for production logging.
- Introducing heavy dependencies without cost/benefit rationale.
- Editing generated files by hand.
