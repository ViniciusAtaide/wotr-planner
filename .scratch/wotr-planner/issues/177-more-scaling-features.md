# More scaling features are granted once, not at each step

Type: bugfix
Status: resolved
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

## Answer

Done 2026-11-13 (branch `ticket-177`). Data only: no engine, schema or migration change. Every feature below is granted again on the row of each step level, on the owner's own level scale (bloodline level, class level, archetype or prestige class level), at the levels the guide's sentence gives.

**Features changed**, each with the guide's sentence (`guide-cache/`, the saved page named):

| Feature | Steps at | Guide's sentence |
| --- | --- | --- |
| Sorcerer Demon Resistances | bloodline 3, 9 | `character-creation`: "At 3rd level, you gain resist electricity 5 and a +2 bonus to saves vs. poison. At 9th level, the resistance increases to 10 and the save bonus increases to +4." |
| Sorcerer Celestial Resistances | 3, 9 | "At 3rd level, you gain resist acid 5 and resist cold 5. At 9th level, the resistances increase to 10." |
| Sorcerer Infernal Resistances | 3, 9 | "At 3rd level, you gain resist fire 5 and a +2 bonus to saves vs. poison. At 9th level, the resistance increases to 10 and the save bonus increases to +4." |
| Sorcerer Elemental Resistances (four elements) | 3, 9 | Read in full (170's extraction stopped at "vs."): "At 3rd level, you gain resistance 10 vs. your bloodline's element. At 9th level, this increases to 20." |
| Sorcerer Undead Death's Gift | 3, 9 | "At 3rd level you gain resist cold 5 and DR 5/magic. At 9th level, this increases to resist cold 10 and DR 10/magic." |
| Sorcerer Serpentine Snakeskin | 9, 13, 17 | "At 9th level, you gain a +1 bonus to natural armour, a +2 racial bonus to saves against poison and a +2 bonus to Mobility checks. At 13th level and 17th level, these bonuses increase by +1." |
| Bloodrager Infernal Resistance | 4, 8 | `bloodrager`: "At 4th level, you gain fire resistance 5 and a +2 bonus to saves against poison. At 8th level, the fire resistance increases to 10 and the save bonus to +4." |
| Bloodrager Abyssal Bloodrage | 12, 16, 20 | "At 12th level, the morale bonuses to attack and damage granted by bloodrage increase by +1, but the AC penalty becomes -4. At 16th and 20th levels, the bonuses increase by an additional +1." |
| Instinctual Warrior Cunning Elusion | 2, 4, 8, 12, 16, 20 | `barbarian`: "At 4th level and every four subsequent levels, this bonus increases by +1 (maximum +5 at 20th level)." |
| Dragonheir Scion Draconic Defences | 3, 7, 13 | `fighter`: "At 3rd level, a Dragonheir Scion gains a +1 natural armour bonus to AC and energy resistance 5 ... At 7th level, the AC bonus increases to +2 and the energy resistance to 10. At 13th level, these increase agains to +3 AC and energy resistance 20." |
| Dragonheir Scion Fearful Might | 2, 6, 10, 14, 18 | "At 2nd level, ... +1 bonus on Intimidate skill checks. This increases by +1 every four additional class levels (max +5 at 18th level)." |
| Reanimator Corpse Studying | 1, 5, 11, 17 | `alchemist`: "At 5th level and every 6 additional levels, this bonus increases by +1 (maximum +4 at 17th level)." |
| Metamorph Adaptive Physiology | 3, 6, 18 | "At 3rd level, the character gains a 25% chance to negate critical hits and precision damage. This increases to 50% chance at 6th level and 75% chance at 18th level." |
| Fake Celebrity Inhibit Courage | 1, 5, 11, 17 | `arcanist`: "At 5th level and every six subsequent levels, this penalty increases by -1 to a maximum of -4 at 17th level." |
| Defender of the True World Fey Stalker | 3, 8, 13, 18 | `druid`: "At 3rd level, ... a +1 morale bonus on attack and damage rolls against fey. This bonus increases at 8th, 13th and 18th levels." |
| Overwhelming Soul Overwhelming Power | 3, 6, 9, 12, 15, 18 | `kineticist`: "At 3rd level, ... a +1 bonus to attack and damage rolls with her blasts. ... This bonus increases by +1 at 6th level and every three subsequent levels (max +6 at 18th level)." |
| Battle spirit's Battle Spirit | shaman 1, 8, 16 | `shaman`: "Allies within 30' of the Shaman receive a +1 morale bonus to attack and damage rolls. At 8th and 16th levels, these bonuses increase by +1." |
| Weretouched Lycanthropic Aspect | 1, 8, 16 | `shifter`: "At 8th level, the bonus increases to +4 and at 16th level, it increases to +6." |
| Rageshaper Invulnerable Defenses | 2, 10, 15, 20 | "At 2nd level, ... +2 natural armour bonus to AC and DR 2/-. At 10th level, the shifter's DR increases to 5/- and 7/- at 15th level. Finally, at 20th level, the DR increases to 10/- and the natural armour bonus increases to +4." |
| Fiendflesh Shifter Fiendish Aspect | 1, 5, 10, 15, 20 | "At 1st level ... DR 1/Good. ... At 5th level, the shifter's DR increases to 2 / Good ... At 10th level ... 5 / Good ... At 15th level ... 7 / Good ... at 20th level, the shifter's DR increases to 10 / Good" |
| Flesheater Crush and Tear | 1, 5, 11, 17 | `barbarian`: "At 5th level, the damage increases to 1d6 (1d4) ... At 11th level, the damage increases to 1d8 (1d6) and deals an additional 1d6 points of bleed damage. Finally, at 17th level, the damage increases to 1d10 (1d8)." |
| Demon Dancer Fiendish Maw | 3, 7, 12, 17 | `skald`: "The damage increases to 1d8 at 7th level, 2d6 at 12th level and 3d6 at 17th level." |
| Assassin Cold Minded | assassin 2, 5, 8 | `prestige-classes`: "From 2nd level, an Assassin gains a +2 bonus to saving throws against mind-affecting, compulsion, fear and emotion effects. This increases to +3 at 5th level and +4 at 8th level." |
| Winter Witch Cold Specialization | winter witch 1, 5, 10 | "Any spells with the cold descriptor have their DCs increased by 1 ... At 5th and 10th levels the bonus and penalty increase by +1 / -1." |

**Struck from the list:** none of the ticket's listed features; every one has a step by class level in the guide's own sentence. Kept out as the ticket says: the bloodrager's Elemental Resistance ("At 4th level, you gain resistance 10 against your bloodline's energy type": no step, as 170 found). The list's "Crush and Tear (flesheater, 5, 11, 17)" and "Cold-Minded (assassin, 5, 8)" are the Flesheater's and the Assassin's; the list's "Cold Specialization (winter witch, 5, 10)" is the prestige class's.

**Rows.** An archetype row replaces the base class's at its level (`features`, `feats`, `feat_slots` and `selections` in full), so each new archetype row also carries what the base row gives there and the archetype's table keeps: the Dragonheir Scion's rows at 10, 14 and 18 keep the combat feat slot (its table gives a Bonus Combat Feat at 8, 10, 12, 14, 16, 18 and 20, none at 4 and 6, which its rows already lacked); the Instinctual Warrior's rows at 4, 8 and 12 keep the rage power (its table: Rage Power at 4, 8, 10, 12, 16, 18, 20); the Flesheater's row at 17 keeps Tireless Rage; the Overwhelming Soul's rows at 6, 12 and 18 keep the wild talent and its row at 15 the expanded element; the Metamorph's rows at 6 and 18 keep the discovery; the Demon Dancer's row at 12 keeps the rage powers and the skald talent and its row at 17 the skald talent; the rows that only add the feature on a base row that is empty or that the archetype does not change carry just it: the Reanimator's 17 (Corpse Studying, the base row 17 is empty), the Defender of the True World's 13 and 18 (Fey Stalker; the base druid rows give only a divine spell level, which an archetype row without one keeps), the Weretouched's 16 (Lycanthropic Aspect; the base row 16 is empty) and the Dragonheir Scion's 7 and 13 (Draconic Defences; the base rows are empty). Option rows (a bloodline, a spirit, a Deceiver way) add to the class's, so they only gain the feature.

**What reads each count:** nothing. A search of `libs/` and `apps/` outside the class files for each key finds no prerequisite (`Feature { count }`; the only `count: 2` is the Arcane Trickster's Sneak Attack), no entry in `bonuses.ron` or `interactions.ron` and no `ability_bonuses`; so no number or pick moves. The snapshot diff confirms it.

**The archetype table** (every archetype of each class touched is built as a possible Build, at class level 20, with the bloodline, spirit or way picked where the feature needs it, and asserted; the counts are the engine's, from `gains` in the test, the levels at which the feature's count rises):

| Feature | Archetype | What it does | Gained at |
| --- | --- | --- | --- |
| Sorcerer resistances (all eight bloodlines above) | Crossblooded (first bloodline), Geomancer, Overwhelming Mage, Nine-Tailed Heir | keep the bloodline's powers | 3, 9 (Snakeskin 9, 13, 17) |
| | Crossblooded's second bloodline | arcana, spells and feats only | none (170) |
| | Seeker | takes every bloodline and skips the rows at 3, 9, 13, 15, 17 | none |
| | Empyreal Sorcerer | fixed Celestial bloodline, skips 1 and 20, keeps 3 and 9 | Celestial Resistances 3, 9 |
| | Sage, Sylvan | fixed Arcane and Fey, no pick | none |
| | Eldritch Scion (magus), Mythic Second Bloodline | by magus level; by sorcerer level once taken (MR 1 at level 6) | 3, 9; Celestial taken second: 6, 9 |
| | Dragon Disciple | takes only a Draconic bloodline (its prerequisite refuses the others) | none of these features |
| Bloodrager Infernal Resistance, Abyssal Bloodrage | Bloodrider, Greenrager, Spelleater, Steelblood, Reformed Fiend (shares the Abyssal and Infernal rows) | keep the bloodline's rows | 4, 8; 12, 16, 20 |
| | Hag-Riven, Mixed Blood Rager | offer neither bloodline | none |
| | Second Bloodrager Bloodline | by bloodrager level | Infernal taken second: 6, 8 |
| | Primalist | see below | with the bloodline's ability at every level: 4, 8; 12, 16, 20 |
| Cunning Elusion | Instinctual Warrior | own feature | 2, 4, 8, 12, 16, 20 |
| | every other barbarian archetype and the class | do not have it | none |
| Crush and Tear | Flesheater | own feature | 1, 5, 11, 17 |
| Draconic Defences, Fearful Might | Dragonheir Scion (all ten colours) | own features | 3, 7, 13; 2, 6, 10, 14, 18 |
| | every other fighter archetype and the class | do not have them | none |
| Corpse Studying; Adaptive Physiology | Reanimator; Metamorph | own features | 1, 5, 11, 17; 3, 6, 18 |
| | every other alchemist archetype and the class | | none |
| Inhibit Courage | Magic Deceiver, Fake Celebrity way | the other five ways hold none | 1, 5, 11, 17 |
| | every other arcanist archetype and the class | | none |
| Fey Stalker | Defender of the True World | own feature | 3, 8, 13, 18 |
| Overwhelming Power | Overwhelming Soul | own feature | 3, 6, 9, 12, 15, 18 |
| Fiendish Maw | Demon Dancer | own feature | 3, 7, 12, 17 |
| Battle Spirit | the shaman and every archetype that picks the Battle spirit (Possessed, Shadow, Spirit Hunter, Spirit Warden, Witch Doctor, Wildland Shaman, which forces it); Second Spirit by shaman level (MR 1: 6, 8, 16) | | 1, 8, 16 |
| | Prophet of Pestilence, Unsworn Shaman | fixed Pestilence spirit; wandering spirits only | none |
| Lycanthropic Aspect; Invulnerable Defenses; Fiendish Aspect | Weretouched; Rageshaper; Fiendflesh Shifter | own features | 1, 8, 16; 2, 10, 15, 20; 1, 5, 10, 15, 20 |
| | every other shifter archetype and the class | | none |
| Cold Minded; Cold Specialization | Assassin; Winter Witch | prestige classes, no archetypes | 2, 5, 8; 1, 5, 10 |

No archetype or sharer leaks: `no_selection_gains_a_step_of_a_feature_whose_first_row_it_skips` passes with the new steps (the Seeker skips 13 and 17 with 3 and 9; the Dragonheir Scion's `skipped_levels` already cover the Draconic rows). Every selection that shares an option with a stepped feature is the one listed above (a search of every selection's `shared_options`: the Reformed Fiend, Wildland Shaman, Eldritch Scion, Crossblooded, Seeker, Empyreal, Dragonheir Scion, Dragon Disciple); no `option_row` or `counts_as` names a stepped feature.

**The Primalist question.** The guide's sentence on it: "Primal Choices: At 4th level and every four levels thereafter, a Primalist can choose to take his bloodline ability or two Rage Powers." Its table lists "Bloodline Ability or Rage Powers" at 4, 8, 12, 16 and 20 and nothing more, so the guide has no sentence on what happens to a step. But the layout says more than I first wrote (the review's finding): each bloodrager table names one ability per Primalist level (Infernal "8 | Diabolical Arrogance"; Abyssal "12 | Abyssal Bloodrage", "16 | Demonic Aura", "20 | Demonic Immunities"), and each step is written into the text of an earlier ability ("At 8th level, the fire resistance increases to 10"). Read that way, "Bloodline Ability or Rage Powers" trades the ability the table names at that level, and the data is backwards in both directions: it grants a step of an ability that was traded away, and drops a step of an ability that was kept. **This is an inference from the guide's layout, not a sentence of the guide.** No rule is invented and the behaviour stays: "Two Rage Powers" replaces the bloodline's whole row at that level, with whatever sits on it. Pinned in `a_primalists_rage_powers_replace_the_row_and_its_step_with_it`:

- Infernal Resistance (first at 4, step at 8): rage powers at neither level gains it at 4 and 8; at 4 only, at 8 alone (a step of a feature it traded away); at 8 only, at 4 alone (the step is lost); at both, never.
- Abyssal Bloodrage (first at 12, steps at 16 and 20): rage powers at none gains it at 12, 16 and 20; at 12 gains 16 and 20 (a count of 2 at 20 without ever holding the first); at 16, 12 and 20; at 20, 12 and 16.

**For the user, two options:** (A) leave it: the Primalist's row choice replaces the row's contents, steps included, as the data does now, and a note says so; or (B) make the choice carry the abilities, so a step belongs to the ability it improves: a step is gained only by a Primalist that holds (kept, not traded) the ability's first grant, and a step on a row it traded for rage powers is still gained when the ability itself was kept. B needs a rule the engine lacks (179 declined the simple form, "a step is not gained without the feature", for the reasons in its Answer; this form would also need the step to stay when its own row is traded). Nothing reads these features, so it shows only on the sheet's gained lines.

**Decided** (user, 2026-11-13): A. "leave the primalist as is". The Primalist's row choice replaces the row's contents, steps included; the test pins it for both bloodlines, and nothing reads these features.

**Snapshot check** (before regenerating; `git status` shows one snapshot file, `bench-multi_prestige20.txt`, whose Assassin levels 5 and 8 are Build levels 11 and 14):

```text
the engine's output differs from the snapshots in 1 of 44 Builds

Changes, each with the Builds it's in:
  + level.11.feature/assassin/cold-minded = Cold Minded in 1 Build: bench-multi_prestige20
  + level.14.feature/assassin/cold-minded = Cold Minded in 1 Build: bench-multi_prestige20

The same, levels and ranks wildcarded:
  level.*.feature/assassin/cold-minded: + Cold Minded in 1 Build, 2 lines
```

Every line is a stepped feature gained at a later level. The other features are in no corpus Build, so the new test pins them. The gold test (`gold.rs`) passes with its two known gaps (the Elk companion, 174, and the Enforcer's Uncanny Dodge, 173) untouched.

**Tests** (`libs/seeds/tests/game_data/scaling_features.rs`): `gains` (the levels a count rises at), `playable` (a possible Build of each archetype, over the races and alignments it needs) and `assert_gains` (every archetype of the class built and asserted; an exception names an archetype the class has); one test per class touched; the sorcerer's eight bloodline resistances against all nine sorcerer rows; the Eldritch Scion, the Mythic Second Bloodline, the Second Bloodrager Bloodline and the Second Spirit; the prestige classes; the Primalist.

**Found and left**, now filed as [182](182-still-more-scaling-features.md): the Herald of the Horn's Horn Call, the Reformed Fiend's Hatred Against Evil, the Flesheater's Devour Essence and Power from Flesh, the Fiendflesh's Fiendish Resilience, the Fake Celebrity's Inhibit Greatness, the Overwhelming Soul's Mental Prowess, and the base classes' stepped features (a question for the user there).
