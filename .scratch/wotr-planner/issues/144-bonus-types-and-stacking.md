# Bonus types and stacking

Type: design
Status: open
Blocked by: [222](222-gear-other-items.md) (the bonus rows it adds), and [139](139-attack-line-weapon-features-and-mythic-feats.md)
Map: [WotR Build Planner](../map.md)

## Question

The planner adds every bonus it counts: item bonuses (ticket [119](119-caster-line.md), [120](120-initiative-and-skill-totals.md)) and assumed gear (ticket [122](122-assumed-gear-and-buffs.md)). Ticket 120 added no stacking column, because the guide gives no bonus types. In the game, two bonuses of the same type (two enhancement or two competence bonuses to one number) don't stack: only the highest counts. So a Build or an assumption with two same-typed bonuses reads too high, and nothing says so.

Options:
- **A.** The user gives each bonus's type, as they gave the skills table (`research/skills-from-user.md`). A `bonus_type` column on item bonuses and gear (a new migration), and a stacking rule in `rules`: untyped and dodge bonuses add, while the rest take the highest per type and target.
- **B.** Say it: each line's and the totals' `leaves_out` gains "bonus types: every bonus is added, whatever its type", and the MCP honesty boundary repeats it.
- **C.** Leave it as it is.

*Recommend:* B now, as it's one line and makes the numbers honest. A if the user wants to supply the types.

## Done when

For B: every line's and the totals' `leaves_out` says it, and the MCP instructions do too. For A: two same-typed bonuses on one target count once, with a real-data test, and `scripts/check.sh --schema` passes.

## Answer

**Decided** (user, 2026-11-10): A. The user gives each bonus's type, as they gave the skills table, and it goes in `research/`. Since [148](148-naked-numbers.md) removes assumed gear, the types cover the item bonuses (`bonuses.ron`) and whatever 139, 141 and 143 add.

**Revised** (user, 2026-11-14, in ticket [169](169-gear-second-stage.md)'s grilling session, its question 7): the user no longer gives a list. The types come from the guide's own words: gear already carries them (Enhancement, Inherent, Competence, Circumstance, Armour, Profane, Alchemical, Luck, Resistance). Feats and features take a type where the guide names one, and are untyped (always stacking) where it doesn't. A later correction is a data edit. This ticket is now the build of stacking, third in 169's order: after the weapons and the other items, before armour class.
