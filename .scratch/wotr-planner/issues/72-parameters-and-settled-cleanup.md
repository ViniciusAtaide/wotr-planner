# Fold the repeats into parameters and prune the dead report filters

Type: task
Status: open
Blocked by: 71
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 71](71-ticket-30-families-where-owned.md)'s recommendations (taken, user, 2026-11-02):

1. Ticket 32's repeats and mythic values are still a pass (`follow_ups::apply`), because they rewrite what `parameters::apply` builds. Fold them into `parameters`' own tables, so they're built with the parameters, and update `libs/extract/tests/extract/parameters.rs` to cover the same cases.
2. Some `settled` report-filter entries now match nothing, because those items are no longer reported. Remove each entry that matches no report item, and keep a test that fails if an entry matches nothing (so dead entries can't pile up again).

## Build

AFK; the only agent in the repo. A refactor of `libs/extract` only: no change to `rules`, the schema, `libs/game-data/` or the link format. The user is asleep and said to take every recommendation: decide by recommendation, record it, and go on.

**Done when:**
- `libs/game-data/` is byte-identical (`git diff HEAD -- libs/game-data` empty).
- A fresh extraction into the scratchpad, report included, is byte-identical (`diff -r`) to one made with HEAD's extractor before the change.
- The `cargo test --workspace -- --list` diff is only the new test (and any moved name, explained).
- The lighter per-ticket checks pass (fmt, clippy native and wasm, `cargo test --workspace`, the `rules` wasm build, seed `--check-only`).
- The Answer ends with the updated list of passes still in `build.rs`, each with its reason.

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer
