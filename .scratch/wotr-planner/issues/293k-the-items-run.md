# Importer S1, 293k: the items run and the +8 headband

Type: build (the importer, stage S1)
Status: resolved
Blocked by: 293e, 293j
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [293](293-the-importer.md)'s stage S1, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/293/slices.md](../research/293/slices.md). The decisions are in [research/293/decisions.md](../research/293/decisions.md) and in that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as written in `slices.md`, 293k. `game-check --write items` wrote 4 fields in 4 files (log: the lane's scratchpad `293k/items.log`): `belt-of-the-venerated-champion` (Fortitude +2 untyped added), `dirge-for-sarkoris` (Strength +2 untyped to Enhancement), `half-plate-of-vigor` (Constitution +2 untyped to Enhancement), `trusted-friend` (components emptied). Note: `libs/game-data/notes/293-items.md`.

- **Headband +8 by hand** (the writer's pre-write data check refuses an item with no bonus, so the file went in with its three components): `headband-of-mental-perfection-8`, `HeadbandOfPerfection8` f94d69b19fe07ca4a8ade7993a1a7510, Headwear, Int/Wis/Cha +8 Enhancement, no guide; `user-facts.txt` line `applied 288` (the user's, ticket 288 line 15).
- **Hand edits before the run:** the Venerated Champion's `other_effects` keeps only "+1 insight bonus to AC against evil" (game: `BeltOfVeneratedChampionFeature` 49fb12e9…); Trusted Friend's bonuses became an `other_effects` line (decision 2: the main session's).
- **A decision of mine outside the listed files:** Trusted Friend with no bonus failed the data check (`DataIssue::GearWithoutBonus`), so `libs/rules/src/check.rs` accepts an item with no bonus when `other_effects` is not empty; `libs/rules/tests/engine/check.rs` pins it (a toggle item raises no issue). Reject it if the main session wants another form.
- `known.txt`: deleted the 288 lines of the Venerated Champion, Dirge and Half-Plate (3 lines). Trusted Friend's stays for its slot, with Broken Trickster and Razmir's Mask.
- **Tests** (`libs/seeds/tests/game_data/gear.rs`): `the_game_has_a_headband_of_mental_perfection_8` (un-ignored), `exactly_the_users_twenty_three_generic_items_have_no_guide_reference`, `the_venerated_champions_fortitude_bonus_is_always_on` (un-ignored, asserts the GearStat), `the_half_plate_of_vigor_gives_con_2_enhancement` (the Con part; `the_half_plate_of_vigor_is_a_2_half_plate` stays ignored for 288), `sarkoris_dirge_gives_str_2_enhancement`, `trusted_friends_bonuses_are_its_toggles_not_always_on`; gear count 93; two tests accept an item with no bonus. Each cites its blueprint and GUID.
- **Proof:** game check exits 0, items 8 findings to 3 (the 3 `slot differs`, 288), 83 compared; a second `--write items` writes 0 files; form proof (base recorded before the write): 2161 Builds compared (2000 open), 2161 old links reopened, 0 differences.
- **Checks:** `cargo test -p seeds -p corpus`, `-p rules`, `-p game-check` pass; wasm build of rules, `check-data` (93 gear), `cargo fmt --all --check` and `scripts/test-modules.sh` (OK) pass.
