# 253: a progression counts its class levels with the archetype's offset

**What the data says now.**

- `option_level_archetypes` is a map from an archetype to the levels it adds (`FeatureSelection::option_level_archetypes`,
  the game's `ArchetypeWithLevel.AdditionalLevel`): `{"witch/stigmatized-witch": 0}` on the oracle's curse, the
  Possessed Oracle's curse and the Stigmatized Witch's curse (as before), and `{"hunter/divine-hunter": -2}` on
  `hunter/divine-hunter/domain`. An option's level adds that archetype's class levels plus the offset, and 0 when the
  sum is below 0. A Divine Hunter's domain is at hunter level less 2: its 1st-level power at hunter 3, its 8th-level
  power at hunter 10. The selection's summary says so.
- `later_rows_behind` (was `levels_behind`; the Separatist's second domain, `cleric/separatist/domain`, 2) lags the
  rows after the first only: the option level is the cleric's levels less 2, but at least 1. Its 1st-level power comes at cleric 1, the later
  rows two levels late. The Separatist's summaries say its numbers count two levels lower, which the planner doesn't
  compute.

**The game.** `Blueprints/Classes/BlueprintProgression.cs`, `CalcLevel` (WrathDecompiled 6ce4a6b): each listed class
counts unless it has a listed archetype; each class taken with a listed archetype counts its level plus that
archetype's `AdditionalLevel`, at least 0. Every domain progression (`NobilityDomainProgression`
8480f2d1ca764774895ee6fd610a568e, `LuckDomainProgression` and the others) lists `DivineHunterArchetype`
f996f0a18e5d945459e710ee3a6dd485 with -2. The Separatist's domains are progressions of their own
(`LuckDomainProgressionSeparatist` 1f26551e7e61436c8ee188064038a896 and 36 more), counting the cleric's level with no
offset, whose rows are the domain's first row at 1 and each later row two levels later (Luck: 1 and 8 for 1 and 6;
Fire: 1, 8, 14 for 1, 6, 12, 20, so the 20th-level row never comes). `SeparatistCasterLevelPropery`
93d211bd5f4946f3bbed1b651d107377 lowers the numbers by two.

**Not modelled.** The game's domain progressions list the cleric, the inquisitor, the hunter and the arcanist with the
Divine Hunter and the Magic Deceiver, so one domain held by a character with levels in two of them counts both. The
planner levels each domain by the selection it was taken through (one class), as before.

**Tests.** `libs/seeds/tests/game_data/option_follow_ups.rs`:
`a_separatists_second_domain_gives_its_1st_level_power_from_cleric_1`,
`a_separatists_second_domain_comes_two_levels_late`, `a_divine_hunters_domain_runs_two_levels_behind_its_hunter_level`.
