# Search by category, and say when nothing matches

Type: task
Status: resolved
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

## Choices settled

The user approved building this with the recommended option (2026-11-08): no effect tags.

1. **`category`** is one text or a list, matched against a match's own `category` ignoring case and spacing. An option also matches its Feature selection's key (`barbarian/rage-power`), since its category is the selection's name.
2. **An empty `query`** (or none) is allowed when `kinds` or `category` is given; it matches every item they allow, sorted by kind, then name. Without either filter it's still an error, which now names both filters.
3. **Limits:** the default stays 10 for a query; an empty query defaults to 50, and the most is 100 (was 50), because the real data's largest category, Weapon feats, has 51. A new `total` counts every match, so an agent sees when `limit` cut the list.
4. **No match:** `matches` is empty, `total` is 0, and a new `note` says "No item in the planner's data matches `<query>`" (plus "with these filters" when filtered), then "The data holds what the guide lists: an item that isn't there can't be taken in a Build." It adds "Near names: …" only for names at or above the floor, among the items the filters allow. A `category` no item has adds "No item has the category …", with the close categories.
5. **The floor** (`apps/mcp/src/similar.rs`, shared by `search` and `choose`): closeness is the share of the given words that a word of the candidate starts with (three letters or more) or nearly spells (edit-distance likeness ≥ 0.75), or the likeness of the two texts with their spaces dropped when that is ≥ 0.75 ("powerattack"). The floor is 0.6, so one shared word of two ("Furious Focus", "Shield Focus") isn't near, while "improved critcal" and "power-atack" are.
6. **`choose`'s `not_an_option`** lists only offers at or above the floor, closest first (then edit distance, then open before closed, then the picker's order). It compares the value's plain keys (not its JSON) and the offer's name. With none close, it says "and no offered value is close to it" and points to `options`.
7. Summary text isn't searched yet: that waits on 112's data (open question 1).

## Progress log

- 2026-11-08: Fast-forwarded the worktree to `main` (03ee7df), which filed tickets 112–118. Read the map's Notes, the plan (104) and tickets 108–111.
- Wrote the tests first: `tests/server/search.rs` gains four (the category filter, an empty query listing a kind with `total`, the no-match note and its hints, and every Melee mythic feat of the real data in one call), the empty-query error test now wants both filters named, and `tests/server/choose.rs` gains one (Furious Focus gets no closest names). Then wrote `similar.rs`, the search filter and note, and the floor in `choose`.
- A smoke test of the binary on real data showed "Armor Focus" as a near name of "Furious Focus": the two whole texts were alike enough (0.62). Whole-text likeness now counts only at a misspelling's level (≥ 0.75, spaces dropped), and two unit tests in `similar.rs` pin the cases (Shield, Arcane, Armor and Weapon Focus aren't close; "improved critcal", "power-atack", "powerattack", "Rogue Talent" are).
- Smoke test after the fix: "Furious Focus" → no match and no hints; "improved critcal" → Improved Critical and its kin; `category: "Rage Power"` → the barbarian's rage powers; `choose` of `furious-focus` in a fighter's feat slot → `not_an_option`, "no offered value is close to it".
- `SQLX_OFFLINE=true scripts/check.sh`: ALL-OK.

## Answer

Done 2026-11-08. `search` takes `category` (one or a list, any case) and an empty `query` when `kinds` or `category` is given, so "every Melee mythic feat" is one call; `total` counts every match. When nothing matches, `note` says "No item in the planner's data matches …", and names near names only above a similarity floor. `choose`'s `not_an_option` uses the same floor, so Furious Focus no longer gets Shield Focus and Arcane Focus as "closest". 77 server tests pass (5 new), plus 2 unit tests of the floor.
