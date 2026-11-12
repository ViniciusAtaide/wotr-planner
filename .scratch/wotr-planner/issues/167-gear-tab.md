# The Gear tab in the web app

Type: task
Status: open
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
- **A refused pick after its item is removed** looks like any refused pick, with its reason. Check the existing refused-pick display handles a pick refused by a change on another tab.
- **The text export** (`build_sheet.rs`, "WotR build v1"): gear lines with new keys, one per worn item, written only when the Build has gear. Every pinned export stays byte for byte; add a pinned export with gear. The export's scores keep reading the point buy, as today.
- **An old link** opens with no gear and an empty Gear tab.
- Run the app and check, on both Fold screen sizes and on desktop: wearing a belt, the summary's two numbers, a feat opening and its note, removing the belt, the detail by tap and by hover. Screenshots in the Answer.
- Extend the end-to-end smoke tests if ticket [147](147-end-to-end-smoke-tests.md) has landed; if not, name the flows it should cover in its ticket.

## Done when

- The flows above work in the running app on both Fold sizes and desktop, with screenshots.
- No interaction needs hover.
- A Build without gear renders exactly as before, and its export is byte-identical.
- `scripts/check.sh --full` passes.
