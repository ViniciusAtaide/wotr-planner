# The base classes' stepped features are granted once

Type: design
Status: open
Blocked by: none
Model: the user decides; then Sonnet to build (data against the guide, as tickets 177 and 182), Opus to review the data and the archetype table.
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 182](182-still-more-scaling-features.md) left this open as a question for the user ("The user decides whether the sheet should show these steps at all … before any is built"). A QA session through the MCP server (2026-11-13, rerun on `main` at `ce9cc3b`) shows what leaving it open costs: tickets 170, 177 and 182 stepped the archetypes' features, so where an archetype replaces a base feature, `compare` reports the archetype gaining a step at each level and the base class gaining nothing there.

| Archetype | Its stepped replacement | The base feature it replaces, granted once |
| --- | --- | --- |
| Hospitaler (paladin) | Smite Evil (Hospitaler) at 1, 7, 13, 19 | Smite Evil at 1 (the summary: "+1 per 3 levels") |
| Two-Handed Fighter | Strong Grip at 2, 6, 10, 14, 18; Two-Handed Weapon Training at 5, 9, 13, 17 | Bravery at 2, Armor Training at 3, Weapon Training at 5 |
| Invulnerable Rager (barbarian) | Extreme Endurance at 3, 6, 9, 12, 15, 18 | Danger Sense at 3 |

The rogue's Danger Sense is the same (at 3 only; its summary: "+1 every 3 levels (max +6)"). The base features already stepped (the rogue's Sneak Attack, the ranger's Favored Enemy, the barbarian's Damage Reduction, the slayer's Studied Target) line up with their replacements, so the sheet is inconsistent between features of the same shape.

182's Answer lists the base features found and left: Danger Sense (barbarian and rogue), the cavalier's Banner, the druid's and the warpriest's Wild Shape (Wolf) uses, Channel Energy, the paladin's and the ranger's stepped features, the fighter's Bravery, Weapon Training and Armor Training, and the archetypes' copies of them (Wild Shape (Wolf) in four druid archetypes and the Feral Champion). Its notes stand: the Dragonheir Scion's table has no Bravery, the Defender of the Hearth's Weapon Training at 9 is its delayed first grant, and Advanced Weapon Training reads Weapon Training with `count: 1`.

Options:
- **A.** Step the base features, each at the levels its guide sentence gives, with 182's rules: every archetype that loses, replaces or delays one gets its own rows (`assert_gains` builds every archetype of a class), and what reads each count is checked first. The sheet and `compare` then agree feature for feature.
- **B.** Step none, and unstep the archetypes' replacements of base features, so both sides show one grant. Undoes part of 170, 177 and 182.
- **C.** Leave it. The `wotr-build` skill then says that `compare` reports an archetype's stepped replacement as a gain at each step even where the base class gains the same.

**Recommendation: A.** The archetype side is built and reviewed; B throws that away, and C leaves `compare` misleading for every archetype that replaces a stepped base feature.

## Done when

The user has chosen. For A: each base feature stepped as its guide sentence says or struck with the reason, the archetype table in the Answer, the snapshot diff showing only the new grants, and `scripts/check.sh` passing.
