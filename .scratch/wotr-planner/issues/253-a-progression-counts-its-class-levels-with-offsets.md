# A progression counts its class levels with the archetype's offset

Type: build (engine rule and data)
Status: open
Blocked by: none
Model: Opus to build: the level a domain's rows read changes per archetype.
Map: [WotR Build Planner](../map.md)

## Problem

Found by [242](242-the-tests-expected-values-from-the-game.md)'s research (2026-11-15): the game's code contradicts the engine's reading of a progression's level.
- **The game:** a progression's level is the sum of its listed classes' levels, each archetype it lists adding its `AdditionalLevel` (`Blueprints/Classes/BlueprintProgression.cs`, `CalcLevel`, WrathDecompiled 6ce4a6b).
- **The Divine Hunter** (paladin): the domain progressions list `DivineHunterArchetype` with `AdditionalLevel` −2, so a Divine Hunter's domain runs two levels behind: Nobility's 8th-level power (`NobilityDomainProgression` 8480f2d1ca764774895ee6fd610a568e) at hunter 10, not 8.
- **The Separatist** (cleric): its second domain's progression (`LuckDomainProgressionSeparatist` 1f26551e7e61436c8ee188064038a896, and the other `…Separatist` ones) grants the 1st-level power at cleric 1; only its numbers count from cleric 3 (`SeparatistCasterLevelPropery` 93d211bd5f4946f3bbed1b651d107377). The planner holds the power from cleric 3.

## Change

- A progression's rows read its classes' levels with each listed archetype's offset; the domains' data names the offsets the game lists.
- The Separatist's second domain grants its 1st-level power at cleric 1; the offset applies to its numbers only, which the planner states in the summary.

## Tests 242 marks as waiting on this ticket

- `option_follow_ups.rs`: `a_separatists_second_domain_comes_two_levels_late` (the 1st-level power is held at cleric 1; the greater power at 8 agrees) and `a_divine_hunter_8_holds_its_domains_level_8_power` (the power comes at hunter 10).

## Done when

- The tests above hold the game's values, cited by blueprint and GUID.
- The scoped checks pass.
