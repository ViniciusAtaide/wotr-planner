# Blueprint lookups in the game check

Type: build (tools)
Status: open
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

From the queue's retro (user, 2026-11-16, "let's do 3 and 4"). Every research agent and builder of tickets 237–254 needed to read blueprints. Each one rebuilt its own lookup in a session scratchpad:
- a Python script that scans the 626 MB extraction on every query;
- a TSV index;
- a SQLite database;
- a pickled walk.

These live in temporary folders and cost tokens and minutes each time. The game check (238) already reads the pinned dump in about 2 seconds, without game text.

Every brief also had to pass `--binz /Users/…/blueprints/…`, because the default path points inside the checkout the tool was compiled in. A worktree has no `blueprints/`.

## Change

- **`game-check show <name|guid>…`** prints each blueprint:
  - its asset name, GUID and type;
  - its JSON with every `!bp_<guid>` reference resolved to `<name> <guid>`;
  - the fields that hold the game's display text left out (`m_DisplayName`, `m_Description`, `m_DescriptionShort`, and other localized-string fields), so the output can be pasted into a ticket.
- **`game-check grep <regex>`** lists `guid name type` for each blueprint whose name or raw JSON matches. The text fields are left out of the match.
- **`game-check refs <name|guid>`** lists the blueprints that reference it, such as the progressions and selections that grant a feature.
- **The default dump path:** when `--binz` is absent and the compile checkout has no `blueprints/`, the tool uses the main checkout's. It finds it through `git rev-parse --git-common-dir`, which is shared by every worktree.
- **`CLAUDE.md` "Game sources"** gains one line pointing to these commands as the way to read a blueprint.

## Done when

- Each subcommand has a fixture test, using invented blueprints as the existing fixtures do. The tests cover:
  - `show` resolves a reference and prints no localized text;
  - `grep` finds by name and by a component type;
  - `refs` finds a granter;
  - the default path is found from a worktree.
- From a lane worktree with no `--binz`, `cargo run -p game-check -- show PowerAttackFeature` prints the feature.
- The check's report is unchanged, byte for byte.
- The scoped checks pass.
