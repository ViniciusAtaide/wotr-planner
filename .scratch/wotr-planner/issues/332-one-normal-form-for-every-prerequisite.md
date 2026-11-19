# One normal form for every prerequisite, at load

Type: build (engine)
Status: open
Blocked by: 331
Model: Opus to build.
Map: [WotR Build Planner](../map.md)
Design: [design-P.md, T1](../research/architecture/design-P.md) (interface §1, what goes §2, tests §4, risks §5), in the frame of [overall.md](../research/architecture/overall.md); decisions 1 and 4 in [decisions.md](../research/architecture/decisions.md).

## Problem

Each reader of a Prerequisite re-derives its shape:
- `and_parts`, `all_texts` and three `Not(Not)` guards in labels, and one in ranks;
- `carrier_guard`'s `AtLeast(0 | 1)`;
- game-check's `canon`.

Fixing one place leaves the others. Ticket 329 fixed "and inside and" in three places and left "or inside or" bracketed. `AtLeast(1, [x])` reads "Needs 1 more of: Kinetic Healer" (308f), and "N more of" shows with none held (330 item 3).

## Change

- `Prerequisite::canonicalize` normalises (design-P N1–N5: no and in an and, no or in an or, `AtLeast(1)` is an or, `Not(Not(x))` is `x`, a group of one is its child, children sorted). `compile_prerequisites` applies it to every entry, hand trees included.
- Add `is_canonical`, and the `Unmet::all`/`any` constructors used by `check_in`, `unmet_for` and `Entry::issue`.
- Delete the flattening and the guards listed in design-P §2.
- Make 330 item 3's change in `unmet_text`.

## Done when

- A real-data test shows every tree in the checked data is canonical.
- The form proof's `compare` (`FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1`, recorded on the base in the lane before the first edit) reports 0 status changes. Every changed reason is one of decision 1's first three items, listed in the Answer by count.
- The plain game check prints the same report, byte for byte, and exits 0.
- The exact texts in design-P §4 pass.

## Files

- `libs/rules/src/{prerequisite.rs, component.rs, choice.rs, state.rs}`
- `libs/rules/src/engine/{labels.rs, ranks.rs, walk/judge.rs}`
- `libs/rules/tests/engine/{prerequisites.rs, labels.rs}`
- `libs/seeds/tests/game_data/{wild_talents.rs, barbarian_witch_shaman_prerequisites.rs, canonical_trees.rs (new), main.rs}`
- `apps/mcp/tests/server/wording.rs`

If the build needs `libs/rules/src/engine/catalog.rs`, `libs/rules/src/check.rs` or `libs/seeds/tests/game_data/support.rs`, say so at once: ticket 334 may be running beside this one on those files.

## Checks

- `cargo test -p rules -p seeds -p corpus -p mcp -p game-check`
- `cargo run -p seeds --bin check-data`
- `cargo build -p rules --target wasm32-unknown-unknown`
- the game check
- the form proof as above
- `cargo fmt --all --check`
