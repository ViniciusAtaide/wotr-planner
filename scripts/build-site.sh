#!/usr/bin/env bash
# Builds the app as a static site, the folder GitHub Pages serves (.github/workflows/pages.yml runs it too).
#
#   scripts/build-site.sh <out-dir>
#
# The site is the release server's own output, saved as files: it seeds a new database in a temporary folder from
# libs/game-data/ (the dev database is never touched), builds with `cargo leptos build --release`, starts the release
# server on a free local port, and saves
#   - its page at `/` as index.html, server-rendered, which the wasm hydrates as it does from the server;
#   - pkg/: the wasm, JS and CSS from the site root, without .br/.gz copies (Pages compresses by itself) or .d.ts;
#   - game-data/<hash>.json, the game data the page names.
# The page names those files relative to itself (./pkg/…, ./game-data/…), so the folder works at `/` and under a
# path prefix such as a project site's `/<repo>/`. <out-dir> must be missing or empty.
# Needs: the toolchain in rust-toolchain.toml, cargo-leptos, curl and python3 (to find a free port). Builds with
# SQLX_OFFLINE=true, against the checked-in libs/seeds/.sqlx metadata.
set -euo pipefail

if [[ $# -ne 1 ]]; then
    sed -n '2,15p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
fi
mkdir -p "$1"
out=$(cd "$1" && pwd)
if [[ -n $(ls -A "$out") ]]; then
    echo "build-site: $out isn't empty" >&2
    exit 1
fi

cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
tmp=$(mktemp -d)
server=
cleanup() {
    if [[ -n $server ]]; then
        kill "$server" 2>/dev/null || true
        wait "$server" 2>/dev/null || true
    fi
    rm -rf "$tmp"
}
trap cleanup EXIT

export SQLX_OFFLINE=true
export DATABASE_URL=sqlite://$tmp/site.db
unset LEPTOS_WATCH GAME_DATA_DIR

echo "build-site: seeding $DATABASE_URL"
cargo run -q --release -p seeds --bin seed
echo "build-site: building the release server and wasm"
cargo leptos build --release

port=$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
site=http://127.0.0.1:$port
LEPTOS_SITE_ADDR=127.0.0.1:$port LEPTOS_SITE_ROOT=target/site LEPTOS_SITE_PKG_DIR=pkg \
    LEPTOS_OUTPUT_NAME=wotr-planner LEPTOS_ENV=PROD target/release/server >"$tmp/server.log" 2>&1 &
server=$!
# The server compresses the game data before it listens: about 2 s in a release build.
for _ in $(seq 120); do
    curl -fsS -o "$out/index.html" "$site/" 2>/dev/null && break
    if ! kill -0 "$server" 2>/dev/null; then
        cat "$tmp/server.log" >&2
        echo "build-site: the server stopped" >&2
        exit 1
    fi
    sleep 0.5
done
if [[ ! -s $out/index.html ]]; then
    cat "$tmp/server.log" >&2
    echo "build-site: the server didn't answer at $site" >&2
    exit 1
fi

if grep -Eq '(href|src)="/' "$out/index.html"; then
    echo "build-site: the page names a file by an absolute path, which a project site's prefix would break:" >&2
    grep -Eo '(href|src)="/[^"]*"' "$out/index.html" >&2
    exit 1
fi

game_data=$(grep -Eo 'href="\./game-data/[0-9a-f]+\.json"' "$out/index.html" | sed -E 's/^href="\.\/(.*)"$/\1/')
if [[ -z $game_data ]]; then
    echo "build-site: the page names no game data" >&2
    exit 1
fi
mkdir -p "$out/game-data"
curl -fsS -o "$out/$game_data" "$site/$game_data"

cp -R target/site/pkg "$out/pkg"
# wasm-bindgen's TypeScript declarations aren't served either.
find "$out/pkg" \( -name '*.br' -o -name '*.gz' -o -name '*.d.ts' \) -delete

# Every file the page names is in the folder.
for file in $(grep -Eo '"\./[^"]+"' "$out/index.html" | tr -d '"' | sort -u); do
    if [[ ! -f $out/$file ]]; then
        echo "build-site: the page names $file, which isn't in the site" >&2
        exit 1
    fi
done

echo "build-site: the site is in $out ($(du -sh "$out" | cut -f1)):"
(cd "$out" && find . -type f -exec ls -l {} + | awk '{print "  " $5 "\t" $9}')
