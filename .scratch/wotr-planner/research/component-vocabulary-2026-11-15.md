# The component vocabulary: design (2026-11-15)

Research and design only. Main at be88f9d7; 244's layout read on `ticket/244-the-game-data-laid-out-by-blueprint` (lane-game, ae501df3). Game: BubblePrintsData 2.7.0x_0 (`bpout/blueprints.jsonl`), code WrathDecompiled 6ce4a6b. Every citation below is by blueprint name and GUID or by file and method at 6ce4a6b.

Scripts and outputs beside this file (all mine, run with `python3 -I`):
- `extract.py` → `db.pkl`: the 34,225 blueprints of the kinds a Build can touch (features, selections, progressions, classes, archetypes, races, buffs, abilities, items, enchantments, spell lists, stat tables, the root).
- `bp.py`: the loader and a reference walker. `q.py`, `show.py`: ad-hoc queries; `show.py <width> <name|guid>…` prints a blueprint's header and components with references resolved to names.
- `walk.py` → `reach.pkl`: the reachable set (§1).
- `census.py` → `census.tsv`: every component type over the reachable set, with blueprints using it, instances, and abilities using it.
- `classify.py` → `classified.tsv`: the census sorted into planner categories (§2).
- `fields.py <Component> <field,…>` and `stats.py`: value distributions of one component's fields.

## 1. The reachable set

**Roots** (from `BlueprintRoot` `Progression`): the 40 `m_CharacterClasses` (base and prestige), the 14 `m_CharacterMythics`, the 12 `m_CharacterRaces`, `m_FeatsProgression` (`BasicFeatsProgression` 5b72dd2ca2cb73b49903806ee8986325, whose level 1 grants `BasicFeatSelection`, `BackgroundsBaseSelection` and `DeitySelection`), `m_BasicFeatSelection` (247a4068296e8be42890143f451b4b45) and `m_DeitySelection` (59e7a76987fe3b547b9cce045f4db3e4). 69 roots.

**Grant edges**, each the game's own (file and method):
- structure: a class's `m_Progression` and `m_Archetypes`; an archetype's `AddFeatures` and `m_ReplaceSpellbook`; a progression's `LevelEntries` (`LevelUpHelper.AddFeaturesFromProgression`); a selection's `m_AllFeatures` and `m_Features`; a race's `m_Features`;
- components: `AddFacts` (`AddFacts.UpdateFacts`), `AddFeatureOnClassLevel` (`Apply`/`IsFeatureShouldBeApplied`), `AddFeatureIfHasFact` (`Apply`), `AddFeatureOnApply` (`OnActivate`, which also runs a granted progression through `LevelUpHelper.UpdateProgression`), `AddFeatureOnAlignment`, `AddFeatureOnSkill`, `MonkNoArmorAndMonkWeaponFeatureUnlock`, and two `AddFactContextActions` shapes in `Activated`: `ContextActionAddFeature` (permanent) and `ContextActionApplyBuff` with `Permanent`.
- Not followed: ability and buff actions (combat time), pet and mount grants (`AddFeatureToPet`, `AddPet`), `AddClassLevels` (companions), recommendations and descriptions.

**Reached: 8,528 blueprints.** 4,871 `BlueprintFeature`, 543 progressions, 443 selections, 122 parametrized features, 163 `BlueprintFeatureReplaceSpellbook`, 2 mythic-spellbook selections, 197 archetypes, 54 classes, 12 races, 20 spellbooks, 145 buffs (permanent ones, through `AddFacts` or a permanent apply), 10 `BlueprintUnitFact`, and 1,211 abilities and 735 activatable abilities (granted, combat time). Edges taken: `m_AllFeatures` 2,151, `AddFacts` 2,105, `LevelEntries` 1,818, archetype `AddFeatures` 984, `AddFeatureOnClassLevel` 580, `AddFeatureIfHasFact` 323, the rest under 200 each.

**Check against 237's mapping** (`237/build/mapping.tsv`): every mapped class, prestige class, path, archetype, race, subrace, background, deity, feat, feat list, selection, mythic ability and mythic feat is reached. Features: 1,771 of 1,778 (not reached: the five Razmiri Infiltrator masks, `BlizzardServantTrirdUpgradeFeature` and `LocustClone`, all granted from ability actions). Options: 768 of 778 (not reached: the ten Dragonblood Shifter breath weapons, same reason). Gear, weapons, spells and spell lists aren't facts and need their own roots (the planner's items: 91, all with enchantments).

The walk also reaches **3,497 feature-like blueprints 237 doesn't map**: 1,085 hidden and carrying a planner-relevant component (the carriers: `DragonLevel2BABOverride`, the per-level stat overrides, …), 1,159 shown with one (per-skill Skill Focus, the `NoProgression` copies, deity features' grants, …), 556 selections and progressions, 697 with none.

## 2. The census

559 component types on the reachable set: 462 on its 6,562 facts (features, selections, progressions, classes, archetypes, races, permanent buffs), 118 on the abilities. Of the facts, 4,785 carry at least one planner-relevant component (1,199 of them also runtime ones), 682 carry only runtime ones and 1,095 none (UI, story, or nothing).

By planner category (`classified.tsv`; blueprints using, among the 6,562 facts):

| Category | Types | Uses | What the planner does with it |
| --- | --- | --- | --- |
| Pick legality (`P`) | 26 | 2,793 | a pick being legal: prerequisites, selection rules |
| Grants (`G-grant`) | 11 | 2,763 | facts held: features, selections, progressions |
| Spellbooks and lists (`G-spells`) | 20 | 728 | caster levels, spell lists, known spells (290's area) |
| Proficiencies, class skills, skill points (`G-prof`, `G-skill`) | 7 | 279 | prerequisites, the sheet, the attack line |
| Mechanics flags (`G-flag`, `AddMechanicsFeature`) | 1 | 83 | Weapon Finesse is the attack line's; the rest runtime |
| Gear-state grants (`G-gear`) | 6 | 47 | a grant while unarmoured or with a shield: gear-dependent |
| Stat bonuses (`N-stat`) | 9 | 397 | ability scores, saves, skills, initiative, BAB |
| Values (`N-value`) | 3 | 321 | the numbers a bonus reads |
| Attack line (`A`) | 38 | 179 | weapon focus, training, damage stat, criticals |
| Caster line (`C`) | 13 | 67 | spell DC, caster level, penetration, concentration |
| Unmodelled numbers | 11 | 974 | ability resources (597), ability DC stats, hit points, hit die |
| Save fixers, recalculation hooks | 10 | 178 | no rule (the engine recomputes anyway) |
| Runtime (`R`) | 352 | 1,493 | combat: triggers, resistances, immunities, conditional saves, auras, AC, CMB |
| UI, story, pets (`X`) | 52 | 923 | nothing |

**The planner-relevant ones by reach** (blueprints among the facts): `AddFacts` 1,803; `PrerequisiteFeature` 763; `AddFeatureOnClassLevel` 504; `PrerequisiteNoFeature` 380; `AddKnownSpell` 345; `ContextRankConfig` 266 (and 572 abilities); `PrerequisiteClassLevel` 262; `AddFeatureIfHasFact` 225; `PrerequisiteNoArchetype` 218; `PrerequisiteFeaturesFromList` 189; `PrerequisiteClassSpellLevel` 188; `AddContextStatBonus` 179; `PrerequisiteStatValue` 167; `AddStatBonus` 163; `PrerequisiteCondition` 160; `PrerequisiteArchetypeLevel` 153; `AddClassSkill` 109; `AddProficiencies` 86; `AddMechanicsFeature` 83; `RemoveFeatureOnApply` 83; `AddFactContextActions` 82; `PrerequisiteAlignment` 77; `AddSpecialSpellList` 76; `LearnSpellParametrized` 73; `PrerequisiteNoClassLevel` 70; `ContextCalculateSharedValue` 53; `PrerequisiteProficiency` 45; `AddSpellbookLevel` 41; `AddBackgroundClassSkill` 39; `LearnSpellList` 38; `WeaponTypeDamageStatReplacement` 32; `AddFeatureOnApply` 31; `PrerequisiteNotProficient` 30; `AddStatBonusIfHasFact` 27; `AddBackgroundWeaponProficiency` 24; `AddFeatureOnAlignment` 22; then `PrerequisiteAnySpellsInSpellbook` 21, `IncreaseSpellDescriptorDC` 19, `ReplaceStatBaseAttribute` 15, `WeaponTraining` 14, `WeaponGroupAttackBonus`/`DamageBonus` 13/14, `ConcentrationBonus` 12, `SpellPenetrationBonus` 12, `NoSelectionIfAlreadyHasFeature` 11, `PrerequisiteFullStatValue` 9, `AddParametrizedFeatures` 9, `ClassLevelsForPrerequisites` 7, `DerivativeStatBonus` 7, `SpellFocusParametrized` 3, `WeaponFocusParametrized` 2, `WeaponSpecializationParametrized` 3, `ImprovedCriticalParametrized` 1, `ExpandedArsenalMagicSchools` 1, `AddGoldenDragonSkillBonus` 1.

**What their fields hold** (`stats.py`):
- `AddStatBonus` targets: skills 103, ability scores 82, AC 40, saves 24, initiative 7, CMB/CMD 12, speed 5, other 13; descriptors Racial 137, None 61, NaturalArmor 34, UntypedStackable 20, Mythic 12, others.
- `AddContextStatBonus`: skills 151, ability scores 55, saves 27, AC 11, initiative 7, BAB 3; values Rank 145, Simple 81, Shared 26, property 9.
- `ContextRankConfig` base values: ClassLevel 371, SummClassLevelWithArchetype 218, CasterLevel 77, StatBonus 67, MythicLevel 57, FeatureListRanks 55, BaseStat 52, CustomProperty 51, FeatureRank 33, Bombs 22, CharacterLevel 8, others under 15. Progressions: AsIs 570, Div2 172, DelayedStartPlusDivStep 58, MultiplyByModifier 43, OnePlusDivStep 41, StartPlusDivStep 39, Custom 36, BonusValue 33, nine others under 16.
- `PrerequisiteCondition`'s conditions: `ContextConditionHasFact` (84 progressions, 5 features, …), `ContextConditionCharacterClass` (with HasFact, 32), `ContextConditionBuffRank` (10: the Extra caps), `ContextConditionIsMainCharacter` (2), and 23 empty.
- Items (the planner's 91): `AddStatBonusEquipment` 138 on enchantments, `AddUnitFeatureEquipment`/`AddUnitFactEquipment` 47 (a wearer fact with its own components), `AllSavesBonusEquipment` 1, weapon and armour enhancement 6.

## 3. The vocabulary

The smallest set whose generic implementation covers the planner's picks and numbers. Each kind is our own form, named for what it does, with the game components it stands for, the code that defines its semantics, how the engine evaluates it, and what the engine already has.

Two layers: an entry's **header** (the blueprint's own fields: they decide structure) and its **components**.

### 3a. The header, by entry kind

| Kind | Fields (our form) | Game | Engine today |
| --- | --- | --- | --- |
| every fact | `ranks`, `groups`, `hidden` | `BlueprintFeature.Ranks`, `Groups`, `HideInUI` | ranks = grant count (243), uncapped |
| selection | `options`, `mode` (`Default`/`OnlyNew`/`OnlyRankUp`), `ignores_prerequisites`, `obligatory` | `BlueprintFeatureSelection.m_AllFeatures`, `Mode`, `IgnorePrerequisites`, `Obligatory`; `CanSelect` | `FeatureSelection`, `FeatList::ignores_prerequisites`, `FeatSlot`, `Repeat` |
| parametrized | `parameter`, `requires_proficiency`, `values_of` (its `m_Prerequisite`) | `BlueprintParametrizedFeature.ParameterType`, `RequireProficiency`, `m_Prerequisite`; `CanSelect` | `ParameterKind`, `FeatForSameValue`, 251 |
| progression | `classes: [(class, offset)]`, `archetypes: [(archetype, offset)]`, `other_classes: Half`, `levels`, `rank_increase` | `BlueprintProgression.m_Classes`, `m_Archetypes` (`AdditionalLevel`), `ForAllOtherClasses` + `AlternateProgressionType`, `LevelEntries`, `m_FeaturesRankIncrease`; `CalcLevel` | `option_levels`, `option_level_archetypes`, `half_other_classes`, `levels_behind`, `skipped_levels`, option rows; 253 |
| class | `skill_points`, `bab`, `saves`, `class_skills`, `spellbook`, `prestige`, `mythic` | `BlueprintCharacterClass` fields, `BlueprintStatProgression` | the same, hand-shaped |
| archetype | `adds`, `removes` rows, `skill_points`, `class_skills` (replace or add), save/BAB overrides, `spellbook` | `BlueprintArchetype.AddFeatures`, `RemoveFeatures`, `AddSkillPoints`, `ReplaceClassSkills`, overrides | the same, hand-shaped |
| race | `features`, `selectable_stat` | `BlueprintRace.m_Features`, `SelectableRaceStat` | `ability_modifiers`, `feat_slots`, … |

The planner's slots are selections in the game: `BasicFeatSelection` lists 276 feats explicitly (`Group` Feat), `FighterFeatSelection` 155, `TeamworkFeat` 12, `MythicAbilitySelection` 67, `MythicFeatSelection` 48. So `FeatSlot::General`, `CombatFeat`, `Category`, `OneOf`, `OptionList` all become a row naming a selection, and a feat list is a selection.

**Nested selections**: an option that is itself a selection opens its slot when picked (`CanSelect` lets an `IFeatureSelection` through at any rank; `NestedFeatureSelectionUtils.AllNestedFeaturesUnavailable` refuses it once none of its options is left). This replaces `Feat::extra_pick_of`, `MythicExtraPick` and a mythic ability's `selections`: Extra Rage Power is `ExtraRagePowerSelection` 0c7f01fbbe687bb4baff8195cb02fe6a, a selection of 38 rage powers.

**Repeats are derived from the header**, as `CanSelect` derives them: a non-selection stops at `ranks`; a selection repeats while it has an option left; a parametrized feature repeats per value; `mode` OnlyNew/OnlyRankUp. `Repeat` leaves the data (249, 250's caps are `ranks` or a `FactRank` condition).

### 3b. The components

Ranked by reach. "Runtime" marks what depends on state the planner can't know.

**1. `Requires(condition, group: All | Any)`** — every `Prerequisite*` (2,633 uses on 1,735 facts). Semantics: `BlueprintFeature.MeetsPrerequisites` (every `All` holds, and one `Any` if there are any), `Prerequisite.Check`; `CheckInProgression` as the code reads it. Conditions:
- `Has(key, rank)`, `HasNot(key)`, `AtLeast(n, keys)` (`PrerequisiteFeature`, `PrerequisiteNoFeature`, `PrerequisiteFeaturesFromList` with `Amount`);
- `ClassLevel(class, level)`, `NoClassLevel(class)`, `ArchetypeLevel(archetype, level)`, `NoArchetype(class, archetype)`, `CharacterLevel(n)`, `MythicLevel(n)`;
- `Stat(stat, value, full)` (`PrerequisiteStatValue.GetStatValue`: a skill reads `BaseValue`, another stat `PermanentValue`; `PrerequisiteFullStatValue`);
- `ClassSpellLevel(class, level)`, `CasterTypeSpellLevel(type, level, spontaneous)`, `CasterType(type)`, `AnySpellsInSpellbook`;
- `Alignment(mask)`, `Proficiency(weapons, armour)`, `NotProficient(…)`, `ParametrizedFeature(feat, value)`, `WeaponCategoryCondition`, `SelectionPossible(selection)`;
- `Condition(tree)` for `PrerequisiteCondition`: `HasFact`, `CharacterClass(class, level)`, `FactRank(key, rank)` under And/Or/Not. That covers every reachable use but 2 `IsMainCharacter` and 23 empty ones (story).
- Dropped and counted, as the game check does: pet, etude, main character, DLC.
- **Engine:** the loader compiles them into today's `Prerequisite` tree, so the walk, the pickers, the reasons and `allows` keep working. `HasNot(self)` stays dropped (`MeetsPrerequisites` refuses a held feature at its `Ranks` already). The planner-only leaves (`Story`, `PickedThrough`, `DeityOffers`, `NoOtherOptionOfGroup`) stay on planner entries only; a game entry gets them from components (`NoOtherOptionOfGroup` is the game's `HasNot` of each other option; `DeityOffers` is `Has` of the deity's `…DomainAllowed` fact, below).
- **Modifiers of prerequisites:** `PrerequisiteClassLevels(fake, actual, modifier, summand, for_selection)` (`ClassLevelsForPrerequisites`, read in `PrerequisiteClassLevel.GetClassLevel` and `PrerequisiteArchetypeLevel`): today's `CountsToward::FeatPrerequisites`. `PrerequisiteStat(old, policy)` (`ReplaceStatForPrerequisites.ResultStat`: class level, magus BAB, another stat). `NoSelectionIf(features, any_from_selection)` (`NoSelectionIfAlreadyHasFeature`, `BlueprintFeatureSelection.IsSelectionProhibited`).

**2. `Grant(keys)`** — `AddFacts` (1,803). `AddFacts.UpdateFacts`: the facts are held while the owner is. **Engine:** the grant count (243) and holds (247, `Held`); a granted selection is held as a fact and opens its slot. A granted ability or activatable isn't an entry: dropped and counted.

**3. `GrantAtClassLevel(feature, class, also, archetypes, level, before)`** — `AddFeatureOnClassLevel` (504). `IsFeatureShouldBeApplied`: the class level, summed with `m_AdditionalClasses`, filtered by `m_Archetypes` (`ReplaceCasterLevelOfAbility.CalculateClassLevel`), at least `Level`, or below it with `BeforeThisLevel`; applied on activation and on every level gained, and removed when it stops holding. **Engine:** a conditional hold evaluated at each level; it is how a deity's favoured weapon reaches a cleric (`ErastilFeature` afc775188deb7a44aa4cbde03512c671: `LongbowProficiency` at cleric or warpriest 1), so `favoured_weapon_proficiency` goes.

**4. `GrantIf(fact, not, feature)`** — `AddFeatureIfHasFact` (225). `Apply` on activation and each level gained; it never removes what it added until the source goes. Runtime only when the checked fact is a buff.

**5. `StatBonus(stat, value, descriptor, multiplier, at_least_zero, if)`** — `AddStatBonus` (163), `AddContextStatBonus` (179), `AddStatBonusIfHasFact` (27, the `if`: facts, all or any, inverted). `OnTurnOn` adds a modifier of the descriptor; stacking by `ModifiableValue.ApplyModifiersFiltered` and `ModifierDescriptorHelper.DefaultStackingDescriptors`; permanence by `Modifier.IsPermanent` (252). Stats the planner sums: the six abilities, BAB, the three saves, initiative, each skill. AC, CMB, CMD, HP, speed and the `Check*` sub-skills have no line: counted, and kept in the data for later lines. Runtime: a bonus with `Restrictions` or under a buff an ability applies.
- **Engine:** today's `BonusInForce` with `counted`, `ability_scores`/`permanent_ability_scores`, the totals. It replaces `bonuses.ron`, `ability_bonuses` (`Fixed`, `Every`, `AtLeast` are stat bonuses; Legend's +4 is untyped `AddContextStatBonus` on `LegendFeature` 7f99961610390044391f300c7ef5e0c8, so `IsPermanent` refuses it for prerequisites: the 252 follow-up dissolves), `bab_at_least_mythic_rank_plus` (below), race `ability_modifiers`.

**6. `Value`** — the value a bonus or a grant reads: `Flat(n)`, `Rank(RankConfig)`, `Shared(SharedValue)`, `Property(formula)`.
- `RankConfig(base, progression, min, max)` — `ContextRankConfig.GetValue` = `ApplyMinMax(ApplyProgression(GetBaseValue))`. Bases the planner knows: `ClassLevel(classes, except)`, `ClassLevelWithArchetype` (sum and max), `CharacterLevel`, `MythicLevel`, `FeatureRank(key)`, `FeatureListRanks(keys)`, `StatBonus(ability)`, `BaseStat(stat)` (an ability's base value, a skill's ranks), `BaseAttack`. Every progression is arithmetic (`ApplyProgression`): all 13 met are implemented. `CasterLevel`, `Bombs`, `ShieldBonus` and armour bases are runtime.
- `SharedValue(dice, bonus, modifier)` — `ContextCalculateSharedValue` (53); the planner needs only its flat part (dice count `One`).
- `Property` — `PropertyCalculatorComponent` and `BlueprintUnitProperty` getters. Implement only the getters met in planner-relevant components, each cited; any other is a gap.
- **Engine:** 254's `steps` are a `RankConfig`'s change points; Skill Focus's +3/+6 (`BonusAmount::TenRanks`) is `SkillFocusAcrobatics` 52dd89af385466c499338b7297896ded's `RankConfig(BaseStat(Mobility), Custom[(9,3),(100,6)])`; `HalfMythicRank` is `MythicLevel` + `Div2`; `AbilityModifier` is `StatBonus`. Perfect Body's "at least 18" is `DragonLevel2StrengthOverride` 9ed55389d2079714eab8495957964af2: `RankConfig(BaseStat(Strength))`, a shared value of −18, multiplier −1, at least zero. Perfect Mind's BAB is `DragonLevel2BABOverride` f4af14e9f4408d44c830d891fee8f4a5: a property of BAB − 15 − the Mythic rank.

**7. `ClassSkill(skill)`, `SkillPoints(per_level)`, `Proficiency(weapons, armour, categories)`** — `AddClassSkill` 109, `AddBackgroundClassSkill` 39, `AddSkillPointPerCharacterLevel` 3 (`HumanSkilled`), `AddProficiencies` 86, `ArcaneArmorProficiency` 10, `AddBackgroundWeaponProficiency` 24, `AddBackgroundArmorProficiency` 8. Each `OnTurnOn`. **Engine:** `class_skills` (state), option rows' `class_skills`, background fields, feature `weapons`, race `weapons`/`skill_points`.

**8. `RemoveOnApply(key)`, `GrantOnApply(key)`** — `RemoveFeatureOnApply` 83 (a stage replacing the one before), `AddFeatureOnApply` 31 (runs a granted progression: `LevelUpHelper.UpdateProgression`). **Engine:** replaces `replaced_by` and option rows that trade a feature.

**9. `Flag(name)`** — `AddMechanicsFeature` 83. Only `WeaponFinesse` changes a planner number (the attack line); the rest are runtime. **Engine:** the attack line's Weapon Finesse test reads the flag rather than a feat key.

**10. Attack-line kinds** — `WeaponFocus(bonus)` (`WeaponFocusParametrized`), `WeaponSpecialization`, `ImprovedCritical`, `CriticalEdge` (`ImprovedCriticalEdgeParametrized`, `ImprovedCriticalMythicParametrized`), `WeaponTraining` (`WeaponTraining`, `WeaponTrainingBonuses`), `WeaponGroupBonus(group, attack|damage, value)`, `AttackStat(stat, …)` (`AttackStatReplacement`), `DamageStat(weapons, stat)` (`WeaponTypeDamageStatReplacement` 32, `DamageGrace`, `AnyWeaponDamageStatReplacement`), `TwoWeapon*` (3). **Engine:** `attack.rs` names Weapon Focus, Greater Weapon Focus, the specializations, Improved Critical and the Trickster's feats by key; it would read the kinds instead. The rest of the 38 (natural limbs, Kensai, unarmed) stay left out.

**11. Caster-line kinds** — `SpellFocus(bonus, mythic_focus)` (`SpellFocusParametrized.GetBonusDC`: doubled while the `m_MythicFocus` fact is held), `SchoolDc(school, value)` (`IncreaseSpellSchoolDC`), `SpellDc(value)` (`IncreaseSpellDC`), `DescriptorDc(descriptor, value)` (`IncreaseSpellDescriptorDC` 19, `IncreaseSpellContextDescriptorDC` 6: by spell descriptor, which the caster line's `SpellTraits` would need), `Concentration(value)`, `SpellPenetration(value)`, `MetamagicDc` (`ArcaneBloodlineArcana`), `ExpandedArsenal` (`ExpandedArsenalMagicSchools.OnEventAboutToTrigger`), `SchoolCasterLevel`. **Engine:** today's `BonusTarget::SpellDc`/`SpellScope`, `BonusAmount::SpellFocusOfAnotherSchool`.

**12. Spellbook kinds** — `SpellbookLevel` (`AddSpellbookLevel` 41), `SpellList`, `KnownSpell`, `LearnSpells`, `SpontaneousConversion`, `OppositionSchool`. The caster level and spell-level access read the first; the rest belong to 290 (spells and spell lists). In the vocabulary from its second stage, evaluated only for the caster level and spell-level access first.

**13. Gear kinds** — `GearStat(stat, value, descriptor)` (`AddStatBonusEquipment.OnTurnOn`, `AddItemModifierUnique` with the item as source), `GearSaves` (`AllSavesBonusEquipment`), `GearFact(key)` (`AddUnitFeatureEquipment`, `AddUnitFactEquipment`: the wearer holds a fact whose components count like any). **Engine:** `GearBonus`/`GearTarget`; the game check's items area already reads these.

**14. Patterns of `AddFactContextActions`** (82) — a permanent `ContextActionAddFeature` or `ContextActionApplyBuff` is a `Grant`; anything else is a named pattern the importer recognizes and cites, or a gap. The one met in the planner's numbers is the Dragon Disciple's and Gold Dragon's "the highest ability" (`DragonLevel1MaxAbilityStrengthFeature` 6a7f8ee85c7f42228c83bb9d2f1711f2 and its siblings: a tree of conditions applying one buff per score): `HighestAbilityBonus(amount)`, today's `AbilityBonus::Highest`.

**15. `GrantWithValue(feat, value)`** — `AddParametrizedFeatures` 9, `AddFeatureParametrized` 1: a parametrized feat granted at one value (`DisenchanterSpellFocusAbjuration` 60ed5ba87ac9e3241adc8212d3f6a24c: Spell Focus (Abjuration)). **Engine:** row `feats_for`, `FeatValue`.

**16. One-blueprint kinds kept as their own**: `GoldenDragonSkills` (`AddGoldenDragonSkillBonus.OnTurnOn`, `GoldenDragonSharedSkillUMD` 4e0b919e4fbd85142bce959fae129d1a: today's `skill_ranks_at_least_mythic_rank`), `SkillAbility(skill, ability)` (`ReplaceStatBaseAttribute` 15), `DerivedStat(ability, stat)` (`DerivativeStatBonus` 7), `GrantWhileGear(state, feature)` (`MonkNoArmorAndMonkWeaponFeatureUnlock`, `HasArmorFeatureUnlock`, …: gear-dependent; the planner holds no armour, so a note until gear models it).

**Left out, shown as notes** (the entry's summary already says them): resources and their uses (597 `AddAbilityResources`), ability DCs and caster levels of abilities, the 352 runtime kinds (`SavingThrowBonusAgainst*`, damage resistance, immunities, triggers, auras, favored enemy and terrain, AC/CMB/CMD/HP), pets. The importer counts them per entry; the data holds no marker for them (see open question 5).

**What the engine already does that maps onto the vocabulary:** feature ranks and the grant count (243, `ranks`, `FeatureRank`); steps (254, `RankConfig` on class or character level); permanent values (252, a stat bonus's descriptor and `IsPermanent`); prerequisites (the compiled tree); held selections (247, `Grant` of a selection, progression rows); filtered pools (243, `Catalog::needs_unheld` over the options' `Requires`); repeats (249, 250: `CanSelect` from the header); option levels (243, `CalcLevel` from the progression header; 253 adds the archetype offset); held facts (`CharacterState::features`, `feats`, `selection_options`, `mythic_abilities`, `mythic_feats`, five maps of what the game holds as one).

## 4. The data shape

In 244's per-blueprint files, an entry holds its planner fields, its header in our form, and `components: [...]`:

```ron
Feature(
    key: "skill-focus-mobility",
    blueprint: "52dd89af385466c499338b7297896ded",
    name: "Skill Focus (Mobility)",
    summary: "…",
    guide: Some("feats#skill-focus"),
    components: [
        StatBonus(stat: Skill("mobility"), value: Rank(base: BaseStat(Skill("mobility")), progression: Custom([(9, 3), (100, 6)]))),
    ],
)
```

Keys, never GUIDs, inside components (the loader and the check resolve both ways through `blueprint`). Defaults left out (`descriptor: Untyped`, `group: All`, `multiplier: 1`), so a line reads like the planner's other fields.

**Hand-shaped fields and where they go:**

| Field today | Becomes |
| --- | --- |
| `prerequisite` (feat, option, class, archetype, mythic) | `Requires` components; kept on planner entries |
| class `levels`, archetype `levels`, option `levels` rows | progression `levels` in the header (rows name entries; `feat_slots` become selections) |
| `FeatSlot` (`General`, `CombatFeat`, `Category`, `OneOf`, `OptionList`) | the game's selections, named by rows |
| `FeatList` | a selection (`ignores_prerequisites`, `options`) |
| `steps` | a `Scales(RankConfig)` component, the change points computed by the engine (no derived list) |
| `bab_at_least_mythic_rank_plus` | `StatBonus(BAB, Property(…))` on the carrier, or a cited kind if the property subset isn't built |
| `skill_ranks_at_least_mythic_rank` | `GoldenDragonSkills` |
| `ability_bonuses` | `StatBonus`, `HighestAbilityBonus` |
| `favoured_weapon_proficiency`, feature `weapons`, race `weapons`, background `weapons` | `Proficiency`, `GrantAtClassLevel` on the deity |
| background `class_skills`, option row `class_skills` | `ClassSkill` |
| race `ability_modifiers`, `skill_points` | `StatBonus` (Racial), `selectable_stat`, `SkillPoints` |
| `bonuses.ron` (`ItemBonus`) | components on their entries (`StatBonus`, `SpellFocus`, …) |
| gear `bonuses` | `GearStat` on the item's enchantments |
| `Repeat`, `parameter`, `extra_pick_of`, `MythicExtraPick`, mythic ability `selections` | the header (`ranks`, `parameter`, nested selections) |
| `option_levels`, `option_level_archetypes`, `half_other_classes`, `levels_behind`, `skipped_levels` | the progression header (`classes`, `archetypes`, `other_classes`) |
| `replaced_by`, `replaces_row_of` | `RemoveOnApply`, `GrantAtClassLevel(before)` |
| `counts_as: FeatPrerequisites` | `PrerequisiteClassLevels` |
| `deity_domains`, `DeityOffers`, deity `domains` | the deity's `Grant` of `…DomainAllowed` facts and the domain options' `Requires(Has(…Allowed))` |
| `filtered_by` | derived by the engine from the options' `Requires` (kept as a cache only if the pickers need it) |
| `group` (`OptionGroup`) | `Requires(HasNot)` of each other option |

**What stays hand-shaped** (planner concepts): `key`, `name`, `summary`, `guide`, `aliases`, `interactions`, the data notes, the `planner/` entries (with their hand-shaped `prerequisite` and rows), `daily_choice` (a sheet note), `MythicPath::switch` and `Progression::mythic_ascensions` (the planner's model of path switches, which the game spreads over etudes and dialogue), `Advancement` and `CountsAs` beyond feat prerequisites (prestige advancement is the class's `AddSpellbookLevel`, checked once the spellbook kinds land), the Story leaves, `option_row`, `level_from` (a planner rule for Impossible Domain and Maneuver Mastery until each is checked against its progression), and the `kind` of an entry for links (feat, option, mythic ability, …: the codec's).

## 5. The importer's target

293 writes, for every entry with a blueprint:
- the header fields of §3a, from the blueprint's own fields;
- `components`, each game component through one normaliser into its vocabulary kind, references resolved to keys; a component with no kind is dropped and counted per type (runtime) or listed as a gap (planner-relevant but unexpressed: a property getter not built, an unknown `AddFactContextActions` shape);
- new entries for the reachable blueprints a component, a row or an option names that have none (the carriers: hidden features, permanent buffs), with no summary but the importer's citation; abilities are never entries.

It never touches the planner fields of §4's last paragraph, and never writes game text.

**The game check simplifies** to two comparisons sharing the normaliser with the importer: header against header, components against components (as sorted multisets per entry). Its areas today (prerequisites, class tables, progression rows, option lists, mythic ranks, items) each become "header differs" or "components differ", grouped by kind of component. The hand-written normal forms in `areas/prerequisites.rs` and `areas/items.rs` (the `All`/`Any` flattening, the alignment mask, the stacking of items) move into the vocabulary's own semantics, where the engine uses them too. What stays: the progression-row pairing (archetype application by `ProgressionData.CalculateLevelEntries`), and the mapping area. After an importer run the check reports only gaps and planner entries.

## 6. The feature tickets

- **255** (a parametrised feat at one value): **dissolves into data**. Skill Focus is `SkillFocusSelection` c9629ef9eebb88b479b2fbc5e836656a, a selection of 11 per-skill features; the bloodline lists, Adaptability (26a668c5a8c22354bac67bcd42e09a3f) and the Chirurgeon's and Enlightened Philosopher's grants name those features; the Disenchanter's Spell Focus (Abjuration) is `GrantWithValue`; the ranger's level-10 lists follow the game. Engine work left: a link alias from `skill-focus` with a skill value to the per-skill feature (244's `aliases`), and nested selections (core).
- **257** (Bloodline Ascendance): **dissolves into data**. `BloodlineAscendance` ce85aee1726900641ab53ede61ac5c19 is a selection gated by `Requires(Has(…BloodlineSelection or …Progression), Any)` (247's held selections); the Empyreal's row 20 and its classes (sorcerer only) are progression header data.
- **258** (Mythic Spell Focus): **dissolves into the caster-line kinds**. `SpellFocus` 16fa59cc9a72a6043b566b49184f53fe carries `SpellFocusParametrized(BonusDC 1, m_MythicFocus: SpellFocusMythicFeat)`; the generic `SpellFocus` kind doubles it (`GetBonusDC`). Expanded Arsenal is the `ExpandedArsenal` kind, built from `ExpandedArsenalMagicSchools.OnEventAboutToTrigger`; its "two facts" question is answered there.
- **259** (Witch Wandering Hex): **dissolves into data**. `WitchWanderingHex` fb8490d1721bd5a4883c3b6559310351 is a feature with `Grant(ability)`, a resource and three `Any` prerequisites; slots come only from selections, so the engine's list of abilities that give a pick goes with the vocabulary.
- **260** (Greater Exploits): **dissolves into data**. `ArcanistGreaterExploitsFeature` c7536b93f17c70d4fa3a8cf9aa76bfb7 is a granted feature; the six greater exploits sit in the exploit selections with `Requires(Has(it))`.
- **Gap: mythic abilities that are selections repeat** (Extra Domain, Second Bloodline, Second Mystery): **dissolves** with `CanSelect` from the header (core).
- **Gap: Extra Hex (Shaman), the Vivisectionist's Extra Discovery**: **data** (the importer writes both selections).
- **Follow-up: feature bonuses toward ability prerequisites**: **dissolves** (`StatBonus` descriptor and `IsPermanent`).
- **Follow-up: Perfect Mind's skill floor as its own modifier**: **engine, small**: the `GoldenDragonSkills` kind makes it a modifier rather than "Ranks".
- **Perfect Mind's BAB**: **dissolves** if the property subset holds BAB, a constant and the Mythic rank; otherwise a cited kind.
- **243's out-of-scope items** (Fey Magic's spell parameter, Enchanting Smile's class-level number, Strength of the Abyss's `AddLevel2`): data under `RankConfig` and `GrantAtClassLevel`, checked when imported.

## 7. Build plan

Each stage is one ticket; a form change moves no behaviour (proved), a value change comes only from the game (proved by the check).

1. **295, the vocabulary core** (rules only). `libs/rules/src/component.rs`: the kinds of §3b 1–9 and 15 (requires, grants, stat bonuses with values, class skills, proficiencies, skill points, remove/grant on apply, flags), their serde form, and their evaluation: `Requires` compiled into `Prerequisite`; grants and stat bonuses evaluated by the walk on the holds it already has. An entry may carry `components` beside its hand fields; `GameData::check` refuses a field and the component that replaces it on one entry. **Tests:** rules tests on hand-made components, each kind's semantics cited by file and method; no data change, so the snapshot check, the corpus hash and every real-data test stay as they are.
2. **296, prerequisites as components.** A throwaway script (outside the repo) rewrites every blueprint entry's `prerequisite` into `Requires` from the current data, not the game: the same values, a new form. **Proof:** the 2,163 corpus Builds' `Debug` hash and every link decode identical; the self-checks and gold builds unchanged; `allows` identical.
3. **297, numbers as components.** `bonuses.ron`, `ability_bonuses`, the BAB and skill floors, `steps` (as `Scales`), gear bonuses and the attack and caster lines' named feats move to components (kinds 10, 11, 13, 14, 16). Same proof; the sheet numbers of the corpus identical.
4. **298, selections as the game has them.** Feat slots and feat lists become selections; nested selections replace `extra_pick_of`, `MythicExtraPick` and ability selections; `Repeat` derived from the header; progression header fields replace `option_levels` and kin. A link-compatibility test over the corpus links (as 243's). Same proof.
5. **293, the importer writes headers and components** from the blueprints; the game check compares components to components. **Proof:** the check's findings before the run are each a change after it; the corpus proof's `explain.txt` groups every Build that moved under a game-check finding; the gold builds' changes go to the user.
6. **Then:** the spellbook kinds (290's area), the property subset, each named `AddFactContextActions` pattern, and the remaining attack-line kinds, each a small engine ticket with its citation.

Stages 2–4 can run one after another in one lane (they touch the data files and `game_data.rs`); 295 can start as soon as 244 merges.

### Open questions (recommendations first)

1. **Which reachable blueprints become entries.** (a) Every reachable feature-like blueprint, hidden ones flagged; (b) only those a component, row or option names or that carry a vocabulary component; (c) inline carriers into their granter. **Recommended (b)**: one file per blueprint stays true (244), the graph stays the game's, and nothing derived is stored; at most about 2,800 more entries than today (the 2,244 unmapped carriers and shown features with a vocabulary component, plus the selections and progressions they need), which 295 measures in the browser's JSON before 293.
2. **Evaluate components directly or compile them.** **Recommended: compile** `Requires` into today's `Prerequisite` tree and stat bonuses into `BonusInForce`, so the walk, pickers, reasons and `allows` stay one code path.
3. **One facts map in the state.** The game holds every fact in one collection; the planner has five maps by kind. **Recommended:** unify in 298 (one map from key to rank and values), keeping the kinds as views for links and the sheet.
4. **`AddFeatureIfHasFact`'s one-way add.** The game never removes what it added until its source goes. **Recommended:** follow the code; a rules test pins it.
5. **Runtime components in the data.** Markers (`Runtime("AddDamageResistanceEnergy")`) would feed each pick's "leaves out". **Recommended: no markers**: the summaries say them and the check counts them; revisit if `leaves_out` per pick is wanted.
6. **The property subset.** **Recommended:** only the getters met in planner-relevant components (stat value, a constant, the Mythic rank, a class level), each cited; anything else a gap.
7. **Skill Focus's shape.** **Recommended:** the game's selection of 11 features, with the link alias from the parametrised key; Mythic Skill Focus's same-value prerequisite becomes `Has` of the matching per-skill feature (244's known 11).

## 8. The ticket draft

`295-the-component-vocabulary.md` beside this file.
