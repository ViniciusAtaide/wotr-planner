# QA: make two full builds from a build guide

Type: task
Status: resolved
Blocked by: [86](86-deities.md)
Map: [WotR Build Planner](../map.md)

## Question

The user asked for a QA exercise after the deity work (2026-11-06): a QA agent makes complete builds in the real app, as a player would, from the build write-ups the user saved in `.scratch/builds`, and reports whether each could be finished; if not, what stopped it. Problems become bugfix tickets.

The file holds two builds:
- **Jack-in-the-Green:** Halfling, Warrior of the Linnorm Kings background, Sylvan Sorcerer 20, CHA 19 point buy, mythic Hero then Azata.
- **Enforcer:** Half-Elf, Andoren Diplomat background, Arcane Enforcer 15 / Sylvan Trickster 4 / Mutation Warrior 1, Strength bonus, mythic Hero then Azata.

**Out of scope** (the map's "Out of scope", and the user, 2026-11-06):
- spells: the "suggested spells" tables, and spell picks inside level-ups such as "Bloodline Spell - Entangle (+ Mage Armor)";
- the animal companion's own build (the Elk's level-ups). A choice *of* companion, such as "Animal Companion - Elk", is in scope if the app offers it.

## Build

- **In the real app:** a release build served on its own port with its own target and site dirs (not the dev server on 3900), driven with the browser tools as a player would: the Build start, then each level and mythic rank in the write-up's order. Use the app's pickers; don't type link fragments.
- **For every line of each write-up,** record one of:
  - done as written;
  - done differently (the app names it otherwise, or a choice the write-up implies has to be made), and how;
  - not possible: what stopped it (missing option, wrongly locked with its reason, wrong level, missing slot, crash or console error), with the level, the slot and the app's exact message.
  
  Features granted automatically (Woodland Stride, Studied Target +2, sneak dice) only need checking on the sheet.
- **Judging a block:** a write-up can be wrong too. Before calling something a bug, check the saved guide pages in `guide-cache/` (never fetch GameFAQs or fextralife) and the game data. A real bug is the app disagreeing with the guide or with the game's rules as the data records them.
- **Each real problem gets a bugfix ticket,** numbered from 89, in the style of the other tickets: the repro (the Build's link at that point, the slot, the message), what the guide says, and the expected result. Group problems that share one cause into one ticket.
- **When the report is written,** delete `.scratch/builds` (user, 2026-11-06): it's staged in git but was never committed, so also unstage it (`git rm --cached`) before deleting.

**Done when:**
- Both builds were attempted to their last level and mythic rank (or as far as the app allows), and the Answer has, per build: finished or not, the final `#b=` link, and a table of every deviation and blocker.
- Every real problem has a bugfix ticket, linked from the Answer.
- `.scratch/builds` is gone.

## Progress log

- 2026-11-06: started (QA agent; the user away, choices settled here). Read the ticket, the repo's `CLAUDE.md`, the map's Notes and the two write-ups in `.scratch/builds`. Scratch in `t88/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-7437-4fe9-8ef8-ebb8716f6fbe/scratchpad/t88/`): `build.log`, `server.log`, `links.md` (the `#b=` link after the key levels of each Build).
- 2026-11-06: release build `cargo leptos build --release --precompress` with `CARGO_TARGET_DIR=t88/target` and `LEPTOS_SITE_ROOT=t88/site` (1 m 51 s), the server binary run on `127.0.0.1:3960` against the repo's `wotr-planner.db` (opened read-only by the server, untouched). Driven with the browser pane through the pickers only; the pane was hidden, so pickers showed "Working out the options…" until a screenshot let a frame run (every one filled; not a bug). No console message at any level of either Build.
- 2026-11-06: before calling anything a bug, checked the saved pages (`sorcerer`, `slayer`, `rogue`, `azata`, `trickster`, `mythic-paths`, `feats`, `character-creation`) and `libs/game-data/` (sorcerer, slayer, rogue, fighter, `mythic/`, races, backgrounds, feats). Decisions the user already made were respected (MR timing from [ticket 04](04-progression-model.md), "choices that gate no pick stay labels" from [ticket 54](54-mercies-and-enforcer-exploits.md), the Enforcer's greater-exploit slot from [ticket 57](57-enforcer-and-exploiter-greater-exploits.md)); where a write-up conflicts with one of them, the bugfix ticket asks rather than asserts.
- 2026-11-06: Jack-in-the-Green and the Enforcer both built to level 20 / MR 10. Wrote tickets 89–93. Settled choices: Halfling (Hasty) (the write-up's notes recommend it), Chaotic Good for both (CN/CG/NG allowed; CG suits Azata), Atheism for both (no deity named, no class needs one), the Enforcer's 2 extra rogue skill points a level into Mobility and Stealth (the write-up names only three skills), slots the write-up can't fill left empty. The map is left alone: 89–93 are bugfix tasks, and its "Not yet specified" is for unspecified topics. Stopped the server; deleted `.scratch/builds` (unstaged with `git rm --cached`, then removed); committed nothing.

## Answer

Done 2026-11-06, uncommitted. Both Builds reach level 20 and MR 10 in the app, with no crash and no console error. Neither can be built exactly as written. Jack misses 3 lines (Spell Penetration in the wrong slot, so Mounted Combat drops; two Life-Bonding Friendship teamwork feats). The Enforcer misses 5 lines through the app (three Trickster first-ascension abilities, two teamwork feats) and 3 through its own mistakes (Uncanny Dodge, Greater Two-Weapon Fighting, Mythic Shield Focus's timing), and its saves are wrong throughout.

Spells (the "suggested spells" tables and the bloodline-spell lines) and the Elk's level-up table were skipped (user, 2026-11-06). "Granted" below means the data grants the feature at that level; the Sheet doesn't show granted features ([ticket 93](93-show-granted-features.md)), so they were checked against `libs/game-data/`.

### Bugfix tickets

- [89 Life-Bonding Friendship gives its two teamwork feats](89-life-bonding-friendship-teamwork-feats.md): the superpower grants nothing; the guide gives a teamwork feat at its rank and the next. Blocks two lines in each Build.
- [90 Arcane Enforcer saves](90-arcane-enforcer-saves.md): the app gives the slayer's good Fort and poor Will; the guide gives the Enforcer good Reflex and Will, poor Fortitude. Needs archetype save overrides.
- [91 The first ascension's path-linked ability is a choice](91-first-ascension-path-ability.md): the guide lets MR 1–2 take another path's linked abilities (the Enforcer's Trickster Bit of Fun and Creative Approach, then Azata); the app grants the MR 3 path's rows. Conflicts with tickets 04 and 26, so it asks.
- [92 The sorcerer's level-1 bonus feat: which list?](92-sorcerer-level-1-bonus-feat-list.md): the sample build takes Spell Penetration (off the Fey list) as its level-1 bonus feat; ticket 44 made the slot the bloodline list without in-game evidence. Asks.
- [93 The Sheet shows no granted features](93-show-granted-features.md): no class, bloodline or path feature appears anywhere, so "check it on the sheet" isn't possible. Asks whether to show them.

Seen but not filed: at Arcane Enforcer 13 the app gives both an "Arcanist Exploits" and a "Greater Exploits" slot (ticket 57's reading). The write-up names one exploit there, but it calls the later exploits irrelevant and may simply not list one, so it's no evidence against ticket 57.

### Jack-in-the-Green: finished, 3 deviations

Final link: `#b=AeWOv0tCURTH77n3vqfW60bRllRrkUNj1BCEFQQVZBGJooOLOjxQBx3cdNHJQXRyEAU3UVBc3N1ERREUfyCI-Efo9YHK2908cL7nez58ORyf7C_3gqls_8XWeTK5D7N3s-nR-Z74m3ZifPVX_77VGZje8OqFyeJN4IEfyaF5jAv3pwHWtLpyn5df5sZDJN3ymD6cXTH03qkVtL82OaMPz-VKfWAxPl9ca3AtPzdWi__O7ug4Wq4mh6VY3m5GAIgASIxJ2pOzlYWtiOp1N4x7QAJifPLCq14LoZhDUEOEVUGeUUG-HgAGgjZCiKJUAZRS5eC2ReUBHWC8BA` (Sorcerer 20, MR 10 Azata, Cha 24).

| Where | Write-up | Result |
|---|---|---|
| Start | Halfling | Done differently: Halfling (Hasty), the subrace the notes recommend (Craven has the same scores). |
| Start | Warrior of the Linnorm Kings; Sylvan Sorcerer 20 | Done. |
| Start | Str 10 Dex 16 Con 14 Int 12 Wis 8 Cha 19 | Done: point buy 12/14/14/12/8/17 = 25. |
| Start | Skills Mobility, Persuasion, UMD | Done: one rank in each a level (3/3 every level). |
| Start | Alignment CN, CG, NG; no deity | Chose Chaotic Good; Atheism. |
| 1 | Animal Companion - Elk | Not offered: the companion is a granted label with no choice of animal (ticket 54, "choices that gate no pick"); not a bug. |
| 1 | (Draconic Bloodline Arcana; Grow Claws) | Granted by MR 1's Second Bloodline (Draconic (Bronze)). |
| 1 | Bonus Feat - Spell Penetration | Not possible: the level-1 "Bloodline feat" slot offers the Fey list only ([92](92-sorcerer-level-1-bonus-feat-list.md)). Done differently: Spell Penetration in the level-1 "Feat" slot, Dodge in the bloodline slot. |
| 1 | Mounted Combat | Not taken: its slot went to Spell Penetration, which Greater Spell Penetration and Spell Penetration (Mythic) need. It was open (the level's own Mobility rank counts). |
| 3 | Woodland Stride; (Dragon Resistances I) | Granted. |
| 3 | Metamagic (Bolstered) | Done: "Metamagic (Bolster Spell)". |
| 4, 8, 12, 16, 20 | Charisma +1 | Done. |
| 5 | Spell Focus - Evocation | Done. |
| 7 | Bloodline Feat - Skill Focus (Mobility); Metamagic (Heighten) | Done ("Metamagic (Heighten Spell)"). |
| 9 | Fleeting Glance; (Dragon Resistances II); Outflank | Granted; Outflank done. |
| 11 | Spell Specialization | Done (no spell named: spells are out of scope). |
| 13 | Bloodline Feat - Improved Initiative; Greater Spell Penetration | Done. |
| 15 | Fey Magic; (Dragon Resistances III; Wings); Greater Spell Focus - Evocation | Granted; GSF done. |
| 17 | Indomitable Mount | Done (Mobility 5, as the guide says). |
| 19 | Bloodline Feat - Great Fortitude | Done once MR 1's Second Bloodline was picked: the slot then offers the Draconic list too (before that it isn't offered, which is right). |
| 19 | Elemental Focus - Electricity | Done. |
| 20 | Soul of the Fey (Power of Wyrms) | Granted. |
| MR 1 | Bypass Epic DR; Instrument of Freedom | Granted by the Azata at MR 3 (its MR 1–2 rows). |
| MR 1 | Second Bloodline - Bronze Dragon | Done: Second Bloodline, then Draconic (Bronze). |
| MR 2 | Hard to Kill; Freedom's Child; Summon Azata Bralani | Granted. |
| MR 2 | Extra Mythic Ability - Abundant Casting | Done. |
| MR 3 | Azata; Dragon Companion; Song of Heroic Resolve; Summon Mastodon | Done; granted. |
| MR 3 | Ascendant Element - Electricity | Done differently: Ascendant Element has no element choice (a label, like Finesse Training; the choice gates no pick). |
| MR 4 | Superpower - Zippy Magic; Spell Penetration (Mythic) | Done. |
| MR 5 | Improved Abundant Casting | Done. |
| MR 6 | Superpower - Life-Bonding Friendship (Shake It Off) | LBF done; the teamwork feat not possible: no slot ([89](89-life-bonding-friendship-teamwork-feats.md)). |
| MR 6 | Expanded Arsenal - Enchantment | Done. |
| MR 7 | Life-Bonding Friendship (Allied Spellcaster) | Not possible: no slot ([89](89-life-bonding-friendship-teamwork-feats.md)). |
| MR 7 | Mythical Beast | Done (the Sylvan's companion qualifies). |
| MR 8 | Superpower - Favorable Magic; Spell Focus (Mythic) - Evocation | Done. |
| MR 9 | Greater Abundant Casting | Done. |
| MR 10 | Superpower - Marvelous Endurance; Sorcerous Reflex | Done. |
| MR 4–9 | The Songs and Summons | Granted. |

### Enforcer: finished, 9 deviations (3 of them the write-up's mistakes)

Final link: `#b=AdVNPUhCYRT97veeqT1ToYiWSIogSKmkKOgPm7IgsBoqInHIoR-RighrkOBZ9DO4RAqtRVMUxOstIRlZg0sW4SCBk20NQTRI3ZdpSa-lTS7nfOfc795zt2C89sL_bNP5FntCex4xvd_0VCh6NmVBPTzDZexKzeD_Oqs6Ry4TddWSF2f8Z6d4a1vTmjSMvPcq8MthvI6Xc8qOnZPXqM0y4VI706PWgSOrRxvb3Gi40ZUkKiuWXWZTfXNkwSxWl3rb64LHTvdBKChcPvJ20enwJSddqsP1sKcrvIbrLQaBm766HzLx_Q_diVhbY-fYVFw4D_RFUgHj2657_oWfu4veUq2dAAEGQK_XFqs0ZQQlfBPk2z97VG5OIREQueczgwGSKSXhgBAUFJn-JirTwxPSCpfNoF-QislqyPdUDRTQINGcQmJZ9keCBAVIrGSkQyyiCKEGFf0A` (Slayer 15 / Rogue 4 / Fighter 1, MR 10 Azata, Str 22). Its empty slots: level 8's Slayer Talent, level 13's Arcanist Exploit, level 19's Feat, MR 10's Mythic feat.

| Where | Write-up | Result |
|---|---|---|
| Start | Half-Elf, Ability Bonus Strength | Done: Half-Elf, +2 to Str. |
| Start | Andoren Diplomat | Done: "Andoran Diplomat" (the user's backgrounds table spells it so; the guide "Andoren"). |
| Start | Arcane Enforcer 15 / Sylvan Trickster 4 / Mutation Warrior 1 | Done. |
| Start | Str 18 Dex 16 Con 14 Int 10 Wis 8 Cha 12 | Done: point buy 16/16/14/10/8/12 = 25. |
| Start | Skills Athletics, Perception, Persuasion | Done: one rank in each a level; the rogue levels' 2 extra points into Mobility and Stealth (settled here). |
| Start | Alignment CN, CG, NG; no deity | Chose Chaotic Good; Atheism. |
| 1–15 | (the saves) | Wrong: "Fort +2 · Ref +2 · Will +0" at 1, "Fort +8 · Ref +8 · Will +4" at 13; the guide gives the Enforcer good Ref and Will, poor Fort ([90](90-arcane-enforcer-saves.md)). |
| 1 | Arcane Reservoir; Studied Target +1 | Granted. |
| 1 | Exploit - Wooden Flesh; Two-Weapon Fighting | Done. |
| 1 | Skill Focus - Persuasion | Done, in the Half-Elf's "Adaptability" slot. |
| 2 | Menacing Style - Intimidating Prowess | Done: Slayer Talent "Combat Style Feat (2nd or 4th level)", style "Menacing Feats", feat Intimidating Prowess. |
| 3, 6, 9, 12, 15 | +1d6 sneak | Granted. |
| 3 | Shield Bash | Done. |
| 4 | Exploit - Armored Mask; +1 Strength | Done. |
| 5, 10, 15 | Studied Target +2/+3/+4 | Granted. |
| 5 | Outflank | Done. |
| 6 | Menacing Style - Shatter Defenses | Done: "Combat Style Feat (6th or 8th level)". |
| 7 | Studied Target (Swift); Exploit - Dimensional Slide; Combat Reflexes | Granted; done. |
| 8 | Slayer Talents - Uncanny Dodge | Not possible, and the write-up is wrong: the guide's slayer talents are the rogue talents plus Trapfinding and Slayer Camouflage; Uncanny Dodge is only a Vanguard feature and an Eldritch Scoundrel talent. Slot left empty. |
| 8 | +1 Strength | Done. |
| 9 | Improved Critical - Battleaxe | Done: "Improved Critical (Battle Axe)". |
| 10 | Exploit - Spell Resistance | Done. |
| 11 | Shield Master | Done. |
| 12 | Menacing Style - Dreadful Carnage; +1 Dexterity | Done: "Combat Style Feat (10th level or later)". |
| 13 | Exploit - Greater Spell Resistance | Done, in the "Greater Exploits" slot; the level's "Arcanist Exploits" slot left empty (see "Seen but not filed"). |
| 13 | Improved Two-Weapon Fighting | Done. |
| 14 | Advanced Talents - Crippling Strike | Done (Slayer Talents slot). |
| 15 | Improved Initiative | Done. |
| 16 | Weapon Finesse; Trapfinding; +1d6 sneak | Granted (rogue 1). |
| 16 | +1 Strength | Done. |
| 17 | Evasion | Granted. |
| 17 | Fey Tricks - Iceplant; Bashing Finish | Done. |
| 18 | Danger Sense; +1d6 sneak | Granted. |
| 18 | Finesse Training - Light Shield | Done differently: Finesse Training is granted with no weapon choice (ticket 54: choices that gate no pick stay labels). |
| 19 | Debilitating Injury; Resist Nature's Lure | Granted. |
| 19 | Fey Tricks - Double Debilitation | Done. |
| 19 | Greater Two-Weapon Fighting | Not possible, and the write-up is wrong: locked "Needs Dex 19 (have 17)"; the guide's prerequisite is Dex 19 and the Build's Dex is 17 (gear doesn't count, map Out of scope). Slot left empty. |
| 20 | Bonus Feat - Shield Focus; +1 Strength | Done (Mutation Warrior's "Combat feat" slot). |
| MR 1 | Bypass Epic DR | Granted (Azata row). |
| MR 1 | Bit of Fun | Not possible: a Trickster first-ascension ability, never offered; the Azata's Instrument of Freedom is granted instead ([91](91-first-ascension-path-ability.md)). |
| MR 1 | Defensive Study | Done. |
| MR 2 | Hard to Kill | Granted. |
| MR 2 | Summon Will-o'-Wisp; Creative Approach | Not possible ([91](91-first-ascension-path-ability.md)); the Azata's Freedom's Child and Summon Azata Bralani are granted instead. |
| MR 2 | Two-Weapon Fighting (Mythic) | Done. |
| MR 3 | Azata; Dragon Companion; Song of Heroic Resolve; Summon Mastodon; Inspirational Leader | Done; granted. |
| MR 4 | Superpower - Incredible Might; Improved Critical (Mythic) - Battleaxe | Done. |
| MR 5 | Archmage Armor | Done. |
| MR 6 | Superpower - Life-Bonding Friendship (Shield Wall) | LBF done; the teamwork feat not possible ([89](89-life-bonding-friendship-teamwork-feats.md)). |
| MR 6 | Improved Initiative (Mythic) | Done. |
| MR 7 | Life-Bonding Friendship (Shake It Off) | Not possible ([89](89-life-bonding-friendship-teamwork-feats.md)). |
| MR 7 | Enduring Spells | Done. |
| MR 8 | Superpower - Marvelous Endurance; Extra Mythic Ability - Greater Enduring Spells | Done. |
| MR 9 | Ever Ready | Done. |
| MR 10 | Superpower - Supersonic Speed | Done. |
| MR 10 | Mythic Shield Focus | Not possible, and the write-up's order doesn't fit the app's timing: "Shield Focus (Mythic)" is locked "Needs Shield Focus", because MR 10 comes at level 19 (the user's table, ticket 04) and Shield Focus at level 20. Slot left empty. |

### Looked like bugs but weren't

- "Half-Elf (Kindred Raised" without its closing parenthesis: the guide's own heading has the same typo.
- Agile Maneuvers locked "Needs Bull Rush or Dirty Trick or Sunder Armor or Trip": the guide's prerequisite.
- Greater Enduring Spells open without Enduring Spells, and Indomitable Mount without Mounted Combat: the guide gives neither prerequisite.
- "Spell Focus (Mythic) (Evocation)" and "Improved Critical (Mythic) (Battle Axe)": a parameterised name's usual "Name (Value)" form; cosmetic only.
- Pickers stuck on "Working out the options…": the hidden pane; each filled once a frame ran.
- The alignment wheel's buttons have no accessible name (the browser tools' name search can't find "CG"; the build clicked them by position). Not a bug by this ticket's standard; worth a look if accessibility matters.
