# The component vocabulary: the engine reads the game's components

Type: build (rules)
Status: open
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

## Done when

- `Component` and its evaluation are in `rules`, each kind's semantics cited in its doc by file and method at WrathDecompiled 6ce4a6b.
- The tests above pass on hand-made data; the snapshot files are untouched; the corpus hash is unchanged.
- `GLOSSARY.md` has **Component** and **Header** (the main session writes it); `libs/game-data/README.md` describes the field.
- The scoped checks pass.
