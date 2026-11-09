#!/usr/bin/env bash
# Starts the MCP server `.mcp.json` connects to (http://localhost:3902/mcp) in the background, when nothing listens
# on that port yet. Claude Code runs it as the project's SessionStart hook (`.claude/settings.json`, ticket 131), so
# the client finds the server when the session starts; it never redials a project server mid-session.
#
#   scripts/start-mcp.sh
#
# - Something already listens on the port (the app's server under `cargo leptos watch`, or an earlier wotr-mcp):
#   it does nothing.
# - Otherwise it starts `target/release/wotr-mcp` with WOTR_MCP_PORT set, detached, logging to
#   `target/wotr-mcp.log`, and waits up to about 5 s for it to listen (the game data is read at start).
# - No binary: it prints a hint to build it (`scripts/build-mcp.sh`) and still exits 0, so the session starts.
#
# A lock directory keeps two sessions starting at once from starting two servers. The port is WOTR_MCP_PORT when
# set, else 3902. It always exits 0: a hook that fails would only add noise to the session.
set -uo pipefail

port=${WOTR_MCP_PORT:-3902}
root=$(cd "$(git -C "$(dirname "$0")" rev-parse --show-toplevel)" && pwd -P) || exit 0
binary=$root/target/release/wotr-mcp
log=$root/target/wotr-mcp.log

listening() {
    nc -z 127.0.0.1 "$port" >/dev/null 2>&1
}

if listening; then
    echo "wotr-mcp: something already listens on port $port; not starting another"
    exit 0
fi
if [[ ! -x $binary ]]; then
    echo "wotr-mcp: $binary isn't built, so the wotr-planner MCP server isn't running; build it with scripts/build-mcp.sh (the next session starts it)"
    exit 0
fi

# One starter at a time, machine-wide for this port: mkdir is atomic. A lock older than a minute is a starter that
# died, and is taken over.
lock=${TMPDIR:-/tmp}/wotr-mcp-$port.lock
if ! mkdir "$lock" 2>/dev/null; then
    if [[ -n $(find "$lock" -maxdepth 0 -mmin +1 2>/dev/null) ]]; then
        rm -rf "$lock" && mkdir "$lock" 2>/dev/null || exit 0
    else
        echo "wotr-mcp: another session is starting the server on port $port"
        exit 0
    fi
fi
trap 'rmdir "$lock" 2>/dev/null' EXIT

# Checked again under the lock: another session may have started it meanwhile.
if listening; then
    echo "wotr-mcp: something already listens on port $port; not starting another"
    exit 0
fi

mkdir -p "$(dirname "$log")"
(cd "$root" && WOTR_MCP_PORT=$port nohup "$binary" </dev/null >>"$log" 2>&1 &)
for _ in $(seq 50); do
    if listening; then
        echo "wotr-mcp: started $binary on port $port (log: $log)"
        exit 0
    fi
    sleep 0.1
done
echo "wotr-mcp: started $binary on port $port, but it isn't listening yet; see $log"
exit 0
