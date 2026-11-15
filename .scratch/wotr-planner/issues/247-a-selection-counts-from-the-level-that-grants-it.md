# A selection counts as held from the level that grants it

Type: build (engine rule)
Status: open
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

## Done when

- A fixture test in `rules` shows the form true from the granting row's level with the slot empty, false before it, and false for a Build whose only matching pick came through another selection.
- The game check compares the form with the game's `PrerequisiteFeature` of a selection (a fixture test in `tools/game-check`).
- The scoped checks pass (`CLAUDE.md`, "Checks").
