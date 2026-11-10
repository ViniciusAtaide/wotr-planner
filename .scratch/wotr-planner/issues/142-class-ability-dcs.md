# DCs of class abilities: hexes, exploits, bombs and spell-like abilities

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

`CASTER_LINE_LEAVES_OUT` names "spell-like abilities, hexes, exploits, bombs and other class abilities with DCs of their own". Builds whose save DCs come from these (a witch's hexes, an arcanist's exploits, an alchemist's bombs, a kineticist's infusions) get no DC from the planner, so `compare` and `improve` can't rank them by it. An agent has to work the DC out from its own knowledge.

Options:
- **A.** An ability line beside the caster line: each class ability the guide gives a DC formula for (for example 10 + half the class level + an ability modifier), its formula as data on the ability, and the item bonuses that target it (Ability Focus and the like) as `ItemBonus` targets. It goes in `sheet`/`compare` (MCP) as `ability`, and `compare`'s `by` and `improve` can rank by it.
- **B.** Leave them in `leaves_out`.

*Recommend:* A, starting with the abilities whose formula the guide states. Any other ability stays named in the line's own `leaves_out`.

First step: list the class abilities with a DC on the saved pages, and the formula each page gives.

## Done when

A witch's hex, an arcanist's exploit and an alchemist's bomb each give a DC with its parts, from data. Real-data tests cover them. The caster line's `leaves_out` names only what's still left out. `scripts/check.sh --schema` passes.

## Answer

**Recommendation taken** (user, 2026-11-10): A, starting with the abilities whose formula the guide states.
