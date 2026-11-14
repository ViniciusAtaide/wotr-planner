# Gear, second stage: weapons

Type: feature
Status: open
Blocked by: [139](139-attack-line-weapon-features-and-mythic-feats.md) (the attack line's `ItemBonus` targets)
Model: Sonnet to build the data; Opus to build the engine and review.
Map: [WotR Build Planner](../map.md)

## Build

The first of [169](169-gear-second-stage.md)'s build tickets (its Answer, items 1 to 3, 6 and 9); read that Answer first.

- **Data:** the guide's unique weapons, Finnean and Radiance (`items#unique-weapons`, `#finnean-and-radiance`), each with its base weapon, enhancement and properties (`items#weapon-properties`). Generic weapons: any base weapon of `weapons.ron` with an enhancement from +1 to +5, a planner-made item named as such ("+5 Falchion"), not listed in the data one by one. A note in `libs/game-data/notes/221-gear-weapons.md`.
- **The attack line:** the weapon in `gear.weapon` becomes the line's weapon, in place of the Weapon Focus default (a `weapon` argument still overrides both). Its enhancement counts to attack and damage. Its always-on properties count as rows: energy dice (Flaming 1d6) in the damage, Keen in the threat range.
- **Conditional properties** (Bane, Holy, "vs. undead") count in rows that name the condition, shown beside the line with a second figure "with conditions", never summed into the headline. `compare` and `improve` rank by the headline unless asked for `with_conditions: true`.
- **The off-hand place** `gear.weapon.off`: held in the Build and its link, read only by [140](140-attack-line-trades-and-two-weapon-fighting.md)'s off-hand line; before 140 it counts in no number.
- **MCP:** `search kinds: ["gear"]` finds weapons, the generic ones by their base weapon. `options` on `gear.weapon` lists them. The `wotr-build` skill's Gear section and the server instructions stop saying weapons aren't modelled.
- **Web app:** the Gear tab's weapon place lists them, and the attack line shows the conditional figure beside the headline.

## Done when

The guide-only Build of the third martial run (the map's History entry for 2026-11-14) with Defender's Nemesis in `gear.weapon` shows the falchion line with +5 and its Elder Flaming 1d6, and the same Build with "+5 Falchion" shows +5 alone. A Bane weapon's line has a "with conditions" figure that `compare` ignores unless asked. Real-data and MCP tests cover each. `scripts/check.sh` passes.
