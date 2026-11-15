# What a prerequisite reads: an ability score's permanent value, BAB with every bonus, skill ranks

Type: build (engine rule)
Status: open
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

## Done when

- The tests above hold the game's answers, cited.
- A real-data test shows Draven's Hat's profane Intelligence not meeting an Intelligence prerequisite that a +2 headband of Enhancement meets.
- The scoped checks pass.
