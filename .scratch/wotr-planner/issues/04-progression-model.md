# How a character progresses

Type: grilling
Status: resolved
Blocked by: 01
Map: [WotR Build Planner](../map.md)

## Question

What is the domain model that turns a Build into what you get at each level?

- **Level 1 choices:** what a Build records at level 1 (race, point-buy ability scores and the like) versus at each later character level (class, archetype, the ability increase every 4th level, skill points, feat slots, feature selections).
- **Combined values:** how class levels from several classes add up to BAB, saves, caster level and skill points.
- **Slots:** where general slots, bonus slots and feature-selection slots come from: character level, class level, race or archetype.
- **Mythic ranks:** how they fit alongside character levels (when each rank is gained, which ranks give a mythic feat and which a mythic ability), and exactly what changes on Legend with its cap of 40.
- **Path-specific mythic picks** (in scope; for example Aeon gazes and Trickster bonus feats): which mythic ranks grant them, and how they relate to mythic feats and mythic abilities.
- **Archetypes:** how they replace or alter class features and slots.
- **Prestige classes** (in scope):
  - entry requirements, including the minimum character level
  - how prestige levels that advance spellcasting add to an existing class's spell-level access
  - alignment, chosen at level 1, and what an alignment restriction means for future levels
- **Skill rank cap:** the user says the maximum is 20, or 40 on Legend. Is it tied to character level at every level, or only a flat cap?

Use `grilling` + `domain-modeling`, and record the terms in GLOSSARY.md.

Inputs from [How the guide is laid out](01-guide-anatomy.md):
- Mythic ranks come from story events, and the guide gives only an approximate character level for each. Placing ranks on the level timeline is a design choice.
- Legend has no progression table. The guide says the rank is stuck at 3 and the rank 3–8 picks are lost, and it doesn't say whether the attribute increase every 4 levels continues after level 20.
- Each class table's "Feat" column marks the odd-level feats for a single-class character, not class bonus feats.

## Comments

**2026-10-31, user (round 1):**

- **When each mythic rank arrives** (`source: user`; this replaces the guide's approximate table):

  | Character level | Mythic rank |
  |---|---|
  | 6 | MR 1 |
  | 7 | MR 2 |
  | 9 | MR 3: choose mythic path |
  | 11 | MR 4 |
  | 14 | MR 5 |
  | 15 | MR 6 |
  | 16 | MR 7 |
  | 17 | MR 8 and MR 9 |
  | 19 | MR 10 |

  The mythic path is chosen at MR 3, not MR 1, so MR 1–2 come before any path.
- **Legend, levels 21–40:** a feat at every odd character level up to 39, and +1 to an ability at 24, 28, 32, 36 and 40. Nothing else differs past level 20.
- Not answered, so the recommendations are taken unless the user objects: Q1 Build start, Q2 archetype at class entry, Q6 skill rank cap equals character level, Q7 the spellcasting class to advance is chosen once at prestige class entry, Q8 a single alignment, Q9 terms.
- Q4 (path switch) is revised because the path is chosen at MR 3; it's re-asked in round 2.

**2026-10-31, user (round 2):**

- Gold Dragon, Swarm That Walks and Devil take over at MR 8 (level 17), the same point as Legend.
- Not answered, so the recommendations are taken unless the user objects:
  - **Q10:** the rank table is fixed.
  - **Q11:** MR 1 gives a general mythic ability and MR 2 a general mythic feat.
  - **Q12.1:** Legend switches at MR 8, MR 9–10 never happen, and only the MR 1–2 picks are kept.
- Still open, as data rather than model: whether Gold Dragon, Swarm and Devil keep the picks made before the switch. The model gives each path a switch rule (which earlier ranks it keeps); the guide pages supply it.

## Answer

**What a Build records**

- **Build start:** race and subrace, alignment (one for the whole Build), and the 25-point ability buy.
- **Level-up** (character levels 1–20, or 1–40 on Legend):
  - **Class:** the archetype is chosen the first time you take a class and stays fixed; at most one archetype per class.
  - **Prestige class entry:** the first time you take a prestige class that advances spellcasting, you choose which class's spellcasting it advances. Mystic Theurge advances one arcane and one divine class.
  - **Ability increase:** +1 at 4, 8, … 40.
  - **Skill ranks:** ranks in a skill can't exceed character level.
  - **Feat slots:** a general slot at each odd character level (up to 39), plus bonus slots granted by race, class table or archetype.
  - **Feature selections.**
- **Mythic ascension** (fixed character levels, `source: user`):

  | Level | 6 | 7 | 9 | 11 | 14 | 15 | 16 | 17 | 19 |
  |---|---|---|---|---|---|---|---|---|---|
  | **Rank** | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8, 9 | 10 |

  - MR 1 gives a general mythic ability and MR 2 a general mythic feat.
  - The path is chosen at MR 3. After that, odd ranks give a mythic ability and even ranks a mythic feat, plus the path's own picks from its rank table.
- **Path switch:** at MR 8, level 17, a Build may switch to Legend, Gold Dragon, Swarm That Walks or Devil.
  - Each path has a switch rule naming which earlier ranks' picks it keeps; that's data from the guide.
  - Legend keeps MR 1–2 only and its rank stays at 3. It gets no further mythic ascensions, and its cap is 20 levels per base class and 10 per prestige class.

**What's calculated, not chosen**

- **BAB and saves:** the per-class table values, added together.
- **Spell-level access:** from the class spells-per-day tables, plus prestige class advancement.
- **Caster level:** only where the guide states it.
- **Prerequisites and alignment:** checked against the Build as it stands at that level.
