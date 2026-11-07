# Copy a Build as a readable build sheet

Type: task
Status: open
Blocked by: [86](86-deities.md) (it changes the Build start and the start panel; the sheet shows the deity)
Map: [WotR Build Planner](../map.md)

## Question

The user wanted a Build a player can read like a build sheet and follow in the game (2026-11-06). A review (scratchpad `build-file-review/`: the prototype renderer in `sheet/`, each bench Build's sheet in `*.sheet.txt`, the sizes in `sizes.tsv`, the name checks in `notes.txt`) found the sheet can't be the link itself:
- A 20-level sheet is 3–6 KB. Raw in a URL it is 4.3–8.7 K characters of `%20`/`%0A`, unreadable in an address bar and too long for a Discord message or a QR code.
- Compressed (brotli + base64url), it is 2.5–2.9× today's `#b=` link (895–1,192 characters for the 20- and 40-level bench Builds against 351–469), and unreadable.
- Names break on any rename, where the codec's key hashes don't. A typed file needs lenient parsing, and slot labels and option names repeat (two "Bonus feat"s in a level; "Air Domain" in several selections).

The user chose (2026-11-06): keep the `#b=` link as it is, and add a **"Copy build sheet"** export. An import is a later ticket.

The sheet, as the prototype renders it (an excerpt):

```
WotR build v1
Race: Human
Racial +2: Strength
Alignment: True Neutral
Background: Acolyte
Point buy: Str 17, Dex 10, Con 16, Int 10, Wis 10, Cha 10
Mythic path: Aeon
Path switch: Legend

Level 1: Fighter
  Combat feat: Armor Focus (Light)
  Bonus feat: Alertness
```

## Build

The user is awake: settle small choices yourself and record them here; bring a real open question back.

- **The renderer lives in `rules`:** a pure `Engine` method (for example `build_sheet(&Build) -> String`), built test-first (`tdd`), using the names a player sees (`labels.rs`, `Engine::slot_label`), so it's tested natively and the UI only copies the text. Start from the prototype in `build-file-review/sheet/`, and read it against the code rather than copying it.
- **What it holds:**
  - **Header:** `WotR build v1`, then the full `#b=` link on its own line, so the sheet always reopens the exact Build.
  - **The start:** race, racial bonus, alignment, deity (ticket 86), background, point buy.
  - **The mythic path and switch.**
  - **Each level:** the class (with its archetype where it's first taken) and every pick by slot label, ability increases, and skill ranks. Skills are 28–52 % of a sheet; keep them on one line per level.
  - **The mythic track:** each rank and its picks.
- **Derived values** (BAB, saves, ability totals) appear, if at all, only as clearly marked comments, never as data. The map keeps a Build free of derived values, and a later import must ignore them. Decide what's worth showing and record it.
- **Deterministic:** the same Build always renders the same text, in level order, then slot order as the sheet shows it.
- **The UI:** a "Copy build sheet" action on the sheet page (and, cheaply, "Download as .txt"). It copies the text to the clipboard and confirms. It works on the Fold's cover and inner screens without hover.
- **Out of scope:** importing a sheet, and changing the link format.

**Done when:**
- `build_sheet` has `rules` tests on the small world, and real-data tests in `libs/seeds/tests/game_data/` that render the six bench Builds (`tools/proof/bench-links.tsv`) and pin their sheets (or the parts that matter).
- The sheet's second line opens the same Build (a test decodes it back).
- The browser check: a release build on its own port, with the copied text matching the renderer's, and no console errors.
- `scripts/check.sh` passes, and `--full` once at the end.
