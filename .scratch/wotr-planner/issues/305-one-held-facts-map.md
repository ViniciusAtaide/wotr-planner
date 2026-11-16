# One held-facts map

Type: build (data form and engine)
Status: resolved
Blocked by: 304
Model: Opus to build: the state's maps change shape under every link and sheet.
Map: [WotR Build Planner](../map.md)

## Problem

Split from [298](298-selections-as-the-game-has-them.md) (user, 2026-11-16, "B: forms now, splits in 293"): one of the staged component plan's five form steps (user, 2026-11-15; [295](295-the-component-vocabulary.md)). The design is in [research/component-vocabulary-2026-11-15.md](../research/component-vocabulary-2026-11-15.md), §3 and §4. The per-class selection splits are [293](293-the-importer.md)'s, not this ticket's.

**What the planner holds today:** five held-fact maps in the state, one per kind, where the game holds one list of facts.

## Change

- **One map of held facts,** keyed as the game keys a fact, with the kinds kept as views for the links and the sheet.
- **Links and sheets are byte-identical:** the views give what the five maps gave.

## Done when

- The form proof (`record` on the base in a temporary worktree, `compare` on the tip) shows 0 differences, or each difference is explained in the Answer and comes from the game, cited.
- Every corpus and bench link opens with the same picks (298's link test).
- The self-checks' short runs, the gold builds and every real-data test are unchanged.
- The scoped checks pass, and the game check exits 0.

## Answer

**Built** (2026-11-16) on the main session's decisions (`research/305/decisions.md`: the shape, feat selections as facts, Q1–Q5).

**The map** (`libs/rules/src/state.rs`). `CharacterState::held: BTreeMap<Fact, HeldFact>`, `HeldFact { holds: u8, values: BTreeMap<Parameter, u8> }`, replaces the twelve maps: `feats`/`feat_values`, `features`, `selection_options`/`option_values`, `selections`, `mythic_abilities`/`mythic_ability_values`/`mythic_ability_open_holds`, and `mythic_feats`/`mythic_feat_holds`/`mythic_feat_values`. Each kind keeps today's count (decisions): a count of holds and of each value for feats, features, options and selections; for a mythic ability, `holds` counts its picks without a value and each value is held once; for a mythic feat, `holds` counts its picks (303) and each value is held once. An entry nothing holds is removed, so `CharacterState: Eq` is unchanged. The views (`feats()`, `feat_values()`, `features()`, `options()`, `option_values()`, `selections()`, `mythic_abilities()`, `mythic_feats()`) are `range` scans over one kind, in `Fact`'s order, which is the old maps' order. The lookups (`feat_count`, `has_feat`, `feat_values_of`, `selection_count`, `mythic_feat_values_of`, …) find an entry through a crate-private `FactRef` (`Borrow<dyn AsFactRef>`) and copy no key. The map is keyed by `Fact`, not by GUID (Q1). There's no rank cap (Q2), no second fact for 302's nested selections (Q3), and no source or level (Q5). The option caches are unchanged (Q4).

**Writers and readers.** `Walker::hold`/`release` and `add_feature` (`engine/walk/hold.rs`) write one entry. Readers ported: `check_in`, `Fact::rank`, `engine.rs` (favoured weapons note, daily choices), `attack.rs` (default weapon, the feats for the weapon, Power Attack), `caster.rs`, `catalog.rs` (proficiency), `walk/components.rs` (values of a holder), `hold.rs` (`refresh_bab`, `own_base`), the MCP `sheet.rs` (`new_feats`, `new_features`), and the tests. The main session ported `tools/corpus/tests/form_proof.rs` line 247 (user's allowance).

**Selections of feats as facts** (decisions). The `offers_feats` filters are gone from the class-row (`level.rs`) and option-row (`options.rs`) holds. A path rank holds its feat selections (`ascension.rs`), and `hold_feat_selections` (`level.rs`) holds the race's (level 1) and the general slot's (odd levels). Game: `LevelUpHelper.AddFeaturesFromProgression` adds each feature a progression grants with `unit.AddFact(item)`, and `SelectRace.Apply` calls it with the race's features (WrathDecompiled 6ce4a6b). BasicFeatsProgression 5b72dd2ca2cb73b49903806ee8986325 grants BasicFeatSelection 247a4068296e8be42890143f451b4b45 at levels 1, 3, 5, …, and HumanRace 0a5d473ead98b0646b94495af250fdc4's features hold it too. No offer, number or link moved (the proof below). Known gap: BasicFeatSelection's `Ranks` is 1, so the game holds a human's at rank 1 where the planner counts 2. Nothing reads that count; the rank cap is Q2's follow-up. The half-elf's Adaptability is held as the race's selection; in the game it comes through HalfElfHeritageSelection 9df7b68d60544bcf8e5b56c0a4688e04, and only the test world covers it.

**Tests.**
- `held_selections::a_selection_of_feats_is_held_as_the_game_holds_it` (test world: general, race, class row, path rank).
- `mythic::no_fact_stays_held_once_nothing_holds_it` (after a Legend switch, no empty entry remains, and each view lists its kind alone, in order).
- Real data, `feat_selections_as_facts`:
  - `the_general_feat_selection_is_held_from_level_1`: BasicFeatsProgression and BasicFeatSelection, as cited above.
  - `a_class_rows_selection_of_feats_is_held_from_its_level`: FighterProgression level 1, FighterFeatSelection 41c8486641f7d6d4283ca9dae4147a9f.
  - `a_path_ranks_selection_of_feats_is_held_from_its_rank`: GoldenDragonProgression a6fbca43902c6194c947546e89af64bd level 2, DragonLevel2FeatSelection a21acdafc0169f5488a9bd3256e2e65b ×3.
- The new tests were written after the change, so none was run red on the base.

**Proof and checks.**
- Form proof, `record` on fc584cda, then `compare` on the tip: 2161 Builds compared (2000 open), 2161 old links reopened, 0 differences (offers not recorded: 305 changes none).
- Tests: `cargo test -p rules` 752 passed; `-p seeds` 534 passed (gold builds included); `-p mcp` 246 passed; `-p corpus` passed, links 2 passed.
- Builds and lints: the rules wasm build finished; clippy on corpus and rules (`--all-targets -D warnings`) is clean; fmt is clean.
- The game check exits 0.
