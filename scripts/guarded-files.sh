#!/usr/bin/env bash
# Guarded files need the user (ticket 322; CODING_STANDARDS.md, "An expectation moves only toward the game"): fails
# when the diff changes an expected value of the gold Builds or a `held` line of the user's facts, unless a ticket file
# in the same diff adds a line starting `User approved:` that names the change (ticket 325).
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
#   - the comparison that decides a gold Build's pass, each from its line to the next line `}` at its start: `fn gold`,
#     `expected`, `feat_names`, `held_names`, `picked`, `compare`, `level_class`, `check_line`,
#     `check_bloodline_spell`, `second_bloodline_spells`, `start`, `unexplained_by`, `disagreements` and
#     `unexplained`, and `impl Gains` and `impl Covers`;
#   - `const NAMES` or `const VALUES`, from that line to the next line `];` at its start: the planner names each
#     write-up name expects;
#   - `const KNOWN_GAPS`, likewise, an added line only: a fix deletes its gap (CLAUDE.md, "Behaviour checks"), a new
#     or changed gap needs the user;
#   - an `assert!(`, `assert_eq!(` or `assert_ne!(`, from that line to the first line ending `);`: the tests' expected
#     values and numbers;
#   - an added `#[ignore` line, and a removed `#[test]` line: a test switched off.
#   A removed and an added line whose code is the same, a trailing `//` comment aside, are no change (a citation added
#   to a pick, a line moved).
# - libs/seeds/tests/game_data/gold/ (the write-ups, `source.txt`, and their conversion, `<build>.ron`): any change.
# - tools/game-check/user-facts.txt: a line ending `-> held`, a trailing `#` comment aside.
# The approval: a line the diff adds under .scratch/wotr-planner/issues/ that starts `User approved:` (who decided,
# when, and what). Each guarded line has a name, and an approval clears only the lines whose name it holds, as whole
# words, in any case (one approval in a range clears no other ticket's change):
# - a line inside a function, gold Build or comparison (`jack`, `enforcer`, `compare`), or holding an assertion, or a
#   test's `#[test]` or `#[ignore` line: that function's name;
# - a line of `impl Gains` or `impl Covers`: `Gains` or `Covers`; of `const NAMES` or `const VALUES`: `const NAMES` or
#   `const VALUES`;
# - a line of a `KNOWN_GAPS` entry: `gap <its ticket>`, the `Why`'s number (`gap 173`); the list's own lines:
#   `KNOWN_GAPS`;
# - a `held` line of the user's facts: its last ` / ` field, the fact (`gear item belt-of-giant-strength-2`);
# - a file under gold/: its file name (`enforcer.ron`).
#
# Exit status: 0 when nothing guarded changed or an approval names each change, 1 when a guarded change has none (each
# is printed with its name), 2 when the base isn't a commit.
set -euo pipefail

gold=libs/seeds/tests/game_data/gold.rs
gold_dir=libs/seeds/tests/game_data/gold/
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

# The guarded lines of gold.rs's text on stdin (the regions the header names), on side $1 (`old`, the removed lines'
# side, or `new`, the added lines'): `<line number><TAB><name>`, one a line, the name the header gives it.
guarded_lines() {
    awk -v side="$1" '
        function code() { return $0 !~ /^[[:space:]]*$/ && $0 !~ /^[[:space:]]*\/\// }
        function mark(name) { if (code()) print NR "\t" (name == "" ? "gold.rs" : name) }
        # The name a `fn` line declares.
        function declared(   s) { s = $0; sub(/^.*fn[[:space:]]+/, "", s); sub(/[^A-Za-z0-9_].*$/, "", s); return s }
        # A gap entry, from `gap(` to its closing parenthesis, named `gap <ticket>` by its `Why` once it closes.
        function gap_line(   i, opens, closes, tmp, joined) {
            if (!code()) return
            entry[++lines] = NR; text = text $0
            tmp = $0; opens = gsub(/\(/, "", tmp); tmp = $0; closes = gsub(/\)/, "", tmp); depth += opens - closes
            if (depth > 0) return
            joined = text; gsub(/[[:space:]]/, "", joined)
            name = "KNOWN_GAPS"
            if (match(joined, /Why::[A-Za-z]+\([0-9]+/)) {
                name = substr(joined, RSTART, RLENGTH); sub(/^.*\(/, "", name); name = "gap " name
            }
            for (i = 1; i <= lines; i++) print entry[i] "\t" name
            lines = 0; text = ""; depth = 0
        }
        # A `#[test]` or `#[ignore` line takes the name of the function it heads, the next `fn` line.
        /^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?((const|async|unsafe)[[:space:]]+)*fn[[:space:]]/ {
            fn = declared(); for (n in heads) print n "\t" fn; delete heads
        }
        side == "old" && /^[[:space:]]*#\[test\]/ { heads[NR] = 1 }
        side == "new" && /^[[:space:]]*#\[ignore/ { heads[NR] = 1 }
        region == "fn" || region == "const" {
            mark(region_name)
            if ((region == "fn" && $0 ~ /^}/) || (region == "const" && $0 ~ /^\];/)) region = ""
            next
        }
        region == "gaps" && /^\];/ { if (side == "new") mark("KNOWN_GAPS"); region = ""; next }
        region == "gaps" { if (side == "new") gap_line(); next }
        region == "assert" { mark(fn); if ($0 ~ /\);[[:space:]]*$/) region = ""; next }
        /^fn (jack|enforcer|gold|expected|feat_names|held_names|picked|compare|level_class|check_line|check_bloodline_spell|second_bloodline_spells|start|unexplained_by|disagreements|unexplained)[(<]/ {
            region = "fn"; region_name = fn; mark(region_name); next
        }
        /^impl(<[^>]*>)? (Gains|Covers)[<[:space:]{]/ {
            region = "fn"; region_name = ($0 ~ /Gains/ ? "Gains" : "Covers"); mark(region_name); next
        }
        /^const (NAMES|VALUES):/ { region = "const"; region_name = "const " ($0 ~ /NAMES/ ? "NAMES" : "VALUES"); mark(region_name); next }
        /^const KNOWN_GAPS:/ { region = "gaps"; if (side == "new") mark("KNOWN_GAPS"); next }
        /assert(_eq|_ne)?!\(/ { mark(fn); if ($0 !~ /\);[[:space:]]*$/) region = "assert"; next }
        END { for (n in heads) print n "\t" "gold.rs" }
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
        /^diff --git / { hunk = 0; next }
        !hunk { next }
        /^-/ { print "-" o "\t" substr($0, 2); o++; next }
        /^\+/ { print "+" n "\t" substr($0, 2); n++; next }
    '
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

show "$old_rev" "$gold" | guarded_lines old >"$tmp/old"
show "$new_rev" "$gold" | guarded_lines new >"$tmp/new"
diff_of "$gold" | changed_lines >"$tmp/gold"
# A removed and an added guarded line whose code is the same, trailing `//` comment and spaces aside, cancel: a citation
# added to a pick, or a line moved, changes no value.
# Each hit is `<name><TAB><the line as printed>`.
awk -F'\t' -v path="$gold" '
    FILENAME == ARGV[1] { old[$1] = $2; next }
    FILENAME == ARGV[2] { new[$1] = $2; next }
    {
        side = substr($1, 1, 1); line = substr($1, 2)
        if (side == "-" && line in old) name = old[line]
        else if (side == "+" && line in new) name = new[line]
        else next
        code = $2; sub(/[[:space:]]*\/\/.*$/, "", code); gsub(/^[[:space:]]+|[[:space:]]+$/, "", code)
        n++; sides[n] = side; codes[n] = code; texts[n] = name "\t  " path " " $0
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
        if (text !~ /->[[:space:]]*held[[:space:]]*$/) next
        # The fact: the last ` / ` field before the arrow.
        name = text; sub(/[[:space:]]*->.*$/, "", name); sub(/^.* \/ /, "", name); gsub(/^[[:space:]]+/, "", name)
        print name "\t  " path " " $0
    }
' >>"$tmp/hits"

git diff --no-color --name-only "${range[@]}" -- "$gold_dir" | awk '{ name = $0; sub(/^.*\//, "", name); print name "\t  " $0 " changed" }' >>"$tmp/hits"

if [[ ! -s $tmp/hits ]]; then
    exit 0
fi
# Not `grep -q` on the pipe: it would stop reading, and `pipefail` would take git's SIGPIPE for no match.
approvals=$(diff_of "$issues" | grep '^+User approved:' || true)
# The hits no approval names, the name a whole word of the approval in any case (not `grep -w`: ugrep's and GNU's
# word rules differ), each `<the line as printed> (named \`<name>\`)`.
echo "$approvals" >"$tmp/approvals"
awk -F'\t' '
    function word(c) { return c ~ /[A-Za-z0-9_]/ }
    # Whether $2 holds $1 with no word character either side, in any case.
    function holds(name, line,   at, rest, before, after) {
        rest = tolower(line); name = tolower(name)
        while ((at = index(rest, name)) > 0) {
            before = at > 1 ? substr(rest, at - 1, 1) : ""; after = substr(rest, at + length(name), 1)
            if (!word(before) && !word(after)) return 1
            rest = substr(rest, at + 1)
        }
        return 0
    }
    FILENAME == ARGV[1] { if ($0 != "") approvals[++n] = $0; next }
    {
        name = $1; shown = substr($0, length(name) + 2)
        for (i = 1; i <= n; i++) if (holds(name, approvals[i])) next
        print shown " (named `" name "`)"
    }
' "$tmp/approvals" "$tmp/hits" >"$tmp/unnamed"
if [[ ! -s $tmp/unnamed ]]; then
    echo "guarded-files: $(wc -l <"$tmp/hits" | tr -d ' ') guarded lines changed, each named by an approval:" >&2
    cut -f1 "$tmp/hits" | sort -u | sed 's/^/  `/; s/$/`/' >&2
    echo "$approvals" | sed 's/^+/  /' >&2
    exit 0
fi
{
    echo "guarded-files: these lines change an expected value or a held user fact, and need the user's approval"
    echo "(CODING_STANDARDS.md, \"An expectation moves only toward the game\"), in a \`User approved:\` line that holds"
    echo "each one's name:"
    cat "$tmp/unnamed"
    if [[ -n $approvals ]]; then
        echo "The diff's approvals, which name none of them:"
        echo "$approvals" | sed 's/^+/  /'
    fi
    echo "Ask the user; with their answer, add a line starting \`User approved:\` (who, when, what, each name) to the"
    echo "ticket's file under $issues in the same diff."
} >&2
exit 1
