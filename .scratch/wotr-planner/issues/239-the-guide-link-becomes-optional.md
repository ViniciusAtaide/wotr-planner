# The guide link becomes optional

Type: build
Status: open
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

Decided in [234](234-game-blueprints-as-a-source.md) (user, 2026-11-14), the user taking the recommendation:
- **Summaries.** Written from whichever source says more, never contradicting the game.
- **The guide link.** `guide` stays wherever a guide section exists, and becomes optional, so the data can hold what the guide lacks.
- **A fact the guide gets wrong** keeps its link, and the data's note records the difference.

Today `guide: GuideRef` is required on feats, classes, features, spells and most other entries (`libs/rules/src/game_data.rs`, `spells.rs`; `gear.rs` has it optional already). So an entry the game has and the guide doesn't can't be added.

## Change

- **The field.** `guide` becomes `Option<GuideRef>` on every entry that has it.
- **Its readers** say nothing when it's absent: the MCP server's `guide_url` (`describe`, `search`), the web app's guide link, and anything in `rules` that reads it.
- **Agents.** The MCP instructions' "Guide links (`guide_url`) are for the user to read" still holds for the links that exist.
- **No data change:** every entry keeps its link.

## Done when

- An entry without `guide` loads, passes `GameData::check`, and shows no link in `describe` or the web app, with a test of each.
- `scripts/check.sh` passes.
