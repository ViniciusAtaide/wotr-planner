# A progression counts its class levels with the archetype's offset

Type: build (engine rule and data)
Status: resolved
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

## Answer

**Built** (2026-11-15). The game's progression level is the sum of its listed classes' levels, a class with a listed archetype counting only through it, each archetype adding its `AdditionalLevel` with the sum at least 0 (`Blueprints/Classes/BlueprintProgression.cs`, `CalcLevel`, WrathDecompiled 6ce4a6b). The planner's option level now adds the offset.

- **The data.** `FeatureSelection::option_level_archetypes` is a map from an archetype to its offset (was a set): `{"witch/stigmatized-witch": 0}` on the three curse selections as before, `{"hunter/divine-hunter": -2}` on `hunter/divine-hunter/domain` (every domain progression lists `DivineHunterArchetype` f996f0a18e5d945459e710ee3a6dd485 with -2). The Divine Hunter's domain is at hunter level less 2: the 1st-level power at hunter 3, Nobility's 8th-level power (`NobilityDomainProgression` 8480f2d1ca764774895ee6fd610a568e) at hunter 10.
- **The Separatist.** Its domains are progressions of their own (`LuckDomainProgressionSeparatist` 1f26551e7e61436c8ee188064038a896 and 36 more) counting the cleric's level with no offset, with the domain's first row at 1 and each later row two levels later. `levels_behind` is renamed `later_rows_behind` and now lags the rows after the first (the option level is the classes' levels less 2, at least 1 once there is a level: `lagging` in `engine/walk/options.rs`): the 1st-level power at cleric 1, the rest two levels late, as the game's rows fall. The summaries say its numbers count two levels lower (`SeparatistCasterLevelPropery` 93d211bd5f4946f3bbed1b651d107377), which the planner doesn't compute; the Divine Hunter's summary says its powers come at hunter level less 2.
- **Code.** `class_levels_of` (`engine/walk/options.rs`) adds a listed archetype's class levels plus its offset (`saturating_add_signed`); `GameData::check` names the map's keys. README and note `libs/game-data/notes/253-a-progression-counts-its-class-levels-with-offsets.md`.

**Decisions.**
- Ours: the offset as the map's value, as the game's `ArchetypeWithLevel` holds it, rather than a second field.
- Ours: the Separatist's moved rows as a lag of the later rows (`later_rows_behind`) rather than a copy of every domain's rows; the rows reached are the game's for all 37 Separatist progressions (Fire, Air, Earth, Water's 20th-level row never comes, as in the game). The option level it stores is 1 from cleric 1 to 3, a planner value.
- Not done: the game's domain progressions count the cleric, inquisitor, hunter and arcanist levels together for one domain; the planner levels a domain by the selection it was taken through, as before (in the note).

**Tests.**
- Real data, `libs/seeds/tests/game_data/option_follow_ups.rs`, both off ignore: `a_separatists_second_domain_gives_its_1st_level_power_from_cleric_1` (Luck's Bit of Luck at cleric 1; `LuckDomainProgressionSeparatist`) and `a_divine_hunters_domain_runs_two_levels_behind_its_hunter_level` (Inspiring Command from hunter 10, Inspiring Word, `NobilityDomainBaseFeature` a1a7f3dd904ed8e45b074232f48190d1, from hunter 3; the data's offset -2). `a_separatists_second_domain_comes_two_levels_late` still holds (Divine Fortune at 8).
- Rules, hand-made data, `libs/rules/tests/engine/option_levels.rs`: `a_separatists_second_domain_gives_its_first_row_at_once_and_the_later_ones_two_levels_late` (replaces `a_separatists_second_domain_lags_two_levels_behind`, whose values were the old model's), `an_archetypes_offset_moves_every_row_and_never_goes_below_0`.

## Done when

- The tests above hold the game's values, cited by blueprint and GUID.
- The scoped checks pass.
