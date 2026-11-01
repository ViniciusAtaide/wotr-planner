# Build the planning sheet screen

Type: task
Status: open
Blocked by: 09, 10, 11, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36
Map: [WotR Build Planner](../map.md)

## Question

Build the Leptos screen chosen in [Level-up screen behaviour](06-level-up-screen.md) (variant C, Sheet) on top of the real `GameData` and rules engine. The [prototype](../prototypes/level-up-screen/index.html) is the reference for behaviour only: rewrite it properly and don't port it.

- **Build start:** race, alignment and point buy, ahead of the grid.
- **Levels grid:** levels 1–20 (1–40 on Legend). A class button opens the class/archetype picker; prestige entry asks once which spellcasting class it advances. There's one chip per slot from the rules engine.
- **Mythic track tab:** MR 1–10 at their fixed levels, the path at MR 3, the Path switch at MR 8. Ranks never reached show as such.
- **Picker:**
  - a bottom sheet below 600px and a modal above
  - search and category chips
  - locked options hidden behind "Show N locked", with reasons
  - options, class changes and Clear that would break a later pick are disabled with "Would break: …"
- **The sheet is the summary** (user, 2026-10-31): there's no separate summary screen. A finished Build's summary is the same grid and Mythic track, showing class, feats, mythic feats and abilities up to level 20 (40 on Legend). A shared link opens on the same sheet.
- **Skill ranks:** per level, capped at character level.
- **State:** the Build lives in the URL fragment (`/#b=…`) and is rewritten with `history.replaceState` on every edit. `/` with no fragment is a blank Build. Opening a stale link shows the "Game data changed" banner, with broken picks and removed keys marked. The rules engine runs in the browser on `GameData` loaded once.
- **Fold 7:** check it at cover and inner sizes; tap targets at least 44px.

AFK: driven by the agent. End-to-end smoke tests come later (see the map).
