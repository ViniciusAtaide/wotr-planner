#!/usr/bin/env bash
# Renders the saved guide pages to plain text, so a fact is one grep away instead of a 380 KB HTML page to parse.
#
#   scripts/guide-text.sh [<page>...]
#
# Reads guide-cache/<page>.html (saved with scripts/save-guide-pages.sh; every page, or only those named) and writes
# guide-cache/txt/<page>.txt plus guide-cache/txt/INDEX.txt (`<page>#<anchor>\t<heading>\t<file>:<line>`, one line per
# section of every rendered page). Headings read `## Invulnerable Rager  [barbarian#invulnerable-rager]`, tables are
# one line per row with cells split by ` | `. The format is described in scripts/guide-text.py's header.
# Prints one line per page with its section and table counts. Exits non-zero if guide-cache/ has no pages.
# Run it after saving or refreshing the pages; it is idempotent, and guide-cache/ is git-ignored.
set -euo pipefail

cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
exec python3 -I scripts/guide-text.py "$@"
