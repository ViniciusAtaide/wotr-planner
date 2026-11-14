# Gear, second stage: the other unique items

Type: feature
Status: open
Blocked by: [221](221-gear-weapons.md)
Model: Sonnet to build the data; Opus to review.
Map: [WotR Build Planner](../map.md)

## Build

The second of [169](169-gear-second-stage.md)'s build tickets (its Answer, items 5, 6 and 9).

- **Data:** every unique item the saved pages list that the first stage left out (`items#unique-items` by slot, `#rods`, `#robes`, `#equipable-and-consumable`, and the same sections of `items-inevitable-excess` and `items-lord-of-nothing`), each with its typed bonuses (the guide's types), `uncounted` lines and `other_effects`. Unique armour and shields come with [223](223-armour-class.md). Relics and artifacts are [224](224-relics-and-artifacts.md). A note in `libs/game-data/notes/222-gear-other-items.md`.
- **What counts:** an always-on bonus to a number the planner computes: an extra attack at full bonus (Hasting Expanse), flat damage (Goggles of Ferocious Pact, Fencer's Gift), per-hit dice (Bracers of Slaughter), caster level and DCs (ticket 122's extraction, `git show 4b1f164:libs/game-data/gear.ron`), saves and skills.
- **Conditional bonuses** count in rows that name the condition, beside the line or total and not summed into its headline, as [221](221-gear-weapons.md) set up for weapons: "+1 DC with fire spells" beside the caster line, "+2 vs undead" beside the attack line.
- Until [144](144-bonus-types-and-stacking.md), bonuses of the same type still add; the rows carry their type so 144 can apply it.

## Done when

The guide-only Build of the third martial run, holding Hasting Expanse, Bracers of Slaughter, Fencer's Gift and Goggles of Ferocious Pact, shows five attacks with their damage rows. A caster's item adds its DC to the caster line, and a conditional DC item shows beside it. `search kinds: ["gear"]` finds each new item. Real-data tests cover the bonus kinds. `cargo run -p seeds --bin check-data` and `scripts/check.sh` pass.
