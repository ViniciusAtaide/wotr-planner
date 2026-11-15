# The guide link becomes optional

Type: build
Status: resolved
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

## Answer

**Built** (2026-11-15). `guide` is `Option<GuideRef>` (`#[serde(default)]`) on every entry that had it required: race, subrace, deity, feat, feat list, class, archetype, feature, feature selection, selection option, mythic ability, mythic feat, mythic path (`libs/rules/src/game_data.rs`), spell and spell list (`libs/rules/src/spells.rs`). Gear had it optional already. No data file changes: RON reads the field with implicit `Some`.

- **Readers:** `rules` reads none outside the gear check (already optional). The MCP search index's one `guide` closure now takes `&Option<GuideRef>`, so `guide_url` is absent for every kind (`describe` takes it from the index). The web app's only guide links are the gear item's (`libs/ui/src/gear.rs`, already optional); `ItemInfo::guide` is now `pub(crate)` for its test. The test world's `guide()` helper returns `Some`.
- **Not touched:** the MCP instructions' text and the `wotr-build` skill (ticket 240).

**Decisions:**
- The user, 2026-11-14 (ticket 234): the link becomes optional, no data change.
- Ours: the web app needed no code change; its test pins the behaviour at `item_info` and `plain_guide_link`.

**Tests** (each seen failing first as a compile error, the field being a `GuideRef` then; no game fact changes, so no game citation):
- `libs/seeds/tests/files.rs` `reads_an_entry_without_a_guide_link`: a fixture deity with no `guide` loads and `check()` is empty.
- `libs/rules/tests/engine/check.rs` `an_entry_without_a_guide_link_passes_the_check`: the test world with every feat, class, deity and spell link removed has no check issue.
- `apps/mcp/tests/server/test_world.rs` `an_entry_without_a_guide_link_shows_none`: `describe` of a class has `guide_url` with the link and none without.
- `libs/ui/src/gear.rs` `an_item_without_a_guide_entry_has_no_link`: an item without a guide has no link in its detail or on its row, one with has its URL.

Scoped checks run: `cargo test -p rules`, `-p seeds -p corpus`, `-p mcp`, `-p ui`; `check-data`; the rules wasm build; frontend wasm clippy; `scripts/css-classes.sh`; `cargo fmt --all --check`.

## Done when

- An entry without `guide` loads, passes `GameData::check`, and shows no link in `describe` or the web app, with a test of each.
- The scoped checks of `CLAUDE.md` "Checks" pass.
