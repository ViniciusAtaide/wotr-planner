# What a prerequisite reads: an ability score's permanent value, BAB with every bonus, skill ranks

Type: build (engine rule)
Status: resolved
Blocked by: none
Model: Opus to build: it changes which gear and features count toward prerequisites.
Map: [WotR Build Planner](../map.md)

## Problem

Found by [242](242-the-tests-expected-values-from-the-game.md)'s research (2026-11-15): the game's code contradicts the engine's rule that "an item's always-on bonuses count at every level, for prerequisites".
- **Ability scores:** `PrerequisiteStatValue.GetStatValue` reads an attribute's `PermanentValue` (`Blueprints/Classes/Prerequisites/PrerequisiteStatValue.cs`, WrathDecompiled 6ce4a6b). `ModifiableValueAttributeStat.CalculatePermanentValue` keeps only the modifiers `Modifier.IsPermanent` accepts: a racial modifier, or an Enhancement modifier from an item (`EntitySystem/Stats/ModifiableValueAttributeStat.cs`, `EntitySystem/Stats/ModifiableValue.cs`). An item's profane, insight, alchemical, circumstance or untyped bonus to a score (Draven's Hat's Intelligence +2 profane, `DravensHatFeature` 21007755b76402743bec22faacfb5d02) doesn't count; a Belt of Dexterity's Enhancement does.
- **Inherent bonuses:** 242's research disagrees with itself on the tomes and manuals (its R3 says `IsPermanent` refuses Inherent; its gold row says a tome counts). Read how a tome's bonus is applied (to the base value, or as a modifier) before deciding, and record which.
- **BAB:** every modifier counts (`ModifiableValue.CalculatePermanentValue`). Perfect Mind (`GoldenDragonProgression` a6fbca43902c6194c947546e89af64bd at Golden Dragon 2) adds a `BaseAttackBonus` modifier up to 15 (`DragonLevel2BABOverride` f4af14e9f4408d44c830d891fee8f4a5): a real BAB, for attacks and prerequisites. The planner raises BAB for prerequisites only.
- **Skills:** a skill prerequisite reads `BaseValue`, the ranks. Perfect Mind's skill floor (`GoldenDragonSharedSkillUMD` 4e0b919e4fbd85142bce959fae129d1a; `UnitLogic/FactLogic/AddGoldenDragonSkillBonus.cs`, `OnTurnOn`) raises each skill's total to the Mythic rank, not its ranks, so it meets no rank prerequisite. The planner counts it as ranks for prerequisites.

## Change

- Ability prerequisites count a racial modifier and an item's Enhancement bonus, and no other gear bonus; the tome question above is settled first, from the code.
- Perfect Mind raises the BAB itself (the sheet's BAB and the attack line), and its skill floor raises totals, never ranks.
- The MCP instructions and the `wotr-build` skill say which gear counts for prerequisites (the main session edits instructions; this ticket proposes the words).

## Tests 242 marks as waiting on this rule

- `follow_ups.rs`: `perfect_mind_gives_bab_15_and_skill_ranks_at_the_mythic_rank_for_prerequisites` (BAB 15 is a real BAB; skill-rank prerequisites don't see the floor).
- `gear_engine.rs`: `greater_two_weapon_fighting_at_dex_17_needs_the_belt_of_incredible_dexterity_2` agrees (an Enhancement belt counts); it gains the citation and a case where a non-Enhancement item bonus doesn't count.

## Answer

### Built (2026-11-15)

**What was built:**
- **Ability scores:** ability prerequisites read `CharacterState::permanent_ability_scores`.
  - It is the scores every number reads (`ability_scores`), but of the gear's bonuses it keeps only those `BonusType::counts_toward_prerequisites` accepts, Enhancement and Racial, stacked among themselves.
  - `WornGear` holds both arrays (`scores`, `permanent`), and `Walker::refresh_ability_scores` builds both from one base (`own_base`, then `with_feature_bonuses`).
  - The features' own `ability_bonuses` count in both, as before.
- **BAB:** Perfect Mind raises the BAB itself to 15 + the Mythic rank.
  - The data field `bab_floor: Some(15)` is now `bab_at_least_mythic_rank_plus: Some(15)`.
  - `Walker::refresh_bab` takes the largest of the class tables', Athletics III's and this. `refresh_floors` and the new `Walker::set_mythic_rank` (`walk/ascension.rs`) refresh it.
  - `CharacterState::prerequisite_bab` and `bab_floor` are gone: a BAB prerequisite, the sheet and the attack line all read `state.bab`.
- **Skills:** a skill-rank prerequisite reads the ranks spent (`skill_ranks`).
  - Perfect Mind's floor raises the skill totals only, through `skill_ranks_in_total`, renamed from `prerequisite_skill_ranks`. The totals and `TenRanks` read it.
- **Docs:** the MCP instructions and the `wotr-build` skill now say that only an item's enhancement bonus counts for ability prerequisites. The data README describes the field, and the note is `libs/game-data/notes/252-what-a-prerequisite-reads.md`.
- **`rules` files touched:**
  - `bonus.rs`, `state.rs`, `game_data.rs`, `engine/catalog.rs`, `engine/totals.rs`
  - `engine/walk/{gear,hold,ascension,bonuses}.rs`
  - `test_world/world/{gear,mod,paths}.rs`
  - `tests/engine/{gear,path_rules}.rs`

**Decisions:**
1. **The builder, from the code: a tome or manual doesn't count toward an ability prerequisite.**
   - A manual's ability adds a permanent feature through `ContextActionAddFeature`, for example `ManualOfGainfulExercisePlus2_Ability` a8b94294f28dec848b11c2651dcbabe0 → `ManualOfGainfulExercisePlus2_Feature` e5ed74f06e40f994da1727662f9c9169.
   - That feature's `AddStatBonus` is Inherent, and `AddStatBonus.OnTurnOn` adds it as a modifier (`AddModifierUnique`), not to the base value.
   - `Modifier.IsPermanent` refuses Inherent. 242's R3 was right and its gold row was wrong.
2. **The builder, from the code: Perfect Mind's BAB is 15 + MR, not 15.** The guide, the game's own text and this ticket all say 15.
   - `DragonLevel2BABOverride` adds −(BAB − 15 − rank), at least 0, and its rank is the `MythicLevel`.
   - `PropertyCalculatorComponentHelper.GetPropertyValue` evaluates the property in the fact's own context, so the rank getter reads the Mythic rank.
   - `EntityFact.Reapply` removes the modifier before working it out again.
   - Following the map's rule (the code's behaviour is modelled where it differs from the text), the data and the summary follow the code, and the summary says where it differs. **This contradicts the ticket's text: the main session should confirm it.**
3. **The builder: a gear penalty doesn't lower a prerequisite's score.** Penalties are untyped, and `IsPermanent` refuses untyped modifiers. The invented world's Scholar's Headband test now pins this.
4. **The builder: the invented world's Bracers of Agility are now Racial Dexterity +2.** They were untyped, which no longer counts. Racial is the one other type that counts and stacks with Enhancement, so the gear-need tests keep a second item that adds up. The "two of three" and "one and either of two" needs now use two invented feats over several scores, `AtLeast` and `All`. The example set in the "two of three" test is the engine's choice of a valid set.
5. **Not done (follow-up):** feature bonuses that `IsPermanent` refuses still count toward prerequisites. Legend's +4 (`LegendFeature` 7f99961610390044391f300c7ef5e0c8) and the Swarm's are untyped (`Descriptor: None`), while the Dragon Disciple's +2s are Racial and do count. Perfect Mind's skill floor also still shows as "Ranks" in the totals rather than as its own modifier.

**Tests:**
- **Real data:**
  - `follow_ups.rs`:
    - `perfect_mind_raises_the_bab_itself_to_15_plus_the_mythic_rank`: a wizard 17 at MR 9 has BAB 24, and BAB 8 at MR 7. It replaces `perfect_mind_gives_bab_15_…`.
    - `perfect_minds_skill_floor_raises_totals_and_meets_no_skill_rank_prerequisite`: Perception has 0 ranks, its 1-rank prerequisite fails, and its total counts 9. It replaces the test that waited on this ticket, now un-ignored.
  - `gear_engine.rs`:
    - `greater_two_weapon_fighting_at_dex_17_needs_the_belt_of_incredible_dexterity_2` (unchanged, already cited).
    - `a_manuals_inherent_dexterity_doesnt_meet_greater_two_weapon_fightings_dexterity`: Dex 19 on the sheet, "Needs Dex 19 (have 17)".
    - `dravens_hats_profane_intelligence_doesnt_meet_combat_expertise_and_an_enhancement_headband_does`: Int 13 on the sheet, "Needs Int 13 (have 11)", and possible with a Headband of Vast Intelligence +2.
- **Invented world:**
  - `path_rules.rs`: `perfect_mind_raises_the_bab_by_the_mythic_rank_and_no_skills_ranks`.
  - `gear.rs`: `a_manuals_inherent_bonus_opens_no_ability_prerequisite`, `an_items_untyped_penalty_lowers_the_score_but_refuses_no_ability_prerequisite`, and the four need tests rebuilt as above.
- **Snapshots:** none changed. The check stays silent: no corpus Build holds Perfect Mind or non-enhancement gear.

**Citations** (WrathDecompiled 6ce4a6b for the code):
- **Prerequisites:** `PrerequisiteStatValue.GetStatValue` (`Kingmaker/Blueprints/Classes/Prerequisites/PrerequisiteStatValue.cs`): a skill reads `BaseValue`, any other stat `PermanentValue`.
- **Ability scores:**
  - `ModifiableValueAttributeStat.CalculatePermanentValue` with `ModifiableValue.FilterIsPermanent`.
  - `ModifiableValue.Modifier.IsPermanent`: a Racial modifier, or an Enhancement one with an `ItemSource`.
  - `AddStatBonusEquipment.OnTurnOn`: `AddItemModifierUnique` sets the item as the source.
- **BAB:** `ModifiableValue.CalculatePermanentValue` counts every modifier toward BAB.
- **Perfect Mind:**
  - BAB: `DragonLevel2BABOverride` f4af14e9f4408d44c830d891fee8f4a5 at `GoldenDragonProgression` a6fbca43902c6194c947546e89af64bd level 2.
  - Skills: `GoldenDragonSharedSkillUMD` 4e0b919e4fbd85142bce959fae129d1a, through `AddGoldenDragonSkillBonus.OnTurnOn`.
- **Items and feats:**
  - Belt: `BeltOfDexterity2` b6af4c1834999e74497b41588d1071cd (`Dexterity2` 2042beb64a5e12443be55b413d7ff733, Enhancement).
  - Manual: `ManualOfQuicknessOfActionPlus2` b8ea6d2f787c7004c9cab9f519f687f8 (`_Feature` 741d033afc9eebf4ca4a5ba014100ee6, Inherent).
  - Draven's Hat: `DravensHatItem` 0160c8eabe704db41b6309af0d26a8e0. Its Intelligence +2 Profane is on `DravensHatEnchantment` b3894ce33c4c0ce4abf0dbcff0d6729f, not on `DravensHatFeature`, which this ticket's Problem named.
  - Headband: `HeadbandOfIntelligence2` 22184af355826444087506d7f2f8f0b1 (`Intelligence2` 0f157adfc02708441b02725a24e8eb71, Enhancement).
  - Feat: `CombatExpertiseFeature` 4c44724ffa8844f4d9bedb5bb27d144a (Intelligence 13).

**MCP instructions** (`apps/mcp/src/server.rs`), still under the 4,000-character test:
- Before: "An item's always-on bonuses count at every level, for prerequisites and every number (never for skill points), so it can open a feat or let a score be lowered".
- After: "An item's always-on bonuses count at every level for every number (never for skill points); ability prerequisites count only its enhancement bonus, as the game does (not a tome's), so it can open a feat or let a score be lowered".

## Done when

- The tests above hold the game's answers, cited.
- A real-data test shows Draven's Hat's profane Intelligence not meeting an Intelligence prerequisite that a +2 headband of Enhancement meets.
- The scoped checks pass.
