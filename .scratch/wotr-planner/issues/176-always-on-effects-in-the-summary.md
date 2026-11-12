# An item's always-on effects belong in its summary, not behind the hover

Type: task
Status: done
Blocked by: none
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

A second report the same day: "fix all hoverable items that has no hover description just a link to the guide". An item with no other effect still gets an info button, and its detail holds nothing but "In the guide". Moving the always-on lines to the summary will leave more items like that.

The rule the user's first sentence gives: **what is always on shows in the base description; only what is conditional, triggered or activated stays in the detail.** Whether the planner counts an effect in a number doesn't decide where it shows.

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
- **The web app:** the row's summary line grows. Check on the cover screen that a long summary wraps without pushing the info button off the row.
- **Nothing to hover when there is nothing to say** (the user's second report): an item is hoverable, and gets its info button, only when its detail has a description to show, which for gear means at least one line left in `other_effects`. A guide link alone is not a description. For such an item there is no info button and no hover; its guide link shows as a small plain link ("Guide") on the worn item's row in the Gear tab, and nowhere in the picker's list. An item with a detail keeps its guide link inside it, as today.
  - Apply the same rule to everything else in the app that opens a detail by hover or by an info button (feats, features, options, classes, races and the rest, in every picker and on the chips): find each case where the detail can be empty or hold only a guide link, list them in the Answer, and give them no hover and no info button. Don't remove a detail that has text.
- **The MCP:** `summary` carries the always-on effects; `other_effects` keeps the rest. Say so in the `search` and `options` descriptions in one clause. They still count in no number: the instructions already say an item's `other_effects` count in nothing; say the same of the always-on effects that aren't bonuses.
- **The engine ignores the new lines:** the snapshot check passes with the snapshot files untouched, and `scripts/check.sh --schema` passes.
- Run the app and look at Mangling Frenzy's row and picker entry on the cover screen and on desktop; screenshots in the Answer.

## Done when

- No item in the app has an info button or a hover whose detail is empty or only a guide link: shown by a browser script that opens every gear picker and one picker of each other kind, at the cover size and on desktop, and asserts it.
- Mangling Frenzy's summary reads `Str, Dex, Con +4 enhancement; DR 3/-` in the Gear tab and in the MCP's `search`, and its detail holds only the critical-hit line.
- Every item's lines are classified, with the counts and the judgement calls in the Answer; a second agent has checked a sample of 15 items against the saved pages.
- No `other_effects` line is always on, and no always-on line is conditional, in that sample.
- Real-data tests pin Mangling Frenzy and two more items, and that no line appears in both lists.
- The snapshot files are untouched and `scripts/check.sh --schema` passes.

## Answer

**Built**, 2026-11-12 on branch `ticket-176`, from `main` at `4f3c267`. Not merged. `scripts/check.sh --schema` passes, and the snapshot check passes with its files untouched. The second agent's check of 15 items (Done when, third item) is the main session's and is not done here. The counts and calls are also in `libs/game-data/EXTRACTION_REPORT.md`; the shape is updated in [research/gear-item-shape.md](../research/gear-item-shape.md).

### What was built

- **The shape** (`libs/rules/src/gear.rs`): `GearItem::always_on: Vec<String>` beside `other_effects`, both `#[serde(default)]`. `GearItem::summary` appends the always-on lines, as written, to the generated bonuses, joined by "; " (`Str, Dex, Con +4 enhancement; DR 3/-`). It is the only summary function: the web app and the MCP both get it through `Engine::gear_summary` (`libs/ui/src/gear.rs`, `apps/mcp/src/search.rs`, `apps/mcp/src/gear.rs`, so `search`, `describe`, `options`, `sheet`), and each passes the engine's skills as before. `always_on` is not sorted by `canonicalize`: its order is the summary's.
- **The check** (`DataIssue::GearEffectInvalid`, widened): an empty or repeated line in `always_on` or `other_effects`, or a line in both. Pinned in `libs/rules/tests/engine/check.rs`.
- **The tables:** migration `20261112092431_176-gear-always-on.sql` adds `gear_always_on_effect` (same columns as `gear_other_effect`, read back in `id` order); `libs/seeds/src/gear.rs` writes and reads it; `libs/seeds/tests/database.rs` round-trips both lists and pins the constraints; `.sqlx` prepared.
- **The data** (`libs/game-data/gear.ron`): all 92 items reclassified.
- **The web app:** `ItemInfo::new` (`libs/ui/src/picker.rs`) is the one place that decides whether an item has a detail: `Some` only with at least one line, and its fields are private, so no other code can build a detail without lines. A guide link alone is no description. An item without a detail has no info button and no hover; on the Gear tab its worn row shows a small plain "Guide" link in the info button's column (`.guide-link.plain`), and the picker's list shows no link for it. An item with a detail keeps "In the guide" inside it.
- **The MCP:** `summary` carries the always-on lines, `other_effects` the rest; the `search`, `describe` and `options` descriptions and the field docs say so in a clause, and the instructions say the always-on lines that aren't bonuses count in no number, as `other_effects` already did.

### The classification

Of 74 `other_effects` lines on 54 items: **32 were always on and moved** to `always_on` (33 lines on 26 items: Razmir's Mask's one line became two), and **42 stayed unchanged**, plus the Offroad Boots' remainder (43 lines on 38 items). The 22 hand-added items have neither list. 16 items have no conditional line left and so no detail: Display of Power, Flawless Belt and Flawless Headband of Mental Perfection +8, Goggles of Ferocious Pact, Hammer Fists, Bookworm's Headband, Darkness Caress, Headband of Confidence, of the Wise Negotiator and of Reshaping, Razmir's Ring, Earth's Embrace, Half-Plate of Vigor, Life Infuser, Snakeskin, Sin Mage's Staff. Moved lines were shortened ("Damage reduction 3/-." became "DR 3/-"); lines that stayed keep their text.

Calls that weren't obvious (each checked against the item's row on the saved pages):

- **Saves against one kind of effect** (+4 vs mind-affecting: Headband of Confidence, Headband of the Wise Negotiator; vs fire: Earth's Embrace; vs poison: Amulet of Alchemical Knowledge) moved: they work like a resistance, with no trigger. **Against a kind of creature** stays (Midnight Hunter's Ring vs lycanthropes, Ustlavic Pride's DR 5/- vs undead, Dirge for Sarkoris' holy damage vs undead, Belt of the Venerated Champion vs evil: the ticket's "against <kind of enemy>"). The two readings are close; this is the call to overrule if the user disagrees.
- **A bonus scoped to the wearer's own spells** (Headband of Reshaping +1 caster level for Transmutation, Jester Cap +2 DC for fused spells) moved, with the scope in the line.
- **Extra uses a day** (Flamboyant Hat's bardic rounds, Headband of Rigorous Training's Perfect Strike, the Amulet's bombs) stayed: limited in uses.
- **Offroad Boots** was one line of two effects: "ignores difficult terrain" moved, the +10 feet of speed for one round on crossing it stayed. **Razmir's Mask:** immunity to Divination magic and Spell Turning moved (as two lines); casting a spell each round as a free action stayed (activated). **Gorgon Belt:** only "ignores difficult terrain" moved, the breath stayed. **Heart of Ice:** "AC +4 armour" moved, Ice Cascade stayed.
- **Goggles of Ferocious Pact:** "AC -2" and "Damage +4" are in the row's one sentence with the score changes, so always on. **Display of Power:** the aura's -2 to enemies' Will saves has no trigger in its row, so always on. **Hammer Fists:** "AC +2 morale". **Mangling Frenzy:** DR moved, the critical hit while raging stayed. **Cat's Boots:** no attacks of opportunity from moving moved, the set bonus with the Owl's Cowl stayed.
- **The armour and weapon lines** ("A +4 leather armour") moved, as the ticket's own example has it.
- **Belt of the Venerated Champion** keeps its line, with its parenthetical about the ambiguous wording.

### Every place where a detail could come out empty or link-only

1. **`ItemInfo::new`** (`libs/ui/src/picker.rs`), which made a detail from lines or a guide link: a gear item with no `other_effects` and a guide reference got an info button whose detail held only "In the guide". Now `None` without a line. It serves both the picker rows (`item_row`, from `Item::info`) and the Gear tab's worn rows (`place_row`).
2. **The Gear tab's worn row** (`place_row`): no detail means no info button, no hover, and the guide link as a plain "Guide" on the row; an item with no guide row (the 22 by hand) shows no link.
3. **`item_detail`** itself: it used to guard against empty lines; it now takes only an `ItemInfo`, which always has a line.
4. **Every other picker** (feats, class, race, deity, alignment, background, ability increase, Mythic): `Item::info` is never set by them, so they have no info button and no hover; their other text is a visible line (`Item::detail`, the story note, the reason), rendered only when it is not empty. No change was needed; the browser script asserts it.
5. **Browser tooltips (`title`)** on the chips, the class button, the alignment wheel and the skill rows: each is set only with text (the class button's only while locked, the chip's the label with any gear note). None is empty or a link; the script asserts none is empty.

### Checked in the running app

A release build (`cargo leptos build --release`, this worktree's `target/`), its server on `127.0.0.1:3976` against this worktree's seeded database, stopped afterwards; Playwright (Chromium) at the Fold's cover (412x960) and inner (720x864) screens with touch and no hover, and on desktop (1440x900) with a pointer, on the human Fighter 11 that ticket 167's flows use.

`/private/tmp/claude-501/-Users-thaynamarcela-code-wotr-planner/04de393e-0bc8-4654-8cc8-17fb98e6905c/scratchpad/ticket-176/detail.js` makes 3,356 checks at cover and inner and 2,047 on desktop (two runs, every check passing). It asserts:

- Mangling Frenzy's picker entry reads `Str, Dex, Con +4 enhancement; DR 3/-`, has an info button, and its detail holds exactly one line, "Critical hit while raging: +4d6 slashing damage.", without the DR; the same on its worn Gear tab row, with its guide link inside the detail and no plain link beside it.
- Snakeskin, Sin Mage's Staff, Hammer Fists (guide row, no detail), the hand-added Headband of Vast Intelligence +2 (no guide row) and Cat's Boots (a detail): an info button only with a detail; a plain "Guide" link to the guide only with a guide row and no detail; Snakeskin's summary reads `Dex +4 profane; A +4 leather armour; +20 feet base speed; Immune to acid` and Hammer Fists' `Str +4 enhancement; AC +2 morale`.
- On the cover, a long summary wraps (`scrollWidth <= clientWidth`) with the row inside the screen and the info button or Guide link right of the value, inside the row, at least 40 px square.
- Every gear picker (12 places, 98 rows, 40 with a detail): the rows are exactly the items of the place's slots (read from `gear.ron`, saved as `gear.json` beside the script); an info button exactly where `other_effects` has a line; every detail has at least one non-empty line and is not only the link; no detail, link or `has-info` class on a row without a button; no guide link outside a detail; as many info buttons as details. On desktop, resting the pointer on every row shows a detail only for a row that has one; with touch, no detail shows by itself and a tap on each info button opens exactly its detail.
- The Tomes picker, the Levels tab's chips, a feat picker, the class picker, the ability increase picker, the race, faith, alignment and background pickers (from the Build start) and the Mythic tab and its picker: no info button, detail, guide link or `has-info`, and no empty tooltip.
- No console error.

Ticket 167's scripts, copied to `/private/tmp/claude-501/-Users-thaynamarcela-code-wotr-planner/04de393e-0bc8-4654-8cc8-17fb98e6905c/scratchpad/ticket-176/flows167.js` and `/private/tmp/claude-501/-Users-thaynamarcela-code-wotr-planner/04de393e-0bc8-4654-8cc8-17fb98e6905c/scratchpad/ticket-176/leans167.js` with only the output folder and the port changed (`shots167/`, 3976): **no assertion changed.** `flows167.js` passes its 253 checks (the Assailant's Belt detail by tap and by hover still has its info button: it keeps its critical-hit line) and `leans167.js` its 8. One earlier run failed on desktop at "click: the detail stays open", a timing flake under the other agent's heavy load; the same script passed alone and in a second full run.

Screenshots, in `/private/tmp/claude-501/-Users-thaynamarcela-code-wotr-planner/04de393e-0bc8-4654-8cc8-17fb98e6905c/scratchpad/ticket-176/shots/`, `<screen>` being `cover`, `inner` or `desktop`:

- `<screen>-01-mangling-frenzy-picker-entry.png`: the belt picker filtered to Mangling Frenzy with its detail open.
- `<screen>-02-gear-tab-rows.png`: the Gear tab with Mangling Frenzy, Cat's Boots (info buttons), Hammer Fists, Snakeskin and Sin Mage's Staff (Guide links) and the Headband (neither).
- `<screen>-03-mangling-frenzy-row.png`, `<screen>-04-snakeskin-row-guide-link.png`, `<screen>-05-mangling-frenzy-row-detail.png`: the rows, and Mangling Frenzy's detail opened by tap or hover.
- `<screen>-06-belt-picker.png`: the whole belt picker.

Logs: `run-ci.log` (cover and inner), `run-d.log` (desktop), `run-167b.log`, `run-leans.log`.

### Judgement calls

- `always_on` has no length rule in the data check beyond non-empty; a real-data test holds it to 60 characters, the summary's budget on a cover screen.
- The Guide link on a detail-less row sits in the info button's column, so the row keeps its alignment; "Guide" at 12.5 px in the accent colour with a thumb-sized target.
- No engine rule, target or bonus was added; the snapshot files are untouched.
