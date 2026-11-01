# Extraction report

What the extractor couldn't turn into data from the guide pages saved on 2026-10-31, with where each item is (`page#anchor`). Fix an item by editing the `.ron` files, check with `cargo run -p data --bin seed -- --check-only`, then delete its line here. 82 items.

## Feats (4)

### Prerequisite lines (4)

Each line is left out of its prerequisite, so it doesn't block any pick. A group (`or`, "One of") with an unresolved member is left out whole.

- `feats#armor-focus`: "Armor Proficiency (Light. Medium or Heavy)" (nothing named "Armor Proficiency (Light. Medium or Heavy)")
- `feats#disruptive`: "Fighter Level 6 or Arcane bloodline" (nothing named "Arcane bloodline")
- `feats#extra-channel`: "Channel (Cleric) or Channel Positive Energy (Empyreal Sorcerer / Hospitaler)" (nothing named "Channel (Cleric)")
- `feats#extra-slayer-talent`: "Slayer Talents or Talented Slayer" (nothing named "Talented Slayer")

## Base classes (64)

### Prerequisite lines (11)

Each line is left out of its prerequisite, so it doesn't block any pick. A group (`or`, "One of") with an unresolved member is left out whole.

- `arcanist#icy-tomb`: "Icy Missile" (nothing named "Icy Missile")
- `hunter#push-and-bite`: "Favorite New Toy" (nothing named "Favorite New Toy")
- `shaman#battle-master`: "Battle" (nothing named "Battle")
- `shaman#battle-ward`: "Battle" (nothing named "Battle")
- `shaman#beckoning-chill`: "Waves" (nothing named "Waves")
- `shaman#entangling-curse`: "Nature" (nothing named "Nature")
- `shaman#friend-to-animals`: "Nature" (nothing named "Nature")
- `shaman#hampering-hex`: "Battle" (nothing named "Battle")
- `shaman#hypothermia`: "Frost" (nothing named "Frost")
- `shaman#metal-curse`: "Stone" (nothing named "Stone")
- `shaman#stone-stability`: "Stone" (nothing named "Stone")

### Entry requirements (1)

Left out of the class's or archetype's prerequisite.

- `cleric`: "Alignment must be within 1 place of their chosen deity." (nothing named "Alignment must be within 1 place of their chosen deity.")

### Progression table cells (40)

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
- `monk#student-of-stone`:  (a feat list with no known feats was dropped)
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

### Bonus feat slots (9)

The slot as stored, and why it needs a look.

- `cavalier#tactician-bonus-feat`: "Tactician - Bonus Feat" (the list names no feats; stored as a general slot)
- `character-creation#bloodlines`: "Bloodline Feat" (each bloodline has its own list; stored as one list of every bloodline's feats, checking prerequisites)
- `ranger#nomad`: "Archery Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#crossblooded`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#empyreal-sorcerer`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#geomancer`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#nine-tailed-heir`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#sage-sorcerer`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#sylvan-sorcerer`: "Bonus Feat" (no list found for this slot; stored as a general slot)

### Prose-only rules (3)

Rules stated only in prose that the current data types can't hold.

- `barbarian#rage-powers`:  (totem rage powers are exclusive: one totem line per character. The data has no "not" prerequisite, so only the Lesser, plain, Greater chain is checked)
- `kineticist#kineticist-bonus-feat`:  (a wild talent that grants a bonus feat slot; a selection option can't grant a slot)
- `slayer#advanced-talents`:  (from 10th level slayer talents may be advanced rogue talents; the slayer talent list is prose, not option headings, so the selection isn't built)

## Prestige classes (7)

### Entry requirements (4)

Left out of the class's or archetype's prerequisite.

- `prestige-classes#dragon-disciple`: "Must have a Draconic bloodline or no known bloodline" (doesn't parse)
- `prestige-classes#dragon-disciple`: "Required special: Able to cast 1st level arcane spells spontaneously" (nothing named "Able to cast 1st level arcane spells spontaneously")
- `prestige-classes#loremaster`: "Required feats: Metamagic (any), Skill Focus (Knowledge: Arcana or Knowledge: World)" (nothing named "Metamagic (any)")
- `prestige-classes#winter-witch`: "Required classes: Druid (Winter Child), Shaman (Frost Spirit) or Witch (Winter Patron)" (nothing named "Druid (Winter Child)")

### Prose-only rules (3)

Rules stated only in prose that the current data types can't hold.

- `prestige-classes#hellknight-signifer`:  (advances one class's spellcasting, arcane or divine: stored as advancing both, which the engine reads as one of each)
- `prestige-classes#loremaster`:  (advances one class's spellcasting, arcane or divine: stored as advancing both, which the engine reads as one of each)
- `prestige-classes#winter-witch`:  (advances one class's spellcasting, arcane or divine: stored as advancing both, which the engine reads as one of each)

## Mythic (7)

### Prerequisite lines (2)

Each line is left out of its prerequisite, so it doesn't block any pick. A group (`or`, "One of") with an unresolved member is left out whole.

- `mythic-paths#destiny-beyond-birth`: "Racial attribute penalty" (nothing named "Racial attribute penalty")
- `mythic-paths#second-bloodrager-bloodline`: "Bloodrager Bloodline" (nothing named "Bloodrager Bloodline")

### Bonus feat slots (1)

The slot as stored, and why it needs a look.

- `gold-dragon`: "Dragon Feats" (three feats chosen without prerequisites; slots outside a class can't skip prerequisites, so stored as three general slots)

### Prose-only rules (4)

Rules stated only in prose that the current data types can't hold.

- `gold-dragon`:  (the switch drops the previous path's own powers from MR 3-7 but keeps the mythic abilities and feats; the data keeps whole ranks, so MR 1-7 are kept in full)
- `other-mythic-paths#devil`:  (the Devil switch is open only to an Aeon or Azata; a path switch has no prerequisite in the data)
- `other-mythic-paths#legend`:  (no rank table: the path's ranks are prose only)
- `other-mythic-paths#swarm-that-walks`:  (the switch drops the previous path's own powers from MR 3-7 but keeps the mythic abilities and feats; the data keeps whole ranks, so MR 1-7 are kept in full)

## Races and skills (0)

Nothing left.

