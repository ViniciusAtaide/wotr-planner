# Extract the guide into data files

Type: task
Status: open
Blocked by: 08, 09
Map: [WotR Build Planner](../map.md)

## Question

Build the extractor crate and run it once over `guide-cache/` to produce the real `game-data/` files (layout in [Game data schema and seeding](05-game-data-schema.md)). Parsing facts are in [research/guide-anatomy.md](../research/guide-anatomy.md).

- Read the `div#faqwrap` content and rebuild nesting from heading levels. Keys come from the `<a id>` anchors.
- Parse progression tables, handling these quirks:
  - "+6 / +1" BAB values
  - row 0
  - the `Feat` column, which marks single-class odd levels and is to be ignored
  - varied headers
  - footnote digits glued to cell text
  - bonus slots shown only in the Special columns
- Build the prerequisite-line grammar test-first, using short hand-written examples:
  - `or`, `+`, "One of" / "Two of"
  - missing colons, BAB abbreviations
  - "Ability to cast Nth-level arcane spells"
  - the misspelt "Prerequsites" / "Prerequsities" labels
- Write a report of every line the parser couldn't handle, so it can be fixed by hand.
- The extractor refuses to overwrite existing data files.
- Encode the rules that the guide only states in prose: totem exclusivity, advanced rogue talents at 10, Major/Grand hexes, Trickster tiers, and path switch rules.
- Facts the user gave go straight into `progression.ron`:
  - the mythic rank table
  - combat feats are every category except class, racial, skill, teamwork and magic (miscellaneous counts)
  - skill ranks are capped at character level

Done when `cargo run -p data --bin seed` loads the full data cleanly and the report of unparsed lines is empty or has been fixed by hand. AFK: driven by the agent.
