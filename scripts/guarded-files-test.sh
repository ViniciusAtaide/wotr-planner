#!/usr/bin/env bash
# The test of scripts/guarded-files.sh (ticket 322), on a throwaway git repository holding a small gold.rs, the gold
# write-ups, a user-facts.txt and a ticket file: a staged gold change fails without a `User approved:` line and passes
# with one, a change outside the guarded regions passes, a deleted known gap passes and an added one fails, a test
# switched off fails, a `held` line's change fails and another line's passes, and the same holds for a committed
# range. Last, it runs itself under a decoy hook environment and checks the decoy repository is untouched.
#
#   scripts/guarded-files-test.sh
#
# Prints one line a case and OK, or the failing case and the script's output, and exits 1. Run by
# scripts/queue-checks.sh, CI, and the pre-commit hook when the script or this test is staged.
set -euo pipefail

# A hook runs with git's variables pointing at the repository being committed (in a linked worktree, at that
# worktree): every git below would follow them, not `-C`, and commit, reset and delete there. They are dropped first.
unset GIT_DIR GIT_INDEX_FILE GIT_WORK_TREE GIT_PREFIX GIT_OBJECT_DIRECTORY GIT_COMMON_DIR \
    GIT_ALTERNATE_OBJECT_DIRECTORIES

here="$(cd "$(dirname "$0")" && pwd)"
script="$here/guarded-files.sh"
self="$here/$(basename "$0")"
repo=$(mktemp -d)
decoy=
trap 'rm -rf "$repo" ${decoy:+"$decoy"}' EXIT

git_in() {
    local dir=$1
    shift
    git -C "$dir" -c user.name=test -c user.email=test@example.invalid -c commit.gpgsign=false "$@"
}

git_() {
    git_in "$repo" "$@"
}

gold=libs/seeds/tests/game_data/gold.rs
ron=libs/seeds/tests/game_data/gold/jack.ron
source=libs/seeds/tests/game_data/gold/source.txt
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

fn compare(gold: &Gold) -> Vec<Disagreement> {
    gold.lines.iter().filter(|line| !line.agrees()).collect()
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
    gap("jack", 5, "Fly", Why::Ticket(4)),
];
EOF
put "$ron" <<'EOF'
(levels: [(level: 1, items: ["Spell Penetration"])])
EOF
put "$source" <<'EOF'
1	Feat	Spell Penetration
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

# Replaces each line of $1 holding $2 with $3 (fixed strings; `\n` in $3 makes two lines), and stages it.
change() {
    local file=$1 from=$2 to=$3 tmp
    tmp=$(mktemp)
    awk -v from="$from" -v to="$to" 'index($0, from) { $0 = to } { print }' "$repo/$file" >"$tmp"
    mv "$tmp" "$repo/$file"
    git_ add "$file"
}

# Deletes each line of $1 holding $2 (a fixed string), and stages it.
drop() {
    local file=$1 from=$2 tmp
    tmp=$(mktemp)
    awk -v from="$from" '!index($0, from)' "$repo/$file" >"$tmp"
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
expect 1 "a known gap changed"
reset

change "$gold" 'Why::Ticket(4)' '    gap("jack", 5, "Fly", Why::Ticket(4)),\n    gap("jack", 7, "Haste", Why::Ticket(5)),'
expect 1 "a known gap added"
reset

drop "$gold" 'Why::Ticket(4)'
expect 0 "a known gap deleted (its fix)"
reset

change "$gold" '        1' '        2'
expect 1 "an expected number inside a multi-line assert"
reset

change "$gold" '!line.agrees()' '    gold.lines.iter().filter(|_| false).collect()'
expect 1 "the comparison weakened"
reset

change "$gold" '#[test]' '#[test]\n#[ignore = "switched off"]'
expect 1 "a test ignored"
reset

drop "$gold" '#[test]'
expect 1 "a test's #[test] removed"
reset

change "$ron" 'Spell Penetration' '(levels: [(level: 1, items: ["Toughness"])])'
expect 1 "a gold conversion (.ron) changed"
reset

change "$source" 'Spell Penetration' '1	Feat	Toughness'
expect 1 "a gold write-up (source.txt) changed"
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

# Under a hook's environment pointing at a decoy repository, the test passes and leaves the decoy as it was: its
# commits, branches, index and files.
if [[ -z ${GUARDED_FILES_TEST_INNER:-} ]]; then
    decoy=$(mktemp -d)
    echo kept >"$decoy/kept.txt"
    git_in "$decoy" init -q
    git_in "$decoy" add kept.txt
    git_in "$decoy" commit -q -m decoy
    state() {
        git -C "$decoy" for-each-ref --format='%(refname) %(objectname)'
        git -C "$decoy" status --porcelain --untracked-files=all
        git -C "$decoy" ls-files --stage
        ls -A "$decoy"
    }
    before=$(state)
    inner=0
    GUARDED_FILES_TEST_INNER=1 GIT_DIR="$decoy/.git" GIT_INDEX_FILE="$decoy/.git/index" GIT_WORK_TREE="$decoy" \
        GIT_COMMON_DIR="$decoy/.git" GIT_OBJECT_DIRECTORY="$decoy/.git/objects" GIT_PREFIX= \
        "$self" >/dev/null 2>&1 || inner=$?
    after=$(state)
    if [[ $inner == 0 && $before == "$after" ]]; then
        echo "ok: under a hook's git variables, the decoy repository untouched"
    else
        echo "FAILED: under a hook's git variables: exit $inner, decoy changed:" >&2
        diff <(echo "$before") <(echo "$after") | sed 's/^/    /' >&2 || true
        failures=$((failures + 1))
    fi
fi

if ((failures > 0)); then
    echo "guarded-files-test: $failures failed" >&2
    exit 1
fi
echo OK
