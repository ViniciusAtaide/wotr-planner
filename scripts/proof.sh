#!/usr/bin/env bash
# The behaviour proof: records about 2,000 Builds on two sides and compares them (tools/proof's docs say more).
#
#   scripts/proof.sh data  <out-dir> [<ref>] [-- <record options>]
#   scripts/proof.sh rules <out-dir> [<ref>] [-- <record options>]
#   scripts/proof.sh both  <out-dir> [<ref>] [-- <record options>]
#
# data:  a game-data change. The working tree's harness records <ref>'s libs/game-data (side A, default HEAD) and
#        the working tree's (side B).
# rules: a rules change. The working tree's harness, compiled against <ref>'s crates (side A, default HEAD) and against
#        the working tree's (side B), records the working tree's libs/game-data. Both record the Builds the working
#        tree's `export` lists (links.tsv), so they record the same Builds.
# both:  a change to the rules and the game data together, which may change the link layout too (ticket 86): as
#        rules, but side A records <ref>'s libs/game-data, which its crates can read, and both record the working
#        tree's Builds as JSON (builds.tsv) rather than links. A side whose Build start has no faith drops it and
#        leaves out the deity Builds (`t86 …`, full ones too).
#
# --explain (ticket 203), a record option the script takes out before it calls `record`: when the comparison isn't empty,
#   scripts/proof.sh both <out-dir> <ref> -- --explain
# both harnesses then dump, in clear, the Builds compare.txt names (A.dump, B.dump: about 1.5 MB a Build, so a few GB for a
# proof that differs everywhere; delete them when done) and explain.txt is written beside compare.txt: its first lines
# say whether the differences come from the data, the code or both (in `both` mode a third record tells: side A's crates on
# side B's game data, or side B's on side A's where A's can't read B's), then `proof explain` groups the differing facts by path and change, largest group first,
# each with its Builds and three examples. Reach for it when compare.txt names more than a few Builds: it says what
# changed, where compare.txt says only where. `proof explain A.dump B.dump --top 0` lists every group. It adds about
# the time of a full record per side. Other record options (--only, --light, --threads, --fold-values) apply to the
# dumps too.
#
# When: once per queue of tickets, by the main session, when the queue is merged (CLAUDE.md, "Behaviour checks"). A
# ticket's own check is the snapshot check.
#
# Record options: --only <name substring>, --threads N (default 8), --light. A full record takes about 45 seconds a
# side on 8 threads, after the two release builds.
# --light (ticket 162) leaves out the `allows` battery and the consistency and shortcut checks, about a third of a
# record's cost, and keeps every other field (sheet, every slot's options, pickers, totals, caster lines): about 29
# seconds a side against 45. Reach for it between the snapshot check and the full proof, to see a change's reach over the whole corpus
# early. It misses a change that only the battery's edits or the checks see; the once-per-queue proof is a full one. A
# light record only compares with a light one (compare refuses a mixed pair).
# Every build goes under one folder, $CARGO_TARGET_DIR (default ${XDG_CACHE_HOME:-$HOME/.cache}/wotr-planner-proof-target:
# not the temp folder, whose cleanup once deleted a generated file and broke the build), outside the
# repo: B/ for the working tree's harness, A/ for the ref's. <out-dir> must be outside the repo too; it gets A.jsonl,
# B.jsonl, compare.txt (empty when the sides agree) and the logs. Exits 1 when the comparison isn't empty.
# Side A's source (rules and both) stays in $CARGO_TARGET_DIR/src-A with a stamp beside it, src-A.stamp: the ref's
# commit, a hash of tools/proof (the harness copied in) and the mode. While the stamp matches, a run reuses the source
# and cargo rebuilds none of side A's crates ("side A's source: reused"). Otherwise the source is extracted again
# (files stamped with the current time, tar -m) and A/ is deleted, so side A builds from clean: cargo once took an
# older ref's build for a newer one (ticket 157). One CARGO_TARGET_DIR holds one src-A: run one proof at a time.
set -euo pipefail

usage() {
    sed -n '2,47p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
}

mode=${1:-}
out=${2:-}
[[ ($mode == data || $mode == rules || $mode == both) && -n $out ]] || usage
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
options=()
explain=0
for option in "$@"; do
    if [[ $option == --explain ]]; then
        explain=1
    else
        options+=("$option")
    fi
done

repo=$(cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)" && pwd -P)
outside_repo() {
    case "$1/" in
    "$repo"/*) echo "$2 must be outside the repo: $1" >&2 && exit 2 ;;
    esac
}
mkdir -p "$out"
out=$(cd "$out" && pwd -P)
outside_repo "$out" "the out dir"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/wotr-planner-proof-target}
mkdir -p "$CARGO_TARGET_DIR"
CARGO_TARGET_DIR=$(cd "$CARGO_TARGET_DIR" && pwd -P)
outside_repo "$CARGO_TARGET_DIR" "CARGO_TARGET_DIR"
# Side A from before ticket 187 still compiles sqlx's query macros: against its checked-in query data, so its build
# never needs a database.
export SQLX_OFFLINE=true
commit=$(git -C "$repo" rev-parse --verify "$ref^{commit}")
echo "proof: $mode, A = $ref ($commit), B = the working tree; outputs in $out" >&2

# Builds the harness of the workspace at $1 (in it, so its toolchain file applies) in the target folder $3 and copies
# it to $2. Each side has its own target folder: cargo fingerprints a workspace's crates by their paths inside it and
# their files' times, and `git archive` gives the ref's files their commit's time, so a side built in the other's
# folder would count as up to date and be the other side's harness (found in ticket 86).
build() {
    echo "proof: building the harness in $1" >&2
    (cd "$1" && CARGO_TARGET_DIR=$3 cargo build --release --quiet -p proof)
    cp "$3/release/proof" "$2"
}

# A hash of the working tree's harness, tools/proof, as it is copied into side A's source (every file, untracked too).
harness_hash() {
    (cd "$repo/tools/proof" && find . -type f -not -path './target/*' | LC_ALL=C sort | xargs shasum -a 256 | shasum -a 256 | cut -d' ' -f1)
}

# Records side $1 with the harness $2: `proof record <the rest>`, timed into $1.err.
record() {
    local side=$1 bin=$2
    shift 2
    echo "proof: recording side $side" >&2
    "$bin" record "$@" ${options[@]+"${options[@]}"} >"$out/$side.jsonl" 2>"$out/$side.err"
    # Data that fails GameData::check is recorded anyway, with a warning (ticket 149).
    if grep '^warning:' "$out/$side.err" >&2; then
        echo "proof: side $side's data problems are listed in $out/$side.err" >&2
    fi
    tail -n 2 "$out/$side.err" >&2
}

case $mode in
data)
    build "$repo" "$out/proof" "$CARGO_TARGET_DIR/B"
    rm -rf "$out/data-A"
    mkdir -p "$out/data-A"
    git -C "$repo" archive "$commit" libs/game-data | tar -x -C "$out/data-A"
    record A "$out/proof" "$out/data-A/libs/game-data"
    record B "$out/proof" "$repo/libs/game-data"
    bin=$out/proof
    ;;
rules | both)
    build "$repo" "$out/proof-B" "$CARGO_TARGET_DIR/B"
    echo "proof: exporting the working tree's Builds" >&2
    if [[ $mode == both ]]; then
        builds=(--builds "$out/builds.tsv")
        "$out/proof-B" export "$repo/libs/game-data" --json ${options[@]+"${options[@]}"} >"$out/builds.tsv" 2>"$out/export.err"
    else
        builds=(--links "$out/links.tsv")
        "$out/proof-B" export "$repo/libs/game-data" ${options[@]+"${options[@]}"} >"$out/links.tsv" 2>"$out/export.err"
    fi
    tail -n 1 "$out/export.err" >&2
    # The ref's crates with the working tree's harness, so both sides fingerprint alike. The source lives beside
    # the build folders and is reused while its stamp (the ref's commit, the harness's hash, the mode) matches.
    src=$CARGO_TARGET_DIR/src-A
    stamp=$CARGO_TARGET_DIR/src-A.stamp
    want=$(printf 'format 1\ncommit %s\nharness %s\nmode %s\n' "$commit" "$(harness_hash)" "$mode")
    if [[ -f $stamp && -d $src && $(cat "$stamp") == "$want" ]]; then
        echo "proof: side A's source: reused ($commit)" >&2
    else
        echo "proof: side A's source: extracted ($commit)" >&2
        # No stamp while the source is half made, and no build of an older source: cargo would take it for this one.
        rm -rf "$stamp" "$src" "$CARGO_TARGET_DIR/A"
        mkdir -p "$src"
        # -m: the files take the current time, not their commit's, so cargo sees them newer than any build.
        git -C "$repo" archive "$commit" | tar -x -m -C "$src"
        # A ref from before the harness moved to tools/ lists it as libs/proof, and one from before ticket 73 not at all.
        rm -rf "$src/libs/proof" "$src/tools/proof"
        mkdir -p "$src/tools"
        cp -R "$repo/tools/proof" "$src/tools/proof"
        perl -pi -e 's{"libs/proof"}{"tools/proof"}' "$src/Cargo.toml"
        grep -q '"tools/proof"' "$src/Cargo.toml" || perl -pi -e 's/^members = \[/members = ["tools\/proof", /' "$src/Cargo.toml"
        # The harness turns on rules' `unchecked` feature (ticket 149), which a ref from before it lacks; there it does
        # nothing, since that ref's engine takes any data.
        rules_toml=$src/libs/rules/Cargo.toml
        if ! grep -q '^unchecked = ' "$rules_toml"; then
            if grep -q '^\[features\]' "$rules_toml"; then
                perl -pi -e 's/^\[features\]$/[features]\nunchecked = []/' "$rules_toml"
            else
                printf '\n[features]\nunchecked = []\n' >>"$rules_toml"
            fi
        fi
        printf '%s\n' "$want" >"$stamp"
    fi
    build "$src" "$out/proof-A" "$CARGO_TARGET_DIR/A"
    data_a=$repo/libs/game-data
    [[ $mode == both ]] && data_a=$src/libs/game-data
    record A "$out/proof-A" "$data_a" "${builds[@]}"
    record B "$out/proof-B" "$repo/libs/game-data" "${builds[@]}"
    bin=$out/proof-B
    ;;
esac

status=0
"$bin" compare "$out/A.jsonl" "$out/B.jsonl" >"$out/compare.txt" 2>"$out/compare.err" || status=$?
cat "$out/compare.err" >&2
head -n 20 "$out/compare.txt" >&2

# The Builds `compare` names, one on a line, as a list for `dump --names`: an empty comparison has none.
differing_builds() {
    awk -F'\t' '$1 == "differs" || $1 ~ /^only in / { print $2 }' "$1" | LC_ALL=C sort -u
}

# How many Builds a comparison names (compare's output at $1).
count_differing() {
    differing_builds "$1" | wc -l | tr -d ' '
}

# Where the differences come from: the first lines of explain.txt. `rules` and `data` change one thing; `both` records
# side A's crates on side B's game data (C), so A -> C is the data's effect and C -> B the code's.
where_from() {
    case $mode in
    data) echo "These differences come from the data: one harness records both sides, on the ref's game data and on the working tree's." ;;
    rules) echo "These differences come from the code: both sides record the working tree's game data, with the ref's crates and with the working tree's." ;;
    both)
        # An intermediate record X changes one thing from each side: C (A's crates on B's data) first, and where A's
        # crates can't read B's data, D (B's crates on A's data).
        local x kind ab from_a to_b data code
        echo "proof: recording side A's crates on side B's game data (C)" >&2
        if "$out/proof-A" record "$repo/libs/game-data" "${builds[@]}" ${options[@]+"${options[@]}"} >"$out/C.jsonl" 2>"$out/C.err"; then
            x=$out/C.jsonl kind=C
        else
            echo "proof: side A's crates can't read side B's game data (C.err); recording side B's crates on side A's (D)" >&2
            if ! "$out/proof-B" record "$data_a" "${builds[@]}" ${options[@]+"${options[@]}"} >"$out/D.jsonl" 2>"$out/D.err"; then
                echo "Where from: not known. Side A's crates can't record side B's game data (C.err) nor side B's side A's (D.err)."
                return
            fi
            x=$out/D.jsonl kind=D
        fi
        ab=$(count_differing "$out/compare.txt")
        "$bin" compare "$out/A.jsonl" "$x" >"$out/compare-A-$kind.txt" 2>/dev/null || true
        "$bin" compare "$x" "$out/B.jsonl" >"$out/compare-$kind-B.txt" 2>/dev/null || true
        from_a=$(count_differing "$out/compare-A-$kind.txt")
        to_b=$(count_differing "$out/compare-$kind-B.txt")
        if [[ $kind == C ]]; then
            data=$from_a code=$to_b
            echo "Where from, by three records (A: the ref's crates on the ref's data; C: the ref's crates on the working tree's data; B: the working tree's on its own):"
            echo "  A -> B differ in $ab Builds; A -> C (the data alone) in $data; C -> B (the code alone, on the new data) in $code."
        else
            code=$from_a data=$to_b
            echo "Where from, by three records (A: the ref's crates on the ref's data; D: the working tree's crates on the ref's data, since the ref's crates can't read the new data; B: the working tree's on its own):"
            echo "  A -> B differ in $ab Builds; A -> D (the code alone, on the old data) in $code; D -> B (the data alone, with the new code) in $data."
        fi
        if [[ $data -gt 0 && $code -eq 0 ]]; then
            echo "  The differences come from the data: the code changes nothing the records hold."
        elif [[ $data -eq 0 && $code -gt 0 ]]; then
            echo "  The differences come from the code: the data changes nothing the records hold."
        elif [[ $data -gt 0 && $code -gt 0 ]]; then
            echo "  The differences come from both, the data and the code (a Build may be in both counts)."
        else
            echo "  Neither alone changes a Build: the differences need the data and the code together."
        fi
        ;;
    esac
}

if [[ $explain -eq 1 && $status -ne 0 && -s $out/compare.txt ]]; then
    differing_builds "$out/compare.txt" >"$out/differing.txt"
    echo "proof: explaining $(wc -l <"$out/differing.txt" | tr -d ' ') Builds" >&2
    if [[ $mode == data ]]; then
        dump_a=("$out/proof" dump "$out/data-A/libs/game-data")
        dump_b=("$out/proof" dump "$repo/libs/game-data")
    else
        dump_a=("$out/proof-A" dump "$data_a" "${builds[@]}")
        dump_b=("$out/proof-B" dump "$repo/libs/game-data" "${builds[@]}")
    fi
    echo "proof: dumping side A" >&2
    "${dump_a[@]}" --names "$out/differing.txt" ${options[@]+"${options[@]}"} >"$out/A.dump" 2>"$out/A-dump.err"
    tail -n 1 "$out/A-dump.err" >&2
    echo "proof: dumping side B" >&2
    "${dump_b[@]}" --names "$out/differing.txt" ${options[@]+"${options[@]}"} >"$out/B.dump" 2>"$out/B-dump.err"
    tail -n 1 "$out/B-dump.err" >&2
    {
        echo "proof explain: A = $ref ($commit), B = the working tree; mode $mode"
        where_from
        echo
        "$bin" explain "$out/A.dump" "$out/B.dump"
    } >"$out/explain.txt"
    echo "proof: explain.txt written ($(du -h "$out/A.dump" "$out/B.dump" | awk '{printf "%s%s ", $1, ""}')for the two dumps, kept: delete them when done)" >&2
    head -n 12 "$out/explain.txt" | cut -c1-240 >&2
fi
exit "$status"
