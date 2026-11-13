# More scaling features are still granted once, not at each step

Type: bugfix
Status: open
Blocked by: [177](177-more-scaling-features.md)
Model: Sonnet to build (data against the guide, as tickets 170 and 177); Opus to review the data and the archetype table.
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 177](177-more-scaling-features.md) stepped the features ticket 170 found. Its work turned up more of the same shape, each granted once in the data where the guide's text names steps. The review of 177 read the sentences of the first six below and the notes on what stepping each would touch; the rows of the last four were **not checked** by it. The same rules apply as in 177: the guide's sentence for each, quoted in the Answer; the steps on the owner's own level scale; every archetype or sharer that loses, replaces or delays the feature listed with what it holds at the steps (`assert_gains` in `scaling_features.rs` builds every archetype of a class); what reads each count checked first; no feature stepped without a guide sentence that gives the levels.

## Build

Each feature: the guide's sentence, then data rows at the step levels, a test, a README note, the snapshots reviewed. An archetype row replaces the base row's `features`, `feats`, `feat_slots` and `selections` in full, so a new row must also carry what the base row gives there that the archetype keeps.

- **The Herald of the Horn's Horn Call** (skald, `skald/herald-of-the-horn/horn-call`): "Sonic spell DCs +1 at 7th level, +2 at 13th and +3 at 19th". It has a row at 7; new archetype rows at 13 and 19 must carry what the skald's base rows give there if the archetype keeps it: Raging Song (Swift Action) at 13 and Damage Reduction at 19.
- **The Reformed Fiend's Hatred Against Evil** (bloodrager): steps at 5, 10, 15 and 20 (+1 attack and damage against evil while raging, +4 at 20). It has a row at 5 and 10; a new row at 15 must keep the base row's bloodline feat slot and one at 20 Mighty Bloodrage. Its Damage Reduction (Reformed Fiend) ("DR 3/evil at 7th level, +3 more at 10th and every three levels after") is the same kind: check it with this one.
- **The Flesheater's Devour Essence** ("+1d8 at 8th, 13th and 18th level") and **Power from Flesh** (9, 18; read the sentence first), barbarian. The Flesheater's rows at 8 and 18 already exist (rage powers lost at 8, kept at 18): carry what they hold.
- **The Fiendflesh Shifter's Fiendish Resilience**: the natural armour at 4, 12 and 20 and the resistance at 8 and 16 ("+1 natural armour (+1 at 4th, 12th, 20th) and fire and electricity resistance 5 (+10 at 8th, 16th)"). Two series on one feature: the count cannot say which, so decide whether it is one feature stepped at 4, 8, 12, 16, 20 or two.
- **The Fake Celebrity's Inhibit Greatness** (arcanist, 17: "-4 at 17th level"), an option row at 17 beside Inhibit Courage's.
- **The Overwhelming Soul's Mental Prowess** (kineticist, uses at 10, 14 and 18: "+1 use at 10th, 14th, 18th"), a row at each, carrying the base row's wild talent at 10, 14 and 18.
- Not checked by the review (read the sentences, then the rows): any others a sweep of the data's summaries for "every", "rising" and "+1 more at" finds.

**A separate question for the user inside this ticket: the base classes' stepped features.** The fighter's Bravery (+1 every four levels, max +5 at 18) and Armor Training, Weapon Training and the like are granted once at their first level in the class's table, as 170's features were. Stepping them would touch every archetype with a row at a step level, so the cost is far larger and each archetype that loses or delays one needs its own row. Notes from the review: the Dragonheir Scion has its own rows at 6, 10, 14 and 18 and its table has no Bravery, so stepping Bravery on the fighter's rows would have to leave it out there; the Defender of the Hearth's Weapon Training at 9 is its delayed first grant, not a second one; Advanced Weapon Training reads Weapon Training with `count: 1`. The user decides whether the sheet should show these steps at all (the level-up of the game names them for some classes and not for others) before any is built.

**Done when:** each feature is stepped as the guide says or struck with the reason, the archetype table is in the Answer, the snapshot diff shows only the new grants, and `scripts/check.sh` passes.
