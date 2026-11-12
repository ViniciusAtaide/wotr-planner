# The Gear tab in the web app

Type: task
Status: done
Blocked by: 165, 166
Model: Opus to build (Leptos UI on two Fold screen sizes, checked in the running app); review on another model.
Map: [WotR Build Planner](../map.md)

## Question

The third gear ticket from the equipment plan ([163](163-equipment.md), user, 2026-11-11): the web app. This reverses ticket [145](145-assumed-gear-in-the-web-app.md)'s won't-do, on a different design: gear the user picks, not gear the app assumes.

Decided in 163:

- **A Gear tab** beside the levels and the summary, "another screen/tab overall independant on the level screen" (user).
- **Items show as races do:** the name, with a small summary of the bonuses below. "I like how races had a small summary bellow telling its main characteristics. equipments should be the same" (user).
- **Other effects** ("On Crit: …") sit in the item's detail, seen on hover (user). On the Fold there is no hover (map, Mobile), so tapping the item shows the same detail.
- **Both numbers** on the summary: with gear, and the naked number beside it when the two differ.
- **Gear-dependent picks** carry a note naming the item.

## Build

AFK. `libs/ui` and the app; no engine or schema change. Mobile-first on both Fold screens, as the map's Notes require.

- **The tab:** one row per slot, in the guide's slot order, with two ring rows. A row shows the worn item's name and summary, or "None".
  - **A Tomes row** (user, 2026-11-11): the manuals and tomes are listed with a tick each, since a character can use all of them. It shows the ticked ones' names and summaries.
  - Robes and armours are picked in one body row.
  - Tapping a row opens the picker the feats use (`libs/ui/src/picker.rs`), listing that slot's items with name and summary, a search box, and "None" first.
  - An item's detail (the picker's existing `detail` area) lists its other effects and its guide link. Desktop shows it on hover as well; touch by tapping the item. A worn item's row expands to the same detail.
  - Picking an item updates the link at once, as every other choice does.
- **The summary grid** (`libs/ui/src/grid.rs`): each number gear changes shows the geared value, with the naked one beside it, smaller and labelled, only when they differ. Use the engine's one call for the naked numbers (166). The leaves-out list says what gear still doesn't count.
- **Gear-dependent picks:** on the level screen and the summary, a pick allowed only by gear shows "With <item name>" where a story note shows today. In a picker, an option open only because of gear shows the same note.
- **An item a pick needs can't be removed or replaced** (user, 2026-11-12; this replaces "the pick stays and is refused"). In the slot's picker, "None" and every item that wouldn't keep the pick legal are locked, with the reason naming the pick ("Greater Two-Weapon Fighting needs it"), as a point-buy step that breaks a later pick is. The engine gives the lock and its reason; a tome that a pick needs can't be unticked either.
- **The text export** (`build_sheet.rs`, "WotR build v1"): gear lines with new keys, one per worn item, written only when the Build has gear. Every pinned export stays byte for byte; add a pinned export with gear. The export's scores keep reading the point buy, as today.
- **An old link** opens with no gear and an empty Gear tab.
- Run the app and check, on both Fold screen sizes and on desktop: wearing a belt, the summary's two numbers, a feat opening and its note, removing the belt, the detail by tap and by hover. Screenshots in the Answer.
- Extend the end-to-end smoke tests if ticket [147](147-end-to-end-smoke-tests.md) has landed; if not, name the flows it should cover in its ticket.

## Done when

- The flows above work in the running app on both Fold sizes and desktop, with screenshots.
- No interaction needs hover.
- A Build without gear renders exactly as before, and its export is byte-identical.
- `scripts/check.sh --full` passes.

## Answer

**Built**, 2026-11-12 on branch `ticket-167`, from `main` at `5b5b5dc`. Not merged. `scripts/check.sh --full` passes, and the snapshot check passes with its files untouched.

### What was built

- **The Gear tab** (`libs/ui/src/gear.rs`), the third tab after Levels and the Mythic track: a row per place in `GearPlace::ALL`'s order (Belt, Boots, Bracers, Cloak, Glasses, Gloves, Headwear, Necklace, Ring 1, Ring 2, Body, Weapon), then a **Tomes** row. A row shows the item's name with `Engine::gear_summary` below it, or "None"; tapping it opens the place's picker. A worn item with other effects or a guide link has a thumb-sized info button that unfolds them under the row. An item some choice can't do without says so on its row ("🔒 Greater Two-Weapon Fighting needs it", from `GearNeed::all_of`).
- **A place's picker:** the feats' `ItemList` with `Engine::gear_options`: "None" first, every item with its summary, a search box from nine items, Robe and Armour filter chips on the body, the Remove button when `withdrawal` allows it. When taking the item off is locked, the engine's reason heads the list ("🔒 Greater Two-Weapon Fighting needs Belt of Incredible Dexterity +2. Clear Greater Two-Weapon Fighting first to take it off."), and "None" and every item that wouldn't keep the pick sit under "Show N locked", each with that reason.
- **The Tomes picker:** `Engine::usable_options` as ticks; a tick applies at once and the picker stays open for the next; a tome a pick needs is greyed with its reason.
- **An item's detail** (other effects, "In the guide ↗"): a picker row with any gets an info button beside it (`aria-controls` names the detail); a tap folds it open under the row. With a pointer (`@media (hover: hover)`), resting on a row for 0.3 s shows the same detail over the rows below, without moving them. Nothing needs hover.
- **The summary** (`grid.rs`): each score gear changes has its naked value beside it, smaller and labelled ("Dex 19 naked 17"), and the attack line has a naked line under it when it differs ("naked +14/+9/+4 · 1d8 · 20/×3"), both from `Engine::naked_sheet`, once per edit in a memo (none for a Build without gear). A "Totals with gear:" line lists every other total the gear changes, with both numbers in the same form ("Reflex +13 naked +6"): the saves, initiative and each skill from `Engine::totals`, and the caster line's caster level, spell penetration, concentration and highest spell level's DC from `Engine::caster_line` (a spell of each school; schools whose DC changes alike are named once, "Spell DC (level 3)" when all do), each from the sheet and the naked sheet, only where they differ; the line isn't there for a Build without gear, nor when no total differs (review fix). A "Gear-dependent:" line lists each pick only gear allows, with its note. The attack line's "Not counted" already says what gear still doesn't count (ticket 166's text).
- **Gear-dependent picks:** a slot chip on the levels and the Mythic track shows "With <item>" under its value, in the gear colour, from `Engine::gear_dependents` (a memo, once per edit); a class only gear lets in shows it under the class. In a slot's picker, `SlotEntry::gear_note` and `SlotOption::with_gear` show where the story note goes.
- **Options that would lean on gear** never look safe: the point buy's steps and the racial +2's buttons that `leans_on_gear` marks are dashed in the gear colour, with a note under them ("Lowering Dex: Greater Two-Weapon Fighting would then need gear"; equal notes share a line); the race, ability increase and class pickers show the same note on the item, named with `Engine::choice_names` ("Open only because of gear" for a class whose own entry needs it).
- **The text export** (`build_sheet.rs`): a `Gear` block after the Build start and before the levels, one line per item worn (`  Belt: …`, the format's own keys, `Ring 1`/`Ring 2` among them), then `  Used: …` per usable item, in the link's order; written only when the Build has gear. The point buy line is unchanged. The pinned v1 texts are byte for byte as before; `a_build_with_gear_lists_it_between_the_start_and_the_levels` pins a geared export (`libs/rules/tests/engine/build_sheet.rs`).

### Checked in the running app

A release build (`cargo leptos build --release`, this worktree's `target/`), its server on `127.0.0.1:3967` against this worktree's seeded database, driven by Playwright (Chromium) with touch and no hover at the Fold's cover (412×960) and inner (720×864) screens, and with a pointer at 1440×900. The Builds: a human Fighter 11 with Dex 17, Two-Weapon Fighting and Improved Two-Weapon Fighting, so Greater Two-Weapon Fighting needs Dex 19; and an elf Wizard 5 with Int 19, for the caster line.

The scripts assert each state and exit non-zero at the first that doesn't hold (the first version only logged and took pictures; the review asked for assertions). `flows.js` makes 253 checks over the three sizes, all passing, and `leans.js` 8 on the cover screen. What they assert, at each size:

- **A link without gear:** the scores read "Str 18 · Dex 17 · Con 10 · Int 10 · Wis 10 · Cha 10"; no naked number, totals line, gear note or mark; the Gear tab's 13 rows all read "None"; the three tabs are `role="tab"` in a `role="tablist"`.
- **The belt picker:** "None ✓" first, a search box. Assailant's Belt's detail is hidden, its info button's `aria-controls` is the detail's id; a tap (touch) expands it, or on desktop a resting pointer shows it and leaving hides it, and a click keeps it; it reads "Landing a critical hit: +2 circumstance on attacks and damage for 3 rounds." with a GameFAQs link.
- **Wearing the belt:** the row reads the belt and "Dex +2 enhancement"; the link's fragment changed. The scores read "Str 18 · Dex 19 naked 17 · …"; with a long bow the attack line reads "+15/+10/+5 · 1d8 · 20/×3" over "naked +14/+9/+4 · 1d8 · 20/×3"; the totals line has Reflex, Initiative and Mobility each one more than naked, and no Fortitude or Will.
- **The feat:** Greater Two-Weapon Fighting is open at level 11 with "With Belt of Incredible Dexterity +2" in the picker; after the pick, the chip holds it with the same note, and the summary reads "Gear-dependent: Greater Two-Weapon Fighting · With Belt of Incredible Dexterity +2".
- **The lock:** the Belt row reads "🔒 Greater Two-Weapon Fighting needs it"; the picker's note reads "🔒 Greater Two-Weapon Fighting needs Belt of Incredible Dexterity +2. Clear Greater Two-Weapon Fighting first to take it off."; there is no Remove button; "None" is disabled with the reason "Greater Two-Weapon Fighting needs Belt of Incredible Dexterity +2", and so is Belt of the Mighty Constitution +2.
- **Clearing, then removing:** the feat's own Remove empties the chip and the summary's gear line; the belt's lock and note go; its Remove leaves the row "None" and the link the gearless one again.
- **A save item** (Grand Festival Cloak, Saves +7 resistance, Cha +2 circumstance): the totals line has Fortitude, Reflex and Will each 7 more than naked, and the scores "Cha 12 naked 10"; the link, opened in a new page, shows the cloak and no belt.
- **Tomes:** nine ticks, none ticked; each tick changes the fragment and keeps the picker open; two ticked; the row reads "Manual of Quickness of Action +2 / Dex +2 inherent / Tome of Clear Thought +2 / Int +2 inherent"; the link, reopened, shows the same row and the cloak.
- **The caster line** (Sin Mage's Staff on the Wizard 5: Int +6, spell DC +1, caster level +3): "Int 25 naked 19"; caster level 8 naked 5; spell penetration +3; concentration +6; "Spell DC (level 3)" 4 more.
- **No console error.**
- `leans.js`: nothing marked without gear; with the belt, the notes read "Str, Con, Int, Wis, Cha: Greater Two-Weapon Fighting would then need gear" and "Lowering Dex: Greater Two-Weapon Fighting would then need gear", those five racial buttons and only "Lower Dexterity" are marked, and the marked step is still open; moving the +2 to Str makes the summary's gear line and the chip's note appear.

Screenshots, `<screen>` being `cover`, `inner` or `desktop`, each in `/private/tmp/claude-501/-Users-thaynamarcela-code-wotr-planner/04de393e-0bc8-4654-8cc8-17fb98e6905c/scratchpad/ticket-167/`: Screenshots, `<screen>` being `cover`, `inner` or `desktop`.

- `<screen>-00-old-link-levels.png`, `<screen>-01-old-link-empty-gear-tab.png`: a link without gear (its bytes are the link it had before gear existed, as the pinned `V1_LINK` holds): no naked number, no gear note, an empty Gear tab.
- `<screen>-02-belt-picker.png`: the belt picker; `cover-03-item-detail-by-tap.png`, `inner-03-item-detail-by-tap.png`: Assailant's Belt's detail opened by a tap on its info button; `desktop-03-item-detail-by-hover.png`: the same by resting the pointer, `desktop-03b-item-detail-by-click.png` by a click.
- `<screen>-04-gear-tab-belt-worn.png`: Belt of Incredible Dexterity +2 worn; the link changed at once.
- `<screen>-05-summary-two-numbers.png`: "Dex 19 naked 17", and with a long bow "+15/+10/+5" over "naked +14/+9/+4".
- `<screen>-06-feat-open-with-belt-note.png`: Greater Two-Weapon Fighting open at level 11 with "With Belt of Incredible Dexterity +2"; `<screen>-07-chip-with-gear-note.png`: the chip after picking it.
- `<screen>-08-gear-tab-belt-needed.png`: the belt's row locked; `<screen>-09-belt-locked-note.png`, `<screen>-09b-belt-none-locked-with-reason.png`: the picker's note, no Remove button, and "None" locked with the reason.
- `<screen>-10-belt-removed.png`: the feat cleared (its own Remove), then the belt taken off.
- `<screen>-11-tomes-ticked.png`, `<screen>-12-gear-tab-tomes.png`: two tomes ticked, and the Tomes row.
- `<screen>-14-totals-block-cloak.png`: the totals line with the cloak (the saves, two skills); `<screen>-15-totals-block-caster.png`: the Wizard's, with the staff (two skills, caster level, spell penetration, concentration, spell DC).
- `cover-13-start-options-that-lean-on-gear.png`: the same Fighter with Dex 19 of its own and the belt: the racial +2's other buttons and Dex's step down marked, with their notes.

The scripts are `/private/tmp/claude-501/-Users-thaynamarcela-code-wotr-planner/04de393e-0bc8-4654-8cc8-17fb98e6905c/scratchpad/ticket-167/flows.js` and `/private/tmp/claude-501/-Users-thaynamarcela-code-wotr-planner/04de393e-0bc8-4654-8cc8-17fb98e6905c/scratchpad/ticket-167/leans.js`.

### Judgement calls

- **Tomes:** the row is "Tomes", as the user named it, though it also lists the elixir and the Potion of Humanity (every `Usable` item).
- **The naked label** is the glossary's word, "naked", in small muted text after the number with gear.
- **The summary's saves** line stays the base saves, which gear doesn't change; the totals gear changes have their own line, shown only with gear, rather than a redesigned summary. Its DC is the highest spell level's, the one a player compares.
- **Details on desktop** float over the rows below, after a short delay, so the list doesn't jump as the pointer moves; the link in a floating detail needs the info button (a click pins it open).
- **Tabs:** three tabs fit one line on the cover screen at 14px, without wrapping; on the wide screens the Gear rows go in two columns.
- **The export's place** for the gear block: after the start, since it sets the scores with the point buy and belongs to no level.

### Ticket 147

147 (end-to-end smoke tests) hasn't been built, so there are no smoke tests to extend. This ticket names the flows here rather than editing 147, which another session owns: wearing a belt and the link changing; the summary's naked numbers; a feat open only with the belt, its note in the picker and on the chip; the belt's picker locked with its reason while the feat holds; clearing the feat, then taking the belt off; ticking two tomes; an item's detail by tap (touch) and by pointer (desktop); a link without gear opening with an empty Gear tab; and a step of the point buy marked as leaning on gear. `/private/tmp/claude-501/-Users-thaynamarcela-code-wotr-planner/04de393e-0bc8-4654-8cc8-17fb98e6905c/scratchpad/ticket-167/flows.js` drives all of them.

### Edits outside `libs/ui` and the app

- `libs/rules/src/engine/build_sheet.rs`: the gear block (no engine change otherwise).
- `libs/rules/tests/engine/build_sheet.rs`: the pinned geared export.

No engine API was missing.
