#!/usr/bin/env bash
# Fails when a class in the web app's stylesheet is used nowhere else, naming each one: a component that stops using a
# class leaves its rule dead, and nothing else notices.
#
#   scripts/css-classes.sh
#
# The classes are those in apps/frontend/style/main.css's selectors (comments, strings, url()s and declarations
# dropped). One counts as used when it is a whole token (letters, digits, `_` and `-`, so `gear` is not used by
# `gear-total`) in a tracked file other than the stylesheet, `.scratch/` and `*.md`. A class whose name the code builds
# at runtime can't be found that way: its prefix goes in `built` below.
set -euo pipefail

cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
css=apps/frontend/style/main.css

# Prefixes of classes whose names the code builds at runtime (a `format!` with a key), one a line: the prefix, then
# where the name is built. None yet: the Mythic paths' colours go by `data-path`, and the alignment wheel's classes are
# literals (libs/ui/src/picker.rs, `wheel_place`).
built=$(
    cat <<'EOF'
EOF
)

# The stylesheet on one line, without comments, strings, url()s and declaration blocks (the innermost braces), so what
# is left is selectors and at-rule preludes. A class starts with a letter, `_` or `-`, which leaves out numbers.
classes=$(
    tr '\n' ' ' <"$css" |
        sed -E -e 's#/\*([^*]|\*+[^*/])*\*+/##g' \
            -e 's/"[^"]*"//g' -e "s/'[^']*'//g" -e 's/url\([^)]*\)//g' \
            -e 's/\{[^{}]*\}/{}/g' |
        grep -oE '\.[A-Za-z_-][A-Za-z0-9_-]*' | sed 's/^\.//' | sort -u
)
tokens=$(git grep -IhoE '[A-Za-z0-9_-]+' -- . ":!$css" ':!.scratch/' ':!*.md' ':!target/' | sort -u)
unused=$(comm -23 <(echo "$classes") <(echo "$tokens"))
while read -r prefix _; do
    if [[ -n $prefix ]]; then
        unused=$(grep -v "^$prefix" <<<"$unused" || true)
    fi
done <<<"$built"

if [[ -n $unused ]]; then
    echo "Classes in $css that nothing uses (delete their rules, or list a built name's prefix in $0):" >&2
    sed 's/^/  ./' <<<"$unused" >&2
    exit 1
fi
echo "$(wc -l <<<"$classes" | tr -d ' ') classes, all used"
