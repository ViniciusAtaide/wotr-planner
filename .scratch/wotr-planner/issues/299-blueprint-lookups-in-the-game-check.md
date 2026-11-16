# Blueprint lookups in the game check

Type: build (tools)
Status: resolved
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

## Answer

Built `game-check show`, `grep` and `refs` and the default dump path from a worktree.

- **`tools/game-check/src/lookup.rs`:** `Library` holds every blueprint of the pinned dump with its JSON (the check's `Dump` keeps JSON only for the kinds it compares). `binz::scan` is the record walk `parse` now shares; `binz::pinned_bytes` is the size and SHA-256 check `read` shares.
  - `show <name|guid>…`: header line `name guid type`, then pretty JSON with each `!bp_<guid>` as `<name> <guid>` (`(not in the dump) <guid>` for one the dump lacks). A name that several blueprints share prints each. A GUID is accepted with or without `bp_`/`!bp_`.
  - Text left out: a value under `m_DisplayName`, `m_Description`, `m_DescriptionShort`, `m_Flavor` or `m_ShortDescription`, and any object with an `m_Key`, `Shared` or `stringkey` key (the shape of every localized string in the dump: about 40 field names such as `Text`, `Description`, `m_DescriptionText`, `LocalizedName`), becomes `<localized text>`.
  - `grep <regex>`: `guid name type`, in name order, for each blueprint whose name matches or whose JSON matches, with the text left out, on one line, references as GUIDs (`"Stat":"Strength"`). The first match runs on the whole JSON and is confirmed on the stripped one.
  - `refs <name|guid>`: `guid name type` of each blueprint whose JSON holds `!bp_<guid>` for it.
  - Exit 0; 2 for a refused dump, an unknown name or a bad pattern. A wrong command or count is refused before the dump is read.
- **`game_check::default_dump`:** `blueprints/<asset>` in the compile checkout when it exists, else in the main checkout from `git rev-parse --git-common-dir`.
- **`CLAUDE.md` "Game sources"** gains the one line. The workspace `Cargo.toml` gains `[profile.dev.package.game-check]` and `regex-automata` at `opt-level = 3`, so a debug `grep` takes about 5 s instead of 16 s; `regex` is a new dependency of the crate.
- **Decisions:** all mine (tooling). The brief's "left out" became a placeholder so a field's presence still shows.

Tests (`tools/game-check/tests/fixture/lookup.rs`, invented blueprints): `show_resolves_a_reference_and_prints_no_localized_text`, `show_of_an_unknown_name_says_so`, `grep_finds_by_name_and_by_a_component_type`, `grep_does_not_match_the_text`, `refs_finds_the_blueprints_that_grant_or_refer_to_one`, `the_default_dump_is_the_main_checkouts_from_a_worktree` (a temporary repo and its worktree). No game fact changes, so no game citation; the real dump was run by hand: `cargo run -p game-check -- show PowerAttackFeature` from this worktree, no `--binz`, prints `PowerAttackFeature 9972f33f977fc724c838e59641b2fca5 BlueprintFeature`. The check's report is byte-identical before and after (`cmp`).

## Done when

- Each subcommand has a fixture test, using invented blueprints as the existing fixtures do. The tests cover:
  - `show` resolves a reference and prints no localized text;
  - `grep` finds by name and by a component type;
  - `refs` finds a granter;
  - the default path is found from a worktree.
- From a lane worktree with no `--binz`, `cargo run -p game-check -- show PowerAttackFeature` prints the feature.
- The check's report is unchanged, byte for byte.
- The scoped checks pass.
