# Oracle rows, curses and mysteries

Type: build (data)
Status: open
Blocked by: [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: the second curse changes what it grants; rows read from the progressions.
Map: [WotR Build Planner](../map.md)

## Problem

The game check, 242's research (half B part 3 `option_families.rs`; part 1 `scaling_features.rs`) and [243](243-ranked-features-and-filtered-pools.md)'s leftovers differ on `oracle.ron`:
- **The second curse** (the Dual-Cursed Oracle, `DualCursedOracleArchetype` 4a6eb63858244f7ea4ed950a7e798a70, adds `SecondCurseSelection` cc6fda79e8c340b88c84689414a9abbe at 1): its nine options are the `<Curse>CurseNoProgression` features (`LameCurseNoProgression` 857c97115ec745d58cc8fa7bdedc794f, …), the curse's level-1 feature only, never its 5/10/15 rows; the planner offers the full curses. The curses' prerequisites exclude their own `NoProgression` copies.
- **Beneficial Curse** (243's known difference): `BeneficialCurse` (2dda67424ee8e0b4d83ef01a73ca6bff) offers the curses' no-penalty progressions (`<Curse>CurseNoPenaltyProgression`); the planner's slot offers `oracle/oracles-curse`, the curses with their penalties and stages.
- **Final revelations:** `OracleFinalRevelation` (0336dc22538ba5f42b73da4fb3f50849) at 20, for the class and each archetype, beside the per-mystery final revelation.
- **Revelations' prerequisites:** a dozen ask arcanist levels the game doesn't (the Living Deity's share, 13 findings) and one an oracle level.
- **Archetypes:** the Lone Strider's and the Wind Whisperer's mystery selections offer the game's mysteries; the Divine Herbalist's Healer's Way at 2, not 1; the Dual-Cursed Oracle's bonus spells at 1, not 2; the Hermit's Fade from Memory at 14; the Purifier's celestial armour and the Possessed Oracle's Two Minds are granted again (ranks); Haste of the Wind (`WIndWhispererArchetype` 77769e43e63647c2b32298efcc577aca, `WindWhispererHasteFeature` 83bb307b3ee74090b7b86c1a8c0a7612) steps at 7, 12, 18; the Lone Strider's steps follow [254](254-a-features-steps-follow-the-class-level.md).
- The Divine Herbalist's and the Enlightened Philosopher's mystery copies, and the revelations' leaves naming them, are [244](244-the-game-data-laid-out-by-blueprint.md)'s.

## Findings (the game check, `known.txt` section 272)

| The check's group (area / kind) | Findings |
| --- | --- |
| option lists / in game only | 21 |
| prerequisites / not in game | 14 |
| progression rows / in game only | 12 |
| option lists / in planner only | 9 |
| prerequisites / missing in planner, a blueprint it doesn't map | 9 |
| progression rows / a further rank in planner only | 5 |
| progression rows / a further rank in game only | 4 |
| progression rows / in planner only | 3 |
| progression rows / at another level | 2 |
| prerequisites / missing in planner | 1 |
| **all** | **80** |

The 34 entries: `archetype oracle/divine-herbalist`, `archetype oracle/dual-cursed-oracle`, `archetype oracle/lone-strider`, `archetype oracle/possessed-oracle`, `archetype oracle/purifier`, `archetype oracle/seeker`, `archetype oracle/wind-whisperer`, `class oracle`, `selection option oracle/oracles-curse/blackened`, `selection option oracle/oracles-curse/demonic`, `selection option oracle/oracles-curse/hellbound`, `selection option oracle/oracles-curse/lame`, `selection option oracle/oracles-curse/plagued`, `selection option oracle/oracles-curse/powerless-prophecy`, `selection option oracle/oracles-curse/pranked`, `selection option oracle/oracles-curse/wasting`, `selection option oracle/oracles-curse/wolf-scarred-face`, `selection option oracle/revelations/ancestor/spirit-of-the-warrior`, `selection option oracle/revelations/ancestor/storm-of-souls`, `selection option oracle/revelations/battle/combat-healer`, `selection option oracle/revelations/battle/iron-skin`, `selection option oracle/revelations/bones/soul-siphon`, `selection option oracle/revelations/bones/spirit-walk`, `selection option oracle/revelations/flame/firestorm`, `selection option oracle/revelations/flame/form-of-flame`, `selection option oracle/revelations/life/lifesense`, `selection option oracle/revelations/nature/life-leech`, `selection option oracle/revelations/waves/blizzard`, `selection option oracle/revelations/waves/punitive-transformation`, `selection option oracle/revelations/wind/invisibility`, `selection option oracle/revelations/wind/thunderburst`, `selection oracle/lone-strider/mystery`, `selection oracle/second-curse`, `selection oracle/wind-whisperer/mystery`.

The report lists each finding under its group, marked `[known: 272]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The second curse and Beneficial Curse offer the game's options, granting what those grant; the rows and prerequisites become the game's; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `option_families.rs`: `the_dual_cursed_oracles_second_curse_is_a_slot_of_its_own` and `a_dual_cursed_oracle_1_picks_two_curses_the_second_not_offering_the_first` (the second curse gives only its level-1 feature), `beneficial_curse_offers_a_dual_cursed_oracle_neither_of_her_curses` (the no-penalty progressions).
- `mythic.rs`: `beneficial_curse_gives_an_oracle_curse_slot_offering_only_curses_not_held`.
- `scaling_features.rs`: `the_oracle_archetypes_step_the_features_the_guide_gives_steps_to` (Haste of the Wind 7, 12, 18).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 272 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
