# The component vocabulary: the engine reads the game's components

Type: build (rules)
Status: resolved
Blocked by: [244, the layout by blueprint](244-the-game-data-laid-out-by-blueprint.md) (the per-blueprint files it adds a field to); 248, 251, 253 (the engine rules it builds on)
Model: Opus to build: a new data form and its evaluation, beside the old one.
Map: [WotR Build Planner](../map.md)

## Problem

The user, 2026-11-15: port the component vocabulary, not the features. The queue was correcting the engine feature by feature (255, 257–260: Mythic Spell Focus, Wandering Hex, Greater Exploits, Ascendance, Skill Focus per skill). The game builds those features from a small set of generic components wired together in blueprints. The planner holds hand-shaped fields instead (`prerequisite`, `bonuses.ron`, `ability_bonuses`, `steps`, `bab_at_least_mythic_rank_plus`, feat slots, `Repeat`, …), each with its own engine code.

**Research** ([research/component-vocabulary-2026-11-15.md](../research/component-vocabulary-2026-11-15.md), from BubblePrintsData 2.7.0x_0 and WrathDecompiled 6ce4a6b):
- **The reachable set.** Walking from `BlueprintRoot`'s classes, mythic classes, races, `BasicFeatsProgression`, `BasicFeatSelection` and `DeitySelection` through progressions, selections and the grant components reaches 8,528 blueprints: 6,562 facts (4,871 features, 543 progressions, 443 selections, 122 parametrized features, 197 archetypes, 54 classes, 12 races, 145 permanent buffs, …) and 1,946 abilities. It reaches every entry 237 mapped but 17 features and options that ability actions grant.
- **The census.** 559 component types. 4,785 of the 6,562 facts carry a component the planner models. By reach: `AddFacts` 1,803; `PrerequisiteFeature` 763; `AddFeatureOnClassLevel` 504; `PrerequisiteNoFeature` 380; `ContextRankConfig` 266; `PrerequisiteClassLevel` 262; `AddFeatureIfHasFact` 225; the other prerequisites; `AddContextStatBonus` 179; `AddStatBonus` 163; `AddClassSkill` 109; `AddProficiencies` 86. The 352 runtime kinds (triggers, resistances, conditional saves, auras) and 597 resource components are left out, as today.
- **The feature tickets dissolve.** 255, 257, 259 and 260 are data under the vocabulary; 258 is the generic `SpellFocusParametrized` (`GetBonusDC` doubles the bonus while its `m_MythicFocus` fact is held).

This ticket builds the vocabulary's core in `rules`, beside the hand fields. Later stages move the data onto it (prerequisites, numbers, selections) and 293 writes it from the blueprints.

## Change

- **`libs/rules/src/component.rs`**: `Component`, in our own form, with keys (never GUIDs) inside:
  - `Requires(condition, group)`: the `Prerequisite*` family (`Has`, `HasNot`, `AtLeast`, `ClassLevel`, `NoClassLevel`, `ArchetypeLevel`, `NoArchetype`, `CharacterLevel`, `MythicLevel`, `Stat` with `full`, `ClassSpellLevel`, `CasterTypeSpellLevel`, `CasterType`, `Alignment`, `Proficiency`, `NotProficient`, `ParametrizedFeature`, `SelectionPossible`, `Condition` over `HasFact`, `CharacterClass` and `FactRank`). Semantics: `BlueprintFeature.MeetsPrerequisites` (every `All`, one `Any` when there are any), each `Prerequisite*.Check`.
  - `PrerequisiteClassLevels` (`ClassLevelsForPrerequisites`, read by `PrerequisiteClassLevel.GetClassLevel`), `PrerequisiteStat` (`ReplaceStatForPrerequisites.ResultStat`), `NoSelectionIf` (`NoSelectionIfAlreadyHasFeature`, `BlueprintFeatureSelection.IsSelectionProhibited`).
  - `Grant(keys)` (`AddFacts`), `GrantAtClassLevel` (`AddFeatureOnClassLevel.IsFeatureShouldBeApplied`, with `m_AdditionalClasses`, `m_Archetypes`, `BeforeThisLevel`), `GrantIf` (`AddFeatureIfHasFact.Apply`, one-way), `GrantOnApply`, `RemoveOnApply` (`AddFeatureOnApply`, `RemoveFeatureOnApply`), `GrantWithValue` (`AddParametrizedFeatures`).
  - `StatBonus(stat, value, descriptor, multiplier, at_least_zero, if)` (`AddStatBonus`, `AddContextStatBonus`, `AddStatBonusIfHasFact`), stacked by `ModifiableValue.ApplyModifiersFiltered` and `ModifierDescriptorHelper.DefaultStackingDescriptors`, permanent by `Modifier.IsPermanent`.
  - `Value`: `Flat`, `Rank(RankConfig)`, `Shared(SharedValue)`. `RankConfig` with the bases `ClassLevel`, `ClassLevelWithArchetype` (sum, max), `CharacterLevel`, `MythicLevel`, `FeatureRank`, `FeatureListRanks`, `StatBonus`, `BaseStat`, `BaseAttack`, and every `ContextRankConfig.ApplyProgression` progression and `ApplyMinMax`. `SharedValue`: `ContextCalculateSharedValue`'s flat part.
  - `ClassSkill`, `SkillPoints` (`AddSkillPointPerCharacterLevel`), `Proficiency` (`AddProficiencies`, `ArcaneArmorProficiency`, the background ones), `Flag` (`AddMechanicsFeature`).
- **The entries.** Every fact kind of 244 (features, options, feats, mythic abilities and feats, races, subraces, backgrounds, deities, classes, archetypes) gains `components: Vec<Component>` (default empty, left out when empty).
- **Evaluation**, on the walk's existing holds:
  - `Requires` compiles into the entry's `Prerequisite` tree (`All` of the `All` group, `Any` of the `Any` group), so the walk, the pickers, the reasons and `allows` read one form.
  - Grants hold their facts through `Held`, as rows do (247); `GrantAtClassLevel` and `GrantIf` are evaluated at each level.
  - Stat bonuses become `BonusInForce` entries (with their descriptor as `BonusType`) and feed `ability_scores`, `permanent_ability_scores`, the BAB, the saves, initiative and the skills; `Value` is worked out from the state at that level.
  - Class skills, skill points and proficiencies join the state's own.
- **`GameData::check`**: a key named in a component must exist (`UnknownKey`); an entry may not hold both a hand field and the component kind that replaces it (`prerequisite` with `Requires`, `ability_bonuses` with an ability `StatBonus`, …: `FieldAndComponent`); a `RankConfig` progression's step may not be 0.
- **No data change.** The files are untouched: the components exist for hand-made data and for the later stages.

**Decided** (user, 2026-11-15): the staged plan 295 → 296 → 297 → 298 → 293; every reachable feature becomes an entry (295 measures the browser JSON first and reports it); components compile at load into the engine's current structures. The design's other recommendations are taken (main session): one-way `AddFeatureIfHasFact` as the code does, no runtime markers, only the property getters the planner's components use, Skill Focus in the game's selection shape with a link alias.

## To decide (recommendations)

1. **Compile or evaluate.** Recommended: compile `Requires` into `Prerequisite`, stat bonuses into `BonusInForce` (one code path; the reasons stay).
2. **Stats with no line** (AC, CMB, CMD, HP, speed, the `Check*` sub-skills). Recommended: the kind accepts them and the engine counts them in no total; the sheet's `leaves_out` names them.
3. **`AddFeatureIfHasFact`'s one-way add.** Recommended: as the code does (`Apply` adds, nothing removes but the source's `OnDeactivate`).
4. **A hand field beside its component.** Recommended: refused by the check (no two sources for one fact).

## Tests

- Rules, hand-made data, `libs/rules/tests/engine/components.rs`, each kind's expected values cited by the decompiled code's file and method:
  - `requires_all_and_any_compile_into_the_prerequisite_tree_with_its_reasons` (`MeetsPrerequisites`);
  - `a_class_level_prerequisite_counts_the_classes_a_held_fact_adds` (`ClassLevelsForPrerequisites`, `PrerequisiteClassLevel.GetClassLevel`, with `Modifier` 0.5);
  - `a_grant_holds_its_facts_and_a_granted_selection_opens_its_slot` (`AddFacts`, `AddFeaturesFromProgression`);
  - `a_grant_at_class_level_comes_at_the_level_and_counts_its_additional_classes_and_archetypes`, and `before_this_level_takes_it_away_at_the_level` (`IsFeatureShouldBeApplied`);
  - `a_grant_if_held_adds_once_and_never_removes` (`AddFeatureIfHasFact.Apply`);
  - `a_stat_bonus_of_one_type_counts_the_largest_and_untyped_ones_stack` (`ApplyModifiersFiltered`);
  - `an_untyped_ability_bonus_counts_on_the_sheet_and_not_for_prerequisites` (`Modifier.IsPermanent`);
  - `a_rank_config_reads_each_base_and_applies_each_progression` (one case per progression, `ApplyProgression`; `ApplyMinMax`);
  - `a_custom_rank_progression_takes_the_first_band_at_or_above_the_value` (Skill Focus's shape);
  - `a_shared_value_below_zero_with_at_least_zero_adds_nothing` (Perfect Body's override shape);
  - `class_skills_skill_points_and_proficiencies_from_components_join_the_state`;
  - `check_refuses_an_unknown_key_in_a_component_and_a_field_beside_its_component`.
- Agreement: `agreement::judge` finds no disagreement on a test world whose entries use components only.
- Real data: none changes; the snapshot check and every real-data test pass unchanged (no data moves). Each `--list` name printed.

## Answer

**Built** (2026-11-16) as written, with the user's decisions of 2026-11-15 (the staged plan; every reachable feature an entry; components compiled at load into the engine's current structures) and the four recommendations above taken (the main session's, under the user's instruction).

**The browser JSON, measured first** (the user asked, for "every reachable feature becomes an entry"). Today's document: 2,097,210 bytes raw, 240,816 gzip -9, 170,591 brotli -q 11 (1,622 features at 413 bytes each, summaries included). The design's walk (`vocab/reach.pkl`) reaches 3,458 feature-like facts the data names by no blueprint (2,483 features, 387 progressions, 167 selections, 163 spellbook replacements, 145 permanent buffs, 101 parametrised features, 12 others), 2,529 of them carrying 5,048 components the vocabulary covers. A throwaway serialisation of a stub `Feature` for each (key and name from the blueprint name, empty summary, every hand field at its default as serde writes it today, and each component in its RON-shaped form) adds 1,607,777 bytes raw (464 per entry): **3.70 MB raw, 324 KB gzip (+83 KB), 227 KB brotli (+57 KB)**. Well under a few MB compressed, so the build went on. A summary of about 90 characters on each of the 1,159 shown features would add about 100 KB raw (an estimate). Leaving the default hand fields out of the JSON (as `components` already is) would take about 200 bytes off each entry.

**What was built** (`rules`, no data change):
- **`component.rs`**: `Component` with the kinds the ticket names (`Requires`, `PrerequisiteClassLevels`, `PrerequisiteStat`, `NoSelectionIf`, `Grant`, `GrantAtClassLevel`, `GrantIf`, `GrantOnApply`, `RemoveOnApply`, `GrantWithValue`, `StatBonus`, `ClassSkill`, `SkillPoints`, `Proficiency`, `Flag`), `Condition` (`Has`, `HasNot`, `FactRank`, `AtLeast`, `ClassLevel`, `NoClassLevel`, `ArchetypeLevel`, `NoArchetype`, `CharacterLevel`, `MythicLevel`, `Stat` with `full`, `ClassSpellLevel`, `CasterTypeSpellLevel`, `CasterType`, `Alignment`, `Proficient`, `NotProficient`, `ParametrizedFeature`, and `All`/`Any`/`Not` for `PrerequisiteCondition`'s tree), `Fact` (a key with its kind), `Value` (`Flat`, `PerRank`, `Rank(RankConfig)`, `Shared(SharedValue)`), `RankConfig` with every base the planner knows and all 15 `ApplyProgression` progressions and `ApplyMinMax`, `Descriptor` with the game's stacking and permanence, `IfHeld`. Each kind's doc cites its game component and the code that gives it its meaning, by file and method at WrathDecompiled 6ce4a6b. Serde form: our own words, keys inside, defaults left out (`group: All`, `descriptor: Untyped`, `multiplier: 1`); the README's example round-trips.
- **Entries**: features, feats, selections, options, mythic abilities and feats, races, subraces, backgrounds, deities, classes and archetypes hold `components: Vec<Component>` (default empty, left out when written, so the JSON and its hash are unchanged).
- **Compile at load** (`engine/catalog/components.rs`): `Requires` into the entry's `prerequisite` (an `All` of the `All` group, one `Any` of the `Any` group, a lone condition as itself); `NoSelectionIf`'s own-options rule into `only_without_option`; the rest indexed by entry kind and key.
- **Evaluation on the walk's holds** (`engine/walk/components.rs`): an entry's components apply when it is first held (`hold`) and go when it is no longer held (`release`), so path switches and traded rows take them along; the race's or own subrace's, the background's, the deity's, a class's and its archetype's apply as those are chosen. Grants hold facts through `Held` (a granted selection opens a slot, `SlotId::OptionSelection`, through a new `Pending::Granted`); `GrantAtClassLevel` and `GrantIf` are looked at on activation and at each class level and Mythic rank gained; stat bonuses feed `ability_scores` and `permanent_ability_scores` (after the gear's and the features'), the BAB (added to the class tables'), and the bonuses in force for the saves, initiative and skills (`BonusSource::Component`, stacked among themselves by the game's rule in `counted`); `SkillPoints` as the game's running total; class skills, proficiencies and flags join the state.
- **Prerequisite leaves added** for conditions with no planner leaf: `FullAbilityScore`, `ClassSpellLevel`, `CasterType` (state: `class_spell_levels` and `caster_types`, worked out only for data whose prerequisites read them); `ClassLevel`, `AbilityScore`, `SkillRanks` and `BaseAttackBonus` read the held facts' `PrerequisiteClassLevels` and `PrerequisiteStat` (state: `prerequisite_class_levels`, `prerequisite_stats`, `flags`). Labels give each new leaf its reason.
- **`GameData::check`**: unknown keys in components (`UnknownKey`, through the compiled tree for `Requires`, which also gets every prerequisite rule); `FieldAndComponent(field)` for `prerequisite`/`Requires`, `ability_bonuses`, `ability_modifiers` and `bab_at_least_mythic_rank_plus` beside the matching `StatBonus`, `weapons` beside a weapon `Proficiency`, `class_skills` beside `ClassSkill`, `skill_points` beside `SkillPoints`, a background's `feats` beside a feat grant, `only_without_option` beside `NoSelectionIf(any_option)`; `ComponentOnWrongEntry(kind)`; `NotGrantable`; `ZeroStep`; `FullSkillValue`.
- **`libs/game-data/README.md`** describes the field.

**Decisions.**
- User: the staged plan, every reachable feature an entry (measured above), compile at load.
- Main session: the four recommendations (compile; stats with no line accepted and counted in no total; `GrantIf` one-way; a hand field beside its component refused).
- Mine:
  - `FactRank` takes a feature only (the one kind with ranks); `Proficient`/`NotProficient` conditions take armour groups only, as the planner's proficiency feats: a weapon category needs the catalog, which a state check has not (not built, below).
  - `AddStatBonus`'s `Value × rank` is its own `Value::PerRank`; `ContextCalculateSharedValue`'s `Modifier` is an integer (1, 2, −1 and 5 in every reachable blueprint); `ClassLevelsForPrerequisites`' `Modifier` is `Share::Full`/`Half` (1.0 and 0.5 in every blueprint) and its `ForSelection` a `Pool` (`Feats` for `BasicFeatSelection`, `Options(selection)`).
  - Component bonuses stack among themselves by the game's rule; gear's keep the planner's rule among themselves, and the two pools don't meet yet (no data has both; 297 moves gear onto components).
  - `GrantIf` and `GrantAtClassLevel` are looked at right after the class level is taken: the game raises `HandleUnitGainLevel` from `UnitProgressionData.AddClassLevel`, before the level's picks.
  - `SkillPoints`: `AddSkillPointPerCharacterLevel` adds the character level to the running total `ApplySkillPoints.Apply` counts over every level, and `ApplySkillPoints` comes before `Features` (`LevelUpActionPriority`), so an entry picked at a level gives its points from the next, all of the earlier levels' at once.
  - An option a component grants is held through its defining selection without its rows by option level (rows are 298's progression header).

**Tests** (`libs/rules/tests/engine/components.rs`, hand-made data on the test world; expected values from the cited code):
- `requires_all_and_any_compile_into_the_prerequisite_tree_with_its_reasons`: `BlueprintFeature.MeetsPrerequisites`.
- `a_class_level_prerequisite_counts_the_classes_a_held_fact_adds`: `PrerequisiteClassLevel.GetClassLevel`, `(int)(0.5 × 4) = 2`, `(int)(0.5 × 3) = 1`.
- `a_grant_holds_its_facts_and_a_granted_selection_opens_its_slot`: `AddFacts.OnActivate`/`UpdateFacts`, `LevelUpHelper.AddFeaturesFromProgression`.
- `a_grant_at_class_level_comes_at_the_level_and_counts_its_additional_classes_and_archetypes`: `AddFeatureOnClassLevel.IsFeatureShouldBeApplied`, `ReplaceCasterLevelOfAbility.CalculateClassLevel`/`CheckArchetype`.
- `before_this_level_takes_it_away_at_the_level`: `IsFeatureShouldBeApplied` with `BeforeThisLevel`, `Apply`.
- `a_grant_if_held_adds_once_and_never_removes`: `AddFeatureIfHasFact.Apply`; `UnitProgressionData.AddClassLevel` → `OnGainClassLevel`.
- `a_stat_bonus_of_one_type_counts_the_largest_and_untyped_ones_stack`: `ModifiableValue.ApplyModifiersFiltered`, `ModifierDescriptorHelper.DefaultStackingDescriptors`.
- `an_untyped_ability_bonus_counts_on_the_sheet_and_not_for_prerequisites`: `Modifier.IsPermanent`, `ModifiableValueAttributeStat.CalculatePermanentValue`, `PrerequisiteStatValue.GetStatValue`, `PrerequisiteFullStatValue`.
- `a_rank_config_reads_each_base_and_applies_each_progression`: `ContextRankConfig.GetValue`, `GetBaseValue`, `ApplyProgression` (every progression), `ApplyMinMax`.
- `a_custom_rank_progression_takes_the_first_band_at_or_above_the_value`: `ApplyProgression` `Custom`; Skill Focus's shape, `SkillFocusAcrobatics` 52dd89af385466c499338b7297896ded.
- `a_shared_value_below_zero_with_at_least_zero_adds_nothing`: Perfect Body's shape, `DragonLevel2StrengthOverride` 9ed55389d2079714eab8495957964af2; `ContextCalculateSharedValue.Calculate`, `AddContextStatBonus.OnTurnOn` with `HasMinimal`.
- `a_stat_bonus_if_held_follows_its_fact`: `AddStatBonusIfHasFact.ShouldApplyBonus`.
- `class_skills_skill_points_and_proficiencies_from_components_join_the_state`: `AddClassSkill.OnTurnOn`, `AddSkillPointPerCharacterLevel.HandleUnitCalculateSkillPointsOnLevelup`, `ApplySkillPoints.Apply`, `LevelUpActionPriority`, `AddProficiencies.OnTurnOn`.
- `a_class_spell_level_and_a_caster_type_read_the_classes_spellbooks`: `PrerequisiteClassSpellLevel.GetClassSpellLevel`, `PrerequisiteCasterType.CheckInternal`.
- `check_refuses_an_unknown_key_in_a_component_and_a_field_beside_its_component`.
- `every_picker_agrees_with_the_walk_on_a_world_of_components`: the test world with every prerequisite a condition exists for moved into `Requires` (55 entries), `agreement::judge_all` over the sample corpus finds no disagreement, and each sample Build walks to the very sheet the hand-shaped world gives.
- The code was written before these tests (not test-first); three mutations (no `Custom` fallback, a full share for `Half`, every descriptor stacking) each make a test above fail.

**Real data**: no file changed; every real-data test, the gold builds and the self-checks' short runs pass unchanged; the JSON the browser gets is byte-for-byte the same (an empty `components` is left out). The state gained fields (`class_spell_levels`, `caster_types`, `prerequisite_class_levels`, `prerequisite_stats`, `flags`), empty for real data, so a hash of a state's `Debug` text changes while every value stays.

**Not built** (each a later stage's or a gap): weapon categories in `Proficient`/`NotProficient` (a state check has no catalog); `SelectionPossible` (needs `CanSelect` over a selection's options: 298's nested selections); a skill's full value (`FullSkillValue` refuses it); the `Property` value and the attack, caster, spellbook, gear and one-blueprint kinds (297 and after); a feature `Requires` (refused: features are never picked until 298 makes options of them); the sheet's `leaves_out` naming the stats with no line (no data has one yet).

## Done when

- `Component` and its evaluation are in `rules`, each kind's semantics cited in its doc by file and method at WrathDecompiled 6ce4a6b.
- The tests above pass on hand-made data; the snapshot files are untouched; the corpus hash is unchanged.
- `GLOSSARY.md` has **Component** and **Header** (the main session writes it); `libs/game-data/README.md` describes the field.
- The scoped checks pass.
