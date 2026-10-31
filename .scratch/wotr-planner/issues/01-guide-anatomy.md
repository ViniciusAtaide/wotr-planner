# How the guide is laid out

Type: research
Status: resolved
Blocked by:
Map: [WotR Build Planner](../map.md)

## Question

How is the [GameFAQs guide](https://gamefaqs.gamespot.com/ps4/324475-pathfinder-wrath-of-the-righteous/faqs/80843/introduction) organised, and how cleanly can its data be extracted?

- Which pages or sections hold each of these:
  - races
  - classes and their level-by-level progression tables (BAB, saves, class features, bonus feat levels)
  - archetypes and what each one replaces
  - feats
  - mythic paths and their rank progression, including Legend
  - mythic abilities and mythic feats
  - feature selections (rogue talents, rage powers, hexes, discoveries and the like)
  - WotR's skill list
- How prerequisites are written: free text or a consistent format? Give real examples of the messy cases (OR, "any one of", class features, skill ranks, caster level).
- The HTML structure and how pages are split. Can it be fetched and parsed? What do robots.txt and the terms of use say about automated fetching?
- Gaps: anything the planner needs that the guide doesn't cover.

## Answer

Full write-up: [guide-anatomy.md](../research/guide-anatomy.md).

- **Layout:** 70 pages (454 TOC entries). About 38 matter: `character-creation` (races, attributes, skills), `feats`, 26 base-class pages (archetypes as H3 sections, feature selections as H4/H5), `prestige-classes`, `mythic-paths` (rank table, mythic abilities, mythic feats), 7 path pages and `other-mythic-paths` (Legend, Devil, Swarm).
- **HTML:** a flat list of `h2–h5 / p / table.ffaq / ul / blockquote` elements inside `div#faqwrap`. Every heading has an `<a id>` anchor and there is no pagination. Class, archetype and mythic progressions are real tables. Feats, abilities and selections are prose entries.
- **Prerequisites:** `<p>Prerequisites:<br>…</p>`, one ANDed condition per line, but free text inside each line: OR, "One of / Two of", mixed `or`/`+`, missing colons, abbreviations, typos (16 of 201 labels are misspelt). Plan for a tolerant grammar plus a hand-maintained override file.
- **Fetching:** robots.txt disallows ClaudeBot, anthropic-ai and Claude-Web, the Fandom terms forbid unauthorized scraping, the author allows personal use only, and the site returns 403 to our fetch tool. Don't build a crawler. Instead, the user saves the ~38 pages by hand into a git-ignored cache, we parse those offline, and we commit only facts, our own summaries and links.
- **Top gaps:** no combat-feat tag (fighter bonus slots); archetype replacements must be diffed from the tables; caster level is mostly implicit and CL prerequisites look missing; no mythic-rank minimums, and ranks come from story events; Legend has no table; no max-skill-ranks rule; alignment restrictions; exclusivity rules live only in prose.
