# A guide to `decompiled/` (WrathDecompiled at 6ce4a6b, patch 2.7.0x)

For an agent who will reimplement the game's character rules in another language (TypeScript, say). Written 2026-11-19 from WrathDecompiled commit `6ce4a6ba6d35039cc73ba16faf70c2d9952a63c2` ("2.7.0x", 2025-05-15), the pin in `scripts/game-sources.pins`.

## The rule that comes first

**The game's code and text never enter the repo, not even quoted.** This guide names files, namespaces, classes, methods and fields, and describes what they do in its own words. It holds no code from the folder: no method bodies, no snippets, no game strings. Where it sketches a rule's logic, the sketch is a summary of a few lines, not a transliteration.

A port follows the same rule. **Reimplement behaviour from understanding and citations; never translate the decompiled code line by line.** This is how the planner works today: its Rust states the rule in its own terms and cites the game's file and method with the WrathDecompiled commit (`libs/rules/src/component.rs` is the densest example). A fact from the game's data is cited by blueprint name and GUID; a rule from the code by file and method (CLAUDE.md, "Game sources"; map Notes, ticket 234).

Paths below are relative to `decompiled/`. The planner's own citations are mostly relative to `Assembly-CSharp/Kingmaker/` (they write `UnitLogic/FactLogic/AddFacts.cs`, a few `Kingmaker/...`); prefix `Assembly-CSharp/Kingmaker/` to open one.

## 1. Layout

`decompiled/` is a shallow clone, git-ignored and only in the main checkout (an agent in a worktree reads it by absolute path). It holds one folder per .NET assembly, each with a `.csproj` and its namespaces as folders: 14,725 `.cs` files, about 82 MB.

| Assembly | Files | What it holds | For character rules |
| --- | --- | --- | --- |
| `Assembly-CSharp` | 10,150 | The game. `Kingmaker/` is everything that matters; beside it `AK*` (Wwise audio), `TurnBased/`, `UnityEngine/`, `IL2CPP/` | **Yes**, `Kingmaker/` only |
| `Assembly-CSharp-firstpass` | 2,913 | Third-party plugins: DOTween, Rewired, Steamworks, Dreamteck, RootMotion, QuickGraph, Razer Chroma | No |
| `Owlcat.Runtime.*` | ~760 | Owlcat's engine layer: core utilities, UI framework, visuals, validation, UniRx | Rarely (a helper's meaning) |
| `Owlcat.SharedTypes`, `Core.*`, `Kingmaker.GameInfo`, `Utility.*` | ~60 | Shared enums, cheats console, overlays, crash reporting | No |
| `DungeonArchitect.*` | ~840 | Procedural dungeon generation | No |

Within `Assembly-CSharp/Kingmaker/` (file counts in brackets):

- **Read:** `Blueprints/` (783: the blueprint types, `Classes/`, `Classes/Prerequisites/`, `Classes/Selection/`, `Classes/Spells/`, `Root/`, `JsonSystem/`), `UnitLogic/` (1,161: facts, features, progression data, the level-up controller, `FactLogic/` components, `Mechanics/` for context values and rank configs), `Designers/Mechanics/` (the designer-facing components: `Facts/` 408 files, `Buffs/`, `Prerequisites/`, `EquipmentEnchants/`, `Recommendations/`), `EntitySystem/` (facts, `Stats/`), `Enums/` (`ModifierDescriptor`, weapon categories), `RuleSystem/` (the combat-time rules), `Items/`.
- **Ignore:** `UI/` (3,036, though its view models show how the game *displays* a rule), `Visual/`, `View/`, `Sound/`, `Kingdom/`, `Armies/`, `Crusade/`, `Globalmap/`, `AreaLogic/`, `Dungeon/`, `DialogSystem/` (except the mythic story gates, below), `Tutorial/`, `QA/`, `Settings/`, `AI/`, `Cheats/`, `Achievements/`.

Unity, Newtonsoft.Json, UniRx and the other libraries are referenced by the `.csproj` files but their code is not in the folder.

## 2. The blueprint system

### Types and loading

- `Assembly-CSharp/Kingmaker/Blueprints/SimpleBlueprint.cs`: the root of every blueprint. Holds `name` and `AssetGuid` (a `BlueprintGuid`, `Blueprints/BlueprintGuid.cs`).
- `Blueprints/BlueprintScriptableObject.cs`: adds the component array (`Components`, read through `ComponentsArray`) and, on `OnEnable`, points each component's `OwnerBlueprint` back at the blueprint.
- `Blueprints/BlueprintComponent.cs`: the base of every component. A component has a `name` (unique within its blueprint) and a `Disabled` flag; `EntitySystem/EntityFact.cs` skips disabled components when it builds a fact's runtime components.
- `Blueprints/JsonSystem/BlueprintsCache.cs`: the shipped game reads a binary pack (`blueprints-pack.bbp`) with a table of GUID → offset, and `Load` deserialises a blueprint on first request, then calls its `OnEnable`. `Blueprints/ResourcesLibrary.cs` `TryGetBlueprint` is the front door.
- `Blueprints/BlueprintReferenceBase.cs`, `BlueprintReference.cs` and the many typed `*Reference` classes (`BlueprintFeatureReference`, `BlueprintCharacterClassReference`, …): a field that names another blueprint stores only a GUID and resolves it lazily through the cache. `ReferenceArrayProxy` wraps arrays of them.
- `Blueprints/Root/BlueprintRoot.cs` and `Root/ProgressionRoot.cs`: the singleton root. `ProgressionRoot` lists the playable classes (`m_CharacterClasses`), mythic classes (`m_CharacterMythics`), races, the feats progression (`m_FeatsProgression`), the starting and Legend mythic classes, the XP tables and the deity selection. A port's reachable set starts here (`research/component-vocabulary-2026-11-15.md` §1 walked it: 8,528 blueprints).

### From the JSON to the class

`blueprints/` is BubblePrints' dump of the same blueprints as JSON. Read it with `cargo run -p game-check -- show <name|guid>` (resolves references, drops text).

- **`$type`** is `"<type GUID>, <ClassName>"`. The type GUID is the class's `[TypeId(...)]` attribute; `Blueprints/JsonSystem/GuidClassBinder.cs` binds one to the other. To find the class: `grep -rn 'TypeId("<guid>")' decompiled/`, or search the class name.
- **Keys are C# field names**, including private ones (`m_Facts`, `m_AllFeatures`, `m_Classes`), unless a `[JsonProperty(PropertyName=...)]` renames it. Properties are not serialised; `[JsonIgnore]` fields are absent. `[FormerlySerializedAs]` is a Unity attribute and does not change the JSON key.
- **Inheritance flattens:** a `BlueprintProgression` object carries its own fields plus those of `BlueprintFeature`, `BlueprintFeatureBase`, `BlueprintUnitFact` and `BlueprintScriptableObject`. Find a field by walking up the class chain.
- **References** are strings `"!bp_<guid>"` in the raw dump (`Blueprints/JsonSystem/Converters/BlueprintReferenceConverter.cs`); `game-check show` prints them as `<name> <guid>`.
- **Components** are the `Components` array; each has its own `$type`, its fields, and editor artefacts (`name` like `$AddFacts$<uuid>`, `PrototypeLink`, `m_Overrides`) that carry no rule.
- **Text** fields (`m_DisplayName`, `m_Description`) are localisation keys; `show` replaces them with `<localized text>`. They never go in the repo.
- **Enums** are written by name (`"Group": "All"`, `"Stat": "Strength"`), so the enum file gives the full set of values (`EntitySystem/Stats/StatType.cs`, `Enums/ModifierDescriptor.cs`, `Blueprints/Classes/FeatureGroup.cs`).

So a JSON field leads to its class through `$type`, and to its consumer by searching the field name or the C# property that wraps it (`m_Features` is read through `Features`).

### How components act

Most rule components derive from `UnitLogic/UnitFactComponentDelegate.cs`, itself from `EntitySystem/EntityFactComponentDelegate.cs`, which offers lifecycle hooks: `OnActivate` / `OnDeactivate` (the fact is added or removed), `OnTurnOn` / `OnTurnOff` (the fact's effects switch on or off), `OnRecalculate`, `OnPostLoad`. Others implement event handlers: `IOwnerGainLevelHandler` (each level gained), `IUnitCalculateSkillPointsOnLevelupHandler`, and the rulebook handlers below. Prerequisites are plain `BlueprintComponent`s that the level-up code calls directly. A blueprint becomes a *fact* on a unit (`EntitySystem/EntityFact.cs`, `UnitLogic/UnitFact.cs`, `UnitLogic/Feature.cs`); its components run only through that fact.

## 3. The character-rules core

### Classes, progressions and level-up

- `Blueprints/Classes/BlueprintCharacterClass.cs`: hit die, skill points, class skills, BAB and save tables (`BlueprintStatProgression`, `Classes/BlueprintStatProgression.cs`), `m_Progression`, `m_Spellbook`, `m_Archetypes`, `PrestigeClass`, `IsMythic`, `IsHigherMythic`. `MeetsPrerequisites` caps a class at 20 levels and a prestige class at 10, forces the next obligatory mythic class when one is queued, then evaluates prerequisites (below); `Check` lets an archetype's alignment prerequisite replace the class's.
- `Blueprints/Classes/BlueprintProgression.cs`: a feature with `LevelEntries` (`Classes/LevelEntry.cs`: a `Level` and its `m_Features`). `CalcLevel` is the progression's own level: the sum of the listed classes' levels plus each `AdditionalLevel` (archetype entries apply to classes that have the archetype), plus the ranks of the facts in `m_FeaturesRankIncrease`; `ForAllOtherClasses` or `m_AlternateProgressionClasses` add other classes' levels, halved when `AlternateProgressionType` is `Div2`; with no classes or archetypes it is the character level. `GiveFeaturesForPreviousLevels` and `m_ExclusiveProgression` change when entries are granted. Tickets 253 and 304 studied this.
- `UnitLogic/ProgressionData.cs`: a unit's copy of one progression, with its level and archetypes; `GetLevelEntry` returns the entries with archetype removes and adds applied.
- `UnitLogic/UnitProgressionData.cs`: the unit's classes (`ClassData`, `UnitLogic/ClassData.cs`), character and mythic levels, selections made (`Selections`, `AddSelection`), `MaxCharacterLevel` (20, or 40 with Legend), `AddClassLevel`, `CanAddArchetype`, `ReplaceAllMythicLevelsWithMythicHeroLevels`.
- `UnitLogic/Class/LevelUp/LevelUpController.cs`: the level-up session. It keeps a list of `ILevelUpAction`s sorted by `LevelUpActionPriority` (`Actions/LevelUpActionPriority.cs`: race, class, archetype, class mechanics, spellbook, attributes, skills, … features, spells, alignment) and, on every change, rebuilds a preview unit by replaying them in order (`UpdatePreview`, `ApplyLevelUpActions`). `LevelUpState.cs` is the session's state: next character, class and mythic levels, the open `FeatureSelectionState`s, skill and attribute points.
- `UnitLogic/Class/LevelUp/Actions/`: one class per step. `SelectClass.Apply` adds the class level and gives an attribute point at every fourth non-mythic character level. `ApplyClassMechanics.Apply` adds BAB, saves, hit points and class skills and updates every progression the unit holds. `LevelUpHelper.UpdateProgression` grants the entries between the old and new progression level; `AddFeaturesFromProgression` adds each feature whose `CheckInProgression` prerequisites hold, opens a selection when the feature is one, and recurses into nested progressions. `ApplySkillPoints` and `LevelUpHelper.GetTotalSkillPoints`: class skill points per level, plus a halved Intelligence modifier that alternates rounding by level, never less than one point per level. `AddArchetype`, `SelectFeature`, `ApplySpellbook`, `SelectSpell` are the rest.

### Feature selections and parametrized features

- `Blueprints/Classes/Selection/BlueprintFeatureSelection.cs`: `m_AllFeatures` (what `CanSelect` accepts; `m_Features` is a UI subset), `Mode` (`Default`, `OnlyNew`, `OnlyRankUp`), `Group`, `IgnorePrerequisites`, `ExceptWhiteListed`, `Obligatory`. `CanSelect`: the item is in the pool; its prerequisites hold unless the selection ignores them (with `ExceptWhiteListed`, only prerequisites marked `IsFeatureSelectionWhiteList` are checked); its rank is below `Ranks` (a selection as an item is exempt); the mode's rank rule; and not every nested choice inside it is closed (`NestedFeatureSelectionUtils.cs`). `IsSelectionProhibited` reads a `NoSelectionIfAlreadyHasFeature` component. Tickets 249, 302, 303, 306d.
- `Selection/BlueprintParametrizedFeature.cs`: a feat taken for a parameter (`FeatureParameterType`: weapon category, spell school, skill, spell, a blueprint from a list). Its `CanSelect` refuses a parameter already taken, checks the `m_Prerequisite` feat for the same parameter (Greater Weapon Focus needs Weapon Focus for that weapon; spell specialisation checks by the spell's school), proficiency when `RequireProficiency`, components implementing `IParamPrerequisite`, and the parameter feature's own prerequisites. Tickets 249, 251.
- `UnitLogic/Class/LevelUp/FeatureSelectionState.cs`: one open slot, with its source, level and parent (nested selections); `IsSelectedInChildren` stops a feature counting as held while it is itself being picked in the same chain.
- `Actions/SelectFeature.cs` `Apply`: records the pick, adds the fact (with its parameter), opens the nested selection the item is, swaps the spellbook for a `BlueprintFeatureReplaceSpellbook` item, and runs a granted progression. Tickets 248 (no order among one level's picks) and 306aa.

### Prerequisites

- `Blueprints/Classes/Prerequisites/Prerequisite.cs`: the base, with `Group` (`All`, `Any`, `ForcedTrue`), `CheckInProgression`, `HideInUI`, `IsFeatureSelectionWhiteList`; `Check` returns true early when `ConsiderFulfilled` says so.
- **How groups combine** (`BlueprintFeature.MeetsPrerequisites`): every `All` prerequisite must hold, and if there is any non-`All` prerequisite, at least one of them must hold. In a feature `ForcedTrue` falls into the same bucket as `Any`; only `BlueprintCharacterClass.MeetsPrerequisites` treats a passing `ForcedTrue` as enough on its own. A feature granted by a progression checks only its `CheckInProgression` prerequisites, and fails when already at its `Ranks` unless it is a selection.
- The prerequisite types, one file each in the same folder: `PrerequisiteFeature`, `PrerequisiteFeaturesFromList` (n of a list), `PrerequisiteNoFeature`, `PrerequisiteStatValue`, `PrerequisiteClassLevel`, `PrerequisiteArchetypeLevel`, `PrerequisiteNoClassLevel`, `PrerequisiteNoArchetype`, `PrerequisiteCharacterLevel`, `PrerequisiteMythicLevel`, `PrerequisiteAlignment`, `PrerequisiteCasterType`, `PrerequisiteCasterTypeSpellLevel`, `PrerequisiteClassSpellLevel`, `PrerequisiteProficiency`, `PrerequisiteNotProficient`, `PrerequisiteParametrizedFeature`, `PrerequisiteCondition`, `PrerequisiteSelectionPossible`, `PrerequisiteIsPet`, `PrerequisiteMainCharacter`, `PrerequisiteEtude`, among others; plus `Designers/Mechanics/Prerequisites/PrerequisiteFullStatValue.cs`.
- **What a prerequisite reads** (ticket 252): `PrerequisiteStatValue` reads a skill's ranks (`BaseValue`) and an ability's `PermanentValue` (base plus racial modifiers plus an item's enhancement bonus, not an inherent bonus: see `ModifiableValueAttributeStat.CalculatePermanentValue` and `ModifiableValue.Modifier.IsPermanent`), after any `ReplaceStatForPrerequisites` (`Designers/Mechanics/Facts/ReplaceStatForPrerequisites.cs`). `PrerequisiteClassLevel` adds the levels a `ClassLevelsForPrerequisites` component lends (`Designers/Mechanics/Facts/ClassLevelsForPrerequisites.cs`).
- `Prerequisites/IgnorePrerequisites.cs` is a context flag (cheats and scripted level-ups), not the selection field of the same name. `BlueprintFeatureSelection.IsIgnorePrerequisites` also honours the main character's Trickster flag for the basic feat selection.

### Archetypes

`Blueprints/Classes/BlueprintArchetype.cs`: `AddFeatures` and `RemoveFeatures` (arrays of `LevelEntry`), `m_ReplaceSpellbook` / `RemoveSpellbook`, replacement BAB and save tables, class skills, `AddSkillPoints`, caster-type changes. `UnitLogic/ProgressionData.cs` `GetLevelEntry` / `CalculateLevelEntries` merges them into the class progression: an archetype's removes come out of a level's list and its adds go in. `ProgressionData.CanAddArchetype` refuses two archetypes that both touch the spellbook and an archetype whose removes are no longer in the merged list (the archetype-compatibility rule). `Actions/AddArchetype.cs` and `UnitProgressionData.AddArchetype` apply one; `BlueprintArchetype.MeetsPrerequisites` checks its own components.

### Mythic paths and ranks

Mythic paths are classes with `IsMythic` (listed in `ProgressionRoot.m_CharacterMythics`). `UnitProgressionData.AddClassLevel` counts a mythic class level toward `MythicLevel` (capped at 10 in `CanLevelUpMythic`) and a normal one toward `CharacterLevel`. `ObligatoryMythicClassesQueue` / `NextObligatoryMythicClass` force the story's path order. A class with `IsHigherMythic` (the later-tier paths) triggers `ReplaceAllMythicLevelsWithMythicHeroLevels`: the earlier path's levels become levels of the starting mythic class, keeping the mythic feat and ability picks (the planner's Path switch, `libs/rules/src/engine/switch.rs`). `FeatureGroup.MythicFeat` and `MythicAbility` mark the mythic selections; `SystemMechanicsRoot` names them. Which paths a player may take is decided by the story (`DialogSystem/Blueprints/BlueprintMythicsSettings.cs`, `MythicClassLockComponent`), not by rules a planner can check.

### Stats and modifiers

- `EntitySystem/Stats/ModifiableValue.cs`: a stat's `BaseValue`, its `Modifier`s grouped by `ModifierDescriptor`, and the computed `ModifiedValue` and `PermanentValue` (`UpdateValue`, `ApplyModifiersFiltered`). Subclasses per stat kind: `ModifiableValueAttributeStat`, `ModifiableValueSkill`, `ModifiableValueSavingThrow`, `ModifiableValueArmorClass`, …; `CharacterStats.cs` holds them; `StatType.cs` names them.
- **Stacking:** a modifier stacks when its descriptor is in `Enums/ModifierDescriptorHelper.cs` `DefaultStackingDescriptors` (untyped, racial, dodge, feat, penalty, focus, natural armour and a few more) or its `StackMode` forces it. Within one non-stacking descriptor only the largest bonus and the largest penalty count; stacking ones all add. Armour and encumbrance penalties have their own rule.
- Components that add modifiers: `UnitLogic/FactLogic/AddStatBonus.cs` (value times the fact's rank), `AddContextStatBonus.cs` (a context value, often from `ContextRankConfig`), `Designers/Mechanics/Buffs/AddStatBonusIfHasFact.cs`, `Designers/Mechanics/EquipmentEnchants/AddStatBonusEquipment.cs` (gear).
- `UnitLogic/Mechanics/Components/ContextRankConfig.cs`: how a value scales (`ContextRankBaseValueType`: class level, character level, a fact's rank, a stat, …; then a progression formula, then min and max). Ticket 254.

### Spellbooks

`Blueprints/Classes/Spells/BlueprintSpellbook.cs` (spells-per-day and spells-known tables, `BlueprintSpellsTable`; `m_SpellList`; `CastingAttribute`; `Spontaneous`; `CasterLevelModifier`; `IsMythic`), `BlueprintSpellList.cs`. Runtime: `UnitLogic/Spellbook.cs` (`BaseLevel`, `CasterLevel` = base plus mythic contribution, `EffectiveCasterLevel` adds the `BonusCasterLevel` stat, `MaxSpellLevel` from the per-day table). `Actions/ApplySpellbook.cs` raises the book each class level unless the class's `SkipLevelsForSpellProgression` lists that level, and merges a replaced book's known spells. Prestige classes advance an existing book through a `BlueprintFeatureReplaceSpellbook` pick (`Classes/BlueprintFeatureReplaceSpellbook.cs`); mythic spellbooks through `BlueprintFeatureSelectMythicSpellbook`. `Designers/Mechanics/Facts/AddSpellbookLevel.cs` adds levels from features.

### Feature ranks

`BlueprintFeature.Ranks` is the cap. `UnitLogic/FeatureCollection.cs` `AddFact` adds a rank to a held feature instead of a second copy (a parametrized feature with a new parameter is a new fact); `UnitLogic/Feature.cs` `AddRank`, `RemoveRank`, `SetSource`, `RankToSource`. `FeatureCollection.GetRank` is what prerequisites and selections read. Ticket 243.

### Grants

- `UnitLogic/FactLogic/AddFacts.cs`: while its fact is active, the listed facts are held (`OnActivate`, `UpdateFacts`); note its difficulty gate (`HasDifficultyRequirements`, `MinDifficulty`).
- `Designers/Mechanics/Facts/AddFeatureOnClassLevel.cs`: gives a feature once a class's level (with `m_AdditionalClasses` and `m_Archetypes`, through `ReplaceCasterLevelOfAbility.CalculateClassLevel`) reaches `Level`, or until it does with `BeforeThisLevel`; re-run on every level gained.
- `Designers/Mechanics/Facts/AddFeatureIfHasFact.cs`, `AddFeatureOnApply.cs` (one-shot, no removal), `RemoveFeatureOnApply.cs`, `UnitLogic/FactLogic/AddParametrizedFeatures.cs`, `AddFeatureOnAlignment`, plus the proficiency, class-skill and mechanics-flag components (`AddProficiencies`, `AddClassSkill`, `AddMechanicsFeature`) in `UnitLogic/FactLogic/`.
- `research/component-vocabulary-2026-11-15.md` §2 is a census of the 559 component types on the reachable set, sorted into planner categories; start there before reading components one by one.

## 4. Entry points for tracing a rule

1. **From a blueprint to its type:** `cargo run -p game-check -- show <name|guid>` gives `$type`; open `Blueprints/**/<ClassName>.cs`, or find it with `find decompiled -name '<ClassName>.cs'`. `granted-by` and `refs` say what grants a blueprint.
2. **From a component to its behaviour:** open its class; look for overridden lifecycle hooks (`OnActivate`, `OnTurnOn`, `Apply`) and handler interfaces in its declaration.
3. **From a field to its consumer:** search the field and its property across the code.
4. **From a rule to who decides it:** start at `LevelUpController`, `LevelUpHelper`, `BlueprintFeature.MeetsPrerequisites` or `BlueprintFeatureSelection.CanSelect` and follow calls.

Search patterns (plain `grep -rn`; in this shell `grep` is ugrep, which rejects some regex forms such as `(\b|,)`, so keep alternations simple and quote globs):

```sh
D=/Users/thaynamarcela/code/wotr-planner/decompiled/Assembly-CSharp/Kingmaker
grep -rn 'TypeId("d79ad4ed064aa4f43ace0e9c238fb9b9")' "$D"         # a $type GUID to its class
grep -rn 'class [A-Za-z]* : Prerequisite' "$D/Blueprints/Classes/Prerequisites"  # every prerequisite type
grep -rln 'GetComponent<NoSelectionIfAlreadyHasFeature>' "$D"       # who consumes a component
grep -rln 'SelectFactComponents<ClassLevelsForPrerequisites>' "$D"  # facts' components read across the unit
grep -rn 'IOwnerGainLevelHandler' "$D/Designers" "$D/UnitLogic"    # components that run on each level gained
grep -rn 'IInitiatorRulebookHandler<RuleCalculateAttackBonus' "$D"  # what feeds one combat rule
grep -rn 'ModifierDescriptor.Insight' "$D/EntitySystem" "$D/UnitLogic"  # special cases of one bonus type
find "$D" -name '*.cs' -path '*Prerequisites*' | sort             # browse by folder
```

## 5. The planner today

The planner's Rust in `libs/rules/` already models each area, with its own vocabulary (`GLOSSARY.md`) and the game cited beside the code:

| Area | Planner | Game files its tickets and code cite |
| --- | --- | --- |
| Progressions, class levels, level steps | `engine/walk/level.rs`, `game_data.rs` (`Progression`, `Class`) | `BlueprintProgression.CalcLevel`, `LevelUpHelper.UpdateProgression` / `AddFeaturesFromProgression`, `ApplyClassMechanics` (tickets 243, 247, 253, 304) |
| Selections, repeats, nesting | `engine/slots.rs`, `parameter.rs`, `engine/walk/options.rs` | `BlueprintFeatureSelection.CanSelect` / `IsSelectionProhibited`, `FeatureSelectionExtensions`, `SelectFeature.Apply`, `LevelUpController` (248, 249, 302, 303, 306aa, 306d) |
| Parametrized feats, proficiency | `parameter.rs` | `BlueprintParametrizedFeature.CanSelect`, `KensaiChosenWeapon`, `WeaponCategoryExtension` (251) |
| Prerequisites | `prerequisite.rs`, `state.rs`, `engine/walk/judge.rs` | `BlueprintFeature.MeetsPrerequisites`, `PrerequisiteStatValue`, `PrerequisiteFullStatValue`, `PrerequisiteClassLevel`, `PrerequisiteFeature`, `PrerequisiteNoFeature`, `PrerequisiteNotProficient`, `PrerequisiteCondition`, `ReplaceStatForPrerequisites`, `ClassLevelsForPrerequisites` (250, 252, 308a, 308d) |
| Prestige classes | `game_data.rs` | `BlueprintCharacterClass.MeetsPrerequisites` (287) |
| Mythic paths, ranks, Path switch | `engine/switch.rs`, `engine/ranks.rs`, `engine/walk/ascension.rs`, `game_data.rs` (`MythicPath`) | mostly the data; little code cited yet |
| Grants | `component.rs`, `engine/walk/components.rs`, `engine/walk/hold.rs` | `AddFacts`, `AddFeatureOnClassLevel`, `AddFeatureIfHasFact`, `AddFeatureOnApply`, `RemoveFeatureOnApply`, `AddParametrizedFeatures` (254, 267, 308h) |
| Ranks | `engine/walk/hold.rs`, `component.rs` | `Feature.AddRank`, `FeatureCollection`, `ContextRankConfig.GetValue` (243, 254) |
| Stats and stacking | `bonus.rs`, `engine/walk/bonuses.rs`, `engine/totals.rs`, `component.rs` (`Descriptor::stacks`) | `ModifiableValue.ApplyModifiersFiltered`, `ModifiableValueAttributeStat`, `ModifierDescriptorHelper`, `AddStatBonus`, `AddContextStatBonus`, `AddStatBonusIfHasFact`, `AddStatBonusEquipment`, `AddGoldenDragonSkillBonus` (244, 252; 144 is the open bonus-type ticket) |
| Skills | `component.rs`, `engine/totals.rs` | `AddClassSkill`, `AddSkillPointPerCharacterLevel`, `ApplySkillPoints.Apply` |
| Spellbooks and the caster line | `spells.rs`, `engine/caster.rs` | `Spellbook` (330), `SpellFocusParametrized`, `ExpandedArsenalMagicSchools` (258), `SpellPenetrationBonus`, `ConcentrationBonus`, `AddMetamagicFeat` |
| Attack line | `engine/attack.rs` | `WeaponFocusParametrized`, `WeaponSpecializationParametrized`, `ImprovedCritical*Parametrized` (289) |

Where to read the studies: `libs/game-data/notes/<ticket>-*.md` (243, 248–254, 302–304, 306aa) and the tickets in `.scratch/wotr-planner/issues/` listed above; `research/component-vocabulary-2026-11-15.md`, `research/importer-design-2026-11-16.md`, `research/tests-from-the-game-242-half-*.md`, `research/305/`–`308/`, and `research/architecture/5-prerequisite-reading.md` and `6-expected-refusals.md`. Find more with `grep -rn "WrathDecompiled\|decompiled/" .scratch/wotr-planner libs tools CODING_STANDARDS.md`. The game check (`tools/game-check`, ticket 238) compares the data with the dump, not with the code.

## 6. Pitfalls

**Decompiler artefacts.** The decompile is clean by ILSpy standards: lambdas and iterators are rebuilt, and no `<>c__DisplayClass` classes remain. What is lost: local variable names (`num`, `num2`, `flag3` stand for meaningful values; name them yourself), some control flow (40 files keep `goto IL_xxxx` labels, mostly in combat and commands), and occasionally generic intent behind casts. Nullable-bool accumulators (`bool?` meaning "no prerequisite of this group seen yet") are a recurring idiom: an empty group counts as passed, which matters.

**Unity and engine coupling.** `Mathf` clamps, `ScriptableObject`-style lifecycle (`OnEnable`), `[SerializeField]` and Newtonsoft attributes, `EventBus` events (`PubSubSystem/EventBus.cs`) and context-scoped flags (`ElementsSystem/ContextData.cs`, `IgnorePrerequisites`) all stand between a rule and its data. A port keeps the rule and drops the plumbing. The level-up code mutates a live unit (and a preview clone) step by step; a planner computes the same outcome from a list of choices, so it must reproduce the *order* the actions run in (`LevelUpActionPriority`) and the moment a fact becomes visible to the next check, not the mutation.

**Combat time versus level-up.** Many components act only inside a combat rule: they handle `RuleCalculateAttackBonusWithoutTarget`, `RuleSavingThrow`, `RuleCalculateAC` and the other classes in `RuleSystem/Rules/` through `OnEventAboutToTrigger` (Weapon Focus, Spell Focus, most conditional bonuses). They never touch a stat, so a stat sheet built from `ModifiableValue` alone misses them, and they cannot be tested by reading the unit's stats. Level-up time is the `LevelUp/` actions, prerequisites, progressions and the `OnActivate`/`OnTurnOn` grants. Abilities, buffs from abilities, and their `ContextAction`s are combat time; the planner does not follow them (`research/component-vocabulary-2026-11-15.md`, "Not followed").

**Names that mislead.** `IgnorePrerequisites` is both a static context flag and a selection field. `ForcedTrue` behaves differently in classes and features. `m_Features` versus `m_AllFeatures` on a selection: only the second decides. A component's `Disabled` flag is honoured at runtime. Difficulty gates (`AddFacts.HasDifficultyRequirements`) and story gates (mythic paths) are not rules of the build.

**Patch version.** The clone is 2.7.0x, the game's last update, and so is the dump; the planner pins both and moves them together. Community guides and the wiki often describe earlier patches; when they disagree with this code, the code wins (map Notes, ticket 234). Owlcat also patched behaviour through data, so a rule can change with no code change: check the blueprint as well as the class.

## Not settled

- Whether the dump records a component's `Disabled` flag: `m_Flags` turns up only on quest objectives in a grep of the 2.7.0x dump, so either no rule component is disabled or the dump omits the field.
- Where `FeatureSelectionState.IgnorePrerequisites` is set for a given slot was not traced.
- How the character-level feats (`ProgressionRoot.m_FeatsProgression`, `BasicFeatsProgression`) are attached to a new character in code; the component-vocabulary research takes it from the root, and `LevelUpController.SetupNewCharacter` is the place to confirm.
