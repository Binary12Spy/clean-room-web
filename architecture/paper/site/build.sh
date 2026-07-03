#!/usr/bin/env bash
# Render the paper (../what-the-web-could-have-been.md) into a single,
# self-contained static HTML file: CSS and fonts inlined, no JavaScript, no
# network at view time. The output honors the paper's own argument - a document
# should render with zero runtime and be inspectable by construction.
#
# Requires: pandoc (provided by the repo flake dev shell).
#
# Usage:
#   ./build.sh            # writes what-the-web-could-have-been.html
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
src="$here/../what-the-web-could-have-been.md"
out="$here/what-the-web-could-have-been.html"

if ! command -v pandoc >/dev/null 2>&1; then
  echo "error: pandoc not found. Enter the dev shell first: nix develop" >&2
  exit 1
fi

# The Markdown opens with its own title, subtitle, byline, and a '---' rule.
# The HTML template renders those in a masthead from metadata, so strip that
# leading block from the body to avoid showing the title twice. Everything from
# the first line after the '---' onward is the paper body.
body="$(mktemp)"
trap 'rm -f "$body"' EXIT
awk 'seen { print } /^---[[:space:]]*$/ && !seen { seen=1 }' "$src" > "$body"

# --embed-resources --standalone inlines the stylesheet and woff fonts as
# data: URIs, producing one portable file that works over file:// too.
# --shift-heading-level-by=1 pushes the paper's top-level '#' sections down to
# <h2>, leaving the masthead <h1> (the paper title) as the single document h1.
pandoc "$body" \
  --from=gfm \
  --to=html5 \
  --standalone \
  --embed-resources \
  --shift-heading-level-by=1 \
  --template="$here/template.html" \
  --css="$here/assets/tufte.css" \
  --css="$here/assets/paper.css" \
  --metadata title="What the Web Could Have Been" \
  --metadata subtitle="A clean-room redesign of the web stack, and a proof that a simpler one was possible." \
  --metadata author="Ethan Smith" \
  --metadata date="2026" \
  --metadata lang="en" \
  --output="$out"

echo "wrote $out ($(wc -c < "$out") bytes, self-contained)"
