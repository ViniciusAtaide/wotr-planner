# Importer S2, 306d: no option left reads the options' prerequisites

Type: build (the importer, stage S2)
Status: resolved
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as the section of [slices.md](../research/306/slices.md) says, with one difference: the tests build their world by editing the test data in the test (`engine_with`), so `test_world/world/nested.rs` is not touched.

**What was built**
- `libs/rules/src/engine/slots.rs` `Catalog::has_option_left`: a feat, mythic ability or option counts when it is under its repeat count **and** its prerequisite holds in the state (`CharacterState::check`). A selection with `ignores_prerequisites` skips the prerequisite, as the game's `BlueprintFeatureSelection.CanSelect` does for `IsIgnorePrerequisites()`.
- `libs/rules/src/choice.rs`: the `Issue::NoOptionLeft` doc no longer says the planner ignores the options' prerequisites.

**Game citation:** `FeatureSelectionExtensions.CanSelectAny` (`Assembly-CSharp/Kingmaker/Blueprints/Classes/Selection/FeatureSelectionExtensions.cs`, WrathDecompiled 6ce4a6b) is `selection.Items.Any(item => selection.CanSelect(...))`; `BlueprintFeatureSelection.CanSelect` returns false when `!IsIgnorePrerequisites()` and the feature's `MeetsPrerequisites` fails; `NestedFeatureSelectionUtils.AllNestedFeaturesUnavailable` calls `CanSelectAny`.

**Decisions:** the prerequisite is read in the state at the pick, so a later pick of the level that would meet it does not count (the game's order too: the selection is judged when it is offered). The ignore flag is read from the selection (the engine's `ignores_prerequisites`).

**Tests** (`libs/rules/tests/engine/nested_selections.rs`): `a_selection_whose_options_all_fail_their_prerequisites_is_no_option_left` (a fighter 3 picks Slayer Trick made to grant the teamwork feats, whose two feats need base attack 4 or 20: `NoOptionLeft`; failed before the change), `one_takeable_option_keeps_the_pick_open` (Shield Wall's prerequisite dropped: the build is possible and the nested slot opens).

**Proof (offers, `FORM_PROOF_OFFERS=1`, base `ab42864f`, this lane):** links 2000 the same, 0 changed; 18,079,504 options recorded and now, none only on one side; 6 statuses changed in 2 groups, both a selection now refused because none of its options' prerequisites hold, in 1 Build: extra-mercy ok -> NoOptionLeft 3; extra-wild-talent ok -> NoOptionLeft 3. 0 Build differences; no corpus Build stops opening.

**Order of the refusals:** the walk's `check_pick_with` (`engine/walk/judge.rs`) now checks the pick's own prerequisite before the selection it grants, so an unmet prerequisite is still the reason shown (and stays deferrable); a first run with the selection checked first changed 32,383 offers' reasons from Unmet to NoOptionLeft and broke `extra_wild_talent_needs_kineticist_level_2`.
