# Initiative and skill totals

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

In the Max DC run (2026-11-09, see [119](119-caster-line.md)) two picks depended on numbers the sheet doesn't give:
- **Initiative.** Ambuscading Spells gives flat-footed enemies −2 on their saves, so acting first matters. The run took Improved Initiative, Improved Initiative (Mythic) and the Pickpocket background (+2) for it, but couldn't state the total.
- **Skill totals.** Arcane Spirit is a Persuasion (Bluff) check against the target's Perception. `sheet` gives ranks, not the total (ranks + ability modifier + class skill + racial and background bonuses + Skill Focus).

## Options

- **A. Nothing:** the agent adds them up.
- **B. Totals in `sheet`:** an `initiative` line (Dex + feats + background + mythic feat at MR) and a `total` beside each skill's ranks, each with its sources. Unconditional only (no gear, no buffs), as for 114 and 119.

## Open questions (with recommendations)

1. **Which option?** *Recommend:* B. The sources are a handful of items whose summaries already state a flat number; the sheet already holds the ranks, the scores and the class skills.
2. **Where do the bonuses live?** *Recommend:* the same per-item bonus data as 119's question 2 (a flat `bonus` with a target: initiative, a skill, a save), so both tickets share one shape.
3. **Saves too?** Iron Will, Mythic Resolve and the halfling's +1 change Will, and `sheet` gives saves from class progressions only. *Recommend:* yes, in the same pass.

## Done when

`sheet` gives initiative, each skill's total and saves with their flat feat and race bonuses, each broken down by source, with real-data tests; `scripts/check.sh` passes.
