# The Instinctual Warrior's monk rage powers

Type: task
Status: open
Blocked by: 65
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 58](58-monk-ki-powers.md)'s decision 3 (user, 2026-11-02): the Instinctual Warrior (a barbarian archetype) can take monk powers as rage powers: Deflect Arrows, Purity of Body, Diamond Body, Diamond Soul and Perfect Self, at the levels its guide section gives. Add them as rage power options only an Instinctual Warrior can take, with those levels: separate options, not the monk's ki powers, since the levels differ.

## Build

AFK, test-first (`tdd`); the only agent in the repo. Same data method as tickets 54–60: extractor module, one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, before/after proof with the latest proof program (only Builds that have an Instinctual Warrior differ). Ticket 48's picker shortcut must stay exact. Data from the saved guide pages in `guide-cache/` only; a question the guide doesn't settle is written in this ticket with a recommendation, and the recommendation is taken (user, 2026-11-02).

**Done when:** the lighter per-ticket checks pass (fmt, clippy native and wasm, `cargo test --workspace` with the `-- --list` diff, the `rules` wasm build, seed `--check-only`, `sqlx prepare --check` if the schema changed); plus real-data tests: an Instinctual Warrior's rage power picker offers each power from its level and not before; a plain barbarian isn't offered them; Extra Rage Power's slot offers them to an Instinctual Warrior; the sheet shows one picked (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer
