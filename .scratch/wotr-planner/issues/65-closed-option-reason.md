# Reason text for an option closed to an archetype

Type: task
Status: open
Blocked by: 64
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 59](59-kineticist-infusions.md)'s decision 5 (user, 2026-11-02): the lock reason for an option an archetype can't take reads "Needs not the Kinetic Sharpshooter archetype". It should read "Not for the Kinetic Sharpshooter archetype". Fix it where `rules`' labels render a negated archetype prerequisite (`libs/rules/src/engine/labels.rs`), for every negated archetype, and check other negated prerequisites for the same "Needs not …" wording (fix each to read naturally, and list them in the Answer).

## Build

AFK, test-first (`tdd`); the only agent in the repo. A labels change only: no data, schema or link change. Before/after proof with the latest proof program: only lock-reason text may differ, and only for negated prerequisites.

**Done when:** the lighter per-ticket checks pass (fmt, clippy native and wasm, `cargo test --workspace` with the `-- --list` diff, the `rules` wasm build, seed `--check-only`, `sqlx prepare --check` if the schema changed); plus an engine test and a real-data test: a Kinetic Sharpshooter's closed infusion reads "Not for the Kinetic Sharpshooter archetype"; no reason anywhere starts with "Needs not" (a real-data sweep over every picker of the proof's Builds).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer
