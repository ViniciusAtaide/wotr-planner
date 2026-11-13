#!/usr/bin/env bash
# Builds the app as a static site, the folder GitHub Pages serves (.github/workflows/pages.yml runs it too).
#
#   [SITE_URL=https://<user>.github.io/<repo>/] scripts/build-site.sh <out-dir>
#
# The site is the release server's own output, saved as files: it seeds a new database in a temporary folder from
# libs/game-data/ (the dev database is never touched), builds with `cargo leptos build --release`, starts the release
# server on a free local port, and saves
#   - its page at `/` as index.html, server-rendered, which the wasm hydrates as it does from the server;
#   - pkg/: the wasm, JS and CSS from the site root, without .d.ts files, uncompressed (Pages compresses them);
#   - game-data/<hash>.json, the game data the page names;
#   - the files of apps/frontend/public/ (the link preview image, the font), cargo-leptos's assets-dir.
# The page names those files relative to itself (./pkg/…, ./game-data/…) and the stylesheet relative to itself
# (../fonts/…), so the folder works at `/` and under a path prefix such as a project site's `/<repo>/`. Only the
# link preview's image is named by an absolute URL, made from SITE_URL, the site's public address (the Pages
# workflow sets it); without it the page names no preview image. <out-dir> must be missing or empty.
# Needs: the toolchain in rust-toolchain.toml, cargo-leptos, curl and python3 (to find a free port). Builds with
# SQLX_OFFLINE=true, against the checked-in libs/seeds/.sqlx metadata.
set -euo pipefail

if [[ $# -ne 1 ]]; then
    sed -n '2,19p' "$0" | sed 's/^# \{0,1\}//' >&2
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
# SITE_URL as the server reads it (`ui::SiteUrl`): without surrounding spaces or trailing slashes.
site_url=$(printf '%s' "${SITE_URL:-}" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//; s#/+$##')
if [[ -z $site_url ]]; then
    echo "build-site: no SITE_URL, so the page names no link preview image" >&2
fi

echo "build-site: seeding $DATABASE_URL"
cargo run -q --release -p seeds --bin seed
echo "build-site: building the release server and wasm"
cargo leptos build --release

port=$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1])')
site=http://127.0.0.1:$port
# Its MCP server takes any free port (0), so it never holds the sessions' 3902 while the site builds.
LEPTOS_SITE_ADDR=127.0.0.1:$port WOTR_MCP_PORT=0 LEPTOS_SITE_ROOT=target/site LEPTOS_SITE_PKG_DIR=pkg \
    LEPTOS_OUTPUT_NAME=wotr-planner LEPTOS_ENV=PROD target/release/server >"$tmp/server.log" 2>&1 &
server=$!
# The server serialises the game data before it listens.
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
find "$out/pkg" -name '*.d.ts' -delete
# The assets (apps/frontend/public/), which the server serves from the site root.
cp -R apps/frontend/public/. "$out/"

# Every file the page names is in the folder.
for file in $(grep -Eo '"\./[^"]+"' "$out/index.html" | tr -d '"' | sort -u); do
    if [[ ! -f $out/$file ]]; then
        echo "build-site: the page names $file, which isn't in the site" >&2
        exit 1
    fi
done
# And every file the stylesheet names by url() (the font): relative to the stylesheet and inside the site, never
# by a path from the host's root or on another host.
css=$out/pkg/wotr-planner.css
while IFS= read -r ref; do
    case $ref in
        data:*) continue ;;
        /* | *:*)
            echo "build-site: the stylesheet names $ref, which isn't a file in the site" >&2
            exit 1
            ;;
    esac
    file=${ref%%[?#]*}
    dir=$(cd "$(dirname "$css")/$(dirname "$file")" 2>/dev/null && pwd) || dir=
    if [[ -z $dir || $dir/ != "$out"/* || ! -f $dir/$(basename "$file") ]]; then
        echo "build-site: the stylesheet names $ref, which isn't in the site" >&2
        exit 1
    fi
done < <(grep -Eo 'url\([^)]*\)' "$css" | sed -E "s/^url\([[:space:]]*['\"]?//; s/['\"]?[[:space:]]*\)$//" | sort -u)
# And so is the preview image the page names under SITE_URL.
for image in $(grep -Eo '<meta property="og:image" content="[^"]+"' "$out/index.html" | sed -E 's/.*content="([^"]+)"/\1/'); do
    file=${image#"$site_url/"}
    if [[ $file == "$image" || ! -f $out/$file ]]; then
        echo "build-site: the page's preview image $image isn't in the site" >&2
        exit 1
    fi
done

echo "build-site: the site is in $out ($(du -sh "$out" | cut -f1)):"
(cd "$out" && find . -type f -exec ls -l {} + | awk '{print "  " $5 "\t" $9}')
