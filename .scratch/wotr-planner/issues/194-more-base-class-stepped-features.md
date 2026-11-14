# More base-class features are granted once, where the guide gives steps

Type: bugfix
Status: open
Blocked by: [188](188-base-classes-stepped-features.md)
Model: Sonnet to build (data against the guide, as tickets 188 and 182); Opus to review the data and the archetype table.
Map: [WotR Build Planner](../map.md)

## Problem

[Ticket 188](188-base-classes-stepped-features.md) stepped the base classes' stepped features that ticket 182 listed (the fighter's Bravery, Armor Training and Weapon Training, Danger Sense, the cavalier's Banner, Tactician and Challenge, the paladin's Smite Evil, Wild Shape (Wolf), the warpriest's Channel Energy and Fervor). Its research found more of the same shape: a class feature whose guide sentence gives named levels or a start, an interval and a cap level, and which the data grants once (at its first level). The user kept them out of 188 ("do as the recommended for all questions": a follow-up ticket). The same rules apply as in 188: the feature granted again on the row of each step, on the owner's own level scale; every archetype that loses, replaces or delays the feature gets its own rows (`assert_gains` in `scaling_features.rs` builds every archetype); a new row carries what the base row gives at that level, minus the stepped feature; what reads each count is checked first (188 found nothing that reads a count above 1 for its features; check these keys again).

## The features

The levels come from the guide's sentence (page named). **Unverified:** the archetype counts below are derived from the data only ("keeps" = the archetype's row at the first level lists the feature, or it has no row there; "new rows" = the archetype loses it and has no row at a step, so it would inherit the step). Check each archetype against the guide's own table for it, as 188 did, before building; where they disagree the guide wins.

| Feature | Class file | Data grants at | Steps | Archetypes needing new rows (data-derived) |
| --- | --- | --- | --- | --- |
| Inspire Courage (`bard/inspire-courage`) | `bard.ron` | 1 | 1, 5, 11, 17 | none (Archaeologist, Beast Tamer and Tranquil Whisperer replace it, with rows at every step; add to the Chelaxian Diva's 5 and 11, the Dirge Bard's 5, the Flame Dancer's and Thundercaller's 11) |
| Inspire Competence (`bard/inspire-competence`) | `bard.ron` | 3 | 3, 7, 11, 15, 19 | Archaeologist 19 (it has a row at 11 holding it: a delayed grant? check its table), Beast Tamer 19, Flame Dancer 7, 15, 19, Tranquil Whisperer 15, 19 |
| Poison Resistance (`alchemist/poison-resistance`) | `alchemist.ron` | 2 | 2, 5, 8 | Chirurgeon 8, Grenadier 8 |
| AC Bonus (`monk/ac-bonus`) | `monk.ron` | 1 | 1, 4, 8, 12, 16, 20 (max +5 at 20). **Flag:** the grant at 1 only adds Wisdom; the +1 starts at 4, so six grants would stand for a +5 (the count is one above the bonus). Decide whether the count should be the bonus (steps 4, 8, ..., 20 with the grant at 1 left out of it) | none (24 existing rows to add to) |
| Fast Movement (`monk/fast-movement`) | `monk.ron` | 3 | 3, 6, 9, 12, 15, 18 **if it passes the rule**: the guide's heading gives a start (3) and an interval (+10' for every 3 levels above 3rd) but names no cap level, so by the rule 188 struck the cleric's Channel Energy on (a formula, no named levels or cap) it fails; step it only if the user wants the rule relaxed | Sensei 12, 15, 18; Sohei 9, 15 |
| Infusion Specialization (`kineticist/infusion-specialization`) | `kineticist.ron` | 5 | 5, 8, 11, 14, 17, 20 | none (existing rows only) |
| Defensive Instinct (`shifter/defensive-instinct`) | `shifter.ron` | 2 | 2, 4, 8, 12, 16, 20 (max +5 at 20). **Flag:** the same as AC Bonus: the grant at 2 adds Wisdom, the +1 starts at 4, so six grants stand for +5 | Rageshaper 8, 12, 16; Wild Effigy 8, 12, 16, 20 (the Fiendflesh Shifter has rows at every step) |
| Inspired Rage (`skald/inspired-rage`) | `skald.ron` | 1 | 1, 4, 8, 12, 16, 20 (the Will bonus every four levels; attack and damage at 8 and 16 fall on those) | none (existing rows only). Ticket 189 changed the skald's rows: re-read them. |
| Slayer's Advance (`slayer/slayers-advance`) | `slayer.ron` | 13 | 13, 17 | Arcane Enforcer 17, Imitator 17 |
| Divine Weapon Bond (`paladin/divine-weapon-bond`) | `paladin.ron` | 5 (Hospitaler, Tortured Crusader only) | **Two patterns, the choice is open.** The enhancement bonus: +1 for every three levels above 5, max +6 at 20, so 5, 8, 11, 14, 17, 20 (the paladin page's Divine Weapon Bond heading). The uses a day: 5, 9, 13, 17 ("1/day, +1 use per 4 levels" in the data's summary; I found no sentence giving it in the saved paladin page: read it again before building). One feature can't carry both series | Hospitaler 8, 14, 17, 20 and Tortured Crusader 14, 17 (add to their existing rows 11; 8, 11, 20) |

Also found, and not decided: the Martyr's Stigmata (the Tortured Crusader's All Is Darkness and the Stonelord's Stonestrike moved into 188: the guide gives them 1, 5, 10, 15, 20 with a cap at 20), the Cleric's Channel Energy, Lay on Hands and the Paladin's Channel Positive Energy (formulas, struck by 188 for that reason). Monk Stunning Fist, Unarmed Strike damage, Flurry of Blows' second extra attack at 11 and the kineticist's Burn, Elemental Overflow and Kinetic Blast were not resolved: read their sentences first.

## Done when

Each feature above is stepped as the guide's sentence says or struck with the reason, the archetype table is in the Answer, the snapshot diff shows only the new grants, and `scripts/check.sh` passes.
