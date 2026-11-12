# The Dragonheir Scion gains Draconic Resistances at level 9, which it should not hold

Type: bugfix
Status: open
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
