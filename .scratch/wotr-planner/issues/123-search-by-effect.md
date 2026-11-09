# Search by what an item does

Type: design
Status: open
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
