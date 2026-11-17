#!/usr/bin/env bash
# Guarded files need the user (ticket 322; CODING_STANDARDS.md, "An expectation moves only toward the game"): fails
# when the diff changes an expected value of the gold Builds or a `held` line of the user's facts, unless a ticket file
# in the same diff adds a line starting `User approved:`.
#
#   scripts/guarded-files.sh           the staged diff (the pre-commit hook)
#   scripts/guarded-files.sh <base>    <base>..HEAD (scripts/queue-checks.sh with the queue's base; CI with the merge
#                                      base of the default branch)
#
# Run in the repository to check (it reads the git repository of the current directory). Guarded, each a line the
# diff removes or adds (`git diff -U0`), on its own side of the diff:
# - libs/seeds/tests/game_data/gold.rs, a line that isn't blank or a `//` comment, inside
#   - `fn jack()` or `fn enforcer()`, from that line to the next line `}` at the line's start: the gold Builds' picks,
#     scores, skills and Mythic rank levels;
#   - `const KNOWN_GAPS`, `const NAMES` or `const VALUES`, from that line to the next line `];` at its start: the known
#     gaps, and the planner names each write-up name expects;
#   - an `assert!(`, `assert_eq!(` or `assert_ne!(`, from that line to the first line ending `);`: the tests' expected
#     values and numbers.
#   A removed and an added line whose code is the same, a trailing `//` comment aside, are no change (a citation added
#   to a pick, a line moved).
# - tools/game-check/user-facts.txt: a line ending `-> held`, a trailing `#` comment aside.
# The approval: a line the diff adds under .scratch/wotr-planner/issues/ that starts `User approved:` (who decided,
# when, and what).
#
# Exit status: 0 when nothing guarded changed or the user approved it, 1 when a guarded change has no approval (each
# is printed), 2 when the base isn't a commit.
set -euo pipefail

gold=libs/seeds/tests/game_data/gold.rs
facts=tools/game-check/user-facts.txt
issues=.scratch/wotr-planner/issues/

cd "$(git rev-parse --show-toplevel)"

if [[ $# -gt 1 ]]; then
    sed -n '6,8p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
fi
if [[ $# -eq 1 ]]; then
    base=$1
    if ! git rev-parse --verify --quiet "$base^{commit}" >/dev/null; then
        echo "guarded-files: \`$base\` is not a commit" >&2
        exit 2
    fi
    range=("$base" HEAD)
    old_rev=$base
    new_rev=HEAD
else
    range=(--cached)
    old_rev=HEAD
    new_rev= # the index: `git show :<path>`
fi

# The diff of the paths, without context, colour or an external tool.
diff_of() {
    git diff --no-color --no-ext-diff -U0 "${range[@]}" -- "$@"
}

# The file at a revision (`""` for the index), empty when it isn't there.
show() {
    git show "$1:$2" 2>/dev/null || true
}

# The guarded line numbers of gold.rs's text on stdin, one a line (the regions the header names).
guarded_lines() {
    awk '
        function mark() { if ($0 !~ /^[[:space:]]*$/ && $0 !~ /^[[:space:]]*\/\//) print NR }
        region == "fn" { mark(); if ($0 ~ /^}/) region = ""; next }
        region == "const" { mark(); if ($0 ~ /^\];/) region = ""; next }
        region == "assert" { mark(); if ($0 ~ /\);[[:space:]]*$/) region = ""; next }
        /^fn (jack|enforcer)\(\)/ { region = "fn"; mark(); next }
        /^const (KNOWN_GAPS|NAMES|VALUES):/ { region = "const"; mark(); next }
        /assert(_eq|_ne)?!\(/ { mark(); if ($0 !~ /\);[[:space:]]*$/) region = "assert"; next }
    '
}

# The lines a `git diff -U0` of one file on stdin removes or adds: `-<old line number><TAB><text>` and
# `+<new line number><TAB><text>`.
changed_lines() {
    awk '
        /^@@ / {
            split($2, old, ","); split($3, new, ",")
            o = substr(old[1], 2) + 0; n = substr(new[1], 2) + 0; hunk = 1; next
        }
        !hunk { next }
        /^-/ { print "-" o "\t" substr($0, 2); o++; next }
        /^\+/ { print "+" n "\t" substr($0, 2); n++; next }
    '
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

show "$old_rev" "$gold" | guarded_lines >"$tmp/old"
show "$new_rev" "$gold" | guarded_lines >"$tmp/new"
diff_of "$gold" | changed_lines >"$tmp/gold"
# A removed and an added guarded line whose code is the same, trailing `//` comment and spaces aside, cancel: a citation
# added to a pick, or a line moved, changes no value.
awk -F'\t' -v path="$gold" '
    FILENAME == ARGV[1] { old[$1] = 1; next }
    FILENAME == ARGV[2] { new[$1] = 1; next }
    {
        side = substr($1, 1, 1); line = substr($1, 2)
        if (!((side == "-" && line in old) || (side == "+" && line in new))) next
        code = $2; sub(/[[:space:]]*\/\/.*$/, "", code); gsub(/^[[:space:]]+|[[:space:]]+$/, "", code)
        n++; sides[n] = side; codes[n] = code; texts[n] = "  " path " " $0
    }
    END {
        for (i = 1; i <= n; i++) {
            if (sides[i] != "-") continue
            for (j = 1; j <= n; j++) {
                if (sides[j] == "+" && !gone[j] && codes[j] == codes[i]) { gone[i] = gone[j] = 1; break }
            }
        }
        for (i = 1; i <= n; i++) if (!gone[i]) print texts[i]
    }
' "$tmp/old" "$tmp/new" "$tmp/gold" >"$tmp/hits"

diff_of "$facts" | changed_lines | awk -F'\t' -v path="$facts" '
    {
        text = $2; sub(/#.*/, "", text)
        if (text ~ /->[[:space:]]*held[[:space:]]*$/) print "  " path " " $0
    }
' >>"$tmp/hits"

if [[ ! -s $tmp/hits ]]; then
    exit 0
fi
# Not `grep -q` on the pipe: it would stop reading, and `pipefail` would take git's SIGPIPE for no match.
approvals=$(diff_of "$issues" | grep '^+User approved:' || true)
if [[ -n $approvals ]]; then
    echo "guarded-files: $(wc -l <"$tmp/hits" | tr -d ' ') guarded lines changed, approved:" >&2
    echo "$approvals" | sed 's/^+/  /' >&2
    exit 0
fi
{
    echo "guarded-files: these lines change an expected value or a held user fact, and need the user's approval"
    echo "(CODING_STANDARDS.md, \"An expectation moves only toward the game\"):"
    cat "$tmp/hits"
    echo "Ask the user; with their answer, add a line starting \`User approved:\` (who, when, what) to the ticket's file"
    echo "under $issues in the same diff."
} >&2
exit 1
