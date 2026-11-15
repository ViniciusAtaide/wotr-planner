# Agents read the summaries as notes on the game

Type: build
Status: resolved
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

Decided in [234](234-game-blueprints-as-a-source.md) (user, 2026-11-14), the user taking the recommendation:
- **The honesty boundary is unchanged.**
- **The wording changes.** The MCP instructions and the `wotr-build` skill call summaries and `interactions` "the planner's notes on the guide". Since the game is the source of truth, they are the planner's notes on the game.
- **`guide_url`** stays a link for the user that agents never fetch.
- **`describe` shows no blueprint GUIDs or code citations.** They mean nothing to a player and would cost tokens (tickets 225–233).

## Change

- **The MCP instructions** (`apps/mcp`): "Summaries and `interactions` (`describe`) are the planner's notes on the guide, not rules it enforces" becomes "… the planner's notes on the game, not rules it enforces". The instructions stay under their length test.
- **The `wotr-build` skill** (`.claude/skills/wotr-build/SKILL.md`, served as `wotr://build-guide`) and `martial.md`: the same wording wherever they say a summary or the data comes from the guide. A line saying the guide is the source of truth goes.
- **`wotr://leaves-out` and `wotr://glossary`**, wherever they say "the guide's" of a fact rather than of a link.
- **The glossary's Interactions note** says "What the guide says about how an item combines…". It now says what the game does, with the guide's words where they say more.

## Answer

Built 2026-11-15. No game fact changes, so no data note.

- **Reworded** (the instructions in `apps/mcp/src/server.rs`, `SKILL.md` served as `wotr://build-guide`, the glossary's Interactions note): summaries and `interactions` are "the planner's notes on the game"; spell facts are "as the game's data gives them"; the Interactions note says what the game does, with the guide's words where they say more. `martial.md`, `wotr://leaves-out` and the other glossary entries name the guide only as a link or as where a fact is printed, so they stay. `guide_url` and "never fetch them yourself" are untouched.
- **Decisions:** the user's, in 234 (item 13). Left alone, for the main session: `libs/game-data/interactions.ron` entries still say "the guide doesn't say" (data, served by `describe`), and the glossary's Guide entry.
- **Tests** (`apps/mcp/tests/server/`): `listing::no_text_served_to_agents_calls_the_guide_the_source` (instructions, glossary, build guide and leaves-out name no guide as the source; failed first on "notes on the guide"); `describe::describe_shows_no_blueprint_guids_or_code_citations`; `spells::the_guide_makes_the_spell_catalogue_planner_data` follows the new phrase; the instructions' length test passes.
- **Citations:** none; the change states no game fact.

## Done when

- No text served to agents calls the guide the source of the rules or the summaries. `guide_url` and the links are still described as links for the user.
- The instructions' length test passes.
- `cargo test -p mcp` and `cargo fmt --all --check` pass.
