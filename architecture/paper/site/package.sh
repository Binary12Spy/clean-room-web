#!/usr/bin/env bash
# Bundle the rendered paper into a static-hosting zip.
#
# The zip's internal layout is the served URL layout. Deploying its contents at
# the site root of clean-room-web.project802.io yields:
#
#   /                                     ->  index.html          (the index/TOC)
#   /what-the-web-could-have-been         ->  what-the-web-could-have-been/index.html
#
# Every page is fully self-contained (CSS and fonts inlined), so there are no
# sibling assets to carry. Add more papers by copying each rendered HTML into
# its own <slug>/index.html below.
#
# Requires: the rendered HTML to exist (run ./build.sh first). Uses the `zip`
# CLI when available, otherwise falls back to python3's zipfile module.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
html="$here/what-the-web-could-have-been.html"
dist="$here/dist"
zip_out="$here/clean-room-web-site.zip"

if [ ! -f "$html" ]; then
  echo "error: $html not found. Run ./build.sh first." >&2
  exit 1
fi

# The URL slug is the paper's filename without extension.
slug="$(basename "$html" .html)"
index="$here/index.html"

if [ ! -f "$index" ]; then
  echo "error: $index not found. Run ./build.sh first." >&2
  exit 1
fi

# Assemble the served tree:
#   index.html at the root serves as the site index / table of contents;
#   clean-URL hosting maps /<slug> -> <slug>/index.html for each paper.
rm -rf "$dist" "$zip_out"
mkdir -p "$dist/$slug"
cp "$index" "$dist/index.html"
cp "$html" "$dist/$slug/index.html"

# Zip the CONTENTS of dist/ so the archive root is the site root, not dist/.
if command -v zip >/dev/null 2>&1; then
  ( cd "$dist" && zip -r -q "$zip_out" . )
else
  ( cd "$dist" && python3 -c 'import os,sys,zipfile
out=sys.argv[1]
with zipfile.ZipFile(out,"w",zipfile.ZIP_DEFLATED) as z:
    for root,_,files in os.walk("."):
        for name in files:
            p=os.path.join(root,name)
            z.write(p, os.path.relpath(p,"."))' "$zip_out" )
fi

echo "wrote $zip_out"
echo "contents:"
python3 -c 'import sys,zipfile
for n in zipfile.ZipFile(sys.argv[1]).namelist(): print("  "+n)' "$zip_out"
