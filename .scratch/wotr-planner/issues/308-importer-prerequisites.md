# The importer, stage 4: prerequisites from the game

Type: build (data, through the importer)
Status: open; sliced into 308a–308f for ranks 1 and 7 ([research/308/](../research/308/slices.md), 2026-11-17). The prerequisites naming an S3 carrier wait for a later queue.
Blocked by: 306, 307
Model: Opus to build: one bug touches hundreds of entries.
Map: [WotR Build Planner](../map.md)

## Problem

Stage S4 of the importer ([293](293-the-importer.md); staging decided 2026-11-16, user: "Yes, start it", then the three answers in the design). The design is [research/importer-design-2026-11-16.md](../research/importer-design-2026-11-16.md): its §2 area for this stage, §3 (the user's facts and the registry), §4.2 (each stage's proof) and the user's answers at its end.

## Change

- `--write` writes every blueprint entry's `Requires` from the game (§2.4), the 62 hand trees where §2.4 says.
- Shared-blueprint facts: 305 keys held facts by the planner's kind and key; where one game blueprint is two planner entries (Stunning Fist feat and the monk's feature, Wings, the rogue-talent Weapon Focus option; the full list of 90 is in 305's data note), the game's rule (a fact held once) is applied here or in its gap ticket, and each change that refuses a pick goes to the user if it touches a fact of theirs.

## Done when

- The design's §4.2 proof, all nine points: the game check exits 0 with this stage's `known.txt` lines gone; a second `--write` writes nothing; the change census attributes every differing line to a change-log row (0 unattributed); old links counted three ways, each refused one listed by Build; the sampled hand check of 100 fields; the self-checks' long runs, alone on the machine; every gold-build change shown to the user before the merge; the ignored tests of this stage's areas come off ignore or name their gap ticket.
- The scoped checks pass. Estimate: one and a half Opus builds.

## 308 left (after 308f, the last S4 slice of the queue, 2026-11-17)

From a full dry run on 308f's tip (`game-check --write prerequisites --dry-run`, no `--only`): **4 fields to write and
403 entries left whole** (304 selection options, 52 feats, 14 mythic feats, 10 mythic abilities, 12 archetypes, 11
classes).

**The 4 the writer would still write, held on purpose:** Greater Enduring Spells (ticket 208, another session's), and the
Elemental Witch's acid, electricity and fire rays (278). Their game prerequisite is WitchElementsPatronProgression
67e85f52b1f020847aaa738d8999d4cd, which the game's Elemental Witch picks in ElementalWitchPatronSelection
3172b6960c774e19ad029c5e4a96d3e4; the planner holds that selection as a feature, so the written tree refused every ray
but cold (308f). The writer's narrowing guard doesn't count an added requirement, so a run naming them writes them
again (as the Stalwart Defender, 308d).

**By the owner `known.txt` names (346 entries):**

| Owner | Entries | The writer's gap |
| --- | --- | --- |
| 293 | 60 | 46 oracle revelations naming the Divine Herbalist's and Enlightened Philosopher's mysteries (306v); 9 crossblooded, Seeker and second bloodlines (306x/y, 316); the ranger's five 10th-level style lists (narrowings: 304's nested lists, 308e) |
| 307 | 54 | 48 carriers S3 makes (the bomb and Sneak Attack grantors, the Lesser totems' SpiritTotemLesserFeature, Life-Bonding Friendship, …); Defensive Study (a carrier and its narrowings); the four wild shape feats (condition trees); Spell Specialization (and 317's spellbook) |
| 267 | 37 | 25 deity facts (`…DomainAllowed`, deity features, Atheism); 10 hand prerequisites; 2 narrowings (Mantis Zealot, Extra Channel) |
| 294 | 31 | domains (26 with a deity's fact too) and 4 hand prerequisites |
| 270 | 23 | infusions and Fire's Fury naming a blast the vocabulary can't hold (307's decision 3) |
| 321 | 17 | shaman hexes naming a spirit's wandering feature (307's decision 3) |
| 281 | 15 | 10 bloodline hand prerequisites (`NoOtherOptionOfGroup`, …); 5 bloodrager bloodlines' condition trees |
| 315 | 14 | per-value features (Skill Focus, Armor Focus, Elemental Focus) and 3 hand trees |
| 306t | 14 | 11 wild talents naming a second or third element or the Kinetic Knight's focus; the air, earth and fire elements (308f: the planner's Expanded Element offers them for the game's second elements, which refuse no archetype) |
| 272 and 293 | 14 | oracle archetype mysteries (306v) |
| 283 | 13 | the Hexcrafter's major and grand hexes (its rows lack HexcrafterArchetype's grants) |
| 272 | 9 | curses (306w) |
| 282 | 8 | opposition schools (306p) |
| 266 | 7 | the Gold Dragon's breaths (4 hand prerequisites), the Lich's skeletal upgrades (mythic class levels), Holy Water Jet |
| 267 and 294 | 6 | a deity's domain list |
| 313 | 5 | a weapon category or proficiency |
| 174 | 5 | pets and barding |
| 306x | 3 | Bloodline Ascendance, Second Bloodline, Disruptive |
| 287 | 3 | Arcane Trickster and Dragon Disciple (hand prerequisites), the Loremaster's combat feat (306o, a condition tree) |
| 314 | 2 | a feature's rank as a stat |
| 308 | 2 | condition trees (Extended Aspects, Second Spirit; decision 3) |
| 306o, 306p, 278, 275 | 1 each | the magus class, School Tolerance, Battle Master (the Wildland Shaman's rows), Favored Enemy Spellcasting |

**With no `known.txt` line (57 entries):** the check sees no difference, since it doesn't read the gap. 29 hand
prerequisites with no game form, 14 condition trees (the sorcerer's and bloodrager's bloodlines, the three bloodline
archetypes, Dragonheir Scion, Brew Potions; decision 3), Expanded Defense's PrerequisiteSelectionPossible, 7 weapon
categories or proficiencies (313), 3 Eldritch Scion splits (306o), 2 per-value features (315), 1 pet (174).
