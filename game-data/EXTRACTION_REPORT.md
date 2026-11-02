# Extraction report

**Hand-fixed on 2026-11-01** by tickets 17–20 (feats, base classes, prestige classes, mythic). Remaining: Feats 0, Base classes 0, Prestige classes 0, Mythic 0, Races and skills 0. Items marked *narrowed* are partly fixed; items marked *new* were found during the hand-fix. Ticket 25 (shared selection options) fixed `rogue#discovery` and `rogue#fey-tricks`. Ticket 26 (mythic path rules) fixed the Dragon Feats slot, the five switch items (partial keeps, Devil's previous paths and drops, Legend's effects, the Trickster's Perception II feats) and the breaths' "Mythic Rank: 9". Ticket 27 (repeatable and parameterised feats) fixed the Master of All's three Skill Focus and the Stalwart Defender's Increased Damage Reduction "twice". Ticket 28 (class rules) fixed the Assassin's alignment, Diverse Training, Frost Power, the three prestige classes that advance one class, arcane or divine, the Gold Dragon breaths' story conditions and Domain Zealot's druid domain (Nature Bond), and settled the Crusader's bonus feats (they keep skipping every prerequisite, by the user's decision; a comment in `classes/cleric.ron` says so). Ticket 30 (option levels) fixed the 39 option tables (sorcerer and bloodrager bloodlines, wizard schools, cavalier orders, Magic Deceiver ways), the bloodline feat lists, Disruptive's "Arcane bloodline", the Dragon Disciple's entry and Blood of Dragons, and the Winter Witch's Frost spirit, and narrowed the shaman's Second Spirit item. Ticket 31 (domains, mysteries, blessings, elements) narrowed the kineticist's bonus-feat item (Elemental Focus is data) and the Hellknight order item (the Hellknight's order is a selection; the Signifer's isn't). Ticket 32 (feat and rule follow-ups) fixed the slayer's combat style feats (three slayer talents, one per window). Ticket 33 (option follow-ups) fixed the Signifer's order (`hellknight-signifer/order`, sharing the Hellknight's seven). Ticket 43 (the report's gaps) fixed the barbarian's totem rage powers (exclusive by `Not`), the shaman's Wandering Hex (a daily choice) and the Eldritch Scoundrel's sneak attack (the reading checked against the guide: dice at 1, 3, 7, 11, 15 and 19); its 9 items left were its questions for the user. Ticket 44 (the user's decisions on them) fixed the sorcerer archetypes' level-1 bonus feat (every sorcerer's, base included, is from the held bloodline's list but the Geomancer's, which stays a general slot) and Extra Slayer Talent (needs slayer level 2; Talented Slayer stays out), dropped the cleric's deity (deities are out of scope) and moved the kineticist's bonus-feat wild talent to ticket 45 (wild talents become a selection).

What the extractor couldn't turn into data from the guide pages saved on 2026-10-31, with where each item is (`page#anchor`). Fix an item by editing the `.ron` files, check with `cargo run -p data --bin seed -- --check-only`, then delete its line here. 0 items.

## Feats (0)

Nothing left.

## Base classes (0)

Nothing left.

## Prestige classes (0)

Nothing left.

## Mythic (0)

Nothing left.

## Races and skills (0)

Nothing left.
