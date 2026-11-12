# Gear data: the items that give an ability score bonus

Type: task
Status: open
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
