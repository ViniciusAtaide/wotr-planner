# Assumed gear in the web app

Type: design
Status: closed (won't do)
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Ticket [122](122-assumed-gear-and-buffs.md) gave `sheet` and `compare` (MCP) an `assume` argument: enhancement and inherent bonuses per ability, a weapon's enhancement, a resistance bonus, and the guide's caster level and spell DC items. Their numbers show beside the unassisted ones. Its answer 4 was "not the web app", for now. So a player on the site sees only unassisted numbers, which at level 20 sit far below what they'll play (ticket 122's DC run went from 34 to 48).

Gear stays out of the Build, its link and every prerequisite (map, Out of scope).

Options:
- **A.** An "Assume gear" panel under the summary: the same generic bonuses and named items, and the attack line, caster line and totals shown with them, each assumed part labelled. It isn't in the link. It resets on reload, or is remembered per browser in `localStorage`.
- **B.** A, with the assumptions in the link fragment as a separate, optional part that the Build's decoder ignores.
- **C.** Keep it MCP-only.

*Recommend:* the user's call. If they want it, A with `localStorage`, since the map keeps gear out of the link. The panel must work on both Fold screens with thumb-sized controls.

## Done when

The user has chosen. For A or B: the web app's summary shows the assumed numbers beside the unassisted ones, each assumed part labelled, through `Engine::assumed_attack_line`, `assumed_caster_line` and `assumed_totals`. An end-to-end smoke check covers it on both Fold viewports. `scripts/check.sh --full` passes.

## Answer

**Closed, not built** (user, 2026-11-10): "gear should be assumed as base weapon and nothing else. I don't want to build the gear functionality yet so just assumed naked with the corresponding weapon focus as the weapon." Assuming gear is the agent's (skill's) concern, not the planner's. Ticket [148](148-naked-numbers.md) removes ticket 122's `assume` from the MCP server too.
