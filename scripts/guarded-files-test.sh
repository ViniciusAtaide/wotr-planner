#!/usr/bin/env bash
# The test of scripts/guarded-files.sh (ticket 322), on a throwaway git repository holding a small gold.rs, a
# user-facts.txt and a ticket file: a staged gold change fails without a `User approved:` line and passes with one, a
# change outside the guarded regions passes, a `held` line's change fails and another line's passes, and the same
# holds for a committed range.
#
#   scripts/guarded-files-test.sh
#
# Prints one line a case and OK, or the failing case and the script's output, and exits 1. Run by
# scripts/queue-checks.sh, CI, and the pre-commit hook when the script or this test is staged.
set -euo pipefail

script="$(cd "$(dirname "$0")" && pwd)/guarded-files.sh"
repo=$(mktemp -d)
trap 'rm -rf "$repo"' EXIT

git_() {
    git -C "$repo" -c user.name=test -c user.email=test@example.invalid -c commit.gpgsign=false "$@"
}

gold=libs/seeds/tests/game_data/gold.rs
facts=tools/game-check/user-facts.txt
ticket=.scratch/wotr-planner/issues/999-test.md

# Writes $1 (a path in the repository) from stdin.
put() {
    mkdir -p "$(dirname "$repo/$1")"
    cat >"$repo/$1"
}

put "$gold" <<'EOF'
//! The gold standard.

fn jack() -> Build {
    let mut build = Build::default();
    // Game: a citation
    build.picks.extend([(feat_slot(1), feat("spell-penetration"))]);
    build
}

fn helper() -> u8 {
    41
}

#[test]
fn a_number() {
    let x = helper();
    assert_eq!(
        x,
        1
    );
}

const KNOWN_GAPS: &[Gap] = &[
    gap("jack", 3, "Wings", Why::Ticket(1)),
];
EOF
put "$facts" <<'EOF'
# The user's facts.
#   <area> / <kind> -> held
class tables / class skills differ (the user's facts) -> held
class tables / class skills differ (the user's facts) / archetype a/b -> applied 262
EOF
put "$ticket" <<'EOF'
# Ticket 999
EOF
git_ init -q
git_ add -A
git_ commit -q -m base
base=$(git_ rev-parse HEAD)

failures=0
# Runs the script in the repository with the arguments, and checks its exit status is $1.
expect() {
    local want=$1 name=$2
    shift 2
    local got=0 output
    output=$(cd "$repo" && "$script" "$@" 2>&1) || got=$?
    # A failure is the guard's, naming the lines, not an error of the script's.
    if [[ $got == 1 && $output != *"need the user's approval"* ]]; then
        got="1 without the guard's message"
    fi
    if [[ $got == "$want" ]]; then
        echo "ok: $name"
    else
        echo "FAILED: $name: exit $got, not $want" >&2
        echo "$output" | sed 's/^/    /' >&2
        failures=$((failures + 1))
    fi
}

# Replaces the line of $1 holding $2 with $3 (fixed strings), and stages it.
change() {
    local file=$1 from=$2 to=$3 tmp
    tmp=$(mktemp)
    awk -v from="$from" -v to="$to" 'index($0, from) { $0 = to } { print }' "$repo/$file" >"$tmp"
    mv "$tmp" "$repo/$file"
    git_ add "$file"
}

reset() {
    git_ reset -q --hard "$base"
}

expect 0 "nothing staged"

change "$gold" 'feat("spell-penetration")' '    build.picks.extend([(feat_slot(1), feat("toughness"))]);'
expect 1 "a gold pick, staged, without approval"
echo 'User approved: the pick (test, 2026-11-17).' >>"$repo/$ticket"
git_ add "$ticket"
expect 0 "a gold pick, staged, with approval"
reset

echo '- User approved: not at the line start' >>"$repo/$ticket"
git_ add "$ticket"
change "$gold" 'feat("spell-penetration")' '    build.picks.extend([(feat_slot(1), feat("toughness"))]);'
expect 1 "an approval not at the line's start"
reset

change "$gold" 'Why::Ticket(1)' '    gap("jack", 3, "Wings", Why::Ticket(2)),'
expect 1 "a known gap"
reset

change "$gold" '        1' '        2'
expect 1 "an expected number inside a multi-line assert"
reset

change "$gold" '// Game: a citation' '    // Game: another citation'
change "$gold" '    41' '    42'
expect 0 "a comment of a gold Build and a line outside the guarded regions"
reset

change "$gold" 'feat("spell-penetration")' '    build.picks.extend([(feat_slot(1), feat("spell-penetration"))]); // Game: a citation'
expect 0 "a citation added to a gold pick's line"
reset

change "$facts" 'archetype a/b -> applied 262' 'class tables / class skills differ (the user'"'"'s facts) / archetype a/b -> applied 263'
expect 0 "a user fact that isn't held"
reset

change "$facts" 'differ (the user'"'"'s facts) -> held' 'class tables / class skills differ (the user'"'"'s facts) -> applied 999'
expect 1 "a held user fact"
reset

change "$gold" 'Why::Ticket(1)' '    gap("jack", 3, "Wings", Why::Ticket(2)),'
git_ commit -q -m "a gap moved"
expect 1 "a range with a gold change" "$base"
echo 'User approved: the gap (test, 2026-11-17).' >>"$repo/$ticket"
git_ add "$ticket"
git_ commit -q -m "the approval"
expect 0 "a range with a gold change and its approval" "$base"
expect 2 "a base that isn't a commit" no-such-commit
reset

if ((failures > 0)); then
    echo "guarded-files-test: $failures failed" >&2
    exit 1
fi
echo OK
