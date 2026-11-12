# The Dragonheir Scion gains Draconic Resistances at level 9, which it should not hold

Type: bugfix
Status: resolved
Blocked by: none
Model: Opus to build (the fix needs a choice between data and a small engine rule); review on another model.
Map: [WotR Build Planner](../map.md)

## Question

Found by the queue's proof on 2026-11-12 (`scripts/proof.sh both <out> 2a60851`, tickets 157, 170, 171, 172, 175 and 176 against the commit before them): 211 of 2,163 Builds differ, and all but three are the intended changes.

| Builds | Fields | Cause |
| --- | --- | --- |
| 85 with Studied Target (slayer, Sanctified Slayer, Tandem Executioner) | sheet | ticket 170's steps |
| 20 bloodrager, 2 Draconic sorcerer | sheet | ticket 170's steps |
| 67 with rogue levels | sheet and slots; 47 also `possible`, false on the old side | ticket 172's Finesse Training slot: the proof's full Builds fill it, and the old engine has no such slot |
| 34 shaman, each holding Battle Master | sheet | ticket 172: the state now records an option's own value |
| **3 fighter / Dragonheir Scion** | sheet | **not intended: this ticket** |

The three are `fighter fighter/dragonheir-scion`, its full Build and its Legend Build.

**The bug.** The Dragonheir Scion takes a draconic bloodline from the sorcerer's options (`libs/game-data/classes/fighter.ron`, selection `fighter/dragonheir-scion/bloodline`: `shared_options` to `sorcerer/bloodline/draconic-*`, `option_levels: ["fighter"]`, `skipped_levels: [1, 3, 15, 20]`). It skips the bloodline's level 3 row, where Draconic Resistances is first granted, because it has its own Draconic Defences ("From 3rd level, +1 natural armour and resistance 5 …; +2 and 10 at 7th, +3 and 20 at 13th"). Ticket [170](170-scaling-features-granted-once.md) added the resistance's steps to the bloodline's rows at 9 and 15. Level 15 is skipped; level 9 is not, since the archetype takes that row's other power. So a Dragonheir Scion now gains `Draconic Resistances` at fighter level 9, a feature whose first grant it skips.

It is the same kind of leak as the Imitator's in ticket 170's review (an archetype that loses a stepped feature inherits a later step). That review's general check covered the sorcerer's and the bloodrager's own archetypes, not other classes that share their bloodline options.

## Build

AFK. Game data, and a small engine rule only if the data cannot say it.

- **Confirm against the guide** (`guide-cache/fighter.html`, in the main checkout, read-only): what the Dragonheir Scion gains from its bloodline and at which levels, and that it never gains the bloodline's resistances.
- **Fix it so the archetype gains no step of a feature whose first grant it skips.** A skipped level drops the whole row, so adding 9 to `skipped_levels` would also drop the power it does take there. Look at what the data can already say (an archetype's `loses`, a selection's own row overrides, a way to skip one feature of a shared row) before adding anything. If a new rule is needed, the smallest one that reads as the game's: a step of a feature is not gained by a character who does not hold the feature. Say why you chose what you chose. If you add that rule, check it against every stepped feature ticket 170 and its follow-up [177](177-more-scaling-features.md) cover, and the Primalist case 170's Answer records, which it may also settle.
- **Sweep every other sharer.** Find every selection, in every class, prestige class and mythic path, that takes options from a stepped bloodline or class feature through `shared_options`, `option_levels` or `skipped_levels` (the Eldritch Scion, the Dragon Disciple, Crossblooded's second bloodline, the Mythic Second Bloodline, and any other), and every place a stepped feature from ticket 170 is granted by reference. For each, work out from the merged rows what it holds at each step level and whether that is right, as ticket 170's archetype table did. Put the table in the Answer.
- **Tests:** a real-data test that a Dragonheir Scion 20 of each colour never holds Draconic Resistances and still gains what the guide gives it at each level; one assertion for each sharer the sweep finds.
- **Snapshots:** the corpus has no Dragonheir Scion, so the snapshot check should pass with the files untouched; if it reports a difference, say what and why before regenerating.
- **Prove it:** ask the main session to rerun the queue's proof. The three Builds above must no longer differ, and no Build that agreed must start to.

## Done when

- A Dragonheir Scion holds no Draconic Resistances at any level, pinned by a real-data test.
- The Answer has the sweep's table, and every leak it found is fixed or ticketed.
- The gold test still passes with its two known gaps, and `scripts/check.sh` passes.

## Answer

Done 2026-11-12 (branch `ticket-179`). Data only: one number in `classes/fighter.ron`; no engine, schema or migration change.

**The guide** (`fighter`, the Dragonheir Scion): "Gains a Draconic Bloodline / Loses Armor Training and Armor Mastery / Loses Weapon Mastery". Its level progression lists "Draconic Bloodline" at 0, "Arcane Strike" at 1, "Fearful Might" at 2, "Draconic Defences" at 3, "Draconic Strike" at 4, "Draconic Presence" at 6, "Wings" at 15 and "Might of Wyrms" at 20, and no other bloodline power. "Draconic Bloodline: A Dragonheir Scion has a Draconic bloodline." "Draconic Defenses: At 3rd level, a Dragonheir Scion gains a +1 natural armour bonus to AC and energy resistance 5 against the bloodline's energy type. At 7th level, the AC bonus increases to +2 and the energy resistance to 10. At 13th level, these increase agains to +3 AC and energy resistance 20." So the bloodline gives it an energy type and nothing else: its own table gives the resistance (Draconic Defences, in place of "Dragon Resistances": "At 3rd level, you gain resist 5 vs. your bloodline's element … At 9th level, the resistance increases to 10 …", `character-creation`), the Wings and its own 20th-level power. It never gains the bloodline's resistances.

**The row at 9 holds only the step.** The ticket says the archetype keeps the level 9 row for "that row's other power"; there is none. Before ticket 170 the Draconic bloodlines had rows at 1, 3, 15 and 20, which the Dragonheir Scion's `skipped_levels: [1, 3, 15, 20]` all skipped: it took no row of its bloodline at all. Ticket 170 added a row at 9 holding only `draconic-resistances`, and that was the one row it did not skip.

**The fix:** `skipped_levels: [1, 3, 9, 15, 20]` on `fighter/dragonheir-scion/bloodline`. It drops exactly the step and nothing else, and it restores the engine's output for the three Builds the proof found to what it was before the queue (the option level the selection records is unchanged, so the sheet's Debug text matches the old side's), so the proof's rerun should show them agreeing.

**What was weighed and rejected:**

- *What the data can already say:* archetypes have no `loses`, and an option has no per-selection row override. Neither was needed: `skipped_levels` already says it, since the row holds nothing else. No "skip one feature of a shared row" was needed either.
- *Dropping the selection's `option_levels`* (a bloodline with no rows at all) would say the same more strongly, but it removes the option level the state records, so the three Builds would still differ from the old side in the proof. Rejected for that.
- *The engine rule* ("a step of a feature is not gained by a character who does not already hold it"). Rejected, for four reasons found in the data:
  1. The data cannot tell a step from a first grant: a step is a repeated grant, and the only way to call a grant a step is "an earlier row of the same owner grants it". On class and archetype rows that is wrong in several places the guide confirms, where an archetype legitimately gains a feature first at a later row of its class's steps: the Possessed Shaman's Wandering Hex only at 14 (guide table: "14 | - | - | Wandering Hex"; the shaman's at 6 and 14), the Battle Singer's Rage Power at 6, 12 and 18 ("Gains fewer Rage Powers"; the skald's at 3, 6, 9, …), the Demon Dancer's at 12, 15 and 18 ("12 | - | Rage Power", "15 | Feat | Rage Power", "18 | - | Rage Power"). The rule would take all of these away, so it could only apply to option rows.
  2. Two sources granting one feature: none in the data. The only features more than one owner grants are the colour families' (ten Draconic, four Elemental, in each of the sorcerer and the bloodrager), of which a character holds one (`NoOtherOptionOfGroup`), and an option is held once through whichever selection. One feature is granted twice on purpose in one row: the Trickster's Sneak Attack at rank 3 ("+2d6 sneak attack at rank 3"), which the Arcane Trickster's `Feature(..., count: 2)` prerequisite reads. A rule would have to keep that.
  3. The Primalist (ticket 170's Answer, item 4): an option-row rule would settle only one half of it (a step at 8 after the row at 4 was traded for rage powers would be dropped); the other half (the step lost when rage powers replace the row at 8) stays, and the guide does not say what the game does with either. Ticket 177 still holds it.
  4. It would make a grant depend on the state at that point of the walk, a new kind of rule for the engine to keep, against a one-number data fix.

  The protection it would give future steps is had instead from a test over the data: `no_selection_gains_a_step_of_a_feature_whose_first_row_it_skips` checks every selection's `skipped_levels` against every option it offers, and fails (it did, for all ten colours, before the fix) when a selection skips a feature's first row but not a later one.

**The sweep.** Every selection, in every class, archetype, prestige class and mythic path, that offers an option whose rows grant a feature more than once (found by walking the data: the Draconic sorcerer bloodlines' Draconic Resistances, the Abyssal's Strength of the Abyss at 9, 13 and 17, the bloodrager's four resistances), every mythic ability granting such a selection, every `option_row`, and every `counts_as` toward a feature. Counts are the engine's at the step levels.

| Stepped feature (option rows) | Sharer | How it takes the option | Held at the steps | Right? |
| --- | --- | --- | --- | --- |
| Draconic Resistances (3, 9, 15) | Sorcerer, Crossblooded's first bloodline, Geomancer, Overwhelming Mage, Nine-Tailed Heir (`sorcerer/bloodline`) | sorcerer + Dragon Disciple levels | 1, 2, 3 | yes (170) |
| | Mythic Second Bloodline (`second-bloodline` grants `sorcerer/bloodline`) | the same selection, by sorcerer + Dragon Disciple levels | taken at MR 1 (level 6): 1 at 6, 2 at 9, 3 at 15 (the gold Build Jack's) | yes |
| | Dragon Disciple continuing a sorcerer's bloodline (`dragon-disciple` in `sorcerer/bloodline`'s `option_levels`) | "continues your bloodline progression uninterrupted" | by bloodline level | yes |
| | Dragon Disciple's Blood of Dragons (`dragon-disciple/bloodline`, `only_without_option`) | "your bloodline level is your class level" | 1 at DD 3, 2 at DD 9 | yes |
| | Eldritch Scion (`magus/eldritch-scion/bloodline`, shared, no skips) | magus + Dragon Disciple levels; "gains spells and abilities related to his bloodline (for example he might grow Wings at level 15)" (`magus`) | 1, 2, 3 at magus 3, 9, 15 | yes |
| | Crossblooded's second bloodline (`feat_lists_only`) | lists and arcana only | none | yes (170) |
| | Seeker (`sorcerer/seeker-sorcerer/bloodline`, shared, skips 3, 9, 13, 15, 17) | its own rows replace them | none | yes (170's test said the Seeker has no Draconic option; it shares all ten: now asserted with a Build) |
| | **Dragonheir Scion** (`fighter/dragonheir-scion/bloodline`, shared, skipped 1, 3, 15, 20) | fighter levels | 0 at 3, **1 at 9**, 1 at 15 | **leak: fixed**, 0 at every level |
| | Bloodline Ascendance (`option_row`: `sorcerer/bloodline` 20) | the 20th row of each option held that `sorcerer/bloodline` offers | no bloodline's row 20 carries a step (asserted) | no step; but it ignores a sharer's skipped rows: **ticket [180](180-bloodline-ascendance-ignores-skipped-rows.md)** |
| Strength of the Abyss (9, 13, 17) | Sorcerer, Second Bloodline, Eldritch Scion | as above | 1, 2, 3 | yes |
| | Seeker | skips 9, 13, 17 | none | yes |
| Bloodrager's Draconic, Demon, Celestial and Serpentine Resistances | Bloodrager and the keeping archetypes (`bloodrager/bloodline`) | bloodrager levels | as 170's table | yes (170) |
| | Second Bloodrager Bloodline (grants `bloodrager/bloodline`) | the same selection, by bloodrager levels | Draconic taken at MR 1: 1 at 6, 2 at 8, 3 at 16 | yes |
| | Reformed Fiend (shares the Abyssal) | bloodrager levels | Demon 1 at 8, 2 at 16 | yes (170) |
| | Primalist | rage powers replace the row at 4, 8, 12, 16, 20 | 170's known consequence | ticket 177 |
| | Hag-Riven, Mixed Blood Rager | offer none of these | none | yes |
| Studied Target (class rows; the Sanctified Slayer's, Tandem Executioner's and Divine Scion's own keys) | none: no selection, `option_row` or `counts_as` names it | | 170's archetype table | yes (170) |

`counts_as` toward a feature: only the Winter Witch's toward the witch's Major and Grand Hexes, neither stepped. So the Dragonheir Scion's was the one leak; Bloodline Ascendance's skipped-row grant is a separate question (not a step), filed as ticket 180.

**Found and left, for ticket 177** (its file has the note): the Dragonheir Scion's own Draconic Defences (3, 7, 13: the guide's sentence above) and Fearful Might ("At 2nd level, a Dragonheir Scion gains a +1 bonus on Intimidate skill checks. This increases by +1 every four additional class levels (max +5 at 18th level).") are each granted once, the same shape as 177's features.

**Tests** (`libs/seeds/tests/game_data/scaling_features.rs`):

- `a_dragonheir_scion_of_each_colour_never_holds_the_bloodlines_powers`: for each colour, a possible Dragonheir Scion 20 holds no Draconic Resistances (nor the bloodline's arcana, claws or Power of Wyrms) at any of levels 1 to 20, and gains Arcane Strike at 1, Fearful Might at 2, Draconic Defences at 3, Draconic Strike at 4, Draconic Presence at 6, Wings at 15 and Might of Wyrms at 20, none the level before. It failed before the fix (count 1 from level 9).
- `every_sharer_of_a_stepped_bloodline_holds_the_steps_its_rows_give`: the Eldritch Scion, the Seeker (Draconic and Abyssal), a bard 6 / Dragon Disciple 10 with Blood of Dragons, the Mythic Second Bloodline and Second Bloodrager Bloodline, each a possible Build, and Bloodline Ascendance's row carrying no step.
- `no_selection_gains_a_step_of_a_feature_whose_first_row_it_skips`: the data-wide check above.
- In `the_sorcerer_and_bloodrager_archetypes_hold_the_steps_their_bloodline_gives`, the Seeker is out of the "no Draconic option" loop, where it passed only because its options are all shared.

**Checks:** the snapshot check passes with the snapshot files untouched (the corpus has no Dragonheir Scion); the gold test passes with its two known gaps (173, 174) unchanged; `scripts/check.sh` passes. The proof is the main session's: the three Dragonheir Scion Builds should stop differing, and nothing else in the data changed.
