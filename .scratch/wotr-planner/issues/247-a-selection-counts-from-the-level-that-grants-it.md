# A selection counts as held from the level that grants it

Type: build (engine rule)
Status: resolved
Blocked by: none
Model: Opus to build: a new prerequisite form, read by the walk, the pickers and `GameData::check`.
Map: [WotR Build Planner](../map.md)

## Problem

Found by [242](242-the-tests-expected-values-from-the-game.md)'s research (2026-11-15): the game's code contradicts a rule of the engine.
- **The game:** a progression adds each selection blueprint it grants as a fact of its own, whatever is picked in it (`UnitLogic/Class/LevelUp/Actions/LevelUpHelper.cs`, `AddFeaturesFromProgression`, WrathDecompiled 6ce4a6b). A `PrerequisiteFeature` naming a selection (`RagePowerSelection`, `ExploiterExploitSelection`, `DomainsSelection`, …) therefore means "the class gave that selection", from the level that grants it, even while its slot is empty.
- **The planner** has no form for that: the data writes such a prerequisite as "holds one of the selection's options" (an `Any` of options, or `SelectionOption`), which fails until a pick is made and passes for a pick made through another class's slot.
- **Examples** (242's research): Extra Rage Power (`ExtraRagePowerSelection` 0c7f01fbbe687bb4baff8195cb02fe6a) needs `RagePowerSelection`, `InstinctualWarriorRagePowerSelection` or `SkaldRagePowerSelection`, so a skald 3 with no rage power held meets it; Extra Arcanist Exploit (`ExtraArcanistExploitSelection` 0a7065e13f4449fa81bfb33199bf7f6a) is open to an Exploiter Wizard 1 holding no exploit; Extra Rogue Talent (`ExtraRogueTalentSelection` 14d9089df87a43b696fa9451ca2f0a12) is closed to a bard or skald, whose talent selections it doesn't name; Domain Zealot (`DomainMastery` 2de64f6a1f2baee4f9b7e52e3f046ec5) and Impossible Domain (`ExtraDomain` 213a8480d22206b45acbfa0619ca5aaf) name selections, not domains.

## Change

- **The rule:** a prerequisite form for "the Build holds this selection", true from the level of the row that grants it (a class's, an archetype's or a mythic path's), whatever its slot holds. The walk, the pickers, the reasons and `GameData::check` read it; the game check compares it with the game's `PrerequisiteFeature` of the selection.
- **No data change here:** the data tickets that use the form change their prerequisites, each blocked by this one: [263](263-feats-prerequisites.md), [264](264-the-extra-feats.md), [265](265-mythic-abilities-and-feats.md), [276](276-rogue-and-slayer-talents.md).

## Tests 242 marks as waiting on this rule (through the tickets above)

- `skald_rage_powers.rs`: `extra_rage_power_reaches_a_skald` (a skald 3 with no rage power held meets Extra Rage Power).
- `mercies_and_exploits.rs`: `extra_arcanist_exploit_gives_an_exploiter_wizard_one_more_exploit_slot` (an Exploiter 1 holding no exploit meets it).
- `extra_picks.rs`: `each_extra_feat_gives_a_slot_of_its_classs_selection` (a bard 3 and a skald 3 are refused Extra Rogue Talent).

## Answer

**Built** (2026-11-15). The game adds each selection a progression grants as a fact of its own, whatever is picked in it (`UnitLogic/Class/LevelUp/Actions/LevelUpHelper.cs`, `AddFeaturesFromProgression`, WrathDecompiled 6ce4a6b: it calls `unit.AddFact` for every feature of the level entry, an `IFeatureSelection` among them, after queueing its slot). The planner now holds it the same way.

- **The form.** `Prerequisite::Selection(FeatureSelectionKey)` (`libs/rules/src/prerequisite.rs`): the Build holds the selection. RON: `Selection("barbarian/rage-power")`.
- **The state.** `CharacterState::selections` (`state.rs`), each selection a row has granted with how many rows hold it; `check_in` reads it.
- **The walk** holds it through a new `Held::Selection` (`engine/walk.rs`, `hold.rs`), from the row that grants it, before any slot of the level is filled: a class's or archetype's row (`take_class_level`, `walk/level.rs`, the archetype's effective row, so an archetype that removes the selection holds none), an option's row (`grant_row`, `walk/options.rs`; a row a pick replaces takes it back), a mythic path's row (`ascend`, `walk/ascension.rs`, sourced as that selection's so a switch dropping it drops the fact). A feat's extra pick (Extra Rage Power's slot) holds nothing, as the game's Extra feats are selections of their own. The pickers read it through the walk's state, so they need nothing of their own; the agreement test below shows they agree.
- **Rank status.** A path switch that drops a held selection alone doesn't mark the rank partly dropped (`switch_path`'s `named_lost`): the Devil-from-Aeon test's empty MR 7 gaze slot stays `Reached`.
- **Reasons.** "the Rogue Talents selection" (`labels.rs`; "Not with the … selection" under a `Not`).
- **The data check** names the selection (`check.rs`, `UnknownKey`).
- **The game check** reads it as the game's `PrerequisiteFeature` of the selection's blueprint (`tools/game-check/src/areas/prerequisites.rs`, `planner_side`).
- **No data change** (the ticket's scope): the Extra feats still list options. Since 263, 264 and 276 were superseded by 293, the importer writes their prerequisites in this form.

**Decisions.**
- Ours: the fact is held from every row that grants a selection, an option's row too, since the game runs an option's progression through `AddFeaturesFromProgression` as well (`UpdateProgression`). A mythic ability's own selections (Second Bloodline's) aren't rows and aren't held; no game prerequisite is known to name one.
- Ours: counted per row, so a replaced option row releases it.
- Ours: the reason's text "the X selection".

**Tests.**
- Rules, hand-made data, `libs/rules/tests/engine/held_selections.rs`: `a_class_rows_selection_is_held_from_its_level_with_its_slot_empty` (rogue 1 no, rogue 2 yes, slot empty), `an_option_taken_through_another_selection_doesnt_hold_the_selection_that_defines_it` (a slayer's Fast Stealth holds the slayer's selection, not the rogue's), `an_options_row_and_a_mythic_paths_row_hold_their_selections_too`, `a_feat_needing_a_selection_is_open_from_the_rows_level_and_the_walk_agrees` (Extra Rogue Talent needing the rogue's selection: open to a rogue 3 with no talent, closed at 1 and to a slayer 3 holding a rogue talent, with the reason; `agreement::judge` finds no disagreement). `check.rs`'s `every_feature_selection_and_option_named_is_one_the_data_has` adds a `Selection` naming a missing selection.
- Real data, `libs/seeds/tests/game_data/held_selections.rs`, each cited: `a_skald_holds_its_rage_power_selection_from_3_with_no_rage_power_picked` (`SkaldProgression` 26418fed2bc153245972a5b54204ed75 grants `SkaldRagePowerSelection` 2476514e31791394fa140f1a07941c96 at 3; `BattleScionArchetype` 71d41dd6309ea994886908c34c87ccd9 removes it and adds `BattleProwessSelection` 29b480a26a88f9e47a10d8c9fab84ee6), `an_exploiter_wizard_holds_the_exploit_selection_from_1` (`ExploiterWizardArchetype` 6d2027b4eec590c41adb5caeb7bb026d adds `ExploiterExploitSelection` 2ba8a0040e0149e9ae9bfcb01a8ff01d at 1), `a_rogue_talent_taken_in_a_slayer_talent_slot_holds_the_slayers_selection_only` (`SlayerProgression` 1f35d11937a06624b8a60db68c7869a1 grants `SlayerTalentSelection2` 04430ad24988baa4daa0bcd4f1c7d118 at 2).
- Game check fixture, `tools/game-check/tests/fixture/areas.rs`: `prerequisites_read_a_held_selection_as_the_game_s_feature_of_the_selection` (Extra Trick agrees; Extra Boon, planted with the wrong selection, is found on both sides).
- **Still waiting** (renamed to 293, the importer, which writes the data in this form): `skald_rage_powers.rs`'s `a_skald_takes_extra_rage_power_once_it_has_a_rage_power_slot` and `mercies_and_exploits.rs`'s `an_exploiter_1_takes_extra_arcanist_exploit_with_no_exploit_picked`: both need the feats' prerequisites rewritten as the game's selections, a data change outside this ticket. `extra_picks.rs`'s `each_extra_feat_gives_a_slot_of_its_classs_selection` was never ignored; its bard and skald cases wait on the same data.

## Done when

- A fixture test in `rules` shows the form true from the granting row's level with the slot empty, false before it, and false for a Build whose only matching pick came through another selection.
- The game check compares the form with the game's `PrerequisiteFeature` of a selection (a fixture test in `tools/game-check`).
- The scoped checks pass (`CLAUDE.md`, "Checks").
