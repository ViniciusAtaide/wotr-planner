#!/usr/bin/env bash
# Every `<crate>/tests/<dir>/*.rs` other than `main.rs` is declared in that folder's `main.rs` (`mod <file>;`), since
# cargo compiles only the declared modules and a test in an undeclared file never runs (CLAUDE.md, "Rust tests").
#
#   scripts/test-modules.sh
#
# Checks `libs/*/tests`, `tools/*/tests` and `apps/*/tests`; prints each undeclared file and exits 1, else prints OK.
set -euo pipefail

cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"

missing=0
for main in libs/*/tests/*/main.rs tools/*/tests/*/main.rs apps/*/tests/*/main.rs; do
    [[ -f $main ]] || continue
    dir=$(dirname "$main")
    for file in "$dir"/*.rs; do
        name=$(basename "$file" .rs)
        [[ $name == main ]] && continue
        if ! grep -qE "^[[:space:]]*(pub(\([a-z]+\))? )?mod $name;" "$main"; then
            echo "$file is not declared in $main (add \`mod $name;\`)" >&2
            missing=1
        fi
    done
done
if [[ $missing -ne 0 ]]; then
    exit 1
fi
echo "test-modules OK"
