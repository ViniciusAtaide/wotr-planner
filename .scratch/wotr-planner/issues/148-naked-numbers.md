# Naked numbers: no assumed gear, the weapon from Weapon Focus

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The user, 2026-11-10, closing [145](145-assumed-gear-in-the-web-app.md): "gear should be assumed as base weapon and nothing else. I don't want to build the gear functionality yet so just assumed naked with the corresponding weapon focus as the weapon." Asked what happens to ticket [122](122-assumed-gear-and-buffs.md)'s `assume`, the user said: remove it from the server. This reverses 122 and the map's "assumed bonuses on the computed numbers are in scope".

Three parts:

1. **Remove `assume` from the MCP server:** `sheet`'s and `compare`'s `assume` argument and their `assumed` output, `Engine::assumed_attack_line`, `assumed_caster_line` and `assumed_totals`, and `rules::Assumed`. Also remove `libs/game-data/gear.ron` and `rules::GearItem`, with a new migration that drops `gear_item` and `gear_bonus` (never edit `20261109091222_122-assumed-gear.sql`). The `leaves_out` lists drop "unless assumed", and the honesty boundary drops its assumed-numbers paragraph.
2. **The MCP attack line's weapon defaults to the Build's Weapon Focus**, as the web app's does (`Engine::default_attack_weapon`: the Build's first Weapon Focus). `sheet` and `compare` without `weapon` give that weapon's line, and none when the Build has no Weapon Focus. `compare`'s `by` with an `attack_line` but no `weapon` uses each Build's own default. The output names the weapon it used.
3. **The wotr-build skill compares naked numbers:** drop the standard `assume` from `.claude/skills/wotr-build/martial.md` (its line 19 and step 1) and from `SKILL.md` (step 6, the assumed-numbers bullet and the report's "with the `assume` used"). The skill says the planner's numbers assume no gear and the Weapon Focus weapon. Gear stays the agent's own judgement, labelled as such, if the user asks for it.

`apps/mcp` and `SKILL.md` had uncommitted changes from another session when this was written, so start once that work has landed.

## Done when

No tool takes `assume` or returns `assumed`. `gear.ron` and its tables are gone, through a new migration. `sheet` for a Build with Weapon Focus (greatsword) and no `weapon` gives the greatsword's attack line, and with no Weapon Focus gives none. The skill files hold no `assume`. The map's Out of scope says gear assumptions are out (user, 2026-11-10). `scripts/check.sh --schema` passes, and `scripts/build-mcp.sh` rebuilds the server the sessions use.
