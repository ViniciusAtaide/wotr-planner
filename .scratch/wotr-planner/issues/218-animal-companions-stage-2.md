# Animal companions, stage 2: the companion's level-up

Type: feature
Status: open
Blocked by: [217](217-animal-companions-stage-1.md)
Model: Sonnet to build; Opus to review.
Map: [WotR Build Planner](../map.md)

## Build

The second stage of [174](174-animal-companions.md)'s answer (its items 1, 7 and 9).

- **The companion's picks**, as the guide gives them (`character-creation#companion-feats`, `#companion-skills`, `#companion-levelling`): a feat at each odd companion level, the archetype's bonus feats, 2 skill points a level (class skills Athletics, Mobility, Stealth, Perception; others allowed), and its ability increases. Prerequisites are checked against the companion, not the character.
- **Addresses:** `companion.level.N.*`, N the companion level, through every tool. The link holds them.
- **Web app:** the companion's level-up in its own panel, opened from the granting feature's chip, one row per companion level.
- **Gold builds:** the write-ups' companion tables (`gold/source.txt`) are compared, level by level.
- **Numbers:** the companion's feats count in its attack line as the character's do (Weapon Focus, Power Attack).

## Done when

Jack-in-the-Green's companion table matches the gold write-up, or its differences are known gaps with tickets. A companion feat whose prerequisite the companion lacks is refused with a reason. The panel shows the companion's own rows at its own levels. Real-data and MCP tests cover each. `scripts/check.sh` passes.
