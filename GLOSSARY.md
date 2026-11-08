# WotR Build Planner

Planning Pathfinder: Wrath of the Righteous characters level by level, offering only the choices the game would allow.

## Language

### Builds and levels

**Build**:
The plan for one character, from creation to its final level: every choice, and nothing derived from them. It lives
in the URL fragment (`/#b=…`), which is also its share link; nothing is saved on the server. The link holds a short
hash of each key, not its text, which the current game data turns back into the key.
_Avoid_: Character sheet, save, saved plan

**Build start**:
The choices made before level 1: race (and subrace), **Faith**, alignment, **Background** and point-buy ability scores. The alignment comes after the faith: it can't be chosen while no faith is, and the faith can't be removed while an alignment is set. A subrace's ability modifiers and feat slots replace its race's (a Dual Heritage half-elf has no Adaptability).
_Avoid_: Character creation, level 0

**Deity**:
One of the 26 gods of the guide's deity table, which a Build start may follow. It allows the alignments within one step of its own (the table's ticks; Godclaw: LG, LN, LE), offers its domains (a cleric's, inquisitor's or Divine Hunter's domain and a warpriest's blessing come from them; a Separatist's second domain from the others) and has favoured weapons, which a cleric, Ecclesitheurge, inquisitor or Deliverer is proficient with (an exotic one is Exotic Weapon Proficiency for it; Irori's unarmed strike is Improved Unarmed Strike). Its own alignment isn't stored: a "good deity" is one offering the Good domain.
_Avoid_: God, patron

**Faith**:
A Build start choice: a **Deity**, or atheism, which is a choice of its own (it allows every alignment; the Living Deity needs it) and not the same as no faith chosen yet. It is chosen before the alignment, which needs one; a deity then allows only its alignments. A cleric, inquisitor, warpriest, Divine Hunter or Deliverer needs a deity; the Mantis Zealot, Feral Champion and Hag of Gyronna need one named deity.
_Avoid_: Religion, belief

**Background**:
A Build start choice from the user's list (38 backgrounds in eight groups; a Build start without one simply has none). Its skills become **Class skills** and its feats are **Granted feats**, held from the start and counted for prerequisites like any other (a Guard's Armor Proficiency (Light Armor) lets it take Medium Armor at level 1; the Aldori Swordsman's is Exotic Weapon Proficiency (Dueling Sword), for that value). Everything else it gives (other weapon proficiencies, the buckler, save, AC and skill bonuses) is its note, shown with the Build. No background chosen yet isn't the background None.
_Avoid_: Origin, history

**Class skill**:
A skill a character's classes (an archetype's list where it replaces its class's), its **Background** or the option rows it holds make class skills: a class's from the level it's first taken, the background's from the start, an option row's while the row is held (an oracle's Flame mystery: Athletics and Mobility at option level 1; a Divine Herbalist, whose mystery skips that row, gets none). Shown on the sheet; nothing else depends on them.
_Avoid_: Trained skill

**Level-up**:
Everything chosen at one character level: the class, any ability increase, skill ranks, and the picks for that level's slots. The archetype and spellcasting advancement are the class's **Class choice**, not the level's.
_Avoid_: Level entry, level step

**Class choice**:
What a Build fixes for a class when it's first taken: at most one archetype and, for a prestige class, the classes whose spellcasting it advances (already taken; one arcane and/or one divine, or, for a class that **advances one class, arcane or divine** such as the Loremaster, one in all). It belongs to the class, so moving the class's first level doesn't lose it.
_Avoid_: Archetype level, entry choice

**Mythic ascension**:
Everything chosen at one mythic rank: a mythic ability or mythic feat, plus any path-specific picks. It happens at a fixed character level.
_Avoid_: Mythic level-up, tier-up

**First ascension**:
The choice at MR 1 of one path's path-linked ability: its MR 1 row (Aeon's Force Reality, Angel's Close to the Heavens, Azata's Instrument of Freedom, Demon's Close to the Abyss, Lich's Danse Macabre, Trickster's Bit of Fun, each with the fixed Bypass Epic Damage Reduction). MR 2 has no pick: it holds that path's MR 2 row, the ability's follow-ups and the fixed Hard to Kill. It doesn't commit the character to that path: the Mythic path at MR 3 is chosen on its own. A Build without one takes its Mythic path's.
_Avoid_: First path, MR 1 path

**Mythic path**:
The path chosen at MR 3 (Aeon, Angel, Azata, Demon, Lich, Trickster). Its rows give from MR 3 on; MR 1 and 2 hold the First ascension's rows, which are this path's when the Build has no first ascension of its own.
_Avoid_: Mythic class

**Path switch**:
Replacing the mythic path at MR 8 with Legend, Gold Dragon, Swarm That Walks or Devil. A switch keeps the **general picks** (each rank's mythic ability or feat) up to one rank and the old path's own grants (its rows' features and the picks in its slots) up to another, and may drop named grants whatever their rank: Legend keeps MR 1–2 of both and stays at MR 3; Gold Dragon and Swarm keep the general picks of MR 1–7 but the old path's grants of MR 1–2 only; Devil keeps MR 1–7 but is open only from Aeon or Azata and drops the Aeon's gaze and the Azata's dragon. A switch may grant features of its own (Legend's +4 to every score). The picks it takes away at earlier ranks stay in the Build: they were held from their rank until the switch, so they count for everything before it (a Trickster who becomes a Legend keeps the feats Perception II unlocked), and are lost from the switch on. A switch is locked while a choice from its rank on needs one of them.
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
A feat a class level (or a **Background**) gives outright, with no slot and no pick, so it isn't in the Build or its link: a wizard's Scribe Scrolls at 1, a fighter's Armor, Shield and Weapon Proficiency feats, a Martial Disciple's Improved Unarmed Strike. It's held from that level on whatever its prerequisites; one already held adds nothing, and picking it later is "already held". A granted Parameterised feat is held for every value (All-skilled: Exotic Weapon Proficiency in every exotic weapon), unless the row grants it for one value (the Mantis Zealot's Weapon Focus (Sawtooth Saber), the Chirurgeon's Skill Focus (Lore (Religion))).
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
One entry in a Feature selection's list, such as Fast Stealth among the rogue talents. Its key extends the selection's (`rogue/rogue-talents/fast-stealth` extends `rogue/rogue-talents`). It may grant feats, held while it is: one (a rogue talent that is the feat of that name) or several (the Azata's All-skilled: every weapon, armour and shield proficiency). One that is a Parameterised feat of the player's choosing (the rogue talent Weapon Focus) is its **chosen feat**, and the option's pick names the Parameter. An option's pick may name a Parameter for its rows alone (the shaman's Battle Master: Weapon Specialization at 8 and Greater Weapon Focus at 16 in that weapon), and an option may limit the values it takes (the Zen Archer's Way of the Bow: a longbow or a shortbow); a selection whose one option takes a value is still a pick. It may Repeat. It may grant over class levels: see **Option level**. It may belong to an **Option group**. A sorcerer's bloodline names its **arcana**, the first-row feature a Crossblooded sorcerer's second bloodline still gives.
_Avoid_: Option (unqualified), talent

**Parameterised feat**:
A feat taken for something the player chooses, which the guide words as "choose a single weapon" or "select one skill": Weapon Focus, Skill Focus, Spell Focus, Elemental Focus, Armor Focus, Improved Critical, Exotic Weapon Proficiency, the Greater and Specialization feats that follow them, and the Trickster's Improved Improved Critical chain; mythic feats too (Improved Critical, Spell Focus and Skill Focus (Mythic), Expanded Arsenal, School Mastery). Every one Repeats once per value. Each pick names its Parameter, and a prerequisite can need the feat for one value ("Weapon Focus (Dueling Sword)") or for the pick's own ("Weapon Specialization: a weapon for which you have already selected Weapon Focus"). "Weapon Focus" on its own is the feat in any value.
_Avoid_: Feat with a subtype, feat variant

**Parameter**:
What one pick of a Parameterised feat is taken for: a skill (from `skills.ron`), a weapon (from `weapons.ron`, or only an exotic one for Exotic Weapon Proficiency), a school of magic, an energy type or an armour type. The Parameter is part of the Pick and of the link.
_Avoid_: Subtype, argument, option (that's a Selection option)

**Repeat**:
How many times a feat or Selection option can be held: once (the default, and what the guide means when it says nothing), a fixed number ("up to 3 times": Extra Hex, Increased Damage Reduction), any number ("may be taken multiple times": Extra Ki; the rogue talent Combat Trick, by the user's decision), or once per value of its Parameter ("must be applied to a different weapon each time": Weapon Focus, and by the user's decision every Parameterised feat). An option is held once per value only of its chosen feat (by the user's decision, the Skill Focus wild talent and rogue talent: once per skill).
_Avoid_: Stacking, max picks

**Option level**:
The level of a Selection option that grants over class levels (a bloodline, a wizard school, a cavalier order, a shaman spirit, a Magic Deceiver way, a ranger combat style, a domain, an oracle mystery or revelation, a warpriest blessing, a kineticist element): the levels of the classes its Feature selection names, added up. A domain is levelled by whichever class's selection took it (cleric, inquisitor, druid, Hellknight levels). A sorcerer 5 / Dragon Disciple 5 has bloodline level 10; a bard who picked a Draconic bloodline on entering Dragon Disciple has the Dragon Disciple's levels. The option's **rows** (its features, granted feats, feats for the value its pick named, Feat slots, nested selections and its feat list from that level on) apply as its level reaches them, and one taken late (Second Bloodline) gets every row up to its level at once. An archetype can skip option levels (a Seeker: 3, 9 and 15), and its own rows replace them, or lag behind them (the Separatist's second domain: two levels late). A selection naming no classes takes the level of the other options held that it offers (Impossible Domain: the character's other domains, whoever levels them, a Hellknight's Pentamic Faith domain by Hellknight levels; Second Mystery: the first mystery), with one option held elsewhere counting among them (a Living Deity's way, so its Impossible Domain runs on arcanist levels from the pick), or of the option whose row granted its slot (Maneuver Mastery's maneuver: its revelation's). A path's selection may count Mythic ranks instead: the ranks since the pick, its own rank as 1 (the Azata's Life-Bonding Friendship: a Teamwork feat slot at the rank it's taken and the next, each a slot of that rank).
_Avoid_: Bloodline level (one family's name for it), subclass level

**Option list slot**:
A bonus Feat slot on a class's row whose Feat list is the held option's: a sorcerer's bloodline feat at 7, 13 and 19, a bloodrager's at 6, 9, …, a ranger's combat style feat (a list per stage), a slayer's combat style talent (the style the first one picked, or a ranger's; the list of the talent's window's stage, fixed on the slot, whoever levels the style). With two options held (a Crossblooded sorcerer's second bloodline gives only its list) it offers both lists. It's checked after the level's Feature selections, since a style picked at 2 is what the slot at 2 draws on.
_Avoid_: Bloodline slot (the families share it), dynamic list

**Extra pick**:
One more slot of a Feature selection that a feat gives each time it's taken ("You gain one additional hex": Extra Arcana, Extra Arcanist Exploit, Extra Discovery, Extra Hex, Extra Mercy, Extra Rage Power, Extra Rogue Talent, Extra Slayer Talent, Extra Wild Talent), at the level it's taken. The feat names the selections its "Prerequisites" names (Extra Hex: the witch's and the shaman's hexes), and an archetype's own version of one (Extra Discovery: the Vivisectionist's Medical Discovery; Extra Arcana: the Hexcrafter's Hex Arcana); the slot is of the one the character had a slot of first, so a witch who becomes a shaman gets a witch's hex, and a character who had a slot of none gets none. The resource feats (Extra Ki, Extra Rage, Extra Channel) give no pick. Two mythic feats give one too, at the Mythic rank they're taken, after its general pick: Extra Feat a feat slot that offers what the general slot does ("You can select any normal feat"), Extra Mythic Ability a mythic ability slot that offers what a rank's general pick does. A mythic ability that gives a selection slot when it's picked (Second Bloodline, Second Spirit, Second Mystery, Witch Wandering Hex: a witch hex, Beneficial Curse: an oracle curse) does so through its own data, not as an Extra pick.
_Avoid_: Bonus talent, extra slot feat

**Daily choice**:
A choice the game asks for again every day, so it isn't part of a Build and has no slot: a Mixed-Blood Rager's bloodline, a shaman's Wandering Spirit (an Unsworn Shaman's Minor Spirit and Wandering Spirit 2), an Abjurer's Resistance, a hunter's Animal Focus. The sheet shows it as a note.
_Avoid_: Daily pick, rest choice

**Pool selection**:
A Feature selection that offers the **pools** of the options held in another: an oracle's revelation slot offers the revelations of the mysteries held (two with a mythic Second Mystery), and a Dual-Cursed Oracle's its own two besides. A pool is an ordinary selection that only a Pool selection offers. Within a level its slot is checked after the other selections'.
_Avoid_: Dynamic selection, revelation list

**Option group**:
A family of Selection options of which a character holds only one: the sorcerer's ten Draconic bloodlines ("You can only take a single draconic bloodline"), its four Elemental ones, and the bloodrager's dragons and elements, apart from the sorcerer's. The options' prerequisite says "no other option of this group", so a Crossblooded sorcerer or a Second Bloodline can't add a second dragon. (Expanded Element's "not the 7th-level element again" is not a group: the selection gives each option once.)
_Avoid_: Bloodline family, exclusive set

**Shared option**:
A Selection option that one Feature selection offers from another's list, such as the rogue talents a slayer's talents offer. It keeps the key of the selection that defines it, so a character holds it once whichever slot took it, and its prerequisite is the same in both (a gate on one class's feature accepts each sharing class's).
_Avoid_: Copied option, alias

**Slot**:
One place in a Build where something is picked: a Feat slot, a Feature selection slot, or a Mythic ascension's mythic ability, mythic feat or path pick. Each has a stable id naming its level or rank. Within a level, slots are checked in this order: the class's Feature selections, the class's feat slots, the race's, the general slot, then the slots of options' rows and feats' Extra picks; so a race or general pick sees what the class gives at that level (a human warpriest's Dazzling Display on the class's Weapon Focus).
_Avoid_: Choice point, cell

**Pick**:
What fills a slot: a feat, a selection option, a mythic ability or a mythic feat. A pick of a Parameterised feat or mythic feat, or of an option that takes a value, also names its Parameter (Weapon Focus (Longsword)).
_Avoid_: Choice (that is any decision in a Build), selection

**Prerequisite**:
A condition that must hold before a feat, selection option, mythic pick, class or archetype can be chosen. Examples: an ability score, BAB, another feat, skill ranks or a class feature. One can depend on the pick's Parameter (Armor Focus: the proficiency of the armour type it's taken for), on the option picked (no other option of its Option group) or the selection it's picked through (a Champion of the Faith's alignment blessing matches his alignment as his chosen alignment only; a Separatist's second domain doesn't conflict with his), or be one that must not hold (the slayer's first combat style feat: not yet slayer 6). A Story condition is one too, but never blocks. One can ask about the **Faith**: a deity, a given deity, atheism, or a deity offering a domain (the Angelfire Apostle's good deity). An archetype's alignment requirement replaces its class's, and the class's other conditions still hold: a Drunken Master, who can't be Lawful, is a monk though a monk must be Lawful; an archetype without one keeps its class's.
_Avoid_: Requirement, restriction

**Story condition**:
A prerequisite set by the game's story rather than by a Build, such as the Gold Dragon's Vow of Dahak or Uncorrupted. Story is out of scope: the condition always holds and the sheet shows it as a note beside the option.
_Avoid_: Flag, quest requirement

### The planner

**Sheet**:
The one planning screen and its summary: a grid with a row per character level and a chip per slot, plus the Mythic track. A finished Build's summary is the same sheet.
_Avoid_: Level-up screen, summary page, character sheet

**Mythic track**:
The sheet's tab for Mythic ranks 1–10 at their fixed character levels, with the First ascension at MR 1, the path at MR 3 and any Path switch at MR 8, and under each rank the features its path row gives. MR 1 and 2 show the first ascension's path and rows. A rank is reached, partly dropped by a switch (its general pick or the old path's grants lost, or a grant the switch drops by name), dropped (both lost), or never reached; the picks a switch took away are marked lost from the switch on, and an empty slot of such a rank can still be filled (it is held until the switch). A rank the Build doesn't reach holds no picks: Remove level and a switch that ends the ascensions (Legend: MR 8 to 10) drop them, and Remove level drops the path or the switch with its rank.
_Avoid_: Mythic tab, mythic levels

**Stale link**:
A link whose Build the current game data can't hold: a key the data no longer has, a pick that no longer fits, a pick for a slot or Mythic rank the Build doesn't reach, or a rule broken (the game data changed since the link was made). A damaged or hand-edited link counts the same. It opens nothing: the sheet shows one generic error, "This link no longer works with the current game data", and a blank Build, never a reason. Every Build the sheet's edits make is possible, so nothing on the sheet is ever marked broken.
_Avoid_: Problem, broken pick, stale-link banner

**Removable**:
A choice the sheet may remove or change now: no later choice depends on it. Choices come off from the leaves inward: of Power Attack, Cleave and Great Cleave, Great Cleave first, then Cleave. A picker locks an option that would break a later choice, greyed out with no reason.
_Avoid_: Would-break, conflict
