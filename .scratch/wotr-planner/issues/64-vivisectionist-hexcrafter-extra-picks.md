# Extra Discovery and Extra Arcana reach the Vivisectionist and the Hexcrafter

Type: task
Status: open
Blocked by: 63
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 60](60-borrowed-class-selections.md)'s decisions 2 and 3 (user, 2026-11-02): Extra Discovery gives a Vivisectionist one more slot of its `alchemist/vivisectionist/medical-discovery`, and Extra Arcana gives a Hexcrafter one more slot of its `magus/hexcrafter/hex-arcana`, by ticket 49's rule (`extra_pick_of` names the selection; the slot is of the one the character had a slot of first). Check each feat's prerequisite reaches them too.

## Build

AFK, test-first (`tdd`); the only agent in the repo. Same data method as tickets 54–60: extractor module, one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, before/after proof with the latest proof program (only Builds that take Extra Discovery or Extra Arcana differ). Ticket 48's picker shortcut must stay exact. Data from the saved guide pages in `guide-cache/` only; a question the guide doesn't settle is written in this ticket with a recommendation, and the recommendation is taken (user, 2026-11-02).

**Done when:** the lighter per-ticket checks pass (fmt, clippy native and wasm, `cargo test --workspace` with the `-- --list` diff, the `rules` wasm build, seed `--check-only`, `sqlx prepare --check` if the schema changed); plus real-data tests: a Vivisectionist's Extra Discovery gives a Medical Discovery slot at the level it's taken; a Hexcrafter's Extra Arcana gives a Hex Arcana slot; a plain alchemist's and magus's extra picks are unchanged; the sheet labels the slots (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer
