# Extraction report

**Hand-fixed on 2026-11-01** by tickets 17–20 (feats, base classes, prestige classes, mythic). Remaining: Feats 2, Base classes 54, Prestige classes 12, Mythic 2, Races and skills 0. Items marked *narrowed* are partly fixed; items marked *new* were found during the hand-fix. Ticket 25 (shared selection options) fixed `rogue#discovery` and `rogue#fey-tricks`. Ticket 26 (mythic path rules) fixed the Dragon Feats slot, the five switch items (partial keeps, Devil's previous paths and drops, Legend's effects, the Trickster's Perception II feats) and the breaths' "Mythic Rank: 9".

What the extractor couldn't turn into data from the guide pages saved on 2026-10-31, with where each item is (`page#anchor`). Fix an item by editing the `.ron` files, check with `cargo run -p data --bin seed -- --check-only`, then delete its line here. 70 items.

## Feats (2)

### Prerequisite lines (2)

Each line is left out of its prerequisite, so it doesn't block any pick. A group (`or`, "One of") with an unresolved member is left out whole.

- `feats#disruptive`: "Fighter Level 6 or Arcane bloodline" (nothing named "Arcane bloodline"; sorcerer bloodlines aren't a selection in the data)
- `feats#extra-slayer-talent`: "Slayer Talents or Talented Slayer" (nothing named "Talented Slayer"; it appears on no saved page)

## Base classes (54)

### Entry requirements (1)

Left out of the class's or archetype's prerequisite.

- `cleric`: "Alignment must be within 1 place of their chosen deity." (nothing named "Alignment must be within 1 place of their chosen deity."; deities aren't data)

### Progression table cells (39)

The cell's text, and how it was stored instead.

- `arcanist#fake-celebrity`: "Fake Celebrity" (a progression table under a feature heading isn't read, so what it grants is missing)
- `arcanist#living-deity`: "Living Deity" (a progression table under a feature heading isn't read, so what it grants is missing)
- `arcanist#magic-inventor`: "Magic Inventor" (a progression table under a feature heading isn't read, so what it grants is missing)
- `arcanist#razmiri-infiltrator`: "Razmiri Infiltrator" (a progression table under a feature heading isn't read, so what it grants is missing)
- `arcanist#time-manipulator`: "Time Manipulator" (a progression table under a feature heading isn't read, so what it grants is missing)
- `arcanist#way-of-the-charlatan`: "Way of the Charlatan" (a progression table under a feature heading isn't read, so what it grants is missing)
- `bloodrager#abyssal-bloodrager`: "Abyssal Bloodrager" (a progression table under a feature heading isn't read, so what it grants is missing)
- `bloodrager#arcane-bloodrager`: "Arcane Bloodrager" (a progression table under a feature heading isn't read, so what it grants is missing)
- `bloodrager#celestial-bloodrager`: "Celestial Bloodrager" (a progression table under a feature heading isn't read, so what it grants is missing)
- `bloodrager#draconic-bloodrager`: "Draconic Bloodrager" (a progression table under a feature heading isn't read, so what it grants is missing)
- `bloodrager#elemental-bloodrager`: "Elemental Bloodrager" (a progression table under a feature heading isn't read, so what it grants is missing)
- `bloodrager#fey-bloodrager`: "Fey Bloodrager" (a progression table under a feature heading isn't read, so what it grants is missing)
- `bloodrager#hag-bloodrager`: "Hag Bloodrager" (a progression table under a feature heading isn't read, so what it grants is missing)
- `bloodrager#infernal-bloodrager`: "Infernal Bloodrager" (a progression table under a feature heading isn't read, so what it grants is missing)
- `bloodrager#serpentine-bloodrager`: "Serpentine Bloodrager" (a progression table under a feature heading isn't read, so what it grants is missing)
- `bloodrager#undead-bloodrager`: "Undead Bloodrager" (a progression table under a feature heading isn't read, so what it grants is missing)
- `cavalier#order-of-the-cockatrice`: "Order of the Cockatrice" (a progression table under a feature heading isn't read, so what it grants is missing)
- `cavalier#order-of-the-lion`: "Order of the Lion" (a progression table under a feature heading isn't read, so what it grants is missing)
- `cavalier#order-of-the-shroud`: "Order of the Shroud" (a progression table under a feature heading isn't read, so what it grants is missing)
- `cavalier#order-of-the-star`: "Order of the Star" (a progression table under a feature heading isn't read, so what it grants is missing)
- `cavalier#order-of-the-sword`: "Order of the Sword" (a progression table under a feature heading isn't read, so what it grants is missing)
- `sorcerer#abyssal`: "Abyssal" (a progression table under a feature heading isn't read, so what it grants is missing)
- `sorcerer#arcane`: "Arcane" (a progression table under a feature heading isn't read, so what it grants is missing)
- `sorcerer#celestial`: "Celestial" (a progression table under a feature heading isn't read, so what it grants is missing)
- `sorcerer#draconic`: "Draconic" (a progression table under a feature heading isn't read, so what it grants is missing)
- `sorcerer#elemental`: "Elemental" (a progression table under a feature heading isn't read, so what it grants is missing)
- `sorcerer#fey`: "Fey" (a progression table under a feature heading isn't read, so what it grants is missing)
- `sorcerer#infernal`: "Infernal" (a progression table under a feature heading isn't read, so what it grants is missing)
- `sorcerer#serpentine`: "Serpentine" (a progression table under a feature heading isn't read, so what it grants is missing)
- `sorcerer#undead`: "Undead" (a progression table under a feature heading isn't read, so what it grants is missing)
- `wizard#abjurer`: "Abjurer" (a progression table under a feature heading isn't read, so what it grants is missing)
- `wizard#conjurer`: "Conjurer" (a progression table under a feature heading isn't read, so what it grants is missing)
- `wizard#diviner`: "Diviner" (a progression table under a feature heading isn't read, so what it grants is missing)
- `wizard#enchanter`: "Enchanter" (a progression table under a feature heading isn't read, so what it grants is missing)
- `wizard#illusionist`: "Illusionist" (a progression table under a feature heading isn't read, so what it grants is missing)
- `wizard#invoker`: "Invoker" (a progression table under a feature heading isn't read, so what it grants is missing)
- `wizard#necromancer`: "Necromancer" (a progression table under a feature heading isn't read, so what it grants is missing)
- `wizard#transmuter`: "Transmuter" (a progression table under a feature heading isn't read, so what it grants is missing)
- `wizard#universalist`: "Universalist" (a progression table under a feature heading isn't read, so what it grants is missing)

### Bonus feat slots (8)

The slot as stored, and why it needs a look.

- `character-creation#bloodlines`: "Bloodline Feat" (*narrowed*: now one list per class, sorcerer 34 feats and bloodrager 28, checking prerequisites; the bloodline held can't narrow it, because bloodlines aren't a selection)
- `cleric#crusader-bonus-feats`: "Bonus Feat" (*new*: only the BAB and fighter-level prerequisites are skipped; stored as skipping all of them, since a slot can't skip some)
- `sorcerer#crossblooded`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#empyreal-sorcerer`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#geomancer`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#nine-tailed-heir`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#sage-sorcerer`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#sylvan-sorcerer`: "Bonus Feat" (no list found for this slot; stored as a general slot)

### Prose-only rules (6)

Rules stated only in prose that the current data types can't hold.

- `barbarian#rage-powers`:  (totem rage powers are exclusive: one totem line per character. The data has no "not" prerequisite, so only the Lesser, plain, Greater chain is checked)
- `kineticist#kineticist-bonus-feat`:  (a wild talent that grants a bonus feat slot; a selection option can't grant a slot. The kineticist has no selections: wild talents and Elemental Focus aren't data)
- `rogue#skill-focus-master-of-all`:  (*new*: one `skill-focus` feat key can't be held more than once, so the Master of All's three Skill Focus grants are stored as features)
- `rogue#sneak-attack-eldritch-scoundrel`:  (*new*: read from prose as dice at 1, 3, 7, 11, 15, 19; the reading needs checking)
- `shaman`:  (*new*: the Second Spirit mythic ability and Wandering Hex give another spirit's hexes; not modelled)
- `slayer#slayer-talents`:  (*new*: a slayer may instead take a ranger combat style feat three times, at 2 or 4, 6 or 8, and 10+; not offered, since nothing limits how many options carrying a feat are taken)

## Prestige classes (12)

### Entry requirements (4)

Left out of the class's or archetype's prerequisite.

- `prestige-classes#assassin`: "Alignment must be neutral or evil" (*new*: ambiguous; stored as any alignment with a neutral part, or evil, which includes Neutral Good. Read as "not good", Neutral Good would be excluded)
- `prestige-classes#dragon-disciple`: "Must have a Draconic bloodline or no known bloodline" (doesn't parse; there's no "not" or "no bloodline" condition)
- `prestige-classes#dragon-disciple`: "Required special: Able to cast 1st level arcane spells spontaneously" (*narrowed*: `SpellLevel(Arcane, 1)` is stored; there's no spontaneous-caster condition)
- `prestige-classes#winter-witch`: "Required classes: Druid (Winter Child), Shaman (Frost Spirit) or Witch (Winter Patron)" (*narrowed*: stored as `Any` of the Winter Child archetype, the witch's Winter patron and `ClassLevel(class: "shaman", level: 1)`; the shaman branch is a stand-in for the Frost spirit's selection option, `shaman/spirit/frost`)

### Prose-only rules (8)

Rules stated only in prose that the current data types can't hold.

- `prestige-classes#blood-of-dragons`:  (*new*: Dragon Disciple levels continue the sorcerer bloodline progression, and a non-sorcerer picks a Draconic bloodline; neither is in the data)
- `prestige-classes#diverse-training`:  (*new*: Eldritch Knight levels count as Fighter levels and caster levels for feat prerequisites)
- `prestige-classes#frost-power`:  (*new*: Winter Witch levels stack with Shaman and Witch levels for hexes, including Major and Grand hex access)
- `prestige-classes#hellknight-signifer`:  (advances one class's spellcasting, arcane or divine: stored as advancing both, which the engine reads as one of each)
- `prestige-classes#increased-damage-reduction`:  (*new*: "may be selected twice"; a selection option can't be held twice)
- `prestige-classes#loremaster`:  (advances one class's spellcasting, arcane or divine: stored as advancing both, which the engine reads as one of each)
- `prestige-classes#order`, `prestige-classes#order-hellknight-signifer`:  (*new*: a Hellknight who wasn't an Armiger, and every Signifer, picks one of seven orders, but there's no selection for it, so Pentamic Faith is closed to a non-Armiger Hellknight)
- `prestige-classes#winter-witch`:  (advances one class's spellcasting, arcane or divine: stored as advancing both, which the engine reads as one of each)

## Mythic (2)

### Prerequisite lines (2)

Each line is left out of its prerequisite, so it doesn't block any pick. A group (`or`, "One of") with an unresolved member is left out whole.

- `gold-dragon#annihilating-breath`, `#decaying-breath`, `#enlightening-breath`, `#rejuvenating-breath`: "Vow of Dahak", "Uncorrupted" (*new*: story conditions, out of scope by the user's decision R5 in ticket 21; "Mythic Rank: 9" is `MythicRank(9)` since ticket 26)
- `mythic-paths#domain-zealot`: "Domain" (*new*: a druid has a domain only if Nature Bond took one, and Nature Bond isn't a choice in the data; `ClassLevel(druid, 1)` lets every druid in)

## Races and skills (0)

Nothing left.
