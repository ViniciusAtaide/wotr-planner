# Studied Target and Draconic Resistances are granted once, not at each step

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Found by the gold standard ([ticket 160](160-gold-builds.md)). The game's level-up names each step of two scaling features, and the write-ups list them there: the Enforcer's "Studied Target +2" at slayer 5, "+3" at 10 and "+4" at 15; Jack-in-the-Green's "(Dragon Resistances II)" at sorcerer 9 and "(Dragon Resistances III; Wings)" at 15 (the Draconic second bloodline's). The data grants each once: `slayer/studied-target` at slayer level 1 (`libs/game-data/classes/slayer.ron`, in the slayer's table and every archetype's), and `sorcerer/bloodline/draconic/draconic-resistances` at bloodline level 3 (`classes/sorcerer.ron`). Their summaries already say they rise ("+1 more at 5th, 10th, 15th and 20th"; "10 and +2 at 9th, and +4 natural armor at 15th"), but nothing is gained on the sheet at those levels. The write-ups are right (user, 2026-11-11), so the sheet should gain them there.

**Repro:** `gold.rs`'s two Builds (`libs/seeds/tests/game_data/gold.rs`, `builds()`): the engine's sheet gains Studied Target at the Enforcer's level 1 only, and Draconic Resistances at Jack's level 6 (MR 1's Second Bloodline) only. Its known gaps name this ticket:

```text
enforcer level 5: Studied Target +2: Studied Target: not gained here, but at level 1
enforcer level 10: Studied Target +3: Studied Target: not gained here, but at level 1
enforcer level 15: Studied Target +4: Studied Target: not gained here, but at level 1
jack level 9: (Dragon Resistances II): Draconic Resistances: not gained here, but at level 6
jack level 15: (Dragon Resistances III; Wings): Draconic Resistances: not gained here, but at level 6
```

**The guide:** its tables list each feature once (the slayer's "Studied Target" at 1, the Draconic bloodline's "Draconic Resistances" at 3); its text gives the steps ("+1 at 5th, 10th, 15th, 20th" for the slayer's copies; resistance 10 and +2 natural armor at 9th, +4 at 15th). The data already grants a stepped feature at each step elsewhere: the Divine Scion's Studied Target at paladin 1, 5, 10, 15 and 20 (`classes/paladin.ron`), and every class's Sneak Attack per die. So this is how the data records a step, not a disagreement with the guide.

**Expected:** a slayer (and each slayer archetype keeping Studied Target) gains Studied Target again at 5, 10, 15 and 20; a Draconic bloodline gains Draconic Resistances again at bloodline levels 9 and 15. The gold test's five entries come off its known-gaps list.

## Build

- Data only: the extra grants in `slayer.ron` and `sorcerer.ron`, a `libs/game-data/README.md` note, a real-data test. Sweep the other copies of both features (the Sanctified Slayer's and the Tandem Executioner's Studied Target, which their summaries also say rise; the bloodrager's and the Dragon Disciple's draconic resistances, if any) and give each the same treatment, or say in the Answer why not.
- Delete the five entries from `gold.rs`'s `KNOWN_GAPS`; regenerate the snapshots (`SNAPSHOTS=write cargo test -p seeds --test game_data snapshots`) and check that only the stepped features' lines move.

**Done when:** the gold test passes without the five entries; the snapshot diff shows only the new grants; `scripts/check.sh` passes.
