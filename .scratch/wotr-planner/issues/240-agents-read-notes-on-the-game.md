# Agents read the summaries as notes on the game

Type: build
Status: open
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

## Done when

- No text served to agents calls the guide the source of the rules or the summaries. `guide_url` and the links are still described as links for the user.
- The instructions' length test passes.
- `scripts/check.sh` passes.
