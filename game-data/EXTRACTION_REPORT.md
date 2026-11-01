# Extraction report

**Hand-fixed on 2026-11-01** by tickets 17–20 (feats, base classes, prestige classes, mythic). Remaining: Feats 1, Base classes 11, Prestige classes 0, Mythic 0, Races and skills 0. Items marked *narrowed* are partly fixed; items marked *new* were found during the hand-fix. Ticket 25 (shared selection options) fixed `rogue#discovery` and `rogue#fey-tricks`. Ticket 26 (mythic path rules) fixed the Dragon Feats slot, the five switch items (partial keeps, Devil's previous paths and drops, Legend's effects, the Trickster's Perception II feats) and the breaths' "Mythic Rank: 9". Ticket 27 (repeatable and parameterised feats) fixed the Master of All's three Skill Focus and the Stalwart Defender's Increased Damage Reduction "twice". Ticket 28 (class rules) fixed the Assassin's alignment, Diverse Training, Frost Power, the three prestige classes that advance one class, arcane or divine, the Gold Dragon breaths' story conditions and Domain Zealot's druid domain (Nature Bond), and settled the Crusader's bonus feats (they keep skipping every prerequisite, by the user's decision; a comment in `classes/cleric.ron` says so). Ticket 30 (option levels) fixed the 39 option tables (sorcerer and bloodrager bloodlines, wizard schools, cavalier orders, Magic Deceiver ways), the bloodline feat lists, Disruptive's "Arcane bloodline", the Dragon Disciple's entry and Blood of Dragons, and the Winter Witch's Frost spirit, and narrowed the shaman's Second Spirit item. Ticket 31 (domains, mysteries, blessings, elements) narrowed the kineticist's bonus-feat item (Elemental Focus is data) and the Hellknight order item (the Hellknight's order is a selection; the Signifer's isn't). Ticket 32 (feat and rule follow-ups) fixed the slayer's combat style feats (three slayer talents, one per window). Ticket 33 (option follow-ups) fixed the Signifer's order (`hellknight-signifer/order`, sharing the Hellknight's seven).

What the extractor couldn't turn into data from the guide pages saved on 2026-10-31, with where each item is (`page#anchor`). Fix an item by editing the `.ron` files, check with `cargo run -p data --bin seed -- --check-only`, then delete its line here. 12 items.

## Feats (1)

### Prerequisite lines (1)

Each line is left out of its prerequisite, so it doesn't block any pick. A group (`or`, "One of") with an unresolved member is left out whole.

- `feats#extra-slayer-talent`: "Slayer Talents or Talented Slayer" (nothing named "Talented Slayer"; it appears on no saved page)

## Base classes (11)

### Entry requirements (1)

Left out of the class's or archetype's prerequisite.

- `cleric`: "Alignment must be within 1 place of their chosen deity." (nothing named "Alignment must be within 1 place of their chosen deity."; deities aren't data)

### Bonus feat slots (6)

The slot as stored, and why it needs a look.

- `sorcerer#crossblooded`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#empyreal-sorcerer`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#geomancer`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#nine-tailed-heir`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#sage-sorcerer`: "Bonus Feat" (no list found for this slot; stored as a general slot)
- `sorcerer#sylvan-sorcerer`: "Bonus Feat" (no list found for this slot; stored as a general slot)

### Prose-only rules (4)

Rules stated only in prose that the current data types can't hold.

- `barbarian#rage-powers`:  (totem rage powers are exclusive: one totem line per character. Only the Lesser, plain, Greater chain is checked: a `Not` prerequisite exists since ticket 32, but the totems don't use it yet)
- `kineticist#kineticist-bonus-feat`:  (*narrowed*: a wild talent that grants a bonus feat slot. Elemental Focus is a selection since ticket 31, each element with its wild-talent bonus feat list, but wild talents aren't data, so no slot picks from it)
- `rogue#sneak-attack-eldritch-scoundrel`:  (*new*: read from prose as dice at 1, 3, 7, 11, 15, 19; the reading needs checking)
- `shaman`:  (*new*, *narrowed*: the Second Spirit mythic ability is a second spirit pick now; Wandering Hex, a hex from another spirit, is not modelled)

## Prestige classes (0)

Nothing left.

## Mythic (0)

Nothing left.

## Races and skills (0)

Nothing left.
