# Extraction report

**Hand-fixed on 2026-11-01** by tickets 17–20 (feats, base classes, prestige classes, mythic). Remaining: Feats 0, Base classes 0, Prestige classes 0, Mythic 0, Races and skills 0. Items marked *narrowed* are partly fixed; items marked *new* were found during the hand-fix. Ticket 25 (shared selection options) fixed `rogue#discovery` and `rogue#fey-tricks`. Ticket 26 (mythic path rules) fixed the Dragon Feats slot, the five switch items (partial keeps, Devil's previous paths and drops, Legend's effects, the Trickster's Perception II feats) and the breaths' "Mythic Rank: 9". Ticket 27 (repeatable and parameterised feats) fixed the Master of All's three Skill Focus and the Stalwart Defender's Increased Damage Reduction "twice". Ticket 28 (class rules) fixed the Assassin's alignment, Diverse Training, Frost Power, the three prestige classes that advance one class, arcane or divine, the Gold Dragon breaths' story conditions and Domain Zealot's druid domain (Nature Bond), and settled the Crusader's bonus feats (they keep skipping every prerequisite, by the user's decision; a comment in `classes/cleric.ron` says so). Ticket 30 (option levels) fixed the 39 option tables (sorcerer and bloodrager bloodlines, wizard schools, cavalier orders, Magic Deceiver ways), the bloodline feat lists, Disruptive's "Arcane bloodline", the Dragon Disciple's entry and Blood of Dragons, and the Winter Witch's Frost spirit, and narrowed the shaman's Second Spirit item. Ticket 31 (domains, mysteries, blessings, elements) narrowed the kineticist's bonus-feat item (Elemental Focus is data) and the Hellknight order item (the Hellknight's order is a selection; the Signifer's isn't). Ticket 32 (feat and rule follow-ups) fixed the slayer's combat style feats (three slayer talents, one per window). Ticket 33 (option follow-ups) fixed the Signifer's order (`hellknight-signifer/order`, sharing the Hellknight's seven). Ticket 43 (the report's gaps) fixed the barbarian's totem rage powers (exclusive by `Not`), the shaman's Wandering Hex (a daily choice) and the Eldritch Scoundrel's sneak attack (the reading checked against the guide: dice at 1, 3, 7, 11, 15 and 19); its 9 items left were its questions for the user. Ticket 44 (the user's decisions on them) fixed the sorcerer archetypes' level-1 bonus feat (every sorcerer's, base included, is from the held bloodline's list but the Geomancer's, which stays a general slot) and Extra Slayer Talent (needs slayer level 2; Talented Slayer stays out), dropped the cleric's deity (deities are out of scope) and moved the kineticist's bonus-feat wild talent to ticket 45 (wild talents become a selection). Ticket 45 fixed it: wild talents are a selection, and the bonus-feat talent grants a slot from the held elements' lists.

What the extractor couldn't turn into data from the guide pages saved on 2026-10-31, with where each item is (`page#anchor`). Fix an item by editing the `.ron` files, check with `cargo run -p seeds --bin seed -- --check-only`, then delete its line here. 0 items.

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

## Gear (`gear.ron`, ticket 165)

Read from the saved pages `items`, `items-inevitable-excess` and `items-lord-of-nothing` only, with a throwaway
script (kept out of the repo) that lists every table row under the guide's item headings. A row counts when its
Properties (for armour and weapons, the third column) give an always-on `+N <type> bonus` to a score, including the
wordings "all ability scores" and "all attributes" (the first pass matched score names only and missed three
consumables); the 73 rows are checked by hand against the page.

| Page | Belt | Boots | Bracers | Cloak | Glasses | Gloves | Headwear | Necklace | Ring | Robe | Usable | Armour | Weapon | Rows |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `items` | 12 | 2 | 1 | 1 | 1 | 3 | 19 | 1 | 3 | 3 | 10 | 4 | 1 | 61 |
| `items-inevitable-excess` | 2 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 4 |
| `items-lord-of-nothing` | 3 | 0 | 0 | 0 | 0 | 2 | 1 | 1 | 0 | 0 | 0 | 0 | 1 | 8 |

73 rows (by the guide's section; Usable includes the Trusted Friend), 3 repeats (Flawless Headband of Mental Perfection +8 on `items-inevitable-excess`, Hammer Fists and Soaring
Fist on `items-lord-of-nothing`: the same properties as the main page's rows, so one entry each with the main page's
reference), 70 distinct items (the design's 67, plus the three consumables below). Of the 70, 9 are `Usable` (the six
manuals and tomes and the three all-score consumables) and the Trusted Friend is a `Necklace`. Plus 22 items from the user with no page: 18 single-score belts and headbands at +2, +4 and +6,
and Belt of Physical Perfection and Headband of Mental Perfection at +2 and +4 (their +6 are the guide's). 92 items.

Judgement calls:

- **Added after review:** Tome of Ancient Perfection (+4 inherent to all ability scores), Elixir of Inconceivable
  Transmutations of Body, and Soul, and Also Mind (+2) and Potion of Humanity (+1 "to all attributes"), each six
  `Inherent` bonuses, `Usable`. A wider pattern ("all ability scores", "all attributes", "highest", "ability score",
  "attribute", abbreviations) over every row of the three pages found nothing else that is a worn or used item's
  bonus: the rest are damage, penalties on a foe, caps, or what is already listed. Two near-misses are out: the
  Conflagrant Taco (a cooking recipe in the Recipe table, a meal with "+4 bonus to all ability
  scores" and no item slot) and Savamelekh Spine (a penalty on its target).
- **Left out, for the second stage:** Vellexia's Magnifying Amulet (`items#necklaces`): its "+2 bonus to highest
  current ability score" depends on which score is highest, which `GearTarget` cannot say.
- **Left out, though a score is named:** Glasses of Undeniable Truth (+2 Int only worn with a robe), Bestial Rags
  (+2 racial to three scores only in a major form), Robe of False Death (a -6 Con penalty only), Mutilated Angel
  (a -3 Cha penalty only), Nimble Edge (+2 morale Dex for a few rounds after a crit); and every row that names a
  score only as damage, a DC or a modifier (Attrition Dagger, Poisonous Dart, Peacemaker, Rusty Dawn, The
  Debilitator, Eviscerating Doom, Burning Torment, Smasher, Decimation, Rulebreaker, Mystic Grace, Trailblazer's
  Armor, Carapace, Cloak of the Ancient Herald, Skinned Leather Cloak, Gloves of Phlebotomy, Helmet of Weakening
  Torture, Amulet of Agile Fists, Demon's Eye, Dead Man's Hat, Grave Shivering).
- **Relics and artifacts** (`items#relics`, `#artifacts`) are listed in prose with upgrade tiers and a chosen form
  (Face-Stealer's Mask's belts and headbands, the Stone of Ghostly Pathways' circlets, Zaoris's Pin's headbands),
  not as table rows, and the design counted table rows. They are not in the file; a later ticket may add them.
- **Trusted Friend** sits in "Equipable and Consumable", but the guide's Skill Boosting Items table lists it as an
  "Amulet": `Necklace`, keeping its `items#equipable-and-consumable` reference.
- **Belt of the Venerated Champion:** its "+2 bonus to Fortitude saves and a +1 insight bonus to AC against evil
  creatures" is ambiguous about which part is against evil creatures, so by the design it is treated as conditional:
  one other-effect line, and only Con +6 as a bonus.
- **Armour and weapon lines:** each names its own enhancement and base type from the table's sub-heading: Snakeskin a
  +4 leather armour, Life Infuser a +5 breastplate, Half-Plate of Vigor a +1 half-plate, Ustlavic Pride a +3 full
  plate, Sin Mage's Staff a +5 quarterstaff, Dirge for Sarkoris a +2 keen club (`items-lord-of-nothing#weapons`,
  under the "Club" sub-heading).
- **Sin Mage's Staff** (`Weapon`, `items#weapon-properties`): Int +6 untyped, spell DC +1 and caster level +3 are
  bonuses; "a +5 quarterstaff" is an other effect (the table's sub-heading). **Dirge for Sarkoris**: Str +2 untyped.
- Untyped: Half-Plate of Vigor, Dirge for Sarkoris, Sin Mage's Staff and the +3 Will of the Goggles of Ferocious Pact (the guide gives no type). Life Infuser's "+8
  armour bonus" is typed `Armour` as written.

### Always-on effects (ticket 176)

The 92 items' lines were reclassified against the saved pages: of the 74 `other_effects` lines on 54 items, 35 were
always on and moved to a new `always_on` list, and 39 stayed whole; 41 lines on 36 items are now the detail's (the 39,
plus the remainders of the Offroad Boots' and Draven's Hat's lines, each of which held an always-on and a conditional
effect and was split). `always_on` has 36 lines on 28 items: the 35 moved lines, plus one more because Razmir's Mask's
one line became two. The 22 hand-added items have neither list. 18 items (Display of Power, Flawless Belt and Flawless
Headband of Mental Perfection +8, Goggles of Ferocious Pact, Hammer Fists, Bookworm's Headband, Darkness Caress,
Headband of Confidence, of the Wise Negotiator and of Reshaping, Razmir's Ring, Earth's Embrace, Grand Festival Cloak,
Jester Cap, Half-Plate of Vigor, Life Infuser, Snakeskin and Sin Mage's Staff) are left with no conditional line, so no
detail and no info button. The always-on lines count in no number, and the MCP lists them as `uncounted`.

The rule: what applies while the item is worn with no trigger, use or enemy is always on; a number that applies only
when something happens, a number of times, against a kind of enemy or with another item stays. Calls that were not
obvious:

- **Bonuses limited to one use of a skill, one kind of effect or the wearer's own spells** are always on: Persuasion
  (Bluff) +5 on the Grand Festival Cloak and the Jester Cap, saves against one kind of effect (+4 against
  mind-affecting for the Headband of Confidence and the Wise Negotiator, against fire for Earth's Embrace, against
  poison for the Amulet of Alchemical Knowledge), Headband of Reshaping's +1 caster level for Transmutation and the
  Jester Cap's +2 DC for fused spells. Nothing triggers them. **Against a kind of creature** (Midnight Hunter's Ring vs
  lycanthropes, Ustlavic Pride's DR 5/- vs undead, Dirge for Sarkoris' holy damage vs undead, Belt of the Venerated
  Champion vs evil) stays: it depends on the enemy.
- **Extra uses of an ability or resource a day** (Flamboyant Hat, Headband of Rigorous Training, Amulet of Alchemical
  Knowledge's bombs) stay, as limited in uses.
- **Splits.** Offroad Boots: "ignores difficult terrain" is always on; the +10 feet of speed for one round on crossing
  it is a trigger and stays. Draven's Hat: Slow and Haste spells are quickened is always on, the free quicken once a day
  stays (the page lists them as two lines). Razmir's Mask: immunity to Divination magic and Spell Turning are two
  always-on lines; casting a spell each round as a free action is an activated use and stays. Gorgon Belt, Heart of Ice
  and Cat's Boots were not split: one of their lines moved whole and the others stayed ("ignores difficult terrain",
  "AC +4 armour", "moving provokes no attacks of opportunity").
- **Goggles of Ferocious Pact:** "-2 penalty to AC" and "+4 damage" are in the row's one sentence with the score
  changes, so always on. **Display of Power:** the aura's -2 to enemies' Will saves has no trigger in the row, so always
  on. **Mangling Frenzy:** the damage reduction moved, the critical hit while raging stays.
- **Belt of the Venerated Champion:** the page's "+2 to Fortitude saves and +1 insight to AC against evil creatures"
  is ambiguous about which part is against evil creatures, so it is treated as conditional and its one line stays in
  the detail (the wording note is kept here, not in the line).
- **Borderline, for the user:** Doubling Annoyance's "summon two wizards when casting Perpetually Annoyed Wizard"
  stays in the detail, as a trigger; the saves against one kind of effect and the spell-scoped bonuses above could be
  read the other way.
- Lines that moved were shortened to the summary's style, with no full stop ("Damage reduction 3/-." became "DR 3/-");
  the lines that stayed are unchanged, but for the two remainders and the Venerated Champion's note.
