# Expected refusals as cited rows, one file per ticket

Type: build (tests)
Status: open
Blocked by: 332, 334
Model: Opus to build.
Map: [WotR Build Planner](../map.md)
Design: [design-X.md, T8](../research/architecture/design-X.md) (interface §1, what goes §2, tests §5, risks §6), in the frame of [overall.md](../research/architecture/overall.md); decision 8 in [decisions.md](../research/architecture/decisions.md).

## Problem

Several separate exceptions excuse refused picks in the tests:
- helpers A–D in `libs/seeds/tests/game_data/support.rs`;
- three lists in `tools/corpus/tests/links.rs`;
- a file of Build names, `306-refused.txt`.

Two of the lists (Battle Prowess and the rogue-talent prefix) go by key alone, against CODING_STANDARDS:32. A listed Build hides any later refusal in it (`links.rs:149`). These exceptions cost fix rounds in 306s, 308e and 325. Eighteen data tickets edited one shared file.

## Change

Add `rules::refusals`, behind the `testing` feature beside `agreement`. It parses cited rule rows from `libs/game-data/refusals/<ticket>-<slug>.txt`, which `seeds::refusal_files()` reads, and answers `excuse(engine, build)` (design-X §1). A Build is refused-as-expected if and only if every one of its failures is excused.

## Done when

- **Before the first edit,** the lane records each S2 base link's fate and each refused Build's failures.
- **After the change:**
  - Every corpus, bench, S2 and old link's fate is unchanged: the same counts of same, heir and refused.
  - The Builds X calls expected are exactly the 459 that `306-refused.txt` listed.
  - The stripped Builds of `first_ascension` and `ranks_and_pools` are unchanged.
- **Every rule is hit,** and every row parses and is cited. A cited GUID is the entry's own, its place's selection's, or that of a Fact its shape or `when` names.
- **These are gone:** helpers A–D, the three lists, `306-refused.txt` and `fixture-refused.txt`.
- **The docs are updated:**
  - CODING_STANDARDS:32's model is a row.
  - The README's Layout and CLAUDE.md's Game data section name the folder: a data ticket adds rows in a file of its own.

## Files

- `libs/rules/src/{lib.rs, refusals.rs (new)}`
- `libs/rules/tests/engine/{main.rs, refusals.rs (new)}`
- `libs/seeds/src/lib.rs`
- `libs/game-data/refusals/*.txt` (new)
- `libs/game-data/README.md`
- `libs/seeds/tests/game_data/{support.rs, first_ascension.rs, ranks_and_pools.rs, expected_refusals.rs (new), main.rs}`
- `tools/corpus/tests/links.rs` and `links/`
- `CODING_STANDARDS.md`
- `CLAUDE.md`

## Checks

- `cargo test -p rules -p seeds -p corpus`
- `cargo run -p seeds --bin check-data` (the new folder is ignored)
- the rules wasm build
- `cargo fmt --all --check`
- `scripts/test-modules.sh`

No form proof: no data and no engine behaviour moves.
