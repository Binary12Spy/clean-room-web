# Paper site

A static, self-contained HTML rendering of
[`../what-the-web-could-have-been.md`](../what-the-web-could-have-been.md),
typeset with [Tufte CSS](https://edwardtufte.github.io/tufte-css/).

The output is a single file with the stylesheet and fonts inlined as `data:`
URIs. It has **no scripts, makes no network requests, and needs no runtime** -
it renders itself from plain markup. That is deliberate: it is the same property
the paper argues a document should have, so the artifact practices what it
preaches. It also means you can open it straight off disk (`file://`), email it,
or archive it, and it will look the same years from now.

## Building

Inside the repo dev shell (which provides `pandoc`):

```sh
nix develop            # from the repo root
architecture/paper/site/build.sh
```

This writes `what-the-web-could-have-been.html`. Re-run it whenever the Markdown
changes; the committed HTML is a convenience copy so the page can be published
without a build step.

## Files

- `build.sh` - pandoc invocation that produces the static HTML.
- `template.html` - pandoc HTML template (masthead + colophon).
- `assets/tufte.css` - Tufte CSS v1.8.0, trimmed to reference only the `.woff`
  fonts (vendored, unminified so it stays inspectable).
- `assets/paper.css` - small additive styles for code blocks and tables.
- `assets/et-book/` - the ET Book web fonts (vendored, `.woff`).
- `what-the-web-could-have-been.html` - the generated, self-contained output.

## Publishing

Serving `architecture/paper/site/what-the-web-could-have-been.html` is enough -
it depends on nothing else. For a nicer URL, point GitHub / Codeberg Pages at
this directory (or copy the one file wherever you like).
