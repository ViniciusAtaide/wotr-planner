# Importer S2, 306ac: is_selection against the blueprint's type

Type: build (the importer, stage S2)
Status: resolved
Blocked by: 306b
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

**Built** (`tools/game-check/src/areas/mapping.rs`): for each option that names a blueprint, the check compares what the planner takes as a selection (the hand `is_selection`, or a pick that names a value: a `chosen_feat` or a `parameter`) with the blueprint's type (`BlueprintFeatureSelection` and kin, or `BlueprintParametrizedFeature`). A planner selection on any other type is the new finding "mapping / selection or not". The other direction is no difference: a game selection with no hand field (33 options, among them the Loremaster's secrets, the ranger's second style lists, Familiar, Terrain Mastery) is derived from the blueprint, so there are 0 findings for them (the slice's expectation). The `ranks differ` finding moved into a helper shared by both arms.

**Decisions** (the builder's, from the slice: "expect 0 findings; any left gets a line under G5"): one direction only, since the other gives 33 findings that the slice says to read as derived. Value picks count as planner selections, as the slice's list says.

**Findings and their fate:** 3, none in the six hand-marked options (Combat Trick, the rogue Weapon Focus, both Skill Focus talents agree; Finesse Training and Battle Scion's feat have no blueprint and are skipped). The three are options that take a value on a blueprint that is not a selection: Zen Archer's Way of the Bow (`ZenArcherWayOfTheBowLongbowFocus` 2e659493076249f49821925b9b7dc389, `BlueprintFeature`), Oracle Weapon Mastery (`OracleRevelationWeaponMastery` 0a4c3556355747241b2d3dcc0a88ec10, `BlueprintProgression`) and the shaman's Battle Master (`ShamanHexBattleMasterProgression` 840651aac9b854a4980ec19e2fced74f, `BlueprintProgression`). Each is a planner choice the game lacks, listed in `known.txt` in its own block under ticket 316 (G5). Nothing in the data changed.

**Tests:** `areas::an_options_selection_is_checked_against_its_blueprints_type` in `tools/game-check/tests/fixture/areas.rs` (fixture: `pick-kind`, `pick-derived`, `pick-plain`, `pick-valued` in `planner.ron`, blueprints b51–b54): selection type with the hand field agrees, selection type with none agrees, a plain blueprint with `is_selection` and one with a parameter are findings. The expected types are the fixture's own, since the fact checked is the blueprint's type; the real-dump types are cited above by name and GUID (blueprints 2.7.0x_0).

**Checks:** `cargo test -p game-check` ok (107 passed); `cargo clippy --workspace --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `scripts/test-modules.sh` OK; the game check exits 0.
