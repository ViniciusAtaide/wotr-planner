# Copy a Build as a readable build sheet

Type: task
Status: resolved
Blocked by: [86](86-deities.md) (resolved; it changed the Build start and the start panel, and the sheet shows the deity)
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

## Choices settled

Each one the recommended answer (or, where the ticket gives none, the smallest choice consistent with the map's Notes), taken while the user slept, 2026-11-09.

1. **The prototype was found and read against the code.** `build-file-review/` was still in the earlier session's scratchpad (`/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-…/scratchpad/`). `Engine::build_sheet` keeps its layout. It reads each level's archetype from the sheet's class level (as the grid does) instead of a set of classes already seen, and adds the link, the deity, the subrace's own line and the first ascension. For the 20- and 40-level bench Builds its output is the prototype's, apart from the changes below.
2. **The signature is `Engine::build_sheet(&Build, page: &str) -> String`, in `libs/rules/src/engine/build_sheet.rs`.** `page` is the planner's address before the fragment. The engine doesn't know where it is served, so the caller passes it: the UI passes `location.origin + location.pathname`, and the real-data tests pass the GitHub Pages address. Line 2 is `{page}#b={encode_build(build)}`. The blank Build gets a code as well, which reopens it.
3. **The Build start comes in the game's order, the order of the start panel:** Name (when the Build has one), Race, Subrace (its own line, since the test world's subrace names don't name the race), Racial +2 (only for a race that chooses it), Deity (the deity's name or "Atheism", the panel's label), Alignment, Background, Point buy. The ticket listed the alignment before the deity, but the game and the panel ask for the faith first (ticket 94). The point buy always shows, all 10s included, since every score has a value. Any other choice not made yet is left out.
4. **The path choices close the header:** First ascension (its picker's name, `Engine::first_ascension_name`, and only when chosen), Mythic path, Path switch. The ticket puts the path and the switch in the header. The first ascension came after the prototype (ticket 91) and sits beside them.
5. **The levels:** `Level N: Class`, the archetype in brackets where the class is first taken, and a prestige class's advanced classes after it (`, advancing Cleric and Wizard`, in key order). Then the ability increase as `Ability: +1 Strength` (the grid chip's label, in the `label: value` shape of the other lines, where the prototype had `+1 Strength`). Then each pick as `slot label: pick name`, in slot order as the sheet checks picks (`Engine::slot_label`, `Engine::pick_name`). Last, `Skills: …` on one line, in skill key order. A level with nothing chosen shows only its class, and each level follows a blank line.
6. **The Mythic track:** a `Mythic track` heading (the glossary term), then each rank with a pick as `MR N (level L)` and its picks in slot order. Ranks follow one another without blank lines. A rank with no pick is left out, and so is the heading when no rank has a pick.
7. **Derived values: only one, and as a comment.** A pick a Path switch takes away (the track's `dropped_slots`) ends with ` # lost to the Path switch`. A player who follows the sheet takes that pick and loses it at MR 8, so the comment tells them what to expect. Comments run from ` # ` to the end of the line, and a later import ignores them. BAB, saves and ability totals are left out: the player doesn't enter them in the game, the game shows them, and leaving them out keeps the sheet to the choices, as the map's Notes keep the Build to its choices.
8. **The UI:** a bar under the header with "Copy build sheet" and "Download .txt" (`libs/ui/src/export.rs`, `BuildSheetActions`). Both are plain buttons, so they work by touch on both Fold screens. They render the sheet only when tapped, not on every edit. The copy goes through `navigator.clipboard.writeText`. Without a clipboard (a page served over plain HTTP from another host), the message says "Couldn't copy here: try Download .txt". The download is a Blob link named after the Build (`spear-of-the-balance.txt`), or `wotr-build.txt` for an unnamed Build. A message in a `role="status"` span confirms each action ("Build sheet copied", "Build sheet saved") and clears after 3 s.
9. **Real-data pins: the parts that matter, not whole golden files.** The blank and start bench sheets are pinned whole. The 20- and 40-level sheets pin their headers, chosen level blocks, class order and Mythic track, and every bench sheet is checked for one line per level, one line per pick and a link that reopens it. Whole golden files of the long sheets would break on every rename in the data, and several tickets are editing the data in parallel.
10. **The sheet's link holds the Build as the planner opened it.** The bench links predate deities and open as atheists (as `proof`'s bench Builds do), so their sheets' links carry the faith and differ from the bench links.

## Progress log

- 2026-11-09: wrote the tests first (`tdd`). Nine `rules` tests in `libs/rules/tests/engine/build_sheet.rs` (small world) failed to compile without `Engine::build_sheet`. Then the renderer, `libs/rules/src/engine/build_sheet.rs`, and two of the tests' own assumptions corrected (the test world names its feats by key; MR 1 is at level 6). The tests cover: the blank Build; the start in the game's order; atheism; picks by slot label in slot order, with the ability increase and the skills line; the archetype and the advanced classes where the class is first taken; the path choices and the Mythic track; the switch comment; and the second line reopening the Build (`open_link`), with the same text on every render.
- Six real-data tests in `libs/seeds/tests/game_data/build_sheets.rs` render the six bench Builds of `tools/proof/bench-links.tsv` (those made before deities open as atheists, as in `proof`). For every bench Build: the second line reopens it and renders the same text again; there is one `Level N:` line per level, in order; there is one indented line per pick. Pinned: the blank and start sheets whole; for the 20- and 40-level Builds their headers, levels 1, 4 and 40, the multiclass class order, and the Legend track (MR 1–7, with the 10 picks of MR 3–7 marked lost). Against the prototype's sheets of the same Builds, the only differences are the intended ones: the link, the deity, `Ability: +1 …`, the `Mythic track` heading, rank lines without a colon, and the lost comments.
- The worktree's database copy was older than ticket 130's migration, so the query macros failed. I rebuilt it with CLAUDE.md's four steps; `.sqlx` is unchanged.
- The UI: `libs/ui/src/export.rs` (`BuildSheetActions`, under the header), clipboard and download helpers in `libs/ui/src/browser.rs` (web-sys `Navigator`, `Clipboard`, `Blob`, `BlobPropertyBag`, `Url`, `HtmlAnchorElement`, `HtmlElement`), and the `.export` styles. `GLOSSARY.md` gains **Build sheet**.
- The browser check: `cargo leptos build --release --precompress`, `target/release/server` on 127.0.0.1:3911 with its MCP server on 3912 (the release server also serves MCP, by default on the user's 3902), opened in the built-in browser with the `fighter20` bench Build's sheet link.
  - The text "Copy build sheet" passes to `navigator.clipboard.writeText`, captured by a wrapper, is byte-identical (3,564 bytes) to `Engine::build_sheet`'s output for that Build with the page's address.
  - The pane denies clipboard writes (`NotAllowedError`), and the bar showed its fallback, "Couldn't copy here: try Download .txt". With the clipboard stubbed to accept the text, it showed "Build sheet copied", cleared after 3 s.
  - "Download .txt" made a `text/plain;charset=utf-8` Blob of the same text, named `wotr-build.txt`, and showed "Build sheet saved".
  - At 360×800 (Fold cover) and 900×1000 (inner) both buttons sit on one row, 44 px tall, with no horizontal scroll (`scrollWidth` equals the width). On the cover screen the message wraps below them.
  - No console messages. The size was reset afterwards and the server stopped.
- `scripts/check.sh`: ALL-OK (fmt, clippy, wasm-clippy, the workspace tests, rules-wasm, seed-check). `rules`' engine tests: 508, 9 of them new; `seeds`' game-data tests: 6 new. No migration and no query change, so no `--schema` run.

## Answer

Done 2026-11-09. `Engine::build_sheet(&Build, page)` renders a Build as a build sheet: `WotR build v1`, the Build's full link (which reopens it exactly), the start in the game's order (deity included), the path choices, each level's class and picks by slot label, ability increase and skills on one line, then the Mythic track. The one derived fact is a pick a Path switch takes away, shown as a ` # ` comment. On the sheet page, "Copy build sheet" copies it and "Download .txt" saves it, each confirmed by a short message. The `#b=` link stays the share format. Importing a sheet remains a later ticket, and so does giving the MCP server's `sheet` tool the text (out of scope here).
