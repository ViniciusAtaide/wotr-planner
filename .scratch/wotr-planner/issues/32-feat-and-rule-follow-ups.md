# Feat and rule follow-ups

Type: task
Status: open
Blocked by: 31
Map: [WotR Build Planner](../map.md)

## Question

Build the user's decisions (2026-11-01, "all recommended") from the open questions of tickets [26](26-mythic-path-rules.md), [27](27-repeatable-and-parameterised-feats.md) and [28](28-class-rules-follow-ups.md), plus the skill facts the user gave ([research/skills-from-user.md](../research/skills-from-user.md)). Each ticket's `## Decisions on the open questions` is the spec.

- **Pick order within a level** (27-7, 28-1): class selections, then class bonus slots, then race slots, then general slots. A human warpriest's level-1 bonus feat can take Dazzling Display on its Weapon Focus; a druid who picks a companion through Nature Bond can take Boon Companion at level 1.
- **Repeats** (27-1): every parameterised feat repeats once per value.
- **Weapon values** (27-2): Sawtooth Saber is exotic and Exotic Weapon Proficiency can take it; natural attacks are one value per attack the guide names; rays and touch attacks are two values; Kinetic Blast and Bomb are values.
- **Armor Focus** (27-4) checks the armour proficiency matching its armour type.
- **Mythic feats with a value** (27-5): Improved Critical (Mythic), Spell Focus (Mythic), Skill Focus (Mythic), Expanded Arsenal, School Mastery and the Trickster's Improved Improved Critical are parameterised, and need their base feat in the same value.
- **Fixed-value and restricted-choice grants** (27-6) replace the feature-encoded grants that ticket 27's open question 6 lists (Zen Archer, Mantis Zealot, Sword Saint, shaman, half-elf Adaptability, Chirurgeon, Enlightened Philosopher, oracle).
- **Slayer combat style feats** (27-8): three in total, at 2 or 4, 6 or 8, and 10 or later, as the guide says.
- **Perfect Mind's floors** (26-3): BAB at least 15 and every skill rank at least the mythic rank, for prerequisites.
- **Destiny Beyond Birth** (26-4): removes the race's ability penalty from the character's own scores.
- **Razmiri Infiltrator** (28-5): the way needs true neutral or any evil.
- **Skill facts from the user:** Tortured Crusader has 3 skill points and its own class skills (Athletics, Knowledge (Arcana), Knowledge (World), Lore (Nature), Lore (Religion), Perception, Persuasion: the guide's paladin page agrees); the Arcane Trickster loses Use Magic Device; the Dragon Disciple gains Knowledge (World).

Data from the saved guide pages only (offline), plus the user's facts. If the guide doesn't settle something, stop and write the question here.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB). **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a human warpriest takes Dazzling Display at 1; Weapon Focus and Greater Weapon Focus each in two weapons; Armor Focus (Heavy) needs heavy armour proficiency; Improved Critical (Mythic) in a longsword needs Improved Critical in a longsword; a slayer can't take a fourth combat style feat; the Tortured Crusader gets 3 skill points.
