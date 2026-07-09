---
title: what-the-web-could-have-been
subtitle: A clean-room redesign of the web stack, and a proof that a simpler one was possible.
date: 2026-06
display-date: June 2026
byline: Ethan Smith
slug: clean-room-web/what-the-web-could-have-been
summary: A clean-room redesign of the web stack, and a proof that a simpler one was possible.
listed: false
links:
  - text: Codeberg
    url: https://codeberg.org/binaryspy/clean-room-web
  - text: GitHub
    url: https://github.com/Binary12Spy/clean-room-web
---

The web is the most successful cross-platform runtime ever built, while also being so complex that the number of organizations able to implement it has fallen from five to three as it grew.

# History of the Web

[Tim Berners-Lee](https://en.wikipedia.org/wiki/Tim_Berners-Lee), a British computer scientist at CERN, built the HyperText Markup Language (HTML) in 1991. His goal was simple: easily share and organize research documents between him and his colleagues. Tim's invention was met with lukewarm indifference, primarily because his initial web browser was developed exclusively for the NeXT OS, which very few scientists at CERN actually used themselves. After Tim and his colleague [Robert Cailliau](https://en.wikipedia.org/wiki/Robert_Cailliau) built a stripped-down web browser that could run on any basic terminal, adoption began to pick up across CERN.

In 1993 CERN made the decision to [release the web's source code into the public domain](https://repository.cern/records/5r6e9-rh009), royalty free. This marked the beginning of a monumental shift in how individuals would use and interact with their computers. Around 1998 HTML's evolution was effectively frozen; the Document Object Model (DOM) APIs were added separately afterward, DOM Level 1 in 1998, Level 2 from 2000 to 2003, and Level 3 partially in 2004 before the group developing it disbanded and ceased work. The DOM APIs grew as a separate layer, bolted onto the success of HTML, and did not fit HTML's original intention.

[Brendan Eich](https://en.wikipedia.org/wiki/Brendan_Eich), an American software engineer at Netscape Communications, created JavaScript in 1995, famously completing the first prototype in about 10 days. Eich had been recruited to Netscape on the promise that he would get to embed the [Scheme programming language](https://en.wikipedia.org/wiki/Scheme_\(programming_language\)) in the browser. But Netscape's parallel deal with Sun Microsystems to bring Java to the browser changed his mandate: management now insisted the new language carry a Java-like syntax and play the role of Java's lightweight companion rather than be Scheme. Working under that constraint, Eich produced a language that blended Scheme's first-class functions, the prototype-based object model of [Self](https://en.wikipedia.org/wiki/Self_(programming_language)), and the surface syntax of Java. It went through two names, Mocha, then LiveScript, before Netscape and Sun settled on the final, marketing-driven name: JavaScript.

JavaScript did what was needed of it, acting as a small glue language to make the web browser more dynamic. Before JavaScript, if you wanted to validate user input in a text field you had to send that text back to the server for validation. JavaScript paved the way for a new class of website, one that was more dynamic and flexible while keeping network bandwidth low. Microsoft soon shipped its own reverse-engineered clone of the language, JScript, in Internet Explorer 3 (1996); the two implementations diverged enough that developers struggled to support both. Partly in response to that fragmentation, Netscape submitted JavaScript to [ECMA International](https://en.wikipedia.org/wiki/Ecma_International), a nonprofit standards organization, in 1997, which established the specification known as ECMAScript, the official standard underlying the JavaScript language.

[Håkon Wium Lie](https://en.wikipedia.org/wiki/H%C3%A5kon_Wium_Lie), a Norwegian web pioneer, proposed the concept of 'Cascading HTML Style Sheets' in 1994 while at CERN, where Berners-Lee and Cailliau had been developing the web. From 1994 to 1995, Håkon partnered with [Bert Bos](https://en.wikipedia.org/wiki/Bert_Bos), who had been building his own browser styling language, and together they merged their ideas into what became the Cascading Style Sheet (CSS). CSS was intended to separate web content from visual presentation, as HTML at the time lacked formatting options, forcing developers to repeat clunky tags to style text throughout a single document. In 1996 the World Wide Web Consortium (W3C) officially published CSS1 as a W3C recommendation for all browsers and web pages to support.

Microsoft's Internet Explorer 3 was the first commercial browser to offer support for the styling language, and adoption only grew from there. In 2007 Apple introduced the first working drafts of CSS Transitions and CSS 2D Transforms to WebKit. Then in 2009 Apple proposed CSS Animations and CSS 3D Transforms, formally introducing a Z-axis and the perspective property to the visual model. These features were ingested and officially published by the W3C as the 3D Transforms, Transitions, and Animations specifications. By 2012 Firefox and Internet Explorer 10 natively shipped full support for the suite of 3D transforms and keyframe animations. (Worth noting: these transforms operate _after_ layout, in the compositing stage; the layout engine itself stays two-dimensional. The Z-axis lives in the visual model, not the box model.)

The [Web Hypertext Application Technology Working Group](https://en.wikipedia.org/wiki/WHATWG) (WHATWG) was founded in 2004, consisting of representatives from Apple, the Mozilla Foundation, and Opera Software. The WHATWG itself describes the timeline and result of the modern web candidly in its "[Living Standard](https://html.spec.whatwg.org/multipage/introduction.html#design-notes)," noting that "HTML, its supporting DOM APIs, as well as many of its supporting technologies, have been developed over a period of several decades by a wide array of people with different priorities who, in many cases, did not know of each other's existence. \[...] Because of the unique characteristics of the web, implementation bugs have often become de-facto, and now de-jure, standards."

HTML has been a versionless "Living Standard" since 2011, changing continuously. That year, the W3C and the WHATWG concluded that even they had different goals: the W3C wanted a finished HTML standard, settling on "HTML5," while the WHATWG wanted a continuously maintained Living Standard rather than freezing the technology. [This WHATWG blog post](https://blog.whatwg.org/html-is-the-new-html5) officially announced that clear divorce from the W3C's intent. Ultimately, the W3C formally conceded in 2019, [signing an agreement](https://www.w3.org/2019/04/WHATWG-W3C-MOU.html) to defer to the WHATWG Living Standard as the single, authoritative version of HTML.

This history culminates in a platform, the "Web", that is no longer one spec but a sprawl of interdependent ones: HTML, DOM, Fetch, Encoding, Streams, URL, Storage, Web IDL, plus a Compatibility Standard and a Quirks Mode Standard whose entire purpose is documenting non-standard behavior browsers must replicate. The existence of a spec for quirks is itself proof of the technology's accretion.

# The Two Diagnoses

Two structural problems emerge from this history, and crucially, neither was anyone's mistake. Each layer was a reasonable local answer to the question being asked at the time; the trouble is what they compound into.

## A. Conflation

The current technologies of the web conflate two jobs, document formatting and application runtime, both in one stack, and serve them both imperfectly. This is exemplified in a few cases:

- A document is meant to be a static, cacheable, linkable, readable file, and it should work without the need for a runtime engine. An app wants the opposite: state, a live render/compute loop, the ability to react. Modern web technology asks the same `<div>` tag to be the vessel for both. The result is documents that won't render without downloading and executing megabytes of JavaScript, and "apps" that abuse semantic document tags as layout primitives.

- The Document Object Model was designed as an API for documents, not applications. It is a tree of nodes meant to be traversed and annotated programmatically. Apps now drive it as a rendering pipeline, mutating that tree 60 times a second. React was built around the concept of a Virtual DOM: another layer that sits on top of the real DOM and papers over the document-tree nature of an API that has been conscripted into being an app framework. An entire industry of tooling exists to reconcile that mismatch.

- Accessibility is a critical feature of any information-delivery system. Screen readers work because document semantics lay information out in logical sections, marking headers and allowing navigation even with limited capability. Applications that render through Canvas bypass the document layer entirely and create accessibility dead zones. The "app" use case actively erodes the thing that made the document use case worthwhile. The two systems end up degrading each other.

## B. Irreversibility

Backward compatibility is unbreakable: a web page made in 1995 must still render, so nothing can ever be removed, only added. Accretion is therefore structural to the system, not accidental; the governance model of the web itself guarantees it.

- Web technologies as a whole lack a version negotiation mechanism. The standard says a page from 1995 must still render, but that 1995 page and a 2026 Progressive Web App are served the same way, with no declared version the browser engine could use to gate legacy behavior. The engine must hold every past behavior simultaneously, forever.

- The Quirks Mode standard is a formally specified mode whose entire purpose is to force modern browsers to reproduce bugs from the browsers of the 1990s. The web's history is codified by the existence of this standard, and that standard only compounds as time goes on.

- By the WHATWG's own admission, "implementation bugs have often become de-facto, and now de-jure, standards." The specification of what a web browser *is* does not amount to a clean blueprint; it is a description of what browsers throughout history actually did, correctly or not. The causality runs backward from how a standard is supposed to work, and that leaves a mountain of backlog for anyone wanting to build their own browser.

- The rising complexity of what it means to be a browser caused the independent implementations present near the beginning to collapse in number. It would take a modern team of developers years to replicate a browser that passes every mark of the standard *as it is today*, never mind where the standard will have moved by the time they finish. This disparity not only drove consolidation among browser engines, it is anti-competitive by its very nature.

A small lineage note about the main web engines of today: WebKit forked from KHTML, Blink forked from WebKit, so two of the three surviving engines share ancestry. This makes the honest count of *independent* engines closer to two than three.

None of this was inevitable, and other systems prove it. Languages that allow themselves to deprecate evolve cleanly to suit modern requirements. Python 3 broke most Python 2 scripts and, over a painful decade, moved the language forward. The transition was not painless, but it was _possible_, which is the point: the ecosystem had a mechanism to leave its mistakes behind, even at a cost. The web's "backward compatibility forever" was never actively chosen in the way those version breaks were. It was baked in by the absence of coordination and a versioning primitive, and that absence is precisely what makes the backlog unbounded.

The web was built of small parts that each independently answered the specific problem being asked at the time, reasonably. The idea of backward compatibility in perpetuity sounds good, but without careful auditing of what enters the standard, the barrier to entry becomes punishingly high, because of the mountain of backlog that must be supported.

On the other hand, the web's refusal to break is exactly why a 30-year-old page still renders and works. It is the reason the web is the most durable publishing format ever built, and why no one needs permission to publish to it. That durability is real, and valuable. The argument that follows is not that compatibility is bad; it is that perpetual compatibility *without a retirement plan* is what proves fatal.

# What is Worth Keeping

Now that we know the history of how the web came to be what it is today, let's break down what things about it are genuinely genius and should persist.

## 1. Universality
This is the single most valuable property of the existing web. One artifact of development runs on a phone, a decade-old laptop, a kiosk, a screen reader, a smart fridge. There is no per-platform port. This is what native app ecosystems never solved and why the web is everywhere. Any comparable redesign of the web must not require a specific OS, CPU architecture, vendor runtime, or capability. The execution target must be portable by construction.

## 2. Linkability
The web is addressable by a string you can put in an email, a text, a QR Code. Any web resource, anywhere. Deep-links, sharing, bookmarkable, no install. This is such a deep fundamental principle of the web that most people forget it's there. A new system must preserve stable, universal addressing. A world full of opaque app bundles that can't be linked across and into would be a regression in capability.

## 3. Inspectability
You are able to look at and reason through the source code of webpages today. This feature made the web understandable to a generation without gatekeepers, and it keeps the platform honest. You can audit what you can see and what runs. A new system that loses this inspectability loses what made the web easy to learn. Later in this document I propose a new web stack, and this is one place where the document layer will remain open and inspectable by construction. The application layer, being compiled, would require preserving that inspectability deliberately: open formats, documentation, and the option to ship source or source-maps.

## 4. Graceful Degradation
A webpage can still deliver on its core purpose, the text and content included in it, when scripts fail, the network is slow, or the device is under-powered. The content of the webpage survives the failure of the fancy layer. A loss of the same level of graceful degradation would narrow the use cases where the technology is still useful.

## 5. Open Standards
With the web, anyone can publish a web page, anyone can implement, there's no license fee, no gatekeeper. The web today is this free because CERN put it into the public domain back in 1993. A new web would need to be just as openly specified and royalty-free, or it will be objectively worse than what it would replace. As with the following proposal, I do not wish to become a gatekeeper of the spec, it should be with collaborative effort that we prevent a new standard from bloating.

## 6. Backward Durability
The web works the same way as it did 30 years ago, just with a lot tacked on top. That is precisely why a webpage made in the 1990s renders today. A webpage that works should work forever, as of the version it declared, not to be dragged forward or broken. Much like with the Rust programming language, Rust you write today will compile successfully the same way 10 years later, web content should too. A new web should keep the durability of content created, while decoupling it from the mechanism that resulted in unbounded compatibility.

And "forever" does not require every engine to ship every renderer in perpetuity. An old version's renderer can itself become an optional, sandboxed island, downloaded when an old page needs it, exactly the mechanism proposed for rendering the legacy web. "Works forever" becomes "the means to render it remains available," not "every browser carries every version's code built in." That is strictly better than today, where the legacy _is_ built into every engine, with no opt-out.


Moving forward, these six properties will be used as the rubric to grade any proposed stack against, and all must be met or that system will be deemed a direct downgrade to what we have today.

# Clean-Room Principles

In order to build a system that lives up to the rubric while avoiding the pitfalls that grew, we can set forth five principles, each one making the next one possible, working from how the code is run up to how the standard governs itself.

## 1. Single Execution Target
The platform should define a single, neutral, sandboxed execution target that all languages compile to, so that no language is ever privileged or permanent, every higher layer can be an equal swappable library, and one bundle runs unchanged on every device, accepting in exchange that raw executable code is no longer human-readable by default.

On today's web, JavaScript is _the_ language, not just _a_ language. It has privileged, hardcoded status nothing else can have. Every browser must ship a JavaScript engine. Every other language that wants to run in a browser has to either compile to JavaScript or, more recently, to WebAssembly, but JavaScript still sits in the privileged seat as the thing with direct DOM access and first-class platform support.

That privilege was an accident, not a design. JavaScript became a standard because Eich shipped something in ten days back in 1995 and it happened to be the only real option when the web exploded. The web was backed into JavaScript, and the "Never Break" principle welded it in permanently.

## 2. Push Features Into Userland
Because the language is neutral and unprivileged, any feature that can be pushed to userland _must be_ pushed into the swappable userland libraries, separating the layout, widget, and rendering frameworks from the core of the engine. This prevents the versioned core from bloating or cycling versions too quickly, and keeps the core to only what is needed to run the webpage, doing only what genuinely can't happen above it.

Today, when a new idea on how the web should work comes along, a new way to render something, to display a table, a proposal is made, and if it gets popular enough it gets integrated into the living standard. This causes the core to bloat and makes it impossible to freeze the core and call a version done.

By pushing new features into userland, that popular new table-rendering idea instead ships as a library that a webpage imports. That library competes with alternatives on merit, and the core of the web has no concept of what a table is or that the library is even there.

## 3. A Small, Explicitly-Versioned Core
With all new features cleanly separated out into userland libraries, the core of the web is left free to be minimal, frozen, and versioned. Your website can be pinned to a version of the core, and the browser has to support it.

This heals the wound of Irreversibility in the current web. Each version of the core is frozen, while the _line_ of versions evolves with new requirements, and old capabilities and behaviors can be safely retired in a way most of the software world already understands. This principle also addresses the Backward Durability point of the rubric: the web can evolve, but a page built for web 1.0 can still render on a browser that supports 1.0. The standard for 3.0 isn't beholden to the versions before it.

The objection writes itself: a browser in 2050 still has to render v1, v2, and v3 content, so haven't we just renamed "hold every behavior forever"? The difference is isolation. The web's burden is not that it has thirty years of features, it is that all thirty years are live in one entangled engine at once, where features interact and quirks leak across boundaries. Frozen version-renderers do not interact. A v1 renderer never has to interoperate with v3; each is a sealed artifact. The burden is not eliminated, but it changes from _unbounded and entangled_ to _bounded and modular_, and a modular burden is one a new implementer can take on a piece at a time, or skip.

## 4. Capability-Based Security
Because userland execution is separated from the core, the next concept is to let the browser determine what a webpage is allowed to touch. A webpage requests permissions, the core enforces them at the edge, and what we get by default is a webpage that can't call what it wasn't handed. This results in a safer browsing interaction, as every webpage is sandboxed from the get-go to prevent cross-site mischief. It replaces the same-origin, CORS, and CSP patchwork with a single clean model: no language has god-mode access to anything in the browser.

Because the execution target is already a sandbox (Principle 1), the core only has to decide which doors to open in that wall. This principle heals the patchwork of security layers in the existing web with a more coherent and intentionally designed system. It also addresses a point that is not in the rubric but is a given in the modern day: safety while using the internet is important. Making the technology safe to use is just as important as making it able to evolve.

## 5. Spec-as-Conformance-Suite
The final principle is that there must be an executable conformance suite to test any new implementation of the core against. This allows, and even encourages, newcomers to build a browser. An executable conformance suite _is_ the spec, meaning any new browser for any new platform passes a test for validation instead of needing to reverse-engineer Chrome. This directly prevents what we have seen with the web today, where there were once five major browsers and now there are three, and the effort to build a new one without cheating on ancestry is near impossible.

This keeps the spec open for everyone and easily testable against. It directly addresses the WHATWG's own admission that, on the current web, "implementation bugs became de-jure standards." These five principles aim to keep the good parts of the current web while answering the question: how do we avoid re-creating the same problems?

# The Architecture

The five principles describe rules. This section describes the structure those rules produce. The whole thing rests on a single idea: there are two boundaries that matter, and everything else is arranged around them.

## The Two Boundaries

The first boundary is between the **host** and the code it runs. This is the frozen one. It is tiny, explicitly versioned, and it is the only thing a new browser engine has to implement to be conformant. Get this boundary right and a second, third, or fourth engine becomes a tractable project instead of a decade's reverse-engineering.

The second boundary is between an **application** and the **library it uses to describe its interface**. This is the free one. It is not frozen, not owned, and not singular. Many can exist at once, they compete on merit, and the host has never heard of any of them.

One boundary rigid, one boundary plural. Hold those two in mind and the layers below arrange themselves.

| Layer                   | Role            | Statement                            | Boundary    |
| ----------------------- | --------------- | ------------------------------------ | ----------- |
| Layer 4: The App        | (your bundle)   | "I am a todo app"                    |             |
| Layer 3: UI/Layout lib  | (userland WASM) | "I turn an element tree into boxes"  | free/plural |
| Layer 2: The Bundle ABI | (the contract)  | "Draw here, you may use the network" | frozen      |
| Layer 1: The Host       | (native code)   | "I run WASM, I own the GPU"          |             |
| Layer 0: Platform       | (OS/GPU)        | "Actual pixels & sockets"            |             |

## Descending the Stack

Rather than build up from the metal, it is clearer to start where the reader already lives, at the application, and ask what each layer needs from the one beneath it.

**Layer 4, the App.** This is a todo list, a spreadsheet, a blog. It holds its own state and its own logic. It does not talk to the GPU and it never sees a raw socket. What it needs is a way to say what its interface should look like. So it reaches for the layer below.

**Layer 3, the UI / Layout lib.** This is the thing today's web bakes into the engine, and the thing this design refuses to. It takes a tree of interface intent, boxes, text, a list, a button, and turns it into concrete positioned things to draw. It does layout math. It does hit-testing, knowing which box a click landed in. Crucially, it ships *inside the bundle*. App A can use a flexbox-style library, App B a constraint-solver library, and the engine below is identical and oblivious to both. To do its job, this layer needs to actually draw pixels and, sometimes, reach the network or disk. So it reaches for the layer below.

**Layer 2, the Bundle ABI.** This is the frozen boundary itself, expressed as a contract. It is the complete, small list of calls that cross between native code and the sandboxed bundle, in both directions: the host can hand the bundle input events and ask it to render; the bundle can emit draw commands and, *if granted*, call out to the network or storage. This layer is deliberately ignorant of everything above it. It knows how to push a rectangle; it has no concept of a button, a table, or a todo. It needs something to actually honor these calls. So it reaches for the layer below.

**Layer 1, the Host.** This is the "browser." It runs the WASM bundle, owns the real GPU surface, owns every real-world resource, and hands out the capabilities that gate them. When the bundle says "draw this rectangle," the host turns that into a real GPU call. When the bundle says "fetch this URL," the host checks whether that capability was granted and, if so, performs it. The host is a resource broker and a WASM runtime and nothing more. It has no opinion about what a user interface is. To touch actual hardware, it reaches for the layer below.

**Layer 0, the Platform.** The OS, the GPU driver, the network stack. Real pixels, real sockets. The host is the only thing that talks to it.

The pattern to notice: each layer knows nothing about the layer two above it. The host cannot name a button. The layout lib cannot name your app. That enforced ignorance is not an accident of the diagram, it is the entire mechanism that makes the layers swappable and the engine re-implementable.

## A Click, Start to Finish

The architecture is easiest to feel by following one event all the way through.

1. The **Platform** delivers a raw mouse click to the **Host**.
2. The Host wraps it as an input event and passes it across the **ABI** to the **App**.
3. The App asks its **Layout lib**, "what is at this coordinate?" The library walks its box tree and answers.
4. The App mutates its own state, a todo gets checked, and on the next frame asks the Layout lib to render.
5. The Layout lib computes positions and emits draw commands, push this rectangle, draw this text, back across the **ABI**.
6. The **Host** receives those commands and issues the real GPU calls to the **Platform**.
7. Pixels.

And the inverse direction, the capability flow: the Host decides at load time what this bundle may do, wires up only those calls, hands over the capability handles, and from then on the bundle physically cannot reach anything it was not given.

## The Two Contracts, in Code

The frozen boundary, the ABI, is small enough to show in full shape. The host calls *into* the bundle:

```rust
// The bundle exports these. The host calls them.
trait BundleExports {
    fn init(&mut self, caps: CapabilityHandles); // receive your capabilities
    fn on_event(&mut self, event: InputEvent);   // mouse, key, resize
    fn render(&mut self, frame: &mut FrameContext); // emit draw commands
}
```

And the bundle calls *out* to the host, grouped by capability, only the groups it was granted exist at all:

```rust
// Always available: drawing.
fn push_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
fn push_text(font: FontId, text: &str, x: f32, y: f32);
fn measure_text(font: FontId, text: &str) -> Size;

// Only present if the capability was granted:
fn net_fetch(url: &str) -> RequestId;          // networking
fn store_set(key: &str, value: &[u8]);         // storage
```

That is close to the entire platform surface. Note what is absent: there is no `createElement`, no styling, no flexbox, no layout, no notion of a node. The host gives a rectangle of pixels, the means to draw into it, input events, and gated access to the outside world. Everything else is built above, in WASM.

One honest exception hides inside `push_text` and `measure_text`. Text rendering, shaping, bidirectional text, font fallback, and the line-breaking that depends on them, is one of the genuinely hard problems in computing, and it sits _below_ the frozen boundary, in the host. This is deliberate: text is the one thing where a single shared, correct implementation is worth far more than the line-count saved by pushing it up, both because correctness is brutal to get right and because consistent text is itself a user-agency property. So the core is small, but it is not trivial. The conformance suite for text alone would be substantial. The claim is not that the host is easy to write; it is that the host is _finite and specifiable_, which is the property the current web lacks.

The free boundary looks like an ordinary library call, because that is exactly what it is:

```rust
// Layer 3 is just a crate the app bundles. The host has never heard of it.
let mut ui = FlexLayout::new();
let root = ui.column(&[
    ui.text("My todos"),
    ui.button("Add"),
]);
ui.render(root, viewport); // computes boxes, calls push_rect / push_text
```

Swap `FlexLayout` for `ConstraintLayout` and nothing beneath Layer 3 changes. The host does not know, and cannot know, that anything was different.

## What This Buys

Layout is the single hardest thing to reimplement in a browser, the box model, flexbox, grid, line-breaking, and it is the largest reason a new web engine is infeasible today. In this architecture, layout is no longer in the engine at all. It is a userland library the engine has never heard of, shipped with the page, swappable for another, versionable on its own. The thing that made the old web impossible to re-implement is, here, just a dependency.

But this cuts both ways, and it is worth being honest about the cost. When the engine owns the layout, it can *reason* about every page: find-in-page, reader mode, zoom, user stylesheets, a screen reader walking the structure. These are the things that make a browser a *user* agent, not just a delivery pipe, and they exist because the engine understands the box tree. Push layout into userland and the engine loses the understanding for app mode. An app-mode page is, to the host, a sequence of rectangles and glyphs with no semantic structure to interrogate. This is a real loss, and it is the reason the next section exists: document mode is not a lesser sibling to app mode, it is where all of the user-agency properties live. The design's bet is that most of the web should be documents, and that the things which genuinely must be pure apps were never going to offer find-in-page anyway.

# Document Mode

In the section above we went into the shape of the architecture for an application in this new web; in this section we will cover the document mode. The original sin from Diagnosis A was that the current web conflated the same substrate to be both a document and an app runtime. Here we will work through the second half of un-conflating it.

## What a Document is

In this spec a document is a small, frozen, declarative format. It consists initially of concrete semantic structure (headings, sections, lists), links, media, forms. It is a layout model specified by explicit rules rather than "whatever the dominant engine does." There is no execution required to render it. The whole point is that it's the thing that was pointed out as worth keeping: static, cacheable, linkable, readable, and able to survive failure of any fancy layer, because there is no fancy layer it depends on.

## Guarantees

A document must meet these four guarantees, and they map cleanly onto the rubric outlined earlier.

### 1. Renders With Zero Runtime

No WASM, no app layer, no capabilities needed. A conformant document renderer is small enough that one person can build it, proof that a frozen declarative core can be tiny. This is Graceful Degradation as a structural guarantee: the content can't fail to render because of a broken script, since there is no script in the path.

### 2. Inspectable by Construction

App mode is compiled WASM, opaque by default, and I promised in principle #1 that document mode would be the open half: human-readable source. This split of document and app is the resolution to the conflation diagnosed earlier. App mode trades inspectability for power; document mode keeps it by staying declarative.

### 3. Accessible by Construction

Because semantics are real and present in the format (a heading is a heading, not a div styled to look like one), screen readers work without anyone doing any extra labor. Contrast with app mode or the canvas usage of the current web, which as stated are accessibility dead zones. Document mode is where accessibility is free because the structure carries meaning.

### 4. Linkable and Addressable

Stable URLs into and across documents. This directly addresses the Linkability rubric item, and it is almost entirely addressed by document-mode. Documents are the things you cite, bookmark, share. Apps can be addressable too, but the durable, deep-linkable, no-install web for references is fundamentally a document-mode property.

## The Relationship Between Document and App Mode

This is where the roads cross, and the decision on how these two should interact is important. A document can *embed* app-mode islands, but an app cannot dissolve the document.

That is, document mode is the substrate; app mode is something a document can opt into for a region that genuinely needs a live compute loop (a map, an editor), the way the current web embeds a widget, but more firmly grounded due to the capability model. The document around the app island remains inspectable, degradable, accessible. The app island is sealed visibly and locally, not infecting the whole page. This directly answers the "documents that won't render without megabytes of JavaScript" complaint from Diagnosis A: the document renders regardless; the app island is additive, not load-bearing. It also keeps true to principle #4, the island is incapable of reaching out of its bounds and rewriting the document around it because the capability model only hands it what it was explicitly granted.

The current web inverts this. The *app* is the substrate (an empty HTML shell + JavaScript that builds everything), and the document semantics are an afterthought bolted on for SEO and accessibility, often badly. This design flips it back into a natural state: the document is the grounding, app is the opt-in island of additional complexity. That inversion is the exact answer to the conflation of the current web.

This design doesn't ban apps, it removes the default that made everything one. The things that can and should be documents become documents again. The things that genuinely are apps, a design tool, a live editor, become bounded islands. And for those, you lose no openness the current web actually delivers, a canvas-based web app is already an opaque blob today. The difference is that here the blob is contained, with an open, durable, accessible document around it, instead of being the whole page.

A note on what keeps documents universal: the document renderer is itself just a thing the host can run, and it is the one piece of "layout is library" that ships *with* the host as a default, precisely so that the universal, no-install, just-works baseline can exist. But, if the document layout is a library like any other, what makes it universal? The document *format* is frozen and standardized so that any host renders it identically, even though app-mode layout libraries are plural and shipped per-app. So the frozen core/plural library split isn't host-vs-userland, it's complementary. Document-format-frozen, app-layout-plural.

## The Caveat

A frozen document format risks the opposite failure mode from the current web. The web grew uncontrollably because the standard was ever evolving; a frozen format risks being *too* limited, people will want the one feature it doesn't have, and the pressure to add "just one more element" is exactly what caused HTML to become as accreted as it is. Looking back to principle #3, we don't grow Document v1, we ship Document v2 as a new frozen artifact.

This also cleanly maps into principle #2, pushing features up into userland when we can. Anything that is needed, and that *can* be an app-mode island shouldn't go into the document format at all.

These two mechanisms are why the frozen format stays small instead of gathering bloated requirements, the document format can stay tiny and stable precisely because it has somewhere to shift the pressure (a new version) and somewhere to put complexity (an app island). The two valves HTML never had.

# A Proof of Concept

The preceding sections argued that a design like this is sound; this one aims to describe the smallest thing that proves it. My goal in this document and PoC is not to replace the web or even own the standard, but to prove the principles end to end. I do not intend for this to be a product, this is a proof that demonstrates the thesis.

The PoC must prove the clean-room principles and the architecture can work:
1. The engine can be tiny and oblivious.
2. Layout can be a swappable userland library.
3. Capabilities can replace ambient authority.
4. A document renders with zero runtime.

## Milestones

 - M0: A host loads a WASM bundle, and that bundle draws a rectangle. This proves the host <-> bundle boundary works end to end and de-risks the whole stack.
 - M1: Capability system, granted bundle fetches, denied bundle can't. Proves capability-based security is achievable, not aspirational.
 - M2: One layout library in WASM renders actual UI (text, boxes, a button, hit-testing). Proves layout can live entirely in userland.
 - M3: The keystone, a second layout library, different paradigm, swapping in with zero host change. Proves the central architectural claim, that the thing which makes the current web un-reimplementable is here, just as a dependency.
 - M4: Document mode, reference renderer, a working link between two documents. Proves the document half, zero-runtime render and linkability.

I want to clearly state that this deliberately is not a product to be shipped. It is not a usable browser, it is an incomplete format, and it is not performance-tuned. This is intended to prove the principles and architecture can exist.

## Success Indicators

The success indicators of this PoC will be a runnable, bare-bones native client implementing the tiny engine core. Success is that client loading and rendering two pages that use different layout libraries, one swapped for the other, with the client holding no knowledge of either. Successfully hitting milestone 3 alone is the whole argument of this document made visible, and everything past it is corroboration.

# Limitations

We've dissected the current web, kept what was worth keeping, and built a system to the rubric. This section is about where that system is genuinely hard.

## 1. The Install Base Moat
This is the biggest reason the web is stuck today and the reason any new web technology stack that replaces *everything* faces near-impossible odds. A platform's true value is its content and its users, and a brand-new stack starts with zero of either. The web, as crufty as it is, has had 30 years of content and development practices built around it; it cleanly beats any stack with none, every time. This is not a problem I aim to solve, it's a problem that this design has, and there is no easy solution for that. This is a side-project proof that was interesting enough to monkey around with dreaming up a better web. I do not claim to displace the web, nor do I intend to. I'm simply proving a point about how it *could* have been built.

## 2. The Migration Story
If this architecture were to ever get significant adoption, a plan for the old web to bridge would be required. The honest truth is: legacy web content would render inside a sandboxed capability *library*, not in the core. This would be an elegant solution because it would in some way use the new, better architecture. But the catch is that a compat library itself would be enormous, which would defeat the intention of eliminating complexity. That complexity simply gets relocated into an island, quarantined, instead of welded into the core. The beauty of a compatibility layer is that it would be *optional* by design.

## 3. The Blob Risk
The app mode architecture can easily become *less* open than HTML is, becoming a stack of opaque binary blobs, which is roughly what Flash was and what a pure-canvas app already is today. This point is de-risked with document mode, the substrate that sites are meant to be built upon. It would take good stewardship of a site's implementation to properly enforce the mindset of "build what can be in the document, use app islands for the genuinely complex portions." Nothing about this spec, or any spec, can realistically prevent anyone from misusing it. A world where everyone reaches to make one big app island anyways brings back the same opacity issue. The mitigation is incentive and default, not a guarantee.

There is a sharper version of this worth stating plainly. A DOM-based app today, the common case, is not fully opaque: the browser still understands its structure, so accessibility tools, find-in-page, and zoom still mostly work even on a heavy web app. App mode has no equivalent. By moving layout out of the engine, app-mode pages give up the user-agency features that a shared layout engine provides for free. This is not a bug to be fixed later; it is the deliberate price of engine simplicity. The design pays it primarily by making document mode, where the engine _does_ understand structure, the default and the substrate, and confining app mode to the islands that genuinely need a live compute loop. If that confinement fails, if everything becomes an app island, the web loses these properties. That is the same failure as the blob risk above, seen from the user's side instead of the developer's. There is, however, a candidate mechanism that recovers much of this: semantic tags riding the draw stream, so the host can build an accessibility tree from app-mode rendering the same way it does from a document. It is involved enough to deserve its own treatment, and is left to follow-up work.

## 4. Governance
The proposed "spec-as-conformance-suite" is a great concept and opens a lot of doors, but someone must maintain that; that someone has soft power over the spec. In a perfect world I see this spec needing no gatekeeper, that everyone would agree on what *truly* needs to be in the core, and all agree to revise it unanimously. The world does not work like that, and I personally do not feel that I can take the reins to head the governing body of a standard this universal.

## 5. The Seam
The hardest design work in this system lives exactly where document mode meets an app island, and this document does not pretend to have settled it. When a link is clicked inside an island, who navigates? Can an island request navigation of the whole page? Who owns the back button, the island or the document around it? The current web's worst behavior lives at precisely these seams, and a clean split of document and app does not make the seam questions disappear, it just makes them _explicit_ instead of emergent. Answering them concretely is a primary job of the proof of concept, not the essay.


Naming these limitations is the point; the value isn't that it ships, it's that it proves an alternative was possible, that the web's complexity was a path taken, not the only path.

# Conclusion

The web won by being universal, then grew so complex it strangled the universality that made it win.

To prove the point of this document is to shine a spotlight on a better path: a web that is clean, concise, and understandable. One that evolves with us without hindering us, because explicit versioning lets the core move forward while old pages keep rendering exactly as they were written. A technology with intention behind its design, with a core that stays small because layout and features live in userland, and a core that understands needs will change. Something even more free and open than what we have today, simple enough that anyone can implement it, with a conformance suite to prove they did. A solution that I, as a software engineer, would be proud to put into the world knowing that it is good.

