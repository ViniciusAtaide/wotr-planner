# A guide to `decompiled/` for Build creation (WrathDecompiled at 6ce4a6b, patch 2.7.0x)

This guide is for an agent who will reimplement, in another language such as TypeScript, the rules the web app uses to make a Build: race and subrace, ability scores, classes and archetypes, feats and other selections with their prerequisites, skill ranks, Mythic paths with their ranks, feats and abilities, and the Path switch. It covers spellbooks only where they decide which choices are open. It leaves out quests, combat, the kingdom, the UI and story events. It was written on 2026-11-19 from WrathDecompiled commit `6ce4a6ba6d35039cc73ba16faf70c2d9952a63c2` ("2.7.0x", 2025-05-15), the commit pinned in `scripts/game-sources.pins`.

## The rule that comes first

**The game's code and text never enter the repo, not even as a quote.** This guide names files, classes, methods and fields, and describes what they do in its own words. It holds no code from the folder: no method bodies, no snippets, no game strings.

A port follows the same rule. **Reimplement each behaviour from understanding and citations. Never translate the decompiled code line by line.** The planner works this way today: its Rust states each rule in its own terms and cites the game's file and method with the WrathDecompiled commit (`libs/rules/src/component.rs` has the most citations). A fact from the game's data is cited by blueprint name and GUID, and a rule from the code by file and method (CLAUDE.md, "Game sources"; map Notes, ticket 234).

Paths are relative to `decompiled/Assembly-CSharp/Kingmaker/` unless they say otherwise. The planner's tickets cite them the same way.

## 1. Where to look

`decompiled/` is a shallow, git-ignored clone (about 82 MB, 14,725 `.cs` files, one folder per assembly) that exists only in the main checkout; an agent in a worktree reads it by absolute path. Only `Assembly-CSharp/Kingmaker/` matters for Builds: `Blueprints/Classes/` (with `Prerequisites/`, `Selection/`, `Spells/`), `Blueprints/Root/`, `UnitLogic/Class/LevelUp/`, `UnitLogic/` (progression data, features, spellbooks), `UnitLogic/FactLogic/` and `Designers/Mechanics/Facts/` (grant and bonus components), and `EntitySystem/Stats/` with `Enums/`. Skip everything else, including `UI/`, `RuleSystem/` (combat rolls), the kingdom, map and dialog folders, and the other assemblies (third-party plugins, Owlcat's engine layer, `DungeonArchitect.*`).

## 2. Blueprints and the JSON

`blueprints/` is BubblePrints' dump of the game's blueprints as JSON. `cargo run -p game-check -- show <name|guid>` prints one blueprint with its references resolved and its text left out; `refs` and `granted-by` show what uses a blueprint.

- **Classes.** Blueprints derive from `Blueprints/SimpleBlueprint.cs` (`name`, `AssetGuid`) and `BlueprintScriptableObject.cs` (the `Components` array); components from `Blueprints/BlueprintComponent.cs`. `JsonSystem/BlueprintsCache.cs` `Load` reads a blueprint by GUID. A field naming another blueprint is a typed reference (`BlueprintFeatureReference`, on `BlueprintReferenceBase.cs`) holding a GUID.
- **`$type`** has the form `"<type GUID>, <ClassName>"`. The type GUID is the class's `[TypeId]` attribute, and `Blueprints/JsonSystem/GuidClassBinder.cs` binds the two. `grep -rn 'TypeId("<guid>")'` finds the class.
- **Keys** are the C# field names, private ones included (`m_AllFeatures`, `m_Classes`), unless a `JsonProperty` attribute renames them. Inherited fields are flattened into one object, so look up a field's class chain: a progression's JSON holds the fields of `BlueprintProgression`, `BlueprintFeature`, `BlueprintFeatureBase` and `BlueprintUnitFact`. Enums are written by name. References are `"!bp_<guid>"` (`JsonSystem/Converters/BlueprintReferenceConverter.cs`).
- **Noise.** A component's `name`, `PrototypeLink` and `m_Overrides` are editor leftovers. The text fields (`m_DisplayName`, `m_Description`) hold localisation keys and never go in the repo. A component's `Disabled` flag (`m_Flags`) appears in the dump only on quest objectives, so it plays no part in Build blueprints.
- **How a component acts.** Grant and bonus components derive from `UnitLogic/UnitFactComponentDelegate.cs`. They act through lifecycle hooks: `OnActivate`/`OnDeactivate` when their fact is gained or lost, `OnTurnOn`/`OnTurnOff`, and event handlers such as `IOwnerGainLevelHandler`, which runs on every level gained. Prerequisites are plain components that the level-up code calls directly.

## 3. Making a Build, as the game does it

### The level-up session

`UnitLogic/Class/LevelUp/LevelUpController.cs` runs character creation and each level-up. It keeps the choices as `ILevelUpAction`s sorted by `Actions/LevelUpActionPriority.cs` (race, class, race stat, archetype, class mechanics, spellbook, attributes, skills, background, mythic ability, deity, path, features, spells, alignment). After every change `UpdatePreview` replays them all on a fresh preview character (`ApplyLevelUpActions`), dropping any whose `Check` fails. `LevelUpState.cs` holds the counters (`NextCharacterLevel`, `NextClassLevel`, `NextMythicLevel`, open `Selections`, `AttributePoints`, `TotalSkillPoints`, `StatsDistribution`). A planner works from a list of choices, but must apply them in this order.

### Race and subrace

- `Blueprints/Classes/BlueprintRace.cs`: a race is itself a feature. It has `m_Features`, `Size` and `SelectableRaceStat`, and its fixed ability modifiers are `AddStatBonus` components with the `Racial` descriptor. ElfRace also has an `AddStatBonusIfHasFact`, a racial modifier that applies only when a given fact is held.
- `Actions/SelectRace.cs` `Apply` sets the race (`UnitProgressionData.SetRace` swaps the race fact) and grants the race's `m_Features` through `LevelUpHelper.AddFeaturesFromProgression` at level 0. Subraces arrive this way: a race's features include a selection, such as AasimarRace's `AasimarHeritageSelection` (67aabcbce8f8ae643a9d08a6ca67cabd), and that selection opens at character creation. Human's features include a second `BasicFeatSelection`.
- `Actions/SelectRaceStat.cs`: for a race with `SelectableRaceStat`, the player's chosen ability gets +2 with the `Racial` descriptor.
- `SelectRace.Check` refuses a race when `BlueprintRace.GetRestrictedByArchetype` finds an archetype the character holds whose `PrerequisiteFeature` names another race.

### Ability scores

- **Point buy:** `UnitLogic/Class/LevelUp/StatsDistribution.cs`. The budget is 25 points, unless the unit blueprint has a `StartingStatPointsComponent` (`LevelUpState` constructor). Every score starts at 10 and moves between 7 and 18 (`CanAdd`, `CanRemove`), with each step's cost read from the class's cost table (`GetAddCost`). `IsComplete` accepts a spend that leaves no points, or a single point that no score can take at a cost of 1. The actions are `Actions/AddStatPoint.cs` and `RemoveStatPoint.cs`.
- **Increases:** `Actions/SelectClass.cs` `Apply` grants one attribute point at every non-mythic level where the next character level is a multiple of 4. `Actions/SpendAttributePoint.cs` adds it to the score's base value.
- **What ability prerequisites read** (ticket 252): `Prerequisites/PrerequisiteStatValue.cs` `GetStatValue` reads an ability's `PermanentValue` and a skill's ranks (`BaseValue`), then applies any `ReplaceStatForPrerequisites` (`Designers/Mechanics/Facts/ReplaceStatForPrerequisites.cs`). For an ability, `PermanentValue` is the base score plus the racial modifiers plus an item's enhancement bonus. It does not include inherent bonuses (`EntitySystem/Stats/ModifiableValueAttributeStat.cs` `CalculatePermanentValue`; `ModifiableValue.Modifier.IsPermanent`). `Designers/Mechanics/Prerequisites/PrerequisiteFullStatValue.cs` is a variant that reads the full value.

### Classes, progressions and the character-level feats

- `Blueprints/Classes/BlueprintCharacterClass.cs` holds a class's hit die, `SkillPoints`, `ClassSkills`, BAB and save tables (`BlueprintStatProgression`), `m_Progression`, `m_Spellbook`, `m_Archetypes`, `PrestigeClass` and `IsMythic`. `MeetsPrerequisites` refuses a 21st class level and an 11th prestige level, forces the next obligatory Mythic class when one is queued, and then combines the class's prerequisites (see Prerequisites below). `Check` lets the first archetype's alignment prerequisite replace the class's own (ticket 287).
- `Blueprints/Classes/BlueprintProgression.cs` holds `LevelEntries` (each a `LevelEntry`: a `Level` and its `m_Features`). `CalcLevel` works out the progression's own level:
  - with no classes or archetypes listed, it is the character level;
  - otherwise it sums the listed classes' levels, each with its `AdditionalLevel`, and archetype entries count only for classes that have that archetype;
  - it adds the ranks of the facts in `m_FeaturesRankIncrease`;
  - `ForAllOtherClasses` or `m_AlternateProgressionClasses` add the levels of other classes, halved when the type is `Div2` (ticket 253).
- `Actions/ApplyClassMechanics.cs` `Apply` adds BAB, saves, hit points and class skills, then calls `LevelUpHelper.UpdateProgression` on every progression the character holds. `UpdateProgression` grants the entries between the old and new `CalcLevel` (`GiveFeaturesForPreviousLevels` and `m_ExclusiveProgression` change which entries). `AddFeaturesFromProgression` adds each feature whose progression-checked prerequisites hold, opens a selection when the feature is one, and recurses into a granted progression (tickets 247, 304).
- **The character-level feats (settled).** The `UnitProgressionData` constructor (`UnitLogic/UnitProgressionData.cs`) gives every unit the root's `ProgressionRoot.FeatsProgression` as a fact. That progression is `BasicFeatsProgression` (5b72dd2ca2cb73b49903806ee8986325). It lists no classes, so its level is the character level. Its entries give `BasicFeatSelection` at every odd level, and level 1 also gives `BackgroundsBaseSelection` and `DeitySelection`. Because `ApplyClassMechanics` updates every held progression, these open at each odd character level whatever the class. In the data, only the root and a buff's condition reference it. Mythic ranks don't advance it, because `UnitProgressionData.AddClassLevel` raises `MythicLevel` for a Mythic class and `CharacterLevel` otherwise.

### Archetypes

`Blueprints/Classes/BlueprintArchetype.cs` holds `AddFeatures` and `RemoveFeatures` (each a list of `LevelEntry`s), `m_ReplaceSpellbook`/`RemoveSpellbook`, replacement BAB and save tables, class skills and `AddSkillPoints`. `UnitLogic/ProgressionData.cs` `CalculateLevelEntries` merges an archetype into the class progression level by level: the removes come out, and the adds go in. `ProgressionData.CanAddArchetype` refuses a second archetype that also changes the spellbook. It also refuses an archetype whose removes are no longer present in the merged entries, which is how two archetypes clash. `Actions/AddArchetype.cs` and `UnitProgressionData.AddArchetype` apply an archetype, and `BlueprintArchetype.MeetsPrerequisites` checks its own prerequisites.

### Feats and other selections

- `Blueprints/Classes/Selection/BlueprintFeatureSelection.cs` `CanSelect` accepts an item only if all of these hold (tickets 249, 302, 303, 306d):
  - the item is in `m_AllFeatures` (`m_Features` is only the UI's subset);
  - its prerequisites hold, unless they are waived (see below);
  - its rank is below its `Ranks`, unless the item is itself a selection;
  - it obeys `Mode`: `OnlyNew` needs rank 0, and `OnlyRankUp` needs a rank already held;
  - at least one of its nested choices is still open (`NestedFeatureSelectionUtils.cs`).

  `IsSelectionProhibited` closes the whole slot when a `NoSelectionIfAlreadyHasFeature` component matches something the character holds.
- `Selection/BlueprintParametrizedFeature.cs` `CanSelect` covers a feat taken for a parameter: a weapon category, a spell school, a skill or a spell. It refuses a parameter already taken. It requires the `m_Prerequisite` feat for the same parameter (for spell specialisation, for the spell's school). It checks proficiency under `RequireProficiency` and any `IParamPrerequisite` components, and it checks the parameter feature's own prerequisites (tickets 249, 251).
- `Actions/SelectFeature.cs` `Apply` records the pick (`UnitProgressionData.AddSelection`), adds the fact with its parameter, opens the nested selection if the item is one, and runs any progression the pick grants (tickets 248, 306aa).
- **Slots that waive prerequisites (settled; ticket 136).** No code assigns `FeatureSelectionState.IgnorePrerequisites`. It is a read-only property (`UnitLogic/Class/LevelUp/FeatureSelectionState.cs`) that walks from the slot up through its parent slots and is true when any of their selections answers `IsIgnorePrerequisites`. That method returns the selection blueprint's `IgnorePrerequisites` field, or, for `BasicFeatSelection` only, the main character's Trickster flag `TricksterIgnoreFeatPrerequisites`. The field is in the data: `LoremasterRogueTalentSelection` (4b7018b1ed4b27140a5e7adfacaaf9c6) and `LoremasterWizardFeatSelection` (689959eef3e972e458b52598dcc2c752) have it true, while `RogueTalentSelection` and `WizardFeatSelection` have it false. Monk and Crusader bonus feats and Advanced Weapon Training are among the other selections with it true. A waived slot still checks:
  - pool membership, the rank cap, `Mode` and nested availability;
  - with `ExceptWhiteListed`, the prerequisites marked `IsFeatureSelectionWhiteList`;
  - for a parametrized feat, proficiency.

  `SelectFeature.Apply` also copies the waiver onto the new `Feature` as `Feature.IgnorePrerequisites`, which matters for the stat recheck below.

### Prerequisites

- `Prerequisites/Prerequisite.cs` defines the base class, with `Group` (`All`, `Any` or `ForcedTrue`), `CheckInProgression`, `HideInUI` and `IsFeatureSelectionWhiteList`. `Check` passes at once when `ConsiderFulfilled` says so.
- **How the groups combine** (`BlueprintFeature.MeetsPrerequisites`): every `All` prerequisite must hold, and if any non-`All` prerequisites exist, at least one of them must hold. An empty group counts as passed. In a feature, `ForcedTrue` falls into the `Any` group. Only `BlueprintCharacterClass.MeetsPrerequisites` treats a passing `ForcedTrue` as enough on its own. A feature granted by a progression checks only its `CheckInProgression` prerequisites, and a feature already at its `Ranks` fails unless it is a selection.
- Each type has its own file in `Prerequisites/`:
  - feature checks: `PrerequisiteFeature` (it fails while the same feature is being picked further down the slot's chain, `FeatureSelectionState.IsSelectedInChildren`), `PrerequisiteFeaturesFromList`, `PrerequisiteNoFeature`;
  - stat checks: `PrerequisiteStatValue`;
  - level checks: `PrerequisiteClassLevel` (it adds the levels a `ClassLevelsForPrerequisites` component lends), `PrerequisiteArchetypeLevel`, `PrerequisiteNoClassLevel`, `PrerequisiteNoArchetype`, `PrerequisiteCharacterLevel`, `PrerequisiteMythicLevel`;
  - alignment: `PrerequisiteAlignment`;
  - casting: `PrerequisiteCasterType`, `PrerequisiteCasterTypeSpellLevel`, `PrerequisiteClassSpellLevel`;
  - proficiency: `PrerequisiteProficiency`, `PrerequisiteNotProficient`;
  - others: `PrerequisiteParametrizedFeature`, `PrerequisiteSelectionPossible`, `PrerequisiteCondition`.
- **Stat prerequisites keep applying after the pick.** `FeatureCollection.CheckStatPrerequisitesAndUpdate` (`UnitLogic/FeatureCollection.cs`) runs whenever an ability's permanent value changes. It disables (`DisabledBecauseOfPrerequisites`) any picked feature, not taken from a waiving slot, whose non-`Any` stat prerequisites no longer hold.
- `Prerequisites/IgnorePrerequisites.cs` is a context flag for scripted and companion level-ups (`AddClassLevels`). It is not the selection field of the same name, and a Build never uses it.

### Feature ranks and grants

- **Ranks** (ticket 243): `BlueprintFeature.Ranks` is the cap. Adding a feature the character already holds raises its rank instead of adding a copy, and a parametrized feature with a new parameter counts as a new fact (`UnitLogic/FeatureCollection.cs` `AddFact`; `UnitLogic/Feature.cs` `AddRank`). Prerequisites and selections read the rank through `FeatureCollection.GetRank`.
- **Grants:**
  - `UnitLogic/FactLogic/AddFacts.cs` (`OnActivate`, `UpdateFacts`) holds its facts while its own fact is held;
  - `Designers/Mechanics/Facts/AddFeatureOnClassLevel.cs` `Apply` grants its feature from a class level on, or before it with `BeforeThisLevel`, and runs again on every level gained (ticket 254);
  - `AddFeatureIfHasFact` grants a feature while another fact is held;
  - `AddFeatureOnApply` grants once and never removes;
  - `RemoveFeatureOnApply` removes a feature;
  - `UnitLogic/FactLogic/AddParametrizedFeatures.cs` grants parametrized feats with their parameters.
- **Proficiencies and flags:** `AddProficiencies`, `AddClassSkill` and `AddMechanicsFeature` (which raises flags such as `LegendaryHero`).
- `research/component-vocabulary-2026-11-15.md` §2 sorts all 559 component types reachable from the root into planner categories. Read it before reading components one by one.

### Skill ranks

- **Points:** `Actions/ApplySkillPoints.cs` and `LevelUpHelper.GetTotalSkillPoints`. Each non-mythic class level gives the class's `SkillPoints` plus its archetypes' `AddSkillPoints` (`ClassData.CalcSkillPoints`). The Intelligence modifier, read without enhancement, adds half its value per level, rounded up on odd levels and down on even ones. The total is never below one point per level, and `AddSkillPointPerCharacterLevel` and similar handlers add extra points.
- **Spending:** `Actions/SpendSkillPoint.cs` costs one point per rank, whether or not the skill is a class skill, and caps ranks at the character level.
- **Class skill bonus:** `EntitySystem/Stats/ModifiableValueSkill.cs` `CalculateBaseValue` adds 3 to a class skill once it has a rank.

### Mythic paths, ranks, feats and abilities

Each claim below cites the code. The planner's own model (GLOSSARY "Path switch"; `libs/rules/src/engine/switch.rs`, `ranks.rs`, `walk/ascension.rs`) was built mostly from data and the guide.

- **Paths are Mythic classes.** `ProgressionRoot.CharacterMythics` lists them, and `BlueprintCharacterClass.IsMythic` marks them. `UnitProgressionData.AddClassLevel` counts a level of a Mythic class toward `MythicLevel` and any other toward `CharacterLevel`.
- **Ranks.** `UnitProgressionData.CanLevelUpMythic` allows a new rank only while it stays at 10 or less and mythic experience is ahead of the current rank. The story grants that experience (`GainMythicExperience`), so the timing of ranks is a story fact and not a rule.
- **The first ranks and the forced order.** `UnitProgressionData.ObligatoryMythicClassesQueue` (set by `SetObligatoryMythicClasses`, from `UnitHelper`) names the classes the next ranks must take. `LevelUpState.NextObligatoryMythicClass` and `BlueprintCharacterClass.MeetsPrerequisites` enforce it, and `AddClassLevel` removes each class from the queue as it is taken. `ProgressionRoot.MythicStartingClass` is the class of the ranks before a path.
- **Mythic feats and abilities** come from the path progressions' level entries, not from code. `MythicStartingProgression`, `AngelProgression`, `AeonProgression`, `AzataProgression`, `DemonProgression`, `LichProgression`, `TricksterProgression`, `DevilProgression`, `GoldenDragonProgression` and `SwarmThatWalksProgression` each grant `MythicFeatSelection` and `MythicAbilitySelection` (named in `SystemMechanicsRoot`; check any path with `game-check refs MythicFeatSelection`). A path progression lists its own class (AngelProgression's `m_Classes` is `AngelMythicClass`), so its level follows the ranks taken in that class. `SelectFeature.CalculatePriority` gives picks in the `MythicFeat` and `MythicAbility` groups the `MythicAbility` priority. The selection records for these picks are keyed by mythic level (`FeatureSelectionData.IsMythic`).
- **Gold Dragon and Swarm That Walks.** These classes alone have `m_IsHigherMythic` set: `GoldenDragonClass` daf1235b6217787499c14e4e32142523 and `SwarmThatWalksClass` 5295b8e13c2303f4c88bdb3d7760a757 (`BlueprintCharacterClass.IsHigherMythic`). When the character takes one while holding a regular path, `UnitProgressionData.AddClassLevel` calls `ReplaceAllMythicLevelsWithMythicHeroLevels`. That method:
  - removes every mythic level (`RemoveMythicLevel` drops the class level, the features it gave and its selection records);
  - re-adds the recorded mythic feat and ability picks (`GetMythicSelectionsData`);
  - refills the rank count with fake levels of the starting Mythic class (`AddFakeLevel`).

  `LegendProgression` grants neither `MythicFeatSelection` nor `MythicAbilitySelection` (`game-check refs` lists only the eleven progressions above), which fits Legend ending the ascensions.
- **Legend.** The Legend choice is a fake class, `ProgressionRoot.MythicLegenFake`, which `LevelUpController.Commit` turns into `UnitPartLegendPathSelected`. After that the character takes `LegendClass` (3d420403f3e7340499931324640efe96, Mythic but not higher). Its `LegendFeature` (7f99961610390044391f300c7ef5e0c8) carries `AddMechanicsFeature` `LegendaryHero`, which raises `UnitProgressionData.MaxCharacterLevel` from 20 to 40. A Legend character uses `ProgressionRoot.LegendXPTable` (`LevelUpController.GetEffectiveLevel`). `LegendProgression` (905383229aaf79e4b8d7e2d316b68715) removes path features with `RemoveFeatureOnApply`.
- **Devil, and the rest of the Legend switch,** are run by story actions such as `Designers/EventConditionActionSystem/Actions/RemoveMythicLevels.cs` and `ReplaceAllMythicLevelsWithMythicHeroLevels.cs` (which call the `UnitProgressionData` methods of the same names), not by level-up rules. The planner takes what each switch keeps from data and the guide (GLOSSARY "Path switch"). Which paths a player may take at all is gated by the story (`DialogSystem/Blueprints/BlueprintMythicsSettings.cs`, `MythicClassLockComponent`).

### Spellbooks, as far as they gate choices

`Blueprints/Classes/Spells/BlueprintSpellbook.cs` holds the spells-per-day table, `CastingAttribute`, `Spontaneous`, `IsArcane` and `CasterLevelModifier`.

- **Advancing a spellbook.** `Actions/ApplySpellbook.cs` raises a class's spellbook by one level at each class level, except the levels the class's `SkipLevelsForSpellProgression` lists. A prestige class advances an existing spellbook through a `BlueprintFeatureReplaceSpellbook` pick (`SelectFeature.Apply`).
- **Caster level and highest spell level.** In `UnitLogic/Spellbook.cs`, `CasterLevel` is `BaseLevel` plus a mythic contribution (`GetMythicLevelForCasterLevel`), and `MaxSpellLevel` is the highest spell level with a non-negative per-day count.
- **The prerequisites that read them.** `PrerequisiteCasterTypeSpellLevel` takes the best `MaxSpellLevel` among the character's arcane or divine spellbooks, optionally spontaneous ones only, leaving out alchemist and Mythic spellbooks. `PrerequisiteClassSpellLevel` reads one class's spellbook.

### Bonus stacking, where it reaches the sheet

`EntitySystem/Stats/ModifiableValue.cs` (`ApplyModifiersFiltered`) and `Enums/ModifierDescriptorHelper.cs` handle stacking. Bonuses of a type in `DefaultStackingDescriptors` all add: untyped, racial, dodge, feat, penalty, focus, natural armour and a few more. For any other type, only the largest bonus and the largest penalty count. This bears on Builds in two places: the racial and item-enhancement modifiers that ability prerequisites read, and the sheet's totals (`AddStatBonus`, `AddContextStatBonus`, `AddStatBonusEquipment`; ticket 144 is the open bonus-type ticket).

## 4. Tracing a rule

Go from a blueprint's `$type` to its class file (`find decompiled -name '<ClassName>.cs'`); read a component's hooks and handler interfaces; trace a choice from `BlueprintFeatureSelection.CanSelect`, `BlueprintFeature.MeetsPrerequisites` or `LevelUpHelper.UpdateProgression`. The shell's `grep` is ugrep, which rejects forms such as `(\b|,)`, so keep patterns simple:

```sh
D=/Users/thaynamarcela/code/wotr-planner/decompiled/Assembly-CSharp/Kingmaker
grep -rn 'TypeId("d79ad4ed064aa4f43ace0e9c238fb9b9")' "$D"              # a $type GUID to its class (AddFacts)
grep -rn 'class [A-Za-z]* : Prerequisite' "$D/Blueprints/Classes/Prerequisites"  # every prerequisite type
grep -rln 'GetComponent<NoSelectionIfAlreadyHasFeature>' "$D"          # who reads a component
```

## 5. The planner today

`libs/rules/` already models each area. Its vocabulary is in `GLOSSARY.md`, and it cites the game beside its code:

| Area | Planner | Game code its tickets and code cite |
| --- | --- | --- |
| Race, subrace, point buy, ability increases | `game_data.rs` (`Race`), `engine/controls.rs`, `engine/walk/hold.rs` | `PrerequisiteStatValue`, `ModifiableValueAttributeStat` (252); the race and point-buy code above isn't cited yet |
| Classes, progressions, level steps | `engine/walk/level.rs`, `game_data.rs` (`Class`, `Progression`) | `BlueprintProgression.CalcLevel`, `LevelUpHelper`, `ApplyClassMechanics`, `BlueprintCharacterClass.MeetsPrerequisites` (243, 247, 253, 287, 304) |
| Archetypes | `game_data.rs` (`Archetype`) | the data's adds and removes |
| Selections, repeats, nesting | `engine/slots.rs`, `parameter.rs`, `engine/walk/options.rs` | `BlueprintFeatureSelection.CanSelect`/`IsSelectionProhibited`, `FeatureSelectionExtensions`, `SelectFeature.Apply`, `LevelUpController` (248, 249, 302, 303, 306aa, 306d) |
| Parametrized feats | `parameter.rs` | `BlueprintParametrizedFeature.CanSelect`, `KensaiChosenWeapon`, `WeaponCategoryExtension` (251) |
| Prerequisites | `prerequisite.rs`, `state.rs`, `engine/walk/judge.rs` | `BlueprintFeature.MeetsPrerequisites`, the prerequisite files, `ReplaceStatForPrerequisites`, `ClassLevelsForPrerequisites` (250, 252, 308a, 308d) |
| Grants and ranks | `component.rs`, `engine/walk/components.rs`, `engine/walk/hold.rs` | `AddFacts`, `AddFeatureOnClassLevel`, `AddFeatureIfHasFact`, `AddFeatureOnApply`, `RemoveFeatureOnApply`, `Feature`, `FeatureCollection`, `ContextRankConfig` (243, 254, 267, 308h) |
| Skills | `component.rs`, `engine/totals.rs` | `AddClassSkill`, `AddSkillPointPerCharacterLevel`, `ApplySkillPoints.Apply` |
| Mythic ranks and Path switch | `engine/switch.rs`, `engine/ranks.rs`, `engine/walk/ascension.rs` | the data and the guide; the code cited in this guide is new |
| Spell-level gates, caster line | `spells.rs`, `engine/caster.rs` | `Spellbook` (330), `SpellFocusParametrized`, `ExpandedArsenalMagicSchools` (258) |

For the studies behind these citations, read the notes in `libs/game-data/notes/` (243, 248–254, 302–304, 306aa), the tickets with the same numbers in `.scratch/wotr-planner/issues/`, ticket 136 (slots that waive prerequisites), `research/component-vocabulary-2026-11-15.md`, `research/architecture/5-prerequisite-reading.md` and `research/305/`–`308/`. To find more, run `grep -rn "WrathDecompiled\|decompiled/" .scratch/wotr-planner libs tools`.

## 6. Pitfalls for a Build port

- **Lost names.** Lambdas and iterators decompile cleanly, but locals lost their names (`num`, `flag3`). A nullable bool often means "no prerequisite of this group seen yet", which is why an empty group passes.
- **Order and timing.** Each check sees what earlier actions added, in priority order. A port must keep that order and each grant's level: race features at 0, progression entries at the progression's level, mythic picks at the mythic level.
- **Names that mislead:** `IgnorePrerequisites` (flag and field); `ForcedTrue` (class versus feature); `m_Features` versus `m_AllFeatures`; `PermanentValue` without inherent bonuses; Gold Dragon and Swarm are "higher mythic", Legend and Devil are not.
- **Data, not code.** Feat cadence, subraces, mythic slots and most grants live in blueprints the code only walks.
- **Not rules of a Build:** story gates (path unlocks, mythic experience), difficulty gates (`AddFacts.HasDifficultyRequirements`), companions' scripted levels (`AddClassLevels`, `ClassLevelLimit`).
- **Patch.** Code and dump are both 2.7.0x and move together; older guides lose to the code (ticket 234).

## Still open

- The exact keep-and-drop of the Legend and Devil switches is run by story actions and data, and is not fully traced here. `RemoveMythicLevels`, `ReplaceAllMythicLevelsWithMythicHeroLevels` and `LegendProgression`'s `RemoveFeatureOnApply` are the places to read.
- In `GetMythicSelectionsData`, some recorded picks are re-keyed two ranks later when Gold Dragon or Swarm replaces the path. Why was not worked out.
