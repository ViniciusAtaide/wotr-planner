# The human's Skilled: a skill point every level

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The second "strongest martial build" run (2026-11-09, see [131](131-gaps-from-the-second-martial-run.md)) showed 2 skill points a level for a Human Two-Handed Fighter with Int 10. The guide gives the human (`character-creation#human`) and the Human-Raised half-orc (`#half-orc-human-raised`) the "Skilled" trait: +1 skill point every level. Both summaries already said so, but nothing counted it: `skill_ranks` in `libs/rules/src/engine/walk/level.rs` added only the class's points and the Int modifier. [111](111-mcp-end-to-end.md)'s "a human arcanist with Int 20 gets 7 skill points" had recorded the bug as a fact.

## Choices settled

1. **Data, not a race key in the rules:** `Race::skill_points` and `Subrace::skill_points` (`#[serde(default)]`, 0 when none is written). A subrace's replace its race's, as its ability modifiers and feat slots do (`Catalog::racial_skill_points`, beside `racial_modifiers`).
2. **On top of the minimum:** the class's points plus Int stay at least 1, then the race's points are added, so Skilled always adds its point.
3. **Schema:** a new migration, `20261109005859_130-race-skill-points.sql`, adds `skill_points` to `race` and `subrace` (the user's 2026-11-09 rule: a schema change adds a migration). The seed writes it and the loader reads it.
4. **The dev database wasn't rebuilt:** another session ("Strongest DC caster build WOTR") was busy in this checkout, so the checks ran with `SQLX_OFFLINE=true` against the regenerated `.sqlx` data. The main session rebuilds `wotr-planner.db` (CLAUDE.md's four steps) when it's alone; until then, builds that read `DATABASE_URL` need `SQLX_OFFLINE=true`.

## Progress log

- 2026-11-09: test first (`tdd`): `derived::a_race_or_its_subrace_adds_skill_points_every_level` failed with 3 for 4. Then the field, the test world (human 1; the invented `half-elf/adaptable` subrace 1, so the subrace rule is tested), the walk, the seed, the loader and the data (`races.ron`: human and Human-Raised, 1).
  - Two tests assumed a human fighter's old 3 points: `level_pickers::no_skill_steps_up_once_the_levels_points_are_spent` now uses a dwarf (still 3), and `apps/mcp`'s sheet test expects 4.
  - Real-data tests in `libs/seeds/tests/game_data/races.rs`: only the human and the Human-Raised half-orc are skilled (1 each), and a human or Human-Raised fighter has one point more than a half-orc or a Tribal one.
  - `SQLX_OFFLINE=true scripts/check.sh --schema`: ALL-OK.
- Behaviour proof, `scripts/proof.sh both` (2,162 Builds): every one of the 103 Builds whose race isn't skilled is identical on both sides, and every difference is a human Build:
  - 1,125 human Builds that spend skill ranks were impossible on HEAD and are possible now. The Builds come from the working tree's export, which spends the new points, so HEAD finds them overspent.
  - 933 others differ in their sheet: the extra available point. In a few, cascades of the same point: Intelligence becomes removable (lowering it no longer overspends), and the class and edit fingerprints change with it.
  - Nothing else differs.

## Answer

Done 2026-11-09. A race or subrace adds its `skill_points` at every level; the human and the Human-Raised half-orc add 1. The Intelligence part of the formula is a separate question, in [131](131-gaps-from-the-second-martial-run.md).
