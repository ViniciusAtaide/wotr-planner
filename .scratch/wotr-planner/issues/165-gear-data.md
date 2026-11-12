# Gear data: the items that give an ability score bonus

Type: task
Status: done
Blocked by: 158, 159
Model: Sonnet to extract and build (volume work into a fixed shape); Opus for the 15-item sample check. The item's shape is designed first, on Fable, and written into this ticket.
Map: [WotR Build Planner](../map.md)

## Question

The first of the gear tickets from the equipment plan ([163](163-equipment.md), user, 2026-11-11). It extracts the first stage's items into the game data. The engine ([166](166-gear-in-the-engine.md)) reads them.

Decided in 163:

- **Named items from the guide,** one catalogue, each in a slot.
- **First stage:** every item on the three saved pages (`guide-cache/items.html`, `items-inevitable-excess.html`, `items-lord-of-nothing.html`) that gives a bonus to an ability score, about 77 items, with all of its always-on bonuses to numbers the planner already computes. Items with no such bonus wait for the second stage ([169](169-gear-second-stage.md)).
- **Typed bonuses:** each bonus carries its type as the guide words it (enhancement, inherent, morale, untyped and so on).
- **Two texts per item, no descriptions:**
  - a summary of the modelled bonuses, generated from the data ("Dex +6 enhancement"), shown under the name as a race's is;
  - its other effects, one short line each in our own words ("On a critical hit: the target is staggered for 1 round"), shown in the item's detail. The guide's text is never copied (map, Out of scope).
- **A link to the item's guide section,** as other items have.
- **The generic stat items** (user, 2026-11-11): the six plain belts and headbands at +2, +4 and +6, eighteen items the guide doesn't list, added by hand with no guide link. The list is in [research/gear-item-shape.md](../research/gear-item-shape.md).
- **A robe and an armour share one body place; caster level is counted in the first stage** (user, 2026-11-11). Both are in the same file.

## Build

AFK. Game data, its reader and the seed; a schema change (a new migration, `CLAUDE.md`'s "Database").

**The shape is designed: [research/gear-item-shape.md](../research/gear-item-shape.md).** Build to it: the types, the RON format, the summary rule, the tables, the checks and the calls on what becomes text. Where it and this ticket disagree, it holds. In particular: the types are serde-only like the rest of `game_data.rs`, not serde-free; the count is about 67 distinct items, not 77; an unknown slot, target or type is refused by the RON parse, not the data check; the link-hash test needs no edit. Its "Open, for the user" list is answered in this ticket's Question before the build starts.

**Order:** after [159](159-snapshot-check.md) and before [166](166-gear-in-the-engine.md), never beside either. It shares `libs/seeds/tests/game_data/main.rs` and `libs/game-data/README.md` with 159, and `libs/rules/src/game_data.rs` and the codec with 166 (`CLAUDE.md`, "Parallel agents": tickets whose diffs touch the same files run one after the other).

- **The shape,** in `libs/rules/src/game_data.rs`, serde-free like the rest, agreed with 166 before either builds:
  - key, name, slot, guide reference;
  - bonuses: target, amount, bonus type. First-stage targets are an ability score and the targets `BonusTarget` already has (saves, skills, initiative, spell DC, spell penetration, concentration). An attack or damage bonus waits for the second stage;
  - other effects: a list of short lines.
  
  Ticket 122's removed shape is a starting point, not a base to restore: `git show 4b1f164:libs/rules/src/gear.rs` and `git show 4b1f164:libs/game-data/gear.ron` (32 caster-level and DC items, with circumstances this stage doesn't model).
- **Slots:** the guide's sections: belt, boots, bracers, cloak, glasses, gloves, headwear, necklace, ring, robe, plus armour, shield and weapon where a first-stage item sits in one. List the slots found in the Answer. An item in the "Equipable and Consumable" section or among the artifacts needs a slot decided from its text; list each such call.
- **The file:** `libs/game-data/gear.ron`, documented in `libs/game-data/README.md` with the other files.
- **Extraction:** from the saved pages only, never the live site.
  - Find every item whose properties give a bonus to an ability score ("+N <type> bonus to <Ability>", and the lists such as "Strength, Dexterity and Constitution").
  - Turn each always-on bonus into a typed bonus. A bonus that holds only in some circumstance (against a kind of enemy, while raging, a few times a day) is an other effect, in words.
  - Write the other effects by hand, short, in our own words, and check each against the page.
  - Record in `libs/game-data/EXTRACTION_REPORT.md` what was read, the counts per page and slot, and every judgement call.
- **The summary** is a function of the bonuses, written once in `rules` so the web app and the MCP show the same text. Group a bonus shared by several scores ("Str, Dex, Con +6 enhancement").
- **The link:** items are a new key kind for the link's key hashes. Add the `KeyKind` and extend `seeds`' `no_two_keys_of_one_kind_share_a_link_hash` to it, so 166 can encode an item.
- **The seed and the checked data:** tables for items, bonuses and other effects in a new migration; the data check (`Engine::new`'s checked type, ticket 149) refuses an unknown slot, target or type, a duplicate key and an item with no bonus.
- **Real-data tests** in `libs/seeds/tests/game_data/`: every item has at least one ability score bonus; every bonus type is one of the listed set; three items picked by hand match their page line (one single-score belt, one three-score belt, one with other effects); every summary is non-empty.

## Done when

- `libs/game-data/gear.ron` holds every first-stage item from the three pages, and the Answer has the count per page and slot against an independent count of the pages' matching lines.
- A second agent has checked a sample of 15 items against the saved pages: bonuses, types, slot, and that no other-effects line copies the guide's wording.
- No engine behaviour changes: the snapshot check ([159](159-snapshot-check.md)) is unchanged if it has landed; otherwise `scripts/proof.sh data` gives an empty comparison.
- `scripts/check.sh --schema` passes.

## Answer

**Built**, 2026-11-11 on branch `ticket-165`, from `main` at `37fb813`, to [research/gear-item-shape.md](../research/gear-item-shape.md). The second agent's check of 15 items (Done when, second item) is the main session's and is not done here. Documented in `libs/game-data/README.md` (`gear.ron`) and `libs/game-data/EXTRACTION_REPORT.md` (the counts and the judgement calls).

### What was built

- **Types** (`libs/rules/src/gear.rs`, `bonus.rs`, `keys.rs`): `GearItem` (`guide` is an `Option`), `GearSlot` (13), `GearBonus`, `GearTarget` (with `CasterLevel`, after `SpellDc`), `BonusType` (19), `GearKey`, `GearItem::summary(&[Skill])` and `GearTarget::bonus_target()`. `GameData::gear`, its `counts()` entry ("Gear") and its canonical sort. No `BonusItem`, Build, walk or Build-format change; the only codec change is `KeyKind::Gear` and its loop in `LinkKeys::new` (the existing collision test covers it).
- **The check** (`DataAt::Gear`): a duplicate key or name, no bonus, an amount of 0 or a negative one that isn't untyped, two bonuses with one target and type, an unknown skill, `SpellDc(Value)`, an empty or repeated other-effect line.
- **The reader:** `gear.ron` is optional (`SINGLE_FILES`).
- **The migration** `20261111180000_165-gear.sql`: `gear_item` (nullable `guide`, unique name), `gear_bonus` (target and column checks, `caster_level` included), `gear_other_effect`; `libs/seeds/src/gear.rs` writes and reads them; the `.sqlx` metadata is prepared.
- **The data:** `libs/game-data/gear.ron`, 92 items.

### The slots found

`Belt`, `Boots`, `Bracers`, `Cloak`, `Glasses`, `Gloves`, `Headwear`, `Necklace`, `Ring`, `Robe`, `Usable` (the consumables of the guide's "Equipable and Consumable": 9), `Armour` (4: Half-Plate of Vigor, Life Infuser, Snakeskin, Ustlavic Pride) and `Weapon` (2: Sin Mage's Staff, Dirge for Sarkoris). No shield and no rod. Calls on the "Equipable and Consumable" section: the six manuals and tomes are `Usable` (the user: tomes are gear), and so are the Tome of Ancient Perfection, the Elixir of Inconceivable Transmutations of Body, and Soul, and Also Mind and the Potion of Humanity (added after review: their "all ability scores" and "all attributes" wordings slipped past the first recount, six Inherent bonuses each); Trusted Friend is a `Necklace`, since the guide's Skill Boosting Items table lists it as an "Amulet", and keeps its `items#equipable-and-consumable` reference.

### Counts against an independent recount

A script over the saved pages' table rows, not the research file's list, found 73 rows with an always-on ability score bonus (the pattern now also catches "all ability scores" and "all attributes"): `items` 61 (56 in the item sections, 4 armour, 1 weapon), `items-inevitable-excess` 4, `items-lord-of-nothing` 8. Three repeat a main-page row (Flawless Headband of Mental Perfection +8, Hammer Fists, Soaring Fist), so 70 distinct items: the research file's 67 and the three consumables its score-name pattern missed. A last pass with "all", "every", "highest", "attribute" and the abbreviations found nothing else worn or used. Per slot (distinct): Belt 17, Boots 2, Bracers 1, Cloak 1, Glasses 1, Gloves 3, Headwear 21, Necklace 3, Ring 3, Robe 3, Usable 9, Armour 4, Weapon 2. The tables per page and slot are in the extraction report; a real-data test pins the per-slot counts.

### The generic items and the manuals

- **From the guide:** the six manuals and tomes (`Usable`, one Inherent +2 each, no other effect) and the three all-score consumables, Belt of Physical Perfection +6 (`items#belts`) and Headband of Mental Perfection +6 (`items#headwear`).
- **Added by hand, no guide reference, 22 items, for the user to confirm** (names and sizes rest on the main session's reading of the request): Belt of Incredible Dexterity, Belt of Giant Strength, Belt of the Mighty Constitution, Headband of Alluring Charisma, Headband of Vast Intelligence and Headband of Inspired Wisdom, each at +2, +4 and +6 (18, Enhancement to the one score); and Belt of Physical Perfection (Str, Dex, Con) and Headband of Mental Perfection (Int, Wis, Cha), each at +2 and +4 (4, Enhancement to the three scores). No two-score variants.

### Judgement calls

- **Left out for the second stage:** Vellexia's Magnifying Amulet (`items#necklaces`), whose "+2 bonus to highest current ability score" the shape cannot hold.
- **Left out though a score is named:** Glasses of Undeniable Truth, Bestial Rags, Robe of False Death, Mutilated Angel (the research file's four) and Nimble Edge (a few rounds after a critical hit). The rest name a score only as damage, a modifier or a cap, or a penalty on a foe (listed in the report); the Conflagrant Taco, a meal in a recipe table with "+4 bonus to all ability scores", has no item slot and is out.
- **Relics and artifacts** (prose with upgrade tiers, no table rows) are not in the file: Face-Stealer's Mask, the Stone of Ghostly Pathways, Zaoris's Pin and others give score bonuses. The design counted table rows. Worth a decision for the second stage.
- **Belt of the Venerated Champion:** its "+2 bonus to Fortitude saves and a +1 insight bonus to AC against evil creatures" is ambiguous, so as the research file's table says it is one other-effect line; only Con +6 is a bonus.
- **Armour and weapon types:** each of the four armours and two weapons has a line naming its enhancement and base type from the table's sub-heading (Snakeskin a +4 leather armour, Life Infuser a +5 breastplate, Half-Plate of Vigor a +1 half-plate, Ustlavic Pride a +3 full plate, Sin Mage's Staff a +5 quarterstaff, Dirge for Sarkoris a +2 keen club: its row sits under the "Club" sub-heading on `items-lord-of-nothing#weapons`).
- **Caster level:** only Sin Mage's Staff has an always-on unscoped bonus (+3), as `CasterLevel`; Headband of Reshaping's +1 for Transmutation stays a line.
- **Untyped:** Half-Plate of Vigor, Dirge for Sarkoris, Sin Mage's Staff (Int, spell DC, caster level), the +2 Fortitude of the Belt of the Venerated Champion and the +3 Will of the Goggles of Ferocious Pact; Life Infuser is `Armour`, as written.
- **Other effects** are my own wording, at most 131 characters (a test holds 140); none shares a run of five words with its row but game terms, numbers and item names.

### Done when

- `gear.ron` holds all 70 first-stage items from the pages plus the 22 hand-added ones (counts above): 92.
- No engine behaviour changes: `cargo test -p seeds --test game_data snapshots` passes with the snapshot files untouched.
- `scripts/check.sh --schema` passes.
- Left to the main session: the second agent's sample of 15 items.
