# clean-room-web

The web is the most successful cross-platform runtime ever built, and it has grown so complex that the number of organizations able to implement it has fallen from five to three.

This is a clean-room redesign of the web stack, and a proof that a simple one was possible. Ity argues that the web's present complexity was a path taken, not the only path, and it backs that argument with a working proof of concept.

It is not a product, not a browser you should use, and not an attempt to replace the web. It is a thought experiment carried far enough to run: a demonstration that the dingle hardest thing about the modern web, the reason a furth browser engine is effectively infeasible, does not have to be hard.

# Start Here

1. [The paper: What the Web Could Have Been](./architecture/paper/what-the-web-could-have-been.md). The full argument: how the web accreted, the two structural problems underneath it, and the architecture that avoids them.
2. [The proof of concept](./architecture/poc/). The smallest thing that demonstrates the architecture is real.
3. The follow-up (planned). Recovering user-agency in app mode: semantics that ride the render rather than living in a DOM. Not yet started.

# Status

- Paper: complete.
- Proof of concept: not yet started. Milestone ladder defined; see the [PoC README](./architecture/poc/README.md) for the build plan.
- Follow-up paper and PoC: planned.

# The Ideas, in Brief

The web fails at two things structurally, and neither was anyone's mistake:

- Conflation. One stack is forced to be both a document format and an application runtime, and served both imperfectly.
- Irreversibility. Nothing can ever be removed, only added, so complexity only grows and the barrier to a new implementation only rises.

The redesign answers these with five principles, each making the next possible:

- One execution target. A single neutral sandboxed bytecode (WASM); no language is priviledged or permanent.
- Features in userland. Anything that *can* be a library *must* be one. Layout, widgets, and rendering live above the engine, not inside it.
- A small, frozen, versioned core. Each version is sealed; the line of versions evolves. Old pages render in the version they declared.
- Capability-based security. Code can only touch what it was explicitly handed. No ambient authority, no same-origin/CORS/CSP patchwork.
- Spec-as-conformance-suite. The executable test suite *is* the spec, so a new engine passes a test instead of reverse0engineering an incumbent.

# The Keystone

The whole architecture rests on one move: **layout is no longer in the engine**. It is a userland library shipped with the page. The proof of concept demonstrates this directly, one host loading two libraries, one swapped for the other, with the host holding no knowledge of either. The thing that makes today's web impossible to re-implement becomes, here, just a dependency.

That swap, running an oblivious host, is the entire argument made visible.

# Honesty on the hard parts

This is not a pitch, and the paper does not pretend the design is free. It treats its real limitations directly: the install-base moat that makes adoption near-impossible, the migration burden, the unsettled seam where documents meet embeeded app islands, and the user-agency cost of moving layout out of the engine. See the papers [Limitations](./architecture/paper/what-the-web-could-have-been.md#limitations) section. Naming these is part of the point: the value of the proof is not that is ships, it is that it shows an alternative was possible.

# About

A personal exploration by Ethan Smith, 2026. One engineer's answer to a question worth asking: if we started clean, keeping what made the web great and designing against what made it rot, what would it look like?
