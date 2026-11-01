# WotR Build Planner

Planning Pathfinder: Wrath of the Righteous characters level by level, offering only the choices the game would allow.

## Language

### Builds and levels

**Build**:
The plan for one character, from creation to its final level: every choice, and nothing derived from them. It lives
in the URL fragment (`/#b=…`), which is also its share link; nothing is saved on the server.
_Avoid_: Character sheet, save, saved plan

**Build start**:
The choices made before level 1: race (and subrace), alignment, and point-buy ability scores.
_Avoid_: Character creation, level 0

**Level-up**:
Everything chosen at one character level: the class, any ability increase, skill ranks, and the picks for that level's slots. The archetype and spellcasting advancement are the class's **Class choice**, not the level's.
_Avoid_: Level entry, level step

**Class choice**:
What a Build fixes for a class when it's first taken: at most one archetype and, for a prestige class, the classes whose spellcasting it advances (one arcane and/or one divine, already taken). It belongs to the class, so moving the class's first level doesn't lose it.
_Avoid_: Archetype level, entry choice

**Mythic ascension**:
Everything chosen at one mythic rank: a mythic ability or mythic feat, plus any path-specific picks. It happens at a fixed character level.
_Avoid_: Mythic level-up, tier-up

**Path switch**:
Replacing the mythic path at MR 8 with Legend, Gold Dragon, Swarm That Walks or Devil. Each new path keeps only some of the earlier mythic ascensions.
_Avoid_: Path change, respec

**Character level**:
The character's total level, 1–20, or 1–40 on the Legend mythic path.
_Avoid_: Level (unqualified), total level

**Class level**:
The number of character levels taken in one class. A character's class levels add up to its character level.
_Avoid_: Level in class

**Spell-level access**:
The highest spell level a character can cast, for one spell type (arcane or divine), as given by the class's spells-per-day table. Prestige class entry requirements use this, not caster level.
_Avoid_: Caster level (a different thing), spellcasting level

**Mythic rank**:
Progress along a mythic path, 1–10, separate from character level.
_Avoid_: Mythic level, tier

### Choices

**Feat slot**:
A point in a Build where one feat is chosen. A **general slot** accepts any feat whose prerequisites are met. A **bonus slot** accepts only some feats: any Combat feat, any feat of one category, or one feat from a fixed list.
_Avoid_: Feat pick, feat choice

**Bonus-list slot**:
A bonus slot that draws from the class's own fixed bonus-feat list, such as a monk's bonus feats, a ranger's combat style or a kineticist's. Its pick ignores the feat's prerequisites (it must still be on the list and not already held) and counts as held afterwards, so a follow-on feat such as Crane Wing still checks its own. Every other feat slot checks prerequisites, a fighter's "any Combat feat" slot included. In the data it's a fixed-list slot with `ignores_prerequisites` set.
_Avoid_: Free feat, prerequisite-free slot

**Combat feat**:
Any feat outside the class, racial, skill, teamwork and magic categories. These are the feats a fighter's bonus slots accept.
_Avoid_: Fighter feat, martial feat

**Feature selection**:
A class-specific choice from a fixed list, such as a rogue talent, rage power or hex.
_Avoid_: Talent (unqualified), class pick

**Selection option**:
One entry in a Feature selection's list, such as Fast Stealth among the rogue talents. Its key extends the selection's (`selection:rogue/rogue-talents/fast-stealth`).
_Avoid_: Option (unqualified), talent

**Slot**:
One place in a Build where something is picked: a Feat slot, a Feature selection slot, or a Mythic ascension's mythic ability, mythic feat or path pick. Each has a stable id naming its level or rank.
_Avoid_: Choice point, cell

**Pick**:
What fills a slot: a feat, a selection option, a mythic ability or a mythic feat.
_Avoid_: Choice (that is any decision in a Build), selection

**Prerequisite**:
A condition that must hold before a feat, selection option, mythic pick, class or archetype can be chosen. Examples: an ability score, BAB, another feat, skill ranks or a class feature.
_Avoid_: Requirement, restriction

### The planner

**Sheet**:
The one planning screen and its summary: a grid with a row per character level and a chip per slot, plus the Mythic track. A finished Build's summary is the same sheet.
_Avoid_: Level-up screen, summary page, character sheet

**Mythic track**:
The sheet's tab for Mythic ranks 1–10 at their fixed character levels, with the path at MR 3 and any Path switch at MR 8. Ranks never reached, or dropped by a switch, show as such.
_Avoid_: Mythic tab, mythic levels

**Problem**:
A choice in a Build that doesn't hold up, where it is and why: an unmet prerequisite, a key missing from the game data, a pick for a slot that's gone, a cap exceeded. Opening a stale link marks problems; editing blocks them.
_Avoid_: Error, violation

**Would-break**:
The problems an edit would cause later in the Build, chains included (removing Cleave breaks Great Cleave and what needs it). The sheet blocks such edits and says what they'd break.
_Avoid_: Conflict, side effect
