# Search by category, and say when nothing matches

Type: task
Status: open
Blocked by: none (tags wait on [112](112-effect-summaries.md))
Map: [WotR Build Planner](../map.md)

## Question

`search` matches names, keys and initials, filtered by `kinds`. To find damage picks in the Demon Greatsword run (2026-11-08, see 112), the agent had to know a name first. For example it searched "critical" and "power attack", then listed whole pickers with `options`.

The data already gives feats and mythic picks a category ("Weapon", "Melee", "Ranged", "Magic", …), and options a parent selection ("Rage Power").

A name that isn't in the data (Furious Focus, a tabletop feat) reached `choose`. It was refused as `not_an_option` with five unrelated "closest" names (Shield Focus, Arcane Focus, …).

## Scope

1. `search` takes `category` (one or more, case-insensitive) and a `query` that may be empty when a category or kind is given, so "every Melee mythic feat" is one call.
2. An empty result says so in words: "No item in the planner's data matches" (the data is what the guide lists). It doesn't hint at near names below a similarity floor.
3. `choose`'s `not_an_option` uses the same floor, so it doesn't list names that share only a word.

## Open questions (with recommendations)

1. **Effect tags** ("damage", "critical", "attacks of opportunity", "two-handed")? *Recommend:* not in this ticket. Once 112's summaries exist, a `query` that also matches summary text covers most of it without a tag list to maintain.

## Done when

Server tests cover the category filter, an empty-query category search and the no-match wording. `scripts/check.sh` passes.
