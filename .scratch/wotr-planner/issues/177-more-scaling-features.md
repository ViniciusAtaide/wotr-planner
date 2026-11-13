# More scaling features are granted once, not at each step

Type: bugfix
Status: open
Blocked by: none
Model: Sonnet to build (data against the guide, as ticket 170); Opus to review the data and the archetype table.
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 170](170-scaling-features-granted-once.md) granted Studied Target, the Draconic bloodlines' resistances and the bloodrager's four resistances at each step the guide names, and found others of the same shape that it left. The independent review confirmed three against the guide. Each is granted once in the data, where the game's level-up names each step.

**Confirmed against the guide** (saved pages, `guide-cache/`):

- **The sorcerer's other resistance bloodlines** (`classes/sorcerer.ron`, granted once at bloodline level 3; the guide gives a step at 9). `sorcerer.ron` now steps Draconic and not these: Demon Resistances ("At 9th level, the resistance increases to 10 and the save bonus increases to +4"), Celestial Resistances ("At 9th level, the resistances increase to 10"), Infernal Resistances (the same as Demon), Elemental Resistances and the Undead bloodline's Death's Gift ("At 9th level, this increases to resist cold 10 and DR 10/magic"; DR counts too). Check Elemental's second step against the guide before granting.
- **The bloodrager's Infernal Resistance** (`bloodrager.ron`, once at 4): "At 8th level, the fire resistance increases to 10 and the save bonus to +4." The bloodrager's Elemental Resistance does not step ("At 4th level, you gain resistance 10 against your bloodline's energy type") and is rightly left.
- **The Instinctual Warrior's Cunning Elusion** (`barbarian.ron`, once at 2): "At 4th level and every four subsequent levels, this bonus increases by +1 (maximum +5 at 20th level)."

**Found by ticket 170 and not checked against the guide** (a summary says it rises; read the guide's sentence first, and leave out any that is not a step by class level): Snakeskin (sorcerer Serpentine, 9, 13, 17), Abyssal Bloodrage (bloodrager, 12, 16, 20), Corpse Studying (reanimator alchemist, 5 and every 6 levels), Overwhelming Power (kineticist, 3 and every 3), Fey Stalker (druid Defender of the True World, 8, 13, 18), Inhibit Courage (arcanist Fake Celebrity, 5, 11, 17), Battle Spirit (shaman, 8, 16), Adaptive Physiology (metamorph alchemist, 6, 18), Lycanthropic Aspect (weretouched shifter, 8, 16), Cold-Minded (assassin, 5, 8), Cold Specialization (winter witch, 5, 10), Crush and Tear (flesheater barbarian, 5, 11, 17), Fiendish Maw (demon dancer skald, 7, 12, 17), Invulnerable Defenses (rageshaper shifter) and Fiendish Aspect (fiendflesh shifter).

**From ticket [179](179-dragonheir-scion-gains-draconic-resistances.md)** (2026-11-12): two more, confirmed against the guide (`fighter`), the Dragonheir Scion's own features, each granted once at its first level: Draconic Defences ("At 3rd level, … +1 natural armour bonus to AC and energy resistance 5 … At 7th level, the AC bonus increases to +2 and the energy resistance to 10. At 13th level, these increase agains to +3 AC and energy resistance 20.") and Fearful Might ("At 2nd level, … +1 bonus on Intimidate skill checks. This increases by +1 every four additional class levels (max +5 at 18th level)."). And a new check this ticket's steps must pass: `scaling_features.rs`'s `no_selection_gains_a_step_of_a_feature_whose_first_row_it_skips` fails when a selection sharing an option skips a stepped feature's first row but not a later step's (the Seeker skips the sorcerer bloodlines' 3, 9, 13, 15 and 17, so the sorcerer's resistances stepped at 9 pass it; the Dragonheir Scion now skips every Draconic row). 179 weighed an engine rule for the Primalist question ("a step is not gained without the feature") and declined it (its Answer says why); the question stays here.

## Build

- Data only, the same treatment as 170: each feature granted again on the row of each step level, a `libs/game-data/README.md` note, a real-data test pinning the steps, the snapshots regenerated (review the check's grouped message: every changed line is a stepped feature gained at a later level) and the gold test unaffected.
- **Check the engine first, as 170 did:** does a prerequisite read the count (`Feature { count }`: the Arcane Trickster's is `count: 2`), does a bonus in `bonuses.ron` or an `ability_bonuses` entry multiply by it? If a number would move, stop for a decision.
- **The general archetype check** 170's review added, for every feature stepped: list every archetype of that class that loses, replaces or delays the feature (or the bloodline ability that carries it), and work out what its merged rows hold at each step level. An archetype row equal to the base row is refused, and an archetype with no row at a step level falls back to the base row, so one that loses a feature needs its own rows at the step levels (170's Imitator). Assert one case per archetype found, with Builds that are possible (a human for the Imitator-like races or alignments).
- **The Primalist question** (bloodrager): "Two Rage Powers" replaces the bloodline's whole row at 4, 8, 12, 16 or 20, and the resistance steps of 170 and of this ticket sit on those rows. A Primalist taking rage powers at 8 loses the step there; one taking them at 4 but the bloodline at 8 gains a step it traded away at 4. Nothing reads these features, so it shows only on the sheet's gained lines. Decide (a note, or a way for the row's choice to carry the step) before more bloodrager steps land.

**Done when:** each confirmed feature steps as the guide says, each unchecked one is checked and either stepped or struck from the list with the reason, the archetype table is in the Answer, the snapshot diff shows only the new grants, and `scripts/check.sh` passes.
