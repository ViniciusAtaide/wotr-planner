# An item's always-on effects belong in its summary, not behind the hover

Type: task
Status: open
Blocked by: 170
Model: Sonnet to reclassify the data and build; Opus to check a sample of the classification against the saved pages.
Map: [WotR Build Planner](../map.md)

## Question

The user, 2026-11-12, looking at the Gear tab: "DR 3/ on belt mangling frenzy is always on so it should be on the base description not on the hover".

Today an item has two texts (ticket [163](163-equipment.md), built by [165](165-gear-data.md)): a summary generated from its modelled bonuses, shown under the name, and `other_effects`, shown in the detail (hover on desktop, the info button on touch). Everything the planner doesn't count went into `other_effects`, whether it always applies or not. Mangling Frenzy shows it:

```ron
bonuses: [Str +4, Dex +4, Con +4 enhancement],
other_effects: [
    "Damage reduction 3/-.",                              // always on
    "Critical hit while raging: +4d6 slashing damage.",   // only on a crit, only while raging
],
```

The rule the user's sentence gives: **what is always on shows in the base description; only what is conditional, triggered or activated stays in the detail.** Whether the planner counts an effect in a number doesn't decide where it shows.

## Build

AFK. The gear data, its shape, the seed (a schema change: a new migration, `CLAUDE.md`'s "Database"), the web app and the MCP. No engine rule changes: an always-on effect that isn't a modelled bonus still counts in no number.

- **The shape** (`libs/rules/src/gear.rs`, `research/gear-item-shape.md`): an item gains a list of always-on effects beside `other_effects`, each a short line in the planner's own words ("DR 3/-", "Immune to mind-affecting effects", "A +5 breastplate"). `other_effects` keeps only what is conditional, triggered, activated, limited in uses, or a restriction on who may wear the item.
- **The summary** (`GearItem::summary`): the generated bonuses first, then the always-on effects, joined the same way: `Str, Dex, Con +4 enhancement; DR 3/-`. Every place that shows the summary gets it from this one function: the Gear tab's rows and pickers, the MCP's `options`, `search`, `describe` and `sheet`.
- **Reclassify all 92 items.** For each `other_effects` line, decide from the item's row on the saved page (`guide-cache/items*.html`, in the main checkout, read-only) whether it always applies while the item is worn:
  - always on: damage reduction, energy resistance, immunities, an AC bonus, a speed bonus, spell resistance, a weapon's or armour's own enhancement and base type, a critical-range or threat change that needs no trigger;
  - stays in the detail: "on a critical hit", "while raging", "against <kind of enemy>", "N times a day", "when worn with <another item>", "can only be equipped by …".
  
  Record the count moved and every call that wasn't obvious in the ticket's Answer and in `libs/game-data/EXTRACTION_REPORT.md`. Keep each line short: the summary sits under the name on a phone's cover screen.
- **The data check:** an always-on line is non-empty, not repeated, and not also in `other_effects`.
- **The tables:** the always-on lines get a table of their own, or a column that tells the two apart; the seed round-trips both.
- **The web app:** nothing new to design: the row's summary line grows. Check on the cover screen that a long summary wraps without pushing the info button off the row; an item with nothing left in its detail shows no info button unless it has a guide link.
- **The MCP:** `summary` carries the always-on effects; `other_effects` keeps the rest. Say so in the `search` and `options` descriptions in one clause. They still count in no number: the instructions already say an item's `other_effects` count in nothing; say the same of the always-on effects that aren't bonuses.
- **The engine ignores the new lines:** the snapshot check passes with the snapshot files untouched, and `scripts/check.sh --schema` passes.
- Run the app and look at Mangling Frenzy's row and picker entry on the cover screen and on desktop; screenshots in the Answer.

## Done when

- Mangling Frenzy's summary reads `Str, Dex, Con +4 enhancement; DR 3/-` in the Gear tab and in the MCP's `search`, and its detail holds only the critical-hit line.
- Every item's lines are classified, with the counts and the judgement calls in the Answer; a second agent has checked a sample of 15 items against the saved pages.
- No `other_effects` line is always on, and no always-on line is conditional, in that sample.
- Real-data tests pin Mangling Frenzy and two more items, and that no line appears in both lists.
- The snapshot files are untouched and `scripts/check.sh --schema` passes.
