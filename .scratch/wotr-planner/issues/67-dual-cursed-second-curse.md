# The Dual-Cursed Oracle's second curse becomes a pick

Type: task
Status: open
Blocked by: 66
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 62](62-wandering-hex-and-beneficial-curse.md)'s decision (user, 2026-11-02, by recommendation): the Dual-Cursed Oracle's "Second Curse" is a feature in the data, but the guide says the oracle chooses it. Make it a slot of `oracle/oracles-curse` at the level its table gives (1), in place of the feature, so a curse taken there isn't offered again by Beneficial Curse's slot or anywhere else.

## Build

AFK, test-first (`tdd`); the only agent in the repo. Same data method as tickets 54–62: extractor module, one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, before/after proof with the latest proof program (only Builds with a Dual-Cursed Oracle differ). Ticket 48's picker shortcut must stay exact. Data from the saved guide pages in `guide-cache/` only; a question the guide doesn't settle is written in this ticket with a recommendation, and the recommendation is taken (user, 2026-11-02).

**Done when:** the lighter per-ticket checks pass (fmt, clippy native and wasm, `cargo test --workspace` with the `-- --list` diff, the `rules` wasm build, seed `--check-only`, `sqlx prepare --check` if the schema changed); plus real-data tests: a Dual-Cursed Oracle 1 has two curse slots (its own and the second), the second not offering the first's curse; Beneficial Curse doesn't offer either; the sheet labels the slot (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer
