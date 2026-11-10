# Summaries for the 133 items without one

Type: task
Status: open
Blocked by: none (the class features: the user saving pages)
Map: [WotR Build Planner](../map.md)

## Question

Ticket [112](112-effect-summaries.md) wrote a summary for every pickable item but 133, because the guide has nothing usable for those. `summaries::items_without_a_summary_never_grow_in_number` pins the counts:

| Kind | Without a summary |
| --- | --- |
| class feature | 113 |
| class selection option | 6 |
| mythic path | 7 |
| path feature | 7 |

The web app's picker and the MCP server's `search`, `describe` and `options` show nothing for these items, and `search` can't find them by what they do (ticket [123](123-search-by-effect.md)).

Options:
- **A.** The user saves pages that describe these items into `guide-cache/` (for example the fextralife wiki's class and mythic path pages, added to `scripts/save-guide-pages.sh`). Summaries are then written from them in our own words, as in ticket 112: each one checked by the script and read by an independent reviewer.
- **B.** Write the mythic paths' and path features' summaries from what the guide already says about each path's ranks, where it says enough, and leave the class features to A.
- **C.** Leave them empty.

*Recommend:* B now, then A for the class features once the user has saved the pages. Agents never fetch GameFAQs or the fextralife wiki themselves (map Notes).

First step: list the 133 items by kind and owner, so the user knows which pages to save.

## Done when

Every item that gets a summary lowers its kind's count in `EMPTY`. Each summary is one short line in our own words, citing its page, and has been read by an independent reviewer. `libs/game-data/README.md` notes the change. `scripts/check.sh` passes.

## Answer

**Recommendation taken** (user, 2026-11-10): B now, for the 7 mythic paths and 7 path features. For the 113 class features and 6 class selection options, list them by owner, so the user knows which pages to save, and write their summaries once the pages are in `guide-cache/` (A).
