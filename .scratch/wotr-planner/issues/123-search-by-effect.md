# Search by what an item does

Type: design
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

`search` matches a name, a key or initials (`apps/mcp/src/search.rs`). In the Max DC run (2026-11-09, see [119](119-caster-line.md)) the agent searched "DC" to find what raises spell DCs, and got Dreadful Carnage, Death Curse, the Draconic (Copper) bloodlines, Godclaw, the Dual-Cursed Oracle and Defensive Combat Training from initials or keys. Only one hit raises a spell DC: Aeon's Enforcing Gaze - Save DC, because "DC" is in its name.

The items it needed (the Arcane bloodline's +1 on metamagic and its School Power, the Fey arcana's +2 on compulsion, the Kitsune's +1 on enchantment, Expanded Arsenal, Favorable Magic) it found from memory and then checked. An item it didn't remember stays unfound, so the build is only as complete as the agent's memory. Every one of those facts is in a summary already ([112](112-effect-summaries.md)).

## Options

- **A. Nothing:** the agent lists a category (`Magic` feats) and reads the summaries.
- **B. Match the summaries too:** a query word that's in an item's summary is a match, ranked below name and key matches. "DC" or "save DC" would find Spell Focus, the Arcane and Fey bloodlines, Kitsune, Gnome, Oread, Aeon's gaze and Elemental Focus. No new data.
- **C. Effect tags:** a fixed list of tags on items (`raises_spell_dc`, `save_penalty`, `roll_twice`, `caster_level`, `spell_penetration`, `extra_targets`, `initiative`, ...) and a `tags` filter on `search`. New data on thousands of items, each tag a judgement to review.

## Open questions (with recommendations)

1. **Which option?** *Recommend:* B now. It's cheap and uses reviewed text; C only if B's hits prove too noisy, and then for the few tags 119 and 120 need, which their bonus data could supply.
2. **Short words:** "DC" also sits inside other words. *Recommend:* summary matches are whole words only, and the result marks which hits came from a summary (`matched: "summary"`).
3. **Ranking:** *Recommend:* name, then key, then initials, then summary; within summary hits, items whose summary holds every query word first.

## Done when

`search {query: "DC"}` lists the run's DC sources among its hits, marked as summary matches, with a server test on real data; the tool description says summaries are searched; `scripts/check.sh` passes.

## Choices settled

Every recommendation taken while the user slept, 2026-11-09 (user: "where a ticket has open questions, take the recommended answers").

1. **Option B:** `search` also matches the words of each item's `summary` (a background's: its note). No new data. Recommended answer, taken while the user slept, 2026-11-09.
2. **Whole words:** a query word matches a summary word that is the same word or its plural either way (`dc`/`dcs`, `hex`/`hexes`), never a word inside another; case and punctuation don't count. A summary match is any query word in the summary. Hits from a summary carry `matched: "summary"`; name, key and initials hits leave `matched` out. Recommended answer, taken while the user slept, 2026-11-09.
3. **Ranking:** every name, key and initials match (exact, initials, name start, name words, contains, initials start, as before), then summary matches, those whose summary holds every query word first, then by how few words they lack; within a rank, as before (kind, shorter name, name, key). Recommended answer, taken while the user slept, 2026-11-09.
4. **Notes** (the smallest choice that keeps `search`'s honesty about absent names, beyond the recommendations): when every match is a summary match, `note` says "No item's name, key or initials match `<query>`: every match is by its summary's words", so a name the data lacks still reads as absent. When `limit` cut summary matches from the list, `note` says how many and to raise `limit`: at the default limit of 10, "DC"'s ten name and initials matches fill the list (Dreadful Carnage, Death Curse, …), and the note points at the 57 summary matches behind them. The default limit stays 10.
5. `describe`'s "Closest:" hint for an unknown key keeps name, key and initials matches only: a summary holding a word of the key isn't close to it.

## Progress log

- 2026-11-09: Rebuilt the worktree's database (its copy predated ticket 130's migration, so the seeds crate's query macros didn't compile); `.sqlx` unchanged.
- Tests first, in `apps/mcp/tests/server/search.rs`: "DC" on real data at the default limit (the note names the summary matches left out) and at `limit: 100` (Spell Focus, Greater Spell Focus, Elemental Focus, Spell Focus (Mythic), the Arcane and Fey bloodlines, School Power (Enchantment), Kitsune, Gnome and Oread, each marked `matched: "summary"`; the Aeon gaze's Save DC by its name, unmarked; every summary match after every name match); whole words and every-word-first ("compulsion DCs": the Fey bloodline before Spell Focus); and only-summary matches noted ("compulsion"). Two existing tests now read name matches apart from summary ones: "spell focus" (its three name matches, then summaries) and the end-to-end prompt's "animal growth" (no name matches: only summaries holding "animal", with the note).
- Built: `Rank::Summary { missing }` after the name ranks, the summary's words on each index entry, `Hit::matched`, the two notes, and the `search` tool description, its `query` argument doc and the instructions' workflow step 1 saying summaries are searched.
- Measured on real data: `search {query: "DC", limit: 100}` has 67 matches, 10 by name, key or initials and 57 by summary (Winter Witch, Spell Focus, Elemental Focus, the bloodlines, School Power's eight schools, Kitsune, Gnome, Oread, …); 27.8 kB of JSON.
- `scripts/check.sh`: ALL-OK. 94 server tests pass (3 new).

## Answer

Done 2026-11-09 with option B. `search` also matches item summaries by whole words (a plural counts), ranked after every name, key and initials match, those holding every query word first, and each such hit is marked `matched: "summary"`. `search {query: "DC", limit: 100}` now lists the Max DC run's sources: Spell Focus and its kin, Elemental Focus, the Arcane and Fey bloodlines, School Power, Kitsune, Gnome and Oread (the Aeon gaze by its name). At the default limit, `note` says how many summary matches the limit left out; when only summaries match, `note` says no name did. Favorable Magic and Expanded Arsenal don't say "DC" in their summaries, so a "DC" search doesn't find them ("saves" or "Spell Focus" does). Effect tags (option C) wait until summary hits prove too noisy.
