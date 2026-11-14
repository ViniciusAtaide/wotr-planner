# `search` says the guide's address once

Type: feature
Status: open
Blocked by: none
Model: Sonnet.
Map: [WotR Build Planner](../map.md)

## Problem

From the second ranking of the MCP server's token costs (2026-11-14, measured on the live server). Every `search` match carries `guide_url`, and every one starts with the same 85 characters, `https://gamefaqs.gamespot.com/ps4/324475-pathfinder-wrath-of-the-righteous/faqs/80843/`, before the page and anchor (`feats#power-attack`). `search` for "power" returns 10 matches in 3,912 characters, 850 of them the prefix (22%); 50 feats are 17,822, 4,250 of them the prefix (24%). Agents are told never to fetch these links; the user reads them, so the page and anchor are what vary.

## Build

- A match says `guide: "feats#power-attack"` (page and anchor); the result says the base once (`guide_base`). The research checks the exact names, whether any page address differs in its prefix (the base must hold for every kind: classes, archetypes, paths, feats, abilities, options, races, deities, spells), and where else `guide_url` appears (`describe`, `options`, gear): the same change wherever it is repeated, listed in the report.
- The tool description and the skill say: to give the user a link, join `guide_base` and the match's `guide`. A test pins the join for each kind of match.
- `compact` is unchanged (it has no guide link).
- `responses.rs`: the budget of "search, power" (4,300; 3,912 today) drops.

## Measured and expected

"search, power": 3,912 to about 3,070; 50 matches 17,822 to 13,600. Three searches in a session save about 2,500 to 12,000.

## Done when

Every kind's match joins to today's `guide_url` exactly; the budget holds the new size; `scripts/check.sh` passes.
