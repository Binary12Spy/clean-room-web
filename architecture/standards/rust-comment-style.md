# Rust Comment and Documentation Style

This guide defines Rust comment and documentation conventions for TunnelWatch.

Use `docs/style/rust-best-practices.md` for implementation and engineering practices.

## 1) Module-level docs (`//!`)

Start each source file with module-level docs when the module is externally consumed or non-trivial.

```rust
//! # crate_name::path::module
//!
//! One-sentence summary of what this module provides.
```

Guidelines:

- Begin with a short heading/path and a one-line summary.
- Add architecture notes, protocol details, invariants, and operational caveats when needed.
- Use runnable examples only when they are stable and environment-independent; otherwise mark examples `ignore`.

```rust
//! # Examples
//!
//! ```ignore
//! let client = Client::connect(endpoint).await?;
//! let reply = client.ping().await?;
//! ```
```

## 2) Item docs (`///`)

### Structs, enums, traits, type aliases

- Start with a single sentence describing purpose.
- Add additional paragraphs only for non-obvious lifecycle/invariant constraints.

```rust
/// Tracks a single in-flight station command lifecycle.
pub struct PendingCommand { /* ... */ }
```

### Fields and enum variants

- One concise sentence.
- Reference related types/variants using backticks.

```rust
/// Response to `Command::Ping`.
Pong,
```

### Functions and methods

Use this structure for non-trivial functions and all public APIs:

1. Summary sentence.
2. Optional explanatory paragraph.
3. `# Arguments` section for parameters.
4. `# Returns` section for meaningful return values.
5. `# Errors` section for all `Result`-returning functions.
6. Optional `# Examples` section.

```rust
/// Write a framed message with a 4-byte big-endian length prefix.
///
/// # Arguments
/// * `writer` - Async writer receiving the framed payload.
/// * `value` - Value to serialize and send.
///
/// # Returns
/// `Ok(())` when the full frame was written.
///
/// # Errors
/// Returns an error if serialization fails or if the writer returns I/O failure.
pub async fn write_framed<W, T>(writer: &mut W, value: &T) -> Result<(), IpcError>
where
    W: AsyncWrite + Unpin,
    T: serde::Serialize,
{ /* ... */ }
```

## 3) Inline comments (`//`)

- Explain **why**, constraints, and invariants - not line-by-line mechanics.
- Keep comments close to the code they justify.
- For `unsafe` blocks, comments are mandatory and must state the upheld safety invariant.
- Prefer plain ASCII in comments and docs.

Avoid:

```rust
// Increment i
i += 1;
```

Prefer:

```rust
// Increment only after validation to avoid counting rejected records.
i += 1;
```

### Section headers

When a single file contains several logically distinct groups of items — and splitting into focused submodules is not warranted — use a `// #` section header to name the group:

```rust
// # Public API

pub struct SiteWatchInterface { /* ... */ }
pub enum SiteWatchInterfaceCommand { /* ... */ }
```

Rules:

- Use `// # Label` exactly — plain ASCII, no decorative filler characters or trailing dashes.
- The label is a short noun phrase, sentence-cased.
- A header earns its place only when the group is not already obvious from the items' own `///` docs or from the surrounding context. A single item does not need a header.
- Prefer focused modules over section headers. If a file needs many headers it is probably doing too much.
- Section headers are inline comments, not item docs — do not use `///` for them.

Avoid:

```rust
// ── Public API ─────────────────────────────────────  // non-ASCII, decorative
// PUBLIC API                                           // shouting
// --- Public API ---                                   // decoration without meaning
// Public API:                                          // colon implies a definition list
```

## 4) Language and tone

- Use clear American English.
- Keep statements direct and concrete.
- Avoid vague placeholders like "handle this" or "fix later" in docs.

## 5) TODO placeholders

- Use `todo!("what remains and why")` for intentional unfinished work.
- Avoid bare `todo!()` with no context.

## 6) Test comments

- Test functions typically do not need doc comments.
- Add module-level context comments when test intent is non-obvious.
- Keep test comments focused on scenario intent and regression rationale.
