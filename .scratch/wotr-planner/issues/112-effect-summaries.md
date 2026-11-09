# Short effect summaries for every pickable item

Type: task
Status: resolved
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

## Choices settled

The user had every ticket of 112–118 built with its recommended options (2026-11-08), so: one sentence of at most 160 characters with the numbers in it; the guide wins. Settled while building:

1. **Five data stages in parallel**, each in its own worktree on its own files: mythic; feats, races and prestige classes; and the base classes in three groups. Each writer worked only from the saved pages (`guide-cache/`), cut at the item's `guide` anchor.
2. **Every summary was reviewed in full** (the user, 2026-11-08, after a sampled first round found 1 or 2 wrong facts in 40):
   - a check script (the scratchpad's `check_summaries.py`) compared each summary with its anchor's section: anchor found, every number in the section, no 8-word run of the guide's text, at least half its 4-word runs not the guide's, 160 characters;
   - then independent reviewers, given the summary and the section but not the writer's reasoning, read every summary for accuracy (numbers, levels, uses per day, conditions, who benefits) and own words;
   - the writers applied the findings and the script ran again.
   About 52 factual errors and 44 close copies were found in about 3,380 summaries and fixed. Every `numbers` flag that was checked turned out right: the guide states those numbers another way (a level table, "1-4", "every N levels").
3. **The person:** a summary addresses the player as "you"; an archetype's or class's description may be in the third person ("their").
4. **Another page or a table as the source**, where the guide gives a feature only in a level table or says it is another class's (the skald's Bardic Knowledge, the bloodrager's Uncanny Dodge).
5. **Where the guide disagrees with itself**, the summary says so ("18th (the guide's table; its text says 19th)"); where it looks wrong but says nothing (the warpriest Fire blessing's acid damage), the summary follows it, with no hedge.
6. **Left without a summary:** 133 items the guide has nothing usable on (cantrips and orisons, features named only in a level table, guide links without an anchor). Feat lists have no `summary` field and get none.
7. **The honesty boundary:** the MCP instructions no longer say the data has "almost no descriptions"; an agent may cite a summary as the planner's summary of the guide, not as a rule the engine enforces.

## Progress log

- 2026-11-08 (MCP part, scope item 3): `search` matches carry `summary` when it isn't empty (a background's: its note). `describe` gives it for the item itself (moved from `Item` into `Hit`, so its JSON is unchanged) and for each feature and Feature selection option it lists (`Summarised {key, name, summary?}`, features from a lookup the `Planner` builds over class, archetype and path features), and for each Feature selection it lists. `options` lines carry it too: a slot's from the engine's `SlotEntry::summary`, a class's or archetype's, a race's or subrace's, a background's note and a path's from the data. Tested on the test world given summaries inside the test (`tests/server/summaries.rs`, 3 tests; `rules` unchanged); a smoke test on real data shows the one summary filled so far (the Thassilonian Specialist's school selection) in `describe`. `SQLX_OFFLINE=true scripts/check.sh`: ALL-OK. 86 server tests pass. The data stages are left to their own work.
- 2026-11-08 (data): the five stages wrote 3,383 summaries (mythic 391, feats/races/prestige 456, class groups 784, 843 and 909). Two found a shared-build trap on the way: the shared target dir's `seed` binary had another worktree's data folder baked in, so `--check-only` now always names its folder. Review, fixes and re-checks per choice 2; the class group 3 writer moved its summaries to "you", and the 12 gendered third-person archetype descriptions on `main` became "their". All five merged into `main`.
- Real-data test `libs/seeds/tests/game_data/summaries.rs`: the empty summaries per kind (class features 113, mythic path entries 7, path features 7, class options 6; every other kind 0) can't grow, and every summary is one line of at most 160 characters. A README note in `libs/game-data/`. The instructions' honesty boundary names the summaries.

## Answer

Done 2026-11-08. Every pickable item the guide describes has a one-sentence summary in our own words (3,383), checked against the saved guide pages by a script and read in full by independent reviewers; the web app's picker and the MCP server's `search`, `describe` and `options` show it. 133 items have none, because the guide has nothing usable on them, and a real-data test keeps that number from growing.
