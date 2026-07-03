#!/usr/bin/env bash
# Render the site's Markdown sources into single, self-contained static HTML
# files: CSS and fonts inlined, no JavaScript, no network at view time. The
# output honors the paper's own argument - a document should render with zero
# runtime and be inspectable by construction.
#
# Requires: pandoc (provided by the repo flake dev shell).
#
# Usage:
#   ./build.sh            # writes what-the-web-could-have-been.html and index.html
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if ! command -v pandoc >/dev/null 2>&1; then
  echo "error: pandoc not found. Enter the dev shell first: nix develop" >&2
  exit 1
fi

# Render one Markdown file to a self-contained HTML page.
#
# The template renders the title/subtitle/byline in a masthead from metadata, so
# the body must not repeat them. We drop the leading heading block from the
# source: everything up to and including the first blank line after the initial
# H1 (+ optional subtitle/byline) is metadata already shown by the masthead.
#
# Args: <src.md> <out.html> <title> <subtitle> <author-or-empty> <date-or-empty>
render() {
  local src="$1" out="$2" title="$3" subtitle="$4" author="$5" date="$6"

  local body
  body="$(mktemp)"

  # Strip the leading front-matter block. Papers use a '---' rule after the
  # byline; the index has no rule, so fall back to dropping the first H1 and a
  # following italic subtitle line. Emit everything after that intro.
  if grep -qE '^---[[:space:]]*$' "$src"; then
    awk 'seen { print } /^---[[:space:]]*$/ && !seen { seen=1 }' "$src" > "$body"
  else
    awk '
      NR==1 && /^# / { next }              # drop the leading H1 title
      !started && /^\*.*\*[[:space:]]*$/ { next }  # drop a leading italic subtitle
      !started && /^[[:space:]]*$/ { next }        # drop blank lines before body
      { started=1; print }
    ' "$src" > "$body"
  fi

  # --embed-resources inlines the stylesheet and woff fonts as data: URIs.
  # --shift-heading-level-by=1 pushes top-level '#' sections to <h2>, leaving
  # the masthead <h1> as the single document h1.
  local args=(
    "$body"
    --from=gfm
    --to=html5
    --standalone
    --embed-resources
    --shift-heading-level-by=1
    --template="$here/template.html"
    --css="$here/assets/tufte.css"
    --css="$here/assets/paper.css"
    --metadata title="$title"
    --metadata subtitle="$subtitle"
    --metadata lang="en"
    --output="$out"
  )
  [ -n "$author" ] && args+=(--metadata author="$author")
  [ -n "$date" ] && args+=(--metadata date="$date")

  pandoc "${args[@]}"
  rm -f "$body"
  echo "wrote $out ($(wc -c < "$out") bytes, self-contained)"
}

render \
  "$here/../what-the-web-could-have-been.md" \
  "$here/what-the-web-could-have-been.html" \
  "What the Web Could Have Been" \
  "A clean-room redesign of the web stack, and a proof that a simpler one was possible." \
  "Ethan Smith" \
  "2026"

render \
  "$here/index.md" \
  "$here/index.html" \
  "clean-room-web" \
  "A clean-room redesign of the web stack, and a proof that a simpler one was possible." \
  "" \
  ""
