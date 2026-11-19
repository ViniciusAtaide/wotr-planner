# Building a Wrath of the Righteous build planner from the game's sources

You are writing a build planner for Pathfinder: Wrath of the Righteous at patch 2.7.0x, in TypeScript or any language you choose. A **Build** is a character's choices from creation to level 20 (40 on Legend) and Mythic rank 10: race and subrace, ability scores, each level's class and picks, skill ranks, and the Mythic path with its ranks, feats, abilities and switch. The planner offers only legal choices, says why each closed choice is closed, and shows the resulting sheet.

This guide gives you the game's model, the data shape and modules to build, the order of work, how to check your work against the game, and a reference for each Build-creation area. Read sections 1–3 before writing code. Sections 4–6 are your plan. Section 7 is reference: consult it area by area as you build.

## 1. Purpose and sources

Two sources, both at patch 2.7.0x, are the whole truth about the rules:

- **`blueprints/`** holds BubblePrintsData release `2.7.0x_0`: every game blueprint as JSON, packed in one `.binz` file. This is the **data**: classes, feats, selections, progressions and prerequisites.
- **`decompiled/`** holds xADDBx/WrathDecompiled at commit `6ce4a6ba6d35039cc73ba16faf70c2d9952a63c2`: the game's C# code. This is the **behaviour**: how the game applies the data.

Both folders sit at the repo root of the main checkout, git-ignored; `scripts/game-sources.sh` fetches them at the pins in `scripts/game-sources.pins`. Code paths below are relative to `decompiled/Assembly-CSharp/Kingmaker/`.

**Reading the dump.** Each blueprint's `$type` is `"<type GUID>, <ClassName>"`, the class's `[TypeId]` attribute (`Blueprints/JsonSystem/GuidClassBinder.cs`). Keys are the C# field names, private ones included (`m_AllFeatures`), and a blueprint carries the fields of its whole class chain. A reference is the string `"!bp_<guid>"` (`JsonSystem/Converters/BlueprintReferenceConverter.cs`). Enums appear by name. A component's `name`, `PrototypeLink` and `m_Overrides` are editor bookkeeping, and `m_DisplayName` and `m_Description` hold localisation keys.

**The copyright boundary.** Describe the game's behaviour in your own words and cite it by file and method with the WrathDecompiled commit. Cite a fact from the data by blueprint name and GUID. Write each rule fresh from that understanding, so the repo holds your code, your summaries and your citations, while the game's code, text strings and dump stay in the git-ignored folders. The game's mechanics are Open Game Content under Owlcat's OGL notice; carry that notice and its Section 15 credit on the published site.

**What counts as truth.** The game's behaviour as its code applies its data is the rule, including where the code does something other than an item's text says (the Seven Virtues ring works by path). When a community guide and the game differ, model the game, and let summaries borrow the guide's clearer wording where it agrees. Model every DLC and prestige class. Model the rules a player follows, and leave known exploits out.

**Reading the code.** Find a class by file name (`find decompiled -name '<ClassName>.cs'`) and follow a rule from its decision point (`BlueprintFeatureSelection.CanSelect`, `BlueprintFeature.MeetsPrerequisites`, `LevelUpHelper.UpdateProgression`) through its calls. Name each decompiled local (`num`, `flag3`) from what it holds; a nullable bool usually means "no member of this group seen yet". The shell's `grep` is ugrep: keep regexes simple.

## 2. How to think about the game's model

Think in the game's own nouns. Each one below names a concept the planner models one-for-one.

- **Blueprint.** Every game object is a blueprint: a typed record with a `name` and a GUID (`Blueprints/SimpleBlueprint.cs`). The GUID is the identity, and names repeat across blueprints. When several blueprints share a name, identify one by what grants it.
- **Component.** A blueprint carries a list of components (`BlueprintScriptableObject.Components`, base class `Blueprints/BlueprintComponent.cs`). Behaviour lives in components: a prerequisite is a component, and so is a grant (`AddFacts`) or a stat bonus (`AddStatBonus`). Think of a feature as an empty shell whose components say what it does.
- **Fact.** When a character holds a blueprint, it holds a fact of it (`UnitLogic/Feature.cs`, `EntitySystem/EntityFact.cs`). Components act only through a held fact. A character holds each feature blueprint once, with a rank.
- **Rank.** `BlueprintFeature.Ranks` caps how many times a feature can be held. Gaining a feature already held raises its rank (`UnitLogic/FeatureCollection.cs` `AddFact`, `Feature.AddRank`). A feature that improves in steps is granted at each step, so its rank is the step reached. A parametrized feat with a new parameter is a separate fact.
- **Progression and level.** A progression is a feature with level entries (`Blueprints/Classes/BlueprintProgression.cs`, `LevelEntry`). Each has its own level, computed by `CalcLevel` from class levels, archetype levels, offsets and other features' ranks. Classes, bloodlines, mysteries, domains and the character-level feats are all progressions. Every level-up re-evaluates every held progression and grants the entries between its old and new level.
- **Selection and slot.** A selection is a feature that asks for a pick from a list (`Selection/BlueprintFeatureSelection.cs`). Holding one opens a slot. A pick can itself be a selection, which opens a nested slot at the same level. Extra feats are selections, so taking one opens a slot of its list.
- **Prerequisite.** A prerequisite is a component with a group (`All`, `Any`, `ForcedTrue`) and a flag saying whether a progression checks it (`Prerequisites/Prerequisite.cs`). A blueprint's prerequisites form one tree. The game combines them in one place for features and one for classes.
- **Level-up order.** A level-up applies its choices in fixed bands (`Actions/LevelUpActionPriority.cs`: race, class, archetype, class mechanics, spellbook, attributes, skills, background, mythic ability, deity, path, features, spells, alignment). Each choice is checked against the character with every earlier choice applied. Within a band the player picks the order, so a pick whose prerequisite another pick of the same band meets is legal in some order.
- **Mythic.** Paths are classes with `IsMythic` whose levels raise the Mythic rank; their progressions grant the Mythic feat and ability selections.

## 3. The data shape to build

Shape the data as the game shapes it, keyed as the game keys it. Then each import, check and test lines up with a blueprint.

1. **Key every entry by its blueprint GUID, one file per blueprint,** in a folder by kind, with a readable slug beside the GUID as a label; two pieces of work then rarely touch one file.
2. **Keep one entry per game object.** A feature several classes grant is one entry that each grantor references; the sheet finds the holder from the grant, and the character holds the fact once.
3. **Hold references by GUID, exactly as the game does:** a class names its progression, a progression's level entries name features, a selection's `m_AllFeatures` names its options, and an archetype's `AddFeatures` and `RemoveFeatures` name features by level. Keep the pick list exactly as `m_AllFeatures` gives it, because that list decides what a slot accepts.
4. **Copy the header fields the code reads, and compute everything else.** Keep `Ranks`, `Mode`, `IgnorePrerequisites`, `ExceptWhiteListed`, the progression's `m_Classes`, `m_Archetypes`, `AdditionalLevel`, `ForAllOtherClasses`, `AlternateProgressionType`, `GiveFeaturesForPreviousLevels` and `m_FeaturesRankIncrease`, and the components that grant, require or add. Derive the rest in code from those fields, the way the game does: how often a pick repeats, which band a pick belongs to (from its selection's `Group`), BAB and saves (from the `BlueprintStatProgression` tables), and caps. A value derived in code stays correct when its inputs change.
5. **Import hidden blueprints too.** Many grants pass through blueprints hidden from the player: a hidden feature whose `AddFacts` or `AddFeatureOnClassLevel` gives what a prerequisite asks for. Import each blueprint reachable from the root (`Blueprints/Root/ProgressionRoot.cs`: classes, Mythic classes, races, the feats progression) by following grants, so every prerequisite leaf can find its grantors.
6. **Give each planner-made entry a `why`, in its own folder,** so every other entry traces to the game.
7. **Treat a group's children as a set:** sort catalogs by key and group children canonically, so a diff of two data versions shows only real changes.
8. **Write summaries in your own words,** cited by blueprint name and GUID.
9. **Write the data with an importer** that reads the dump and writes each entry's mechanical fields; a second run on the same dump writes nothing.
10. **Check the data once at load:** every GUID resolves to an entry of the expected kind, every planner-made entry has its reason, and every prerequisite tree is in normal form (module P). The engine takes only checked data.

## 4. The modules to build

Each module answers one question. Every other module asks it that question and keeps no copy of the answer. When a second place starts answering the same question, move it into the module.

- **Loader and check.** *Is this data whole?* It reads the files, resolves references and applies the load-time checks above, and it returns checked data for everything else.
- **P, the prerequisite reader.** *What does this prerequisite ask, in one shape?* Normalise each prerequisite tree once, at load:
  - flatten nested groups of one kind (`All` in `All`, `Any` in `Any`);
  - read `AtLeast(1)` as `Any`;
  - cancel a double `Not`;
  - replace a group of one with its child;
  - sort the children and merge equal ones.

  Then offer one read-only view whose leaves carry their polarity (needed or refused) and the fact they name. Every evaluator, wording function, importer and test reads the tree through this view, so all of them agree on what a tree means.
- **G, the grant index.** *Which places grant this fact, and what does this entry grant?* Build one index at load over every kind of grant: progression entries, archetype adds and removes, selection lists, grant components (`AddFacts`, `AddFeatureOnClassLevel`, `AddFeatureIfHasFact`, `AddFeatureOnApply`), race features and Mythic switch grants. Add a Build-aware half that answers who holds a fact in this Build and what a pick gives, including the slots it opens. Answer every "who grants X" question from it.
- **W, the walk.** *What does this Build hold at each level, and which of its choices fail?* It is one pass over the Build, level by level, in the game's band order: class level, progression updates, slots, picks, skill ranks and ability increases, with Mythic ranks placed at their levels. The walk is the single judge of legality. It records each held fact with its source and level.
- **S, slots and pickers.** *What can this slot take?* For each slot it lists the options with each one's status, computed by pausing the walk at the slot and judging every candidate there. A picker's answer matches what the walk would say if that option were chosen.
- **R, the refusal.** *Why can't this pick be taken here, and what would let it in?* One answer per pick at a place: the unmet leaves worded true for this Build (the pick really held, by value, level and rank; only alternatives the Build can still reach), the later choices it would break, and the later level or rank where it opens. Every surface words its text from this answer. Status is behaviour and wording is presentation, so wording changes leave statuses as they were.
- **E, edits.** *Does this edit keep the Build possible?* Every edit goes through one apply function and `allows`, so the Build holds only possible states; removing a choice a later one needs is refused, naming that later choice.
- **C, the link codec.** *Which Build does this link hold?* Every choice in the URL fragment, with stable identifiers and a version; an identifier a link names stays decodable through an alias when its entry changes.
- **I, the importer.** *What does the game say this entry's fields are?* It writes the data from the dump area by area, identifying each selection through what grants it.
- **K, the game check, and L, its ledger.** *Where does the data differ from the dump, and is each difference owned?* The same report for the same inputs; one `key -> owner` ledger with one reader, reconciled both ways (an unlisted difference fails, and so does a line matching none).
- **N, the narrowing check.** *Does this rewrite refuse a character the game allows?* When an import rule changes a tree, record each narrowing (a dropped alternative, an added `Not` or requirement, a raised value) with the blueprint that justifies it.
- **X, expected refusals.** *Is each failure of this old Build one the game refuses, by which rule?* Rows of entry, slot kind, refusal shape and citation, one file per piece of work; an old Build is refused as expected when every failure matches a row.

The modules form a stack: P, then G, then W, then S and R, then E, C and the surfaces. The tooling (I, K, L, N, X) reads P and G's static half. Each module reads only those below it.

## 5. The order of work

Each step ends when its criterion holds. Move to the next step only then.

1. **Read the game's model.** Read section 2 and the classes it cites. *Done when* you can trace, in the code, how a level-up grants a progression's entries and how a selection decides one pick.
2. **Build the dump reader.** Read the `.binz`, streaming one record at a time so memory stays small, and index blueprints by GUID and name. Give it three lookups: `show` (one blueprint, references resolved to `<name> <guid>`, text left out), `refs` (who references it) and `granted-by` (follow `refs` up to a class, race or progression). *Done when* all three work for any GUID.
3. **Define the data shape** from section 3, with the loader and its checks. *Done when* a small hand-written fixture loads and every check has a failing fixture of its own.
4. **Build P,** the normal form and the view. *Done when* every tree in the fixture is normal after load and a property test shows that normalising twice changes nothing.
5. **Write the importer for the reachable set,** in this order: classes with their tables and progressions, archetypes, races, selections with their lists, hidden grant carriers, prerequisites, then Mythic paths. *Done when* every blueprint reachable from the root has an entry, every reference resolves, and a second run writes nothing.
6. **Build G.** *Done when* every prerequisite leaf naming a fact finds at least one grantor, and the ledger lists every leaf that finds none.
7. **Build the walk.** Start with a single class to level 20, then add archetypes, selections, nested selections, ranks, skill ranks, ability scores, race, Mythic ranks and switches. *Done when* each area's real-data tests (section 6) pass.
8. **Build the pickers and R.** *Done when* the agreement self-check passes over the corpus, and every closed option carries a refusal answer.
9. **Build E and C.** *Done when* every corpus Build round-trips through its link unchanged and every edit `allows` accepts leaves the Build possible.
10. **Build K, L, N and X,** and wire them into one check script that runs in a fixed order and stops at the first failure. *Done when* the game check exits 0 with every listed difference owned.
11. **Build the surfaces** (web page, agent tools) on R and S. *Done when* each surface's text for a refusal comes from R's answer.

## 6. How to check against the game

- **Check each rule with a test that cites the blueprint or the method.** The expected value comes from the game, cited beside the assertion, and an expectation moves only toward the game: when a test and the engine disagree, fix the engine.
- **Assert game facts rather than counts of your own data:** compare a list against the dump's `m_AllFeatures`, so the test stays valid as the data grows.
- **Keep self-checks:** every picker agrees with the walk, each shortcut agrees with the full computation, and each option's status agrees with `allows` of its edit. Run them over a **corpus** of about 2,000 generated Builds covering every class, archetype and path, plus a few **gold** Builds from real players. Self-checks prove consistency; the game decides correctness.
- **Prove each form-only change:** record every corpus Build's sheet and every option's status before, compare after, and expect zero differences. A change that moves values attributes every differing line to a row of its change log, against a census predicted beforehand.
- **Carry the owner's gate when an option joins a wider list,** as a prerequisite stated by the game's own blueprint.
- **Run an area's tests while you work** and the full suite once at the end, heavy runs one at a time.
- **Make every exception a cited decision:** each refusal of an old Build a row in X, each known difference an owner in L.

## 7. Reference for each Build-creation area

### The level-up session

`UnitLogic/Class/LevelUp/LevelUpController.cs` `AddAction` inserts each choice by band; `UpdatePreview` replays them all on a fresh preview (`ApplyLevelUpActions`), dropping any whose `Check` fails. `LevelUpState.cs` holds the counters (`NextCharacterLevel`, `NextClassLevel`, `NextMythicLevel`, open `Selections`, `AttributePoints`, `TotalSkillPoints`). A pick's band comes from its selection's `Group` (`SelectFeature.CalculatePriority`): domains, bloodlines, mysteries, curses, spirits, blessings and schools in Path; Mythic feats and abilities in MythicAbility; most others in Features. Within a band, retry a pick refused for an unmet prerequisite after the band's other picks.

### Race and subrace

- `Blueprints/Classes/BlueprintRace.cs`: a race is a feature, with `m_Features`, `Size` and `SelectableRaceStat`. Its fixed ability modifiers are `AddStatBonus` components with the `Racial` descriptor, and ElfRace adds one more racial modifier through `AddStatBonusIfHasFact`.
- `Actions/SelectRace.cs` `Apply` sets the race (`UnitProgressionData.SetRace`) and grants `m_Features` at level 0 through `LevelUpHelper.AddFeaturesFromProgression`. A subrace is a selection among those features: AasimarRace grants `AasimarHeritageSelection` 67aabcbce8f8ae643a9d08a6ca67cabd. HumanRace's features include a second `BasicFeatSelection`.
- `Actions/SelectRaceStat.cs` adds +2 `Racial` to the chosen ability of a race with `SelectableRaceStat`.
- `SelectRace.Check` closes a race when `BlueprintRace.GetRestrictedByArchetype` finds an archetype the character holds whose `PrerequisiteFeature` names another race.

### Ability scores

- **Point buy** (`UnitLogic/Class/LevelUp/StatsDistribution.cs`): 25 points, or a unit's `StartingStatPointsComponent` (`LevelUpState` constructor). Every score starts at 10 and moves between 7 and 18 at the cost of the class's cost table (`GetAddCost`). `IsComplete` accepts no points left, or one point that no score can take at a cost of 1. The racial modifiers apply on top.
- **Increases:** `Actions/SelectClass.cs` `Apply` grants one attribute point at each non-mythic level where the next character level is a multiple of 4, and `Actions/SpendAttributePoint.cs` adds it to the base score.
- **What ability prerequisites read:** `Prerequisites/PrerequisiteStatValue.cs` `GetStatValue` reads an ability's `PermanentValue`, which is the base score plus racial modifiers plus an item's enhancement bonus, leaving out inherent bonuses (`EntitySystem/Stats/ModifiableValueAttributeStat.cs` `CalculatePermanentValue`, `ModifiableValue.Modifier.IsPermanent`). It applies any `ReplaceStatForPrerequisites` first (`Designers/Mechanics/Facts/`). `Designers/Mechanics/Prerequisites/PrerequisiteFullStatValue.cs` reads the full value.
- **Stat prerequisites keep applying after the pick:** `FeatureCollection.CheckStatPrerequisitesAndUpdate` runs when an ability's permanent value changes, and disables (`DisabledBecauseOfPrerequisites`) any picked feature whose non-`Any` stat prerequisites stop holding, unless the feature came from a slot that waives prerequisites.
- **Stacking**, where it reaches the sheet: `ModifiableValue.ApplyModifiersFiltered` with `Enums/ModifierDescriptorHelper.cs`. Bonuses of the stacking types (untyped, racial, dodge, feat, penalty, focus, natural armour and a few more) all add. For any other type, the largest bonus and the largest penalty count.

### Classes, progressions and the character-level feats

- `Blueprints/Classes/BlueprintCharacterClass.cs` holds the hit die, `SkillPoints`, `ClassSkills`, BAB and save tables (`BlueprintStatProgression`), `m_Progression`, `m_Spellbook`, `m_Archetypes`, `PrestigeClass` and `IsMythic`. `MeetsPrerequisites` caps a class at 20 levels and a prestige class at 10, then combines prerequisites, and a passing `ForcedTrue` suffices on its own there. `Check` lets the first archetype's alignment prerequisite stand in for the class's.
- `BlueprintProgression.CalcLevel`:
  - with no classes and no archetypes, it is the character level;
  - otherwise it is the sum of the listed classes' levels, each with its `AdditionalLevel` and floored at 0, where archetype entries count only for a class taken with that archetype;
  - it adds the ranks of the facts in `m_FeaturesRankIncrease`;
  - `ForAllOtherClasses` or `m_AlternateProgressionClasses` add other classes' levels, halved under `Div2`.
- `Actions/ApplyClassMechanics.cs` `Apply` adds BAB, saves, hit points and class skills, then calls `LevelUpHelper.UpdateProgression` on every held progression. `UpdateProgression` grants the entries between the old and new level, or only the new level's entries unless `GiveFeaturesForPreviousLevels` is set, and only to the class in `m_ExclusiveProgression` when one is named. `AddFeaturesFromProgression` checks only `CheckInProgression` prerequisites, opens a slot for a selection, and recurses into a progression.
- **The character-level feats:** the `UnitProgressionData` constructor gives every character the root's `FeatsProgression`, which is `BasicFeatsProgression` 5b72dd2ca2cb73b49903806ee8986325. It lists no classes, so its level is the character level. It grants `BasicFeatSelection` at every odd level, and `BackgroundsBaseSelection` and `DeitySelection` at level 1.

### Archetypes

`Blueprints/Classes/BlueprintArchetype.cs` holds `AddFeatures` and `RemoveFeatures` (each a list of `LevelEntry`s), `m_ReplaceSpellbook`/`RemoveSpellbook`, replacement BAB and save tables, class skills and `AddSkillPoints`. `UnitLogic/ProgressionData.cs` `CalculateLevelEntries` merges the archetype into the class progression level by level: the removes leave the list and the adds join it. `ProgressionData.CanAddArchetype` allows a second archetype only when at most one of the two changes the spellbook and every one of its removes is still present in the merged list; two archetypes that remove the same feature clash. `BlueprintArchetype.MeetsPrerequisites` checks the archetype's own components.

### Feats and selections

- `Selection/BlueprintFeatureSelection.cs` `CanSelect` takes an item when all of these hold:
  - the item is in `m_AllFeatures`;
  - its prerequisites hold, or are waived;
  - its rank is below `Ranks`, unless the item is itself a selection;
  - `Mode` allows it: `OnlyNew` needs rank 0, `OnlyRankUp` a rank already held;
  - one of its nested choices is still open (`NestedFeatureSelectionUtils`).

  `IsSelectionProhibited` closes the slot when a `NoSelectionIfAlreadyHasFeature` component matches a held fact.
- `Selection/BlueprintParametrizedFeature.cs` `CanSelect` covers a feat taken for a weapon category, spell school, skill or spell. It takes a parameter not yet held, requires the `m_Prerequisite` feat for the same parameter (for spell specialisation, for the spell's school), checks proficiency under `RequireProficiency` and any `IParamPrerequisite`, and checks the parameter feature's prerequisites.
- `Actions/SelectFeature.cs` `Apply` records the pick, adds the fact with its parameter, opens a picked selection as a nested slot at the same level (this is how Extra feats give their extra pick), and runs any progression the pick grants.
- **The waiver rule.** `FeatureSelectionState.IgnorePrerequisites` is a read-only property: true when the slot's selection or any parent slot's selection answers `IsIgnorePrerequisites`. That answer is the blueprint's `IgnorePrerequisites` field, and also, for `BasicFeatSelection` only, the main character's `TricksterIgnoreFeatPrerequisites` flag. LoremasterRogueTalentSelection 4b7018b1ed4b27140a5e7adfacaaf9c6 and LoremasterWizardFeatSelection 689959eef3e972e458b52598dcc2c752 set the field, and so do the Monk and Crusader bonus feat selections and Advanced Weapon Training. A waived slot still checks:
  - membership in its list, the rank cap, `Mode` and nested availability;
  - under `ExceptWhiteListed`, the prerequisites marked `IsFeatureSelectionWhiteList`;
  - proficiency for a parametrized weapon feat.

  `SelectFeature.Apply` copies the waiver onto the fact (`Feature.IgnorePrerequisites`).

### Prerequisites

- **How the groups combine** (`BlueprintFeature.MeetsPrerequisites`): every `All` member holds, and when non-`All` members exist, at least one of them holds; an empty group passes. In a feature, `ForcedTrue` joins the `Any` members. A progression's grant checks only `CheckInProgression` members, and a feature at its `Ranks` fails unless it is a selection.
- **The types**, one file each in `Prerequisites/`: `PrerequisiteFeature` (it fails while the same feature is picked further down the chain, `FeatureSelectionState.IsSelectedInChildren`), `PrerequisiteFeaturesFromList`, `PrerequisiteNoFeature`, `PrerequisiteStatValue`, `PrerequisiteClassLevel` (plus levels a `ClassLevelsForPrerequisites` lends), `PrerequisiteArchetypeLevel`, `PrerequisiteNoClassLevel`, `PrerequisiteNoArchetype`, `PrerequisiteCharacterLevel`, `PrerequisiteMythicLevel`, `PrerequisiteAlignment`, `PrerequisiteCasterType`, `PrerequisiteCasterTypeSpellLevel`, `PrerequisiteClassSpellLevel`, `PrerequisiteProficiency`, `PrerequisiteNotProficient`, `PrerequisiteParametrizedFeature`, `PrerequisiteSelectionPossible`, `PrerequisiteCondition`.
- `Prerequisites/IgnorePrerequisites.cs` is a scoped flag for scripted companion levels (`AddClassLevels`); the selection field of the same name is the one Builds use.

### Ranks and grants

- **Ranks:** `FeatureCollection.AddFact` and `GetRank`, `Feature.AddRank` and `SetSource`.
- **Grants:** `UnitLogic/FactLogic/AddFacts.cs` (`OnActivate`, `UpdateFacts`) holds its facts while its own fact is held (its `HasDifficultyRequirements` gate follows the game setting). `Designers/Mechanics/Facts/AddFeatureOnClassLevel.cs` `Apply` grants from a class level on (with `m_AdditionalClasses`, `m_Archetypes`), or below it under `BeforeThisLevel`, re-run on every level gained. `AddFeatureIfHasFact` grants while another fact is held; `AddFeatureOnApply` grants once, for good; `RemoveFeatureOnApply` removes; `UnitLogic/FactLogic/AddParametrizedFeatures.cs` grants a parametrized feat with its parameter. `AddProficiencies`, `AddClassSkill` and `AddMechanicsFeature` give proficiencies, class skills and flags.

### Skill ranks

- **Points** (`Actions/ApplySkillPoints.cs`, `LevelUpHelper.GetTotalSkillPoints`): each non-mythic class level gives its class's `SkillPoints` plus its archetypes' `AddSkillPoints` (`ClassData.CalcSkillPoints`). The Intelligence modifier, read without enhancement, adds half its value per level, rounded up on odd levels and down on even ones. The total is at least one point per level, and handlers such as `AddSkillPointPerCharacterLevel` add more.
- **Spending:** `Actions/SpendSkillPoint.cs`: one point per rank, ranks capped at the character level.
- **Class skill bonus:** `EntitySystem/Stats/ModifiableValueSkill.cs` `CalculateBaseValue` adds 3 to a class skill once it has a rank.

### Mythic paths, ranks, feats, abilities and switches

- **Paths are Mythic classes** (`ProgressionRoot.CharacterMythics`, `BlueprintCharacterClass.IsMythic`). `UnitProgressionData.AddClassLevel` raises `MythicLevel` for a Mythic class and `CharacterLevel` for any other, so the character-level feats follow the character level alone.
- **Ranks:** `UnitProgressionData.CanLevelUpMythic` allows a rank up to 10 while mythic experience is ahead of the rank, and the story grants that experience. Model rank timing as a table of the earliest character level for each rank, and let a Build take a rank at that level or later.
- **The first ranks:** `ObligatoryMythicClassesQueue` (`SetObligatoryMythicClasses`) names the classes the next ranks must take. `LevelUpState.NextObligatoryMythicClass` and `BlueprintCharacterClass.MeetsPrerequisites` enforce it, and `ProgressionRoot.MythicStartingClass` is the class of the ranks before a path. The path is chosen at Mythic rank 3.
- **Mythic feats and abilities** come from the path progressions' level entries. MythicStartingProgression, AngelProgression, AeonProgression, AzataProgression, DemonProgression, LichProgression, TricksterProgression, DevilProgression, GoldenDragonProgression, SwarmThatWalksProgression and MythicCompanionProgression each grant `MythicFeatSelection` and `MythicAbilitySelection`, which `SystemMechanicsRoot` names. A path progression counts its own class's levels (AngelProgression lists `AngelMythicClass`). The selection records for these picks are keyed by Mythic level (`FeatureSelectionData.IsMythic`).
- **Switches** come at rank 8:
  - **Gold Dragon and Swarm That Walks** set `m_IsHigherMythic` (`GoldenDragonClass` daf1235b6217787499c14e4e32142523, `SwarmThatWalksClass` 5295b8e13c2303f4c88bdb3d7760a757). Taking one while holding a path calls `UnitProgressionData.ReplaceAllMythicLevelsWithMythicHeroLevels` (from `AddClassLevel`). That method removes every Mythic level (`RemoveMythicLevel` drops the class level, its features and its selection records), re-adds the recorded Mythic feat and ability picks (`GetMythicSelectionsData`), and refills the rank count with levels of the starting Mythic class (`AddFakeLevel`).
  - **Legend:** `LevelUpController.Commit` turns the stand-in class `ProgressionRoot.MythicLegenFake` into `UnitPartLegendPathSelected`, after which the character takes `LegendClass` 3d420403f3e7340499931324640efe96. Its `LegendFeature` 7f99961610390044391f300c7ef5e0c8 sets `AddMechanicsFeature` `LegendaryHero`, which raises `UnitProgressionData.MaxCharacterLevel` to 40, and a Legend character levels on `LegendXPTable`. `LegendProgression` 905383229aaf79e4b8d7e2d316b68715 grants neither Mythic selection and removes path features through `RemoveFeatureOnApply`, so Legend ends the ascensions.
  - **Devil** and the rest of the Legend switch run through story actions (`Designers/EventConditionActionSystem/Actions/RemoveMythicLevels.cs`, `ReplaceAllMythicLevelsWithMythicHeroLevels.cs`). What each keeps: on the Legend switch and on the Devil switch, the character keeps the choices and Mythic feats taken at Mythic levels 1 and 2, and every Mythic choice and Mythic feat from Mythic level 3 onwards is dropped (the user's account of the game). Keep the dropped picks in the Build: they count until the switch. To cite it, read the two story actions above and the switch blueprints' components.
  - **Which paths open** is a story matter (`DialogSystem/Blueprints/BlueprintMythicsSettings.cs`, `MythicClassLockComponent`). Offer every path at its rank.

### Spellbooks, as gates

- `Blueprints/Classes/Spells/BlueprintSpellbook.cs` holds the spells-per-day table, `CastingAttribute`, `Spontaneous`, `IsArcane` and `CasterLevelModifier`.
- `Actions/ApplySpellbook.cs` raises a class's spellbook at each class level, except the levels its `SkipLevelsForSpellProgression` lists. A prestige class advances an existing spellbook through a `BlueprintFeatureReplaceSpellbook` pick.
- In `UnitLogic/Spellbook.cs`, `CasterLevel` is `BaseLevel` plus a mythic share (`GetMythicLevelForCasterLevel`), and `MaxSpellLevel` is the highest level with a non-negative per-day count.
- `PrerequisiteCasterTypeSpellLevel` takes the best `MaxSpellLevel` among the arcane or divine spellbooks (spontaneous only, when asked), counting neither alchemist nor Mythic spellbooks. `PrerequisiteClassSpellLevel` reads one class's spellbook.

### Open questions to settle as you build

- Why `GetMythicSelectionsData` re-keys some picks two ranks later when Gold Dragon or Swarm replaces a path.
