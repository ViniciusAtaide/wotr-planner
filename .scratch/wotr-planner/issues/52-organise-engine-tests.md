# Organise the engine tests

Type: task
Status: open
Blocked by: 51
Map: [WotR Build Planner](../map.md)

## Question

**User, 2026-11-02:** `rules/tests/engine/` (about 12,400 lines) reads like a library of its own. `support/mod.rs` alone is about 3,000 lines: the test world (an invented game: classes, archetypes, feats, selections, option rows, mythic paths, races, backgrounds, built in Rust) plus the Build helpers (`build`, `times`, `class_slot`, `pick_feat`, …), grown a bit by every ticket since 11. Several test files are named after the ticket that added them (`follow_ups.rs`, `option_follow_ups.rs`, `second_follow_ups.rs`, …), not the rule they test.

1. **Split `support/`** into modules: `support/world/` (one file per part of the world: classes, feats, selections, paths, races, backgrounds, …) and `support/builds.rs` (the helpers), with `support/mod.rs` only declaring and re-exporting them. Move-only: the world's contents don't change.
2. **Name test files by topic:** move the tests from ticket-named files (`follow_ups`, `option_follow_ups`, `second_follow_ups`, and any other named after a ticket) into the topic files they belong to (`parameters`, `option_levels`, `path_rules`, `removable`, …, or a new topic file where none fits), and delete the emptied files. Test names and bodies don't change.

Not in scope: describing the test world in RON files instead of Rust (the user chose the split only).

## Build

AFK; the only agent in the repo; touches only `rules/tests/`. Follow `CLAUDE.md`'s test-crate rules (every file gets its `mod` line in `main.rs`; shared helpers via `mod support;`).

**Done when:** `cargo test -p rules -- --list` lists the same tests before and after, apart from their module paths (save both lists and a mapping in the scratchpad); every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes; no file in `rules/tests/engine/support/` is over about 600 lines; no test file is named after a ticket.

## Progress log

## Answer
