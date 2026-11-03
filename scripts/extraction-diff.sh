#!/usr/bin/env bash
# Compares a fresh extraction made with <ref>'s extractor (side A, default HEAD) against one made with the working
# tree's (side B), both from guide-cache/. For a refactor of libs/extract, the two must be byte-identical, report
# included.
#
#   scripts/extraction-diff.sh <out-dir> [<ref>]
#
# Every build goes to one folder, $CARGO_TARGET_DIR (default ${TMPDIR:-/tmp}/wotr-planner-extract-target), outside
# the repo. <out-dir> must be outside the repo too; it gets A/ and B/ (the two extractions), diff.txt (empty when they
# agree) and the logs. Exits 1 when they differ. Never writes to libs/game-data/.
set -euo pipefail

usage() {
    sed -n '2,10p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
}

out=${1:-}
[[ -n $out && $# -le 2 ]] || usage
ref=${2:-HEAD}

repo=$(cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)" && pwd -P)
outside_repo() {
    case "$1/" in
    "$repo"/*) echo "$2 must be outside the repo: $1" >&2 && exit 2 ;;
    esac
}
mkdir -p "$out"
out=$(cd "$out" && pwd -P)
outside_repo "$out" "the out dir"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-${TMPDIR:-/tmp}/wotr-planner-extract-target}
mkdir -p "$CARGO_TARGET_DIR"
CARGO_TARGET_DIR=$(cd "$CARGO_TARGET_DIR" && pwd -P)
outside_repo "$CARGO_TARGET_DIR" "CARGO_TARGET_DIR"
# The checked-in query data, so a build never needs the database.
export SQLX_OFFLINE=true
guide=$repo/guide-cache
[[ -d $guide ]] || { echo "no saved guide pages in $guide" >&2 && exit 2; }
commit=$(git -C "$repo" rev-parse --verify "$ref^{commit}")
echo "extraction-diff: A = $ref ($commit), B = the working tree; outputs in $out" >&2

# Builds the extractor of the workspace at $2 and extracts into side $1.
extract() {
    local side=$1 workspace=$2
    echo "extraction-diff: building and extracting side $side" >&2
    (cd "$workspace" && cargo build --release --quiet -p extract)
    cp "$CARGO_TARGET_DIR/release/extract" "$out/extract-$side"
    rm -rf "${out:?}/$side"
    "$out/extract-$side" --guide "$guide" --out "$out/$side" >"$out/$side.log" 2>&1
    tail -n 1 "$out/$side.log" >&2
}

rm -rf "${out:?}/src-A"
mkdir -p "$out/src-A"
git -C "$repo" archive "$commit" | tar -x -C "$out/src-A"
extract A "$out/src-A"
extract B "$repo"

status=0
diff -r "$out/A" "$out/B" >"$out/diff.txt" || status=$?
if [[ $status -eq 0 ]]; then
    echo "extraction-diff: identical" >&2
else
    echo "extraction-diff: they differ ($(grep -c '^diff \|^Only in' "$out/diff.txt") files); see $out/diff.txt" >&2
    head -n 20 "$out/diff.txt" >&2
fi
exit "$status"
