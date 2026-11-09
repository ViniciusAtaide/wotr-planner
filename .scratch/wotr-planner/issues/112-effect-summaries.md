# Short effect summaries for every pickable item

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

In an agent run through the MCP server (2026-11-08), the user asked for "the strongest martial build possible, maximum attack damage". The agent built the Demon Greatsword: Human, Two-Handed Fighter 5 / Barbarian 15, Demon to MR 10. Every pick was legal, but the agent chose these from memory, because the data says nothing about what they do:
- the 12 Demon aspects;
- five mythic abilities (Brutality Incarnate, Thundering Blows, …);
- the seven rage powers.

It said so, and it said it wasn't sure.

Every kind already has a `summary`, in `rules::GameData` and as a `NOT NULL` column in the migration: races, subraces, feats, classes, archetypes, features, Feature selections, selection options and mythic abilities. Only one is filled (`classes/wizard.ron`).
- `describe` returns a summary when it isn't empty (`apps/mcp/src/describe.rs`).
- The web app's picker already shows each option's summary (`SlotEntry::detail` in `libs/rules/src/engine.rs`).

So this ticket is data only. It improves the web app's pickers and the MCP server alike.

The map's Notes allow this: "a short summary in our own words and a link to the guide, never the guide's full text". The saved pages in `guide-cache/` hold what's needed; nothing is fetched.

## Scope

1. **No schema change:** the fields and columns exist.
2. **Data, by category, one stage each** (like the hand-fix stages 17–20; each stage validates without the database, then the main session seeds once):
   - mythic: path features, path selection options (aspects, Lich powers, gazes, …), mythic abilities, mythic feats;
   - feats;
   - class features and class selection options (rage powers, talents, exploits, hexes, …).
3. **MCP:** `describe` returns the summary for selection options and features too (today it does so only for classes, archetypes, paths, feats and mythic picks). `options` and `search` add it to each line, so an agent can choose without one `describe` call per candidate.
4. **Real-data test** in `libs/seeds/tests/game_data/`: the count of empty summaries per kind, and that it only goes down.

## Open questions (with recommendations)

1. **How long?** *Recommend:* one sentence, about 160 characters at most, with the numbers in it ("+2 to attack and damage while raging with a two-handed weapon"). Leave out how the guide rates an item.
2. **Which stage first?** *Recommend:* mythic. All of the run's guesses were mythic picks or rage powers. Then class options, then feats.
3. **When the guide and the game disagree?** *Recommend:* the guide wins, as the map's Notes say, and the summary notes the difference only if the guide does.

## Done when

Every item in the finished stages has a summary in our own words. The real-data test passes. `options` lines and `describe` show the summary, and so does the web app's picker. `scripts/check.sh` passes.

## Progress log

- 2026-11-08 (MCP part, scope item 3): `search` matches carry `summary` when it isn't empty (a background's: its note). `describe` gives it for the item itself (moved from `Item` into `Hit`, so its JSON is unchanged) and for each feature and Feature selection option it lists (`Summarised {key, name, summary?}`, features from a lookup the `Planner` builds over class, archetype and path features), and for each Feature selection it lists. `options` lines carry it too: a slot's from the engine's `SlotEntry::summary`, a class's or archetype's, a race's or subrace's, a background's note and a path's from the data. Tested on the test world given summaries inside the test (`tests/server/summaries.rs`, 3 tests; `rules` unchanged); a smoke test on real data shows the one summary filled so far (the Thassilonian Specialist's school selection) in `describe`. `SQLX_OFFLINE=true scripts/check.sh`: ALL-OK. 86 server tests pass. The data stages are left to their own work.
