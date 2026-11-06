#!/usr/bin/env bash
# The behaviour proof: records about 2,000 Builds on two sides and compares them (tools/proof's docs say more).
#
#   scripts/proof.sh data  <out-dir> [<ref>] [-- <record options>]
#   scripts/proof.sh rules <out-dir> [<ref>] [-- <record options>]
#
# data:  a game-data change. The working tree's harness records <ref>'s libs/game-data (side A, default HEAD) and
#        the working tree's (side B).
# rules: a rules change. The working tree's harness, compiled against <ref>'s crates (side A, default HEAD) and against
#        the working tree's (side B), records the working tree's libs/game-data. Both record the Builds the working
#        tree's `export` lists (links.tsv), so they record the same Builds.
#
# Record options: --only <name substring>, --threads N (default 8). A full record takes about 4 minutes on 8 threads.
# Every build goes to one folder, $CARGO_TARGET_DIR (default ${TMPDIR:-/tmp}/wotr-planner-proof-target), outside the
# repo. <out-dir> must be outside the repo too; it gets A.jsonl, B.jsonl, compare.txt (empty when the sides agree)
# and the logs. Exits 1 when the comparison isn't empty.
set -euo pipefail

usage() {
    sed -n '2,16p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
}

mode=${1:-}
out=${2:-}
[[ ($mode == data || $mode == rules) && -n $out ]] || usage
shift 2
ref=HEAD
if [[ $# -gt 0 && $1 != -- ]]; then
    ref=$1
    shift
fi
if [[ $# -gt 0 ]]; then
    [[ $1 == -- ]] || usage
    shift
fi
options=("$@")

repo=$(cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)" && pwd -P)
outside_repo() {
    case "$1/" in
    "$repo"/*) echo "$2 must be outside the repo: $1" >&2 && exit 2 ;;
    esac
}
mkdir -p "$out"
out=$(cd "$out" && pwd -P)
outside_repo "$out" "the out dir"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-${TMPDIR:-/tmp}/wotr-planner-proof-target}
mkdir -p "$CARGO_TARGET_DIR"
CARGO_TARGET_DIR=$(cd "$CARGO_TARGET_DIR" && pwd -P)
outside_repo "$CARGO_TARGET_DIR" "CARGO_TARGET_DIR"
# The checked-in query data, so a build never needs the database.
export SQLX_OFFLINE=true
commit=$(git -C "$repo" rev-parse --verify "$ref^{commit}")
echo "proof: $mode, A = $ref ($commit), B = the working tree; outputs in $out" >&2

# Builds the harness of the workspace at $1 (in it, so its toolchain file applies) and copies it to $2.
build() {
    echo "proof: building the harness in $1" >&2
    (cd "$1" && cargo build --release --quiet -p proof)
    cp "$CARGO_TARGET_DIR/release/proof" "$2"
}

# Records side $1 with the harness $2: `proof record <the rest>`, timed into $1.err.
record() {
    local side=$1 bin=$2
    shift 2
    echo "proof: recording side $side" >&2
    "$bin" record "$@" ${options[@]+"${options[@]}"} >"$out/$side.jsonl" 2>"$out/$side.err"
    tail -n 2 "$out/$side.err" >&2
}

case $mode in
data)
    build "$repo" "$out/proof"
    rm -rf "$out/data-A"
    mkdir -p "$out/data-A"
    git -C "$repo" archive "$commit" libs/game-data | tar -x -C "$out/data-A"
    record A "$out/proof" "$out/data-A/libs/game-data"
    record B "$out/proof" "$repo/libs/game-data"
    bin=$out/proof
    ;;
rules)
    build "$repo" "$out/proof-B"
    echo "proof: exporting the working tree's Builds" >&2
    "$out/proof-B" export "$repo/libs/game-data" ${options[@]+"${options[@]}"} >"$out/links.tsv" 2>"$out/export.err"
    tail -n 1 "$out/export.err" >&2
    # The ref's crates with the working tree's harness, so both sides fingerprint alike.
    src=$out/src-A
    rm -rf "$src"
    mkdir -p "$src"
    git -C "$repo" archive "$commit" | tar -x -C "$src"
    # A ref from before the harness moved to tools/ lists it as libs/proof, and one from before ticket 73 not at all.
    rm -rf "$src/libs/proof" "$src/tools/proof"
    mkdir -p "$src/tools"
    cp -R "$repo/tools/proof" "$src/tools/proof"
    perl -pi -e 's{"libs/proof"}{"tools/proof"}' "$src/Cargo.toml"
    grep -q '"tools/proof"' "$src/Cargo.toml" || perl -pi -e 's/^members = \[/members = ["tools\/proof", /' "$src/Cargo.toml"
    build "$src" "$out/proof-A"
    record A "$out/proof-A" "$repo/libs/game-data" --links "$out/links.tsv"
    record B "$out/proof-B" "$repo/libs/game-data" --links "$out/links.tsv"
    bin=$out/proof-B
    ;;
esac

status=0
"$bin" compare "$out/A.jsonl" "$out/B.jsonl" >"$out/compare.txt" 2>"$out/compare.err" || status=$?
cat "$out/compare.err" >&2
head -n 20 "$out/compare.txt" >&2
exit "$status"
