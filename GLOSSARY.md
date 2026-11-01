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
What a Build fixes for a class when it's first taken: at most one archetype and, for a prestige class, the classes whose spellcasting it advances (already taken; one arcane and/or one divine, or, for a class that **advances one class, arcane or divine** such as the Loremaster, one in all). It belongs to the class, so moving the class's first level doesn't lose it.
_Avoid_: Archetype level, entry choice

**Mythic ascension**:
Everything chosen at one mythic rank: a mythic ability or mythic feat, plus any path-specific picks. It happens at a fixed character level.
_Avoid_: Mythic level-up, tier-up

**Mythic path**:
The path chosen at MR 3 (Aeon, Angel, Azata, Demon, Lich, Trickster). Choosing it grants its MR 1 and 2 rows too, shown at those ranks and held from the MR 3 ascension on, so a pick before MR 3 can't rely on them.
_Avoid_: Mythic class

**Path switch**:
Replacing the mythic path at MR 8 with Legend, Gold Dragon, Swarm That Walks or Devil. A switch keeps the **general picks** (each rank's mythic ability or feat) up to one rank and the old path's own grants (its rows' features and the picks in its slots) up to another, and may drop named grants whatever their rank: Legend keeps MR 1–2 of both and stays at MR 3; Gold Dragon and Swarm keep the general picks of MR 1–7 but the old path's grants of MR 1–2 only; Devil keeps MR 1–7 but is open only from Aeon or Azata and drops the Aeon's gaze and the Azata's dragon. A switch may grant features of its own (Legend's +4 to every score).
_Avoid_: Path change, respec

**Ability bonus**:
What a feature does to the character's own ability scores while it's held: a fixed amount to one score, to every score (Legend +4), to the highest score (Perfect Body +4), or a floor on every score (Perfect Body: at least 14). It counts toward ability-score prerequisites like point buy and level increases do; gear never does.
_Avoid_: Buff, stat boost, enhancement bonus

**Prerequisite floor**:
What a feature sets as the least a prerequisite sees while it's held, without changing the sheet's numbers: Perfect Mind's base attack bonus of 15 and skill ranks at least the Mythic rank. A Build's own BAB and ranks count when higher.
_Avoid_: Minimum, override

**Character level**:
The character's total level, 1–20, or 1–40 on the Legend mythic path.
_Avoid_: Level (unqualified), total level

**Class level**:
The number of character levels taken in one class. A character's class levels add up to its character level.
_Avoid_: Level in class

**Counts as**:
A class's levels counting as another class's levels, toward one thing only: a feat's prerequisites (Diverse Training: Eldritch Knight levels are fighter levels when a feat needs "Fighter level: 4"), or a feature of the other class (Frost Power: Winter Witch levels stack with witch levels, so the witch's rows granting Major and Grand Hexes are reached at the two classes' levels together). Nothing else sees the extra levels. Ticket 28 wrote it as `counts_as (class, as_class, for)`; the data says `toward`.
_Avoid_: Stacks with (unqualified), effective level

**Spell-level access**:
The highest spell level a character can cast, for one spell type (arcane or divine), as given by the class's spells-per-day table. Prestige class entry requirements use this, not caster level.
_Avoid_: Caster level (a different thing), spellcasting level

**Mythic rank**:
Progress along a mythic path, 1–10, separate from character level. Two ranks come at character level 17, so a prerequisite on the rank ("Mythic Rank: 9") checks the rank, not the level. After a Legend switch it stays 3.
_Avoid_: Mythic level, tier

### Classes

**Progression**:
How a class's BAB or one of its saves grows with class level: full, ¾ or ½ BAB, and a good or poor save. Base and prestige classes use different save formulas. The class stores its progressions and the rules compute the numbers; archetypes never change them.
_Avoid_: BAB table, save column

**Save exception**:
A save cell the guide prints differently from the class's progression (a paladin's Will +20 at 20), kept exactly as printed.
_Avoid_: Override, typo fix

**Archetype row**:
One level where an archetype differs from its base class. Its grants replace the base class row's in full, and its spell levels replace the base row's when it sets them. A level without one is the base class's row.
_Avoid_: Archetype table (it isn't a full copy), override row

### Choices

**Feat slot**:
A point in a Build where one feat is chosen. A **general slot** accepts any feat whose prerequisites are met. A **bonus slot** accepts only some feats: any Combat feat, any feat of one category, or one feat from a **Feat list**. A feat the class gives with no choice is a **Granted feat**, not a slot.
_Avoid_: Feat pick, feat choice

**Feat list**:
A named fixed list of two or more feats that a bonus slot picks from, such as the monk's bonus feats at 1, 6 and 10 (one list per stage), or a list of every feat (Gold Dragon's Dragon Feats), which names none. A list of one Parameterised feat is a bonus feat of the player's choosing (the warpriest's Weapon Focus "in a weapon of his choosing", the Master of All's Skill Focus): the pick names the Parameter. It belongs to the class, archetype, path or race whose file defines it (a race's slots name only its own: the half-elf's Adaptability, Skill Focus as a bonus feat), and says whether picks from it ignore prerequisites (see **Bonus-list slot**). The Sword Saint's Chosen Weapon is a one-feat list too.
_Avoid_: Option list, fixed-list slot (the slot names the list)

**Granted feat**:
A feat a class level gives outright, with no slot and no pick, so it isn't in the Build or its link: a wizard's Scribe Scrolls at 1, a fighter's Armor, Shield and Weapon Proficiency feats. It's held from that level on whatever its prerequisites; one already held adds nothing, and picking it later is "already held". A granted Parameterised feat is held for every value (All-skilled: Exotic Weapon Proficiency in every exotic weapon), unless the row grants it for one value (the Mantis Zealot's Weapon Focus (Sawtooth Saber), the Chirurgeon's Skill Focus (Lore (Religion))).
_Avoid_: Bonus feat (that's a slot), one-feat list

**Bonus-list slot**:
A bonus slot that draws from its grantor's own bonus-feat list, such as a monk's bonus feats, a ranger's combat style, a kineticist's, or Gold Dragon's Dragon Feats at MR 9 ("any three feats without having to meet the prerequisites"). Its pick ignores the feat's prerequisites (it must still be on the list and not already held) and counts as held afterwards, so a follow-on feat such as Crane Wing still checks its own. Every other feat slot checks prerequisites, a fighter's "any Combat feat" slot included. In the data the flag is the **Feat list**'s: `ignores_prerequisites` on the list the slot names.
_Avoid_: Free feat, prerequisite-free slot

**Combat feat**:
Any feat outside the class, racial, skill, teamwork and magic categories. These are the feats a fighter's bonus slots accept.
_Avoid_: Fighter feat, martial feat

**Feature selection**:
A class-specific choice from a fixed list, such as a rogue talent, rage power or hex. The list is its own Selection options plus any Shared options. One that offers a single option grants it with no pick (a Sage Sorcerer's Arcane bloodline); one for options with rows names the classes whose levels make their Option level, and may be held twice (a cleric's two domains; a mythic Second Bloodline).
_Avoid_: Talent (unqualified), class pick

**Selection option**:
One entry in a Feature selection's list, such as Fast Stealth among the rogue talents. Its key extends the selection's (`rogue/rogue-talents/fast-stealth` extends `rogue/rogue-talents`). It may grant feats, held while it is: one (a rogue talent that is the feat of that name) or several (the Azata's All-skilled: every weapon, armour and shield proficiency). One that is a Parameterised feat of the player's choosing (the rogue talent Weapon Focus) is its **chosen feat**, and the option's pick names the Parameter. An option's pick may name a Parameter for its rows alone (the shaman's Battle Master: Weapon Specialization at 8 and Greater Weapon Focus at 16 in that weapon), and an option may limit the values it takes (the Zen Archer's Way of the Bow: a longbow or a shortbow); a selection whose one option takes a value is still a pick. It may Repeat. It may grant over class levels: see **Option level**.
_Avoid_: Option (unqualified), talent

**Parameterised feat**:
A feat taken for something the player chooses, which the guide words as "choose a single weapon" or "select one skill": Weapon Focus, Skill Focus, Spell Focus, Elemental Focus, Armor Focus, Improved Critical, Exotic Weapon Proficiency, the Greater and Specialization feats that follow them, and the Trickster's Improved Improved Critical chain; mythic feats too (Improved Critical, Spell Focus and Skill Focus (Mythic), Expanded Arsenal, School Mastery). Every one Repeats once per value. Each pick names its Parameter, and a prerequisite can need the feat for one value ("Weapon Focus (Dueling Sword)") or for the pick's own ("Weapon Specialization: a weapon for which you have already selected Weapon Focus"). "Weapon Focus" on its own is the feat in any value.
_Avoid_: Feat with a subtype, feat variant

**Parameter**:
What one pick of a Parameterised feat is taken for: a skill (from `skills.ron`), a weapon (from `weapons.ron`, or only an exotic one for Exotic Weapon Proficiency), a school of magic, an energy type or an armour type. The Parameter is part of the Pick and of the link.
_Avoid_: Subtype, argument, option (that's a Selection option)

**Repeat**:
How many times a feat or Selection option can be held: once (the default, and what the guide means when it says nothing), a fixed number ("up to 3 times": Extra Hex, Increased Damage Reduction), any number ("may be taken multiple times": Extra Ki), or once per value of its Parameter ("must be applied to a different weapon each time": Weapon Focus, and by the user's decision every Parameterised feat).
_Avoid_: Stacking, max picks

**Option level**:
The level of a Selection option that grants over class levels (a bloodline, a wizard school, a cavalier order, a shaman spirit, a Magic Deceiver way, a ranger combat style, a domain, an oracle mystery or revelation, a warpriest blessing, a kineticist element): the levels of the classes its Feature selection names, added up. A domain is levelled by whichever class's selection took it (cleric, inquisitor, druid, Hellknight levels). A sorcerer 5 / Dragon Disciple 5 has bloodline level 10; a bard who picked a Draconic bloodline on entering Dragon Disciple has the Dragon Disciple's levels. The option's **rows** (its features, granted feats, feats for the value its pick named, Feat slots, nested selections and its feat list from that level on) apply as its level reaches them, and one taken late (Second Bloodline) gets every row up to its level at once. An archetype can skip option levels (a Seeker: 3, 9 and 15), and its own rows replace them.
_Avoid_: Bloodline level (one family's name for it), subclass level

**Option list slot**:
A bonus Feat slot on a class's row whose Feat list is the held option's: a sorcerer's bloodline feat at 7, 13 and 19, a bloodrager's at 6, 9, …, a ranger's combat style feat (a list per stage). With two options held (a Crossblooded sorcerer's second bloodline gives only its list) it offers both lists. It's checked after the level's Feature selections, since a style picked at 2 is what the slot at 2 draws on.
_Avoid_: Bloodline slot (the families share it), dynamic list

**Daily choice**:
A choice the game asks for again every day, so it isn't part of a Build and has no slot: a Mixed-Blood Rager's bloodline, a shaman's Wandering Spirit, an Abjurer's Resistance. The sheet shows it as a note.
_Avoid_: Daily pick, rest choice

**Pool selection**:
A Feature selection with no options of its own that offers the **pools** of the options held in another: an oracle's revelation slot offers the revelations of the mysteries held (two with a mythic Second Mystery). A pool is an ordinary selection that only a Pool selection offers.
_Avoid_: Dynamic selection, revelation list

**Shared option**:
A Selection option that one Feature selection offers from another's list, such as the rogue talents a slayer's talents offer. It keeps the key of the selection that defines it, so a character holds it once whichever slot took it, and its prerequisite is the same in both (a gate on one class's feature accepts each sharing class's).
_Avoid_: Copied option, alias

**Slot**:
One place in a Build where something is picked: a Feat slot, a Feature selection slot, or a Mythic ascension's mythic ability, mythic feat or path pick. Each has a stable id naming its level or rank. Within a level, slots are checked in this order: the class's Feature selections, the class's feat slots, the race's, the general slot, then the slots of options' rows; so a race or general pick sees what the class gives at that level (a human warpriest's Dazzling Display on the class's Weapon Focus).
_Avoid_: Choice point, cell

**Pick**:
What fills a slot: a feat, a selection option, a mythic ability or a mythic feat. A pick of a Parameterised feat or mythic feat, or of an option that takes a value, also names its Parameter (Weapon Focus (Longsword)).
_Avoid_: Choice (that is any decision in a Build), selection

**Prerequisite**:
A condition that must hold before a feat, selection option, mythic pick, class or archetype can be chosen. Examples: an ability score, BAB, another feat, skill ranks or a class feature. One can depend on the pick's Parameter (Armor Focus: the proficiency of the armour type it's taken for) or be one that must not hold (the slayer's first combat style feat: not yet slayer 6). A Story condition is one too, but never blocks.
_Avoid_: Requirement, restriction

**Story condition**:
A prerequisite set by the game's story rather than by a Build, such as the Gold Dragon's Vow of Dahak or Uncorrupted. Story is out of scope: the condition always holds and the sheet shows it as a note beside the option.
_Avoid_: Flag, quest requirement

### The planner

**Sheet**:
The one planning screen and its summary: a grid with a row per character level and a chip per slot, plus the Mythic track. A finished Build's summary is the same sheet.
_Avoid_: Level-up screen, summary page, character sheet

**Mythic track**:
The sheet's tab for Mythic ranks 1–10 at their fixed character levels, with the path at MR 3 and any Path switch at MR 8. MR 1 and 2 show the path chosen at MR 3. A rank is reached, partly dropped by a switch (its general pick or the old path's grants lost, or a grant the switch drops by name), dropped (both lost), or never reached; the picks a switch took away are marked.
_Avoid_: Mythic tab, mythic levels

**Problem**:
A choice in a Build that doesn't hold up, where it is and why: an unmet prerequisite, a key missing from the game data, a pick for a slot that's gone, a cap exceeded. Opening a stale link marks problems; editing blocks them.
_Avoid_: Error, violation

**Would-break**:
The problems an edit would cause later in the Build, chains included (removing Cleave breaks Great Cleave and what needs it). The sheet blocks such edits and says what they'd break.
_Avoid_: Conflict, side effect
