# Abilities switched on in a fight

Type: design
Status: open
Blocked by: [139](139-attack-line-weapon-features-and-mythic-feats.md)
Map: [WotR Build Planner](../map.md)

## Question

Tickets [114](114-attack-line-and-damage.md) and [119](119-caster-line.md) chose unconditional numbers only. So the attack line and the caster line leave out every ability a character switches on in a fight: rage and stances, Demonic Rage and the Demon's aspects, the Aeon's gaze, the Devil's decrees, Studied Target, and situational bonuses such as flanking and charging. For a barbarian or a Demon, these are most of the damage, so ranking such Builds by the line understates them.

Options:
- **A.** An `active` argument on `sheet`, `compare` and `improve` (MCP): the switched-on abilities the Build holds that the agent wants counted. Each adds its parts labelled "(active)", beside the unassisted numbers, as ticket [122](122-assumed-gear-and-buffs.md)'s `assume` did (removed by [148](148-naked-numbers.md)). The abilities' bonuses are `ItemBonus` rows with a condition. They're never in the Build, its link or a prerequisite.
- **B.** A, plus toggles in the web app's summary.
- **C.** Leave them in `leaves_out`.

*Recommend:* A. The MCP server comes first, where the agent runs showed the need. It's blocked by 139, which gives the attack line the per-item bonus shape this builds on.

## Done when

A barbarian's line with rage active shows rage's parts labelled "(active)", and the unassisted line is unchanged. A Demon's line does the same with Demonic Rage. `compare`'s `by` ranks with `active` when given. Real-data tests cover these cases. The `leaves_out` lists say "unless active". `scripts/check.sh --schema` passes.

## Answer

**Recommendation taken** (user, 2026-11-10): A, in the MCP server only, after 139.
