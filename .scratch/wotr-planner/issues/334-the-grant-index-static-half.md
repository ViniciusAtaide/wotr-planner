# The grant index: its static half

Type: build (engine, tooling)
Status: open
Blocked by: 331
Model: Opus to build.
Map: [WotR Build Planner](../map.md)
Design: [design-G.md, T3](../research/architecture/design-G.md) (interface §1 static half, what goes §2's T3 rows, tests §4, risks §5), in the frame of [overall.md](../research/architecture/overall.md); decisions in [decisions.md](../research/architecture/decisions.md).

## Problem

Four crates each answer "what gives this fact" (findings `4-grant-index.md` §1–2), each counting different grant kinds. The carrier guard copies the favoured-weapon rule without its values (308h, 324). Switch features and a feat's selection are in none of them.

## Change

- Add `rules::grants` (design-G §1, static half) and build it in `Catalog::new`.
- Make `catalog.rs`, `check.rs:1925`, game-check's `carrier_guard` and seeds `support` read it (design-G §2's T3 rows).
- `catalog.rs` keeps `GrantedBy` as a temporary projection. `labels.rs` is untouched: ticket 332 edits it.

## Done when

- Every grant kind in design-G §1's `When` has a site, pinned by tests.
- The form proof (`FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1`, recorded on the base in the lane before the first edit) shows 0 differences.
- The game check exits 0.
- `--carrier-audit` is run on the base and the tip and diffed; each changed hit is explained in the Answer. New lines are written `-> ?` for the main session to assign, which is decision 7.
- The wasm and native costs of building the index are measured: under 10 ms native, and under 5% of the browser's "engine" line.

## Files

- `libs/rules/src/{grants.rs (new), lib.rs, engine/catalog.rs, check.rs}`
- `libs/rules/tests/engine/{grants.rs (new), main.rs}`
- `libs/seeds/tests/game_data/{support.rs, grant_index.rs (new), main.rs}`
- `tools/game-check/src/write/carrier_guard.rs`
- `tools/game-check/tests/fixture/prerequisites_write.rs`
- `tools/game-check/carriers-known.txt` (only if hits move)

## Checks

- `cargo test -p rules -p seeds -p corpus -p game-check`
- `cargo build -p rules --target wasm32-unknown-unknown`
- `cargo fmt --all --check`
- the form proof as above
- the game check
- `--carrier-audit` diffed
