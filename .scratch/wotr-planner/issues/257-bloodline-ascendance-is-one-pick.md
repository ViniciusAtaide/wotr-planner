# Bloodline Ascendance is one pick; the Empyreal's 20th row is Ascension

Type: build (engine and data)
Status: superseded (user, 2026-11-15, the component vocabulary): its feature is built from generic components the importer (293) writes; any engine part left is in 295–298
Blocked by: none
Model: Opus to build: a grant becomes a selection gated by the held bloodlines.
Map: [WotR Build Planner](../map.md)

## Problem

Found by [242](242-the-tests-expected-values-from-the-game.md)'s research (2026-11-15), half A (`bloodline_ascendance.rs`).
- **Ascendance is a selection of one** (`BloodlineAscendance` ce85aee1726900641ab53ede61ac5c19, a `BlueprintFeatureSelection`, `Ranks` 1): one of 21 20th-level powers, each needing its bloodline's requisite feature (the Arcane's also the Sage's, the Celestial's also the Empyreal's, the Fey's also the Sylvan's). It is open to a holder of the sorcerer's, the Seeker's, the Nine-Tailed Heir's, the Empyreal's, the Sage's, the Eldritch Scion's or the Sylvan's bloodline. The planner grants the 20th-level row of every bloodline held.
- **The Empyreal's 20th row** (`EmpyrealBloodlineProgression` 8a95d80a3162d274896d50c2f18bb6b1, sorcerer levels only) is the Celestial's Ascension itself (`BloodlineCelestialAscensionAbility` d85e0396d8f68e047b7b67a1290f8dbc); Ascendance offers that same feature to an Empyreal. The planner holds an Apotheosis of its own (`sorcerer/empyreal-sorcerer/apotheosis`).
- **Dragon Disciple levels don't count for the Empyreal:** `EmpyrealBloodlineProgression` lists `SorcererClass` alone, where the Draconic progressions list the sorcerer, the magus and `DragonDiscipleClass` (72051275b1dbb2d42ba9118237794f7c). Ticket 181 left this to the user; it is not one of the user's facts, so the game's answer is applied without asking (234), and the report tells the user.
- **The Dragonheir's dragon** gets nothing from Ascendance (no Draconic requisite feature): agrees, as one chosen option.
- **Ascendance's and Second Bloodline's prerequisites** are in [265](265-mythic-abilities-and-feats.md)'s findings.

## Change

- Ascendance becomes a selection of the 21 powers, each gated by its bloodline's requisite feature; its slot opens at the rank it is taken, as today.
- The Empyreal's row 20 is the Celestial's Ascension (its own `apotheosis` entry goes or maps to that blueprint); "early, held once" then holds for Ascension.
- The Empyreal's options count sorcerer levels only.

## Tests 242 marks as waiting on this ticket

- `bloodline_ascendance.rs`: `a_second_bloodline_gets_its_20th_level_power_too` (one of Demonic Might or the Red Power of Wyrms), `an_eldritch_scions_bloodline_still_gets_its_20th_level_power` (one of the two), `an_empyreal_sorcerers_bloodline_gets_nothing_from_bloodline_ascendance` and the five Empyreal tests after it (Ascension, not an Apotheosis of its own), `dragon_disciple_levels_count_towards_an_empyreal_sorcerers_apotheosis` (they don't).

## Done when

- The tests above hold the game's answers, cited by blueprint and GUID.
- The scoped checks pass.
