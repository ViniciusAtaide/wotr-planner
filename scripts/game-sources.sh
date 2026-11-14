#!/usr/bin/env bash
# Fills blueprints/ and decompiled/ at this checkout's root with the game's two sources, at the pins in
# scripts/game-sources.pins (ticket 236). A missing or empty folder is fetched; a filled one is checked against the
# pin and reported current, or refused (non-zero exit) with both values named. Both folders are git-ignored: the repo
# never holds the game's dump or code.
set -euo pipefail

# A git transfer that stalls (under 1 kB/s for a minute) fails instead of hanging.
export GIT_HTTP_LOW_SPEED_LIMIT=1000 GIT_HTTP_LOW_SPEED_TIME=60

root=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
# shellcheck source=game-sources.pins
. "$root/scripts/game-sources.pins"

sha256() {
  if command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | cut -d' ' -f1; else sha256sum "$1" | cut -d' ' -f1; fi
}

is_empty() { [ ! -d "$1" ] || [ -z "$(ls -A "$1")" ]; }

blueprints() {
  local dir="$root/blueprints" file="$root/blueprints/$BLUEPRINTS_ASSET" got
  if is_empty "$dir"; then
    mkdir -p "$dir"
    # The download lands beside the folder and moves in only once its hash matches; a failed or interrupted
    # download leaves nothing behind.
    part=$(mktemp -d "$root/blueprints.part.XXXXXX")
    trap 'rm -rf "$part"' EXIT
    echo "blueprints: fetching $BLUEPRINTS_REPO $BLUEPRINTS_RELEASE ($BLUEPRINTS_BYTES bytes)"
    if command -v gh >/dev/null 2>&1; then
      gh release download "$BLUEPRINTS_RELEASE" -R "$BLUEPRINTS_REPO" -p "$BLUEPRINTS_ASSET" -D "$part"
    else
      curl -fL -o "$part/$BLUEPRINTS_ASSET" \
        "https://github.com/$BLUEPRINTS_REPO/releases/download/$BLUEPRINTS_RELEASE/$BLUEPRINTS_ASSET"
    fi
    got=$(sha256 "$part/$BLUEPRINTS_ASSET")
    if [ "$got" != "$BLUEPRINTS_SHA256" ]; then
      rm -rf "$part"
      echo "blueprints: REFUSED, the download's SHA-256 is $got, the pin is $BLUEPRINTS_SHA256" >&2
      return 1
    fi
    mv "$part/$BLUEPRINTS_ASSET" "$file"
    rm -rf "$part"
    echo "blueprints: fetched, SHA-256 $got"
    return 0
  fi
  if [ ! -f "$file" ]; then
    echo "blueprints: REFUSED, $dir holds no $BLUEPRINTS_ASSET (the pin); empty it or fix it by hand" >&2
    return 1
  fi
  got=$(sha256 "$file")
  if [ "$got" != "$BLUEPRINTS_SHA256" ]; then
    echo "blueprints: REFUSED, $BLUEPRINTS_ASSET has SHA-256 $got, the pin is $BLUEPRINTS_SHA256" >&2
    return 1
  fi
  echo "blueprints: current ($BLUEPRINTS_RELEASE, SHA-256 $got)"
}

decompiled() {
  local dir="$root/decompiled" head
  if is_empty "$dir"; then
    mkdir -p "$dir"
    echo "decompiled: fetching $DECOMPILED_REPO at $DECOMPILED_COMMIT (shallow)"
    if ! { git -C "$dir" init -q \
      && git -C "$dir" remote add origin "https://github.com/$DECOMPILED_REPO.git" \
      && git -C "$dir" fetch -q --depth 1 origin "$DECOMPILED_COMMIT" \
      && git -C "$dir" checkout -q FETCH_HEAD; }; then
      rm -rf "$dir"
      echo "decompiled: REFUSED, could not fetch $DECOMPILED_COMMIT from $DECOMPILED_REPO" >&2
      return 1
    fi
  fi
  head=$(git -C "$dir" rev-parse HEAD 2>/dev/null || echo "none")
  if [ "$head" != "$DECOMPILED_COMMIT" ]; then
    echo "decompiled: REFUSED, HEAD is $head, the pin is $DECOMPILED_COMMIT" >&2
    return 1
  fi
  echo "decompiled: current ($DECOMPILED_REPO at $head)"
}

status=0
blueprints || status=1
decompiled || status=1
exit $status
