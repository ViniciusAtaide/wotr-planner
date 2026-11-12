# Studied Target and Draconic Resistances are granted once, not at each step

Type: task
Status: resolved
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

- Data only: the extra grants in `slayer.ron` and `sorcerer.ron`, a `libs/game-data/README.md` note, a real-data test. The copies to cover, each granted once today (found in the ticket 160 review; sweep for more):
  - **Studied Target** (`slayer/studied-target`) in the slayer's own table (`slayer.ron` line 121) and in every slayer archetype that keeps it: the Arcane Enforcer (line 155), the Bloodseeker (around 206), the Deliverer (around 232), the Executioner (around 248) and the Vanguard (around 317), and any other the sweep finds. The Sanctified Slayer's (`inquisitor.ron`) and the Tandem Executioner's (`hunter.ron`) copies, whose summaries also say they rise.
  - **Draconic Resistances** (`sorcerer/bloodline/draconic/draconic-resistances`) at bloodline levels 9 and 15, for every Draconic colour.
  - **The bloodrager's resistances,** which rise in steps too, each granted once (`bloodrager.ron`): Draconic Resistance (4; 8 and 16 by its summary, in each Draconic colour's rows), Demon Resistances (8; 16), Celestial Resistances (4; 12) and Serpentine Resistances (4; every four levels from 8).
  
  Give each the same treatment, or say in the Answer why not.
- Delete the five entries from `gold.rs`'s `KNOWN_GAPS`; regenerate the snapshots (`SNAPSHOTS=write cargo test -p seeds --test game_data snapshots`) and check that only the stepped features' lines move.

**Done when:** the gold test passes without the five entries; the snapshot diff shows only the new grants; `scripts/check.sh` passes.

## Answer

Done 2026-11-12 (branch `ticket-170`). Data only: no engine, schema or migration change.

**What a repeated grant does.** The seed and the data check accept it (the Elemental Rampager's Energy Resistance, Strength of the Abyss and the Divine Scion's Studied Target already do). `CharacterState.features` counts the grants of a feature; the engine reads that count in three places only: a prerequisite `Feature { count }` holds when the count is at least `count` (`numeric` in `state.rs`, `>=`), a bonus in `bonuses.ron` holds for a count above 0, and `ability_bonuses` multiply by the count. None of the changed features has an ability bonus, a bonus entry or a prerequisite above `count: 1` (Defensive Study names Studied Target with `count: 1`), so no number moves. The snapshot diff below confirms it: only the new grant lines appear.

**The rows.** An archetype's row replaces the base class's at its level, and the check refuses an archetype row equal to the base's. So the slayer's base rows 5, 10, 15 and 20 carry Studied Target, and an archetype has a row at those levels only where it differs from the base (the Spawn Slayer's 5 with Studied Spawn, the Arcane Enforcer's 10, the Bloodseeker's 10 and 20, the Deliverer's 10, the Executioner's 10, the Stygian Slayer's 10 and the others it already had). The Imitator loses Studied Target and is unchanged. (The ticket's "Vanguard around 317" was the Stygian Slayer; the Vanguard has no level 1 row and inherits it.)

**Features changed**, each with the guide's sentence:

- **Studied Target** (`slayer/studied-target`, `classes/slayer.ron`: the base rows, and the archetype rows that replace a base row at 5, 10, 15 or 20), now at 1, 5, 10, 15 and 20. Guide (`slayer`): "At 5th, 10th, 15th and 20th levels, the bonuses increase by +1 (maximum +5)." The Arcane Enforcer, Bloodseeker, Deliverer, Executioner, Spawn Slayer, Stygian Slayer and Vanguard tables each list "Studied Target" at 1 and say nothing else about it.
- **Sanctified Slayer** (`inquisitor/sanctified-slayer/studied-target`, `classes/inquisitor.ron`, rows 5, 10, 15, 20): guide (`inquisitor`): "Studied Target    This is identical to the Slayer ability." The row 5 keeps Bane and the row 15 the teamwork feat slot, as the base inquisitor's rows give them.
- **Tandem Executioner** (`hunter/tandem-executioner/studied-target`, `classes/hunter.ron`, rows 5, 10, 15, 20): guide (`hunter`): "Studied Target    This is identical to the  Slayer ability . The Tandem Executioner shares the bonuses with her companion as long as it is within 60'."
- **Draconic Resistances** (`sorcerer/bloodline/draconic/draconic-resistances`, `classes/sorcerer.ron`, all ten colours), now at bloodline levels 3, 9 and 15. Guide (`character-creation`, "Dragon Resistances"): "At 3rd level, you gain resist 5 vs. your bloodline's element and a +1 natural armour bonus. At 9th level, the resistance increases to 10 and the armour bonus increases to +2. At 15th level, the armour bonus increases to +4."
- **Bloodrager's Draconic Resistance** (`bloodrager/bloodline/draconic/draconic-resistance`, all ten colours), at 4, 8 and 16. Guide (`bloodrager`): "At 4th level, you gain resistance 5 against your bloodline's energy type and a +1 natural armour bonus to AC. At 8th level, resistance increases to 10 and the armour bonus to +2. At 16th level, your natural armour bonus increases to +4."
- **Demon Resistances** (bloodrager), at 8 and 16: "At 8th level, you gain resistance 5 to acid, cold and fire. At 16th level, the resistances increase to 10."
- **Celestial Resistances** (bloodrager), at 4 and 12: "At 4th level, you gain resistance 5 to acid and cold. At 12th level, the resistances increase to 10."
- **Serpentine Resistances** (bloodrager), at 4, 8, 12, 16 and 20: "At 4th level, you gain a +1 natural armour bonus to AC and +1 to saves vs. poison. At 8th level and every four levels thereafter, these bonuses increase by +1 (max +5 at 20th level)."

**Snapshot check**, before regenerating (every line is a stepped feature gained at a later level; `git status` shows only the three snapshot files `class-slayer.txt`, `gold-enforcer.txt`, `gold-jack.txt` beside the data, the gold test and the docs):

```text
the engine's output differs from the snapshots in 3 of 44 Builds

Changes, each with the Builds it's in:
  + level.10.feature/slayer/studied-target = Studied Target (Slayer) in 2 Builds: class-slayer, gold-enforcer
  + level.15.feature/slayer/studied-target = Studied Target (Slayer) in 2 Builds: class-slayer, gold-enforcer
  + level.5.feature/slayer/studied-target = Studied Target (Slayer) in 2 Builds: class-slayer, gold-enforcer
  + level.15.feature/sorcerer/bloodline/draconic/draconic-resistances = Draconic Resistances in 1 Build: gold-jack
  + level.20.feature/slayer/studied-target = Studied Target (Slayer) in 1 Build: class-slayer
  + level.9.feature/sorcerer/bloodline/draconic/draconic-resistances = Draconic Resistances in 1 Build: gold-jack

The same, levels and ranks wildcarded:
  level.*.feature/slayer/studied-target: + Studied Target (Slayer) in 2 Builds, 7 lines
  level.*.feature/sorcerer/bloodline/draconic/draconic-resistances: + Draconic Resistances in 1 Build, 2 lines
```

The corpus has no Build for the bloodrager's bloodlines, the Sanctified Slayer, the Tandem Executioner or the other slayer archetypes, so their steps show in no snapshot; the new test `scaling_features.rs` pins them.

**Gold test:** the five entries are deleted (`KNOWN_GAPS` has four left, untouched: Mythic Shield Focus (171), Finesse Training (Light Shield) (172), the Elk companion (174) and the Enforcer's Uncanny Dodge); the `STEPS` constant goes with them. `a_known_gap_hides_no_other_failure_on_its_line` used the Draconic entry as its fixture, so it now builds its own one-entry list (a made-up `Why::Bug(0)` on Draconic Resistances) and still proves that an entry hides only its failure, not Wings on the same line.

**Test:** `libs/seeds/tests/game_data/scaling_features.rs` (declared in `main.rs`): the slayer's counts at 1, 4, 5, 9, 10, 14, 15, 19, 20; each slayer archetype that keeps it; the Sanctified Slayer's, Tandem Executioner's and Divine Scion's; each Draconic colour's sorcerer resistances; and the bloodrager's four resistances.

**Features of the same shape, found and left** (the ticket doesn't name them and the guide's text for each is not checked here; none is changed): the sorcerer's other resistance bloodlines, each granted once at 3 and rising at 9 (Demon Resistances, Celestial Resistances, Elemental Resistances 10 then 20, Infernal Resistances, the Undead's Death's Gift) and Serpentine's Snakeskin (9, 13, 17); the bloodrager's Infernal Resistance (4, 8) and Abyssal Bloodrage (12, 16, 20); Corpse Studying (alchemist, 5 and every 6 levels), Cunning Elusion (barbarian, every 4), Overwhelming Power (kineticist, 3 and every 3), Fey Stalker (druid, 8, 13, 18), Inhibit Courage and Battle Spirit (arcanist, shaman, 5/11/17 and 8/16), Adaptive Physiology (alchemist, 6 and 18), Lycanthropic Aspect (shifter, 8, 16), Cold-Minded (assassin, 5, 8), the Cold Specialization (winter witch, 5, 10), Crush and Tear (flesheater, 5, 11, 17), Fiendish Maw (skald, 7, 12, 17) and Invulnerable Defenses. They are one decision each; granting them again the same way needs the guide's sentence for each.
