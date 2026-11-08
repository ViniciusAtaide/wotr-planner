# The Mythic sigil design

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The user explored three design directions over the real sheet on the throwaway branch `prototype/design-language` (2026-11-08):
- `apps/frontend/style/prototype-theme.css`, where each direction is a block keyed on `:root[data-variant=…]`;
- `apps/frontend/public/prototype-theme.js`;
- `apps/frontend/public/prototype-compare.html`, the directions side by side.

The user chose **C, the Mythic sigil**: a night sky with faint stars, glassy surfaces, a row card per level, Marcellus headings, a gold accent, and each Mythic path in its own colour. This ticket builds it into `main` properly. The prototype's CSS is a reference, not code to paste: it was written as overrides on top of `main.css`, with a JS observer tagging the paths.

Settled with the user (2026-11-08):
- **Dark-only.** The site stops following the system's light setting: no light theme.
- **Marcellus is self-hosted.** It's open-licence (SIL OFL 1.1). Download its woff2 from Google Fonts (the user approved this download) and serve it with the site, so visitors' browsers never contact Google.

## Build

- **One theme in `main.css`.** Replace the light `:root` tokens and the `prefers-color-scheme: dark` block with C's tokens, as the only theme, and keep `color-scheme: dark`. Fold C's rules into the sections they style (frame, Build start, tabs, the grid, the Mythic track, the picker, the alignment wheel) rather than appending an override block. Keep the file's section layout and comments. Keep the feat icons' accent tint (`.chip-label .feat-icon`), which takes C's gold.
- **The row cards** (C's `.grid` gap and `.row` card) must keep the grid's density on a 375 px screen. Compare a level row's height with today's on the same Build: it may grow by the card's padding, not by wrapping more chips.
- **Each Mythic path's colour, from real markup.** Today the prototype's JS guesses the path from the text.
  - The UI puts the path's key on what it styles, as `data-path="<key>"`: a rank row (`mythic.rs`, `class="row rank"`), its path name (`rank-path`), the path chips (`path-chip`), and the Levels grid's Mythic badge (`grid.rs`, `mythic-badge`, the Build's path at that level, if it has one).
  - The keys are the data's: `aeon`, `angel`, `azata`, `demon`, `devil`, `gold-dragon`, `legend`, `lich`, `swarm-that-walks`, `trickster`.
  - The CSS maps each key to its colour (the prototype's picks; note `swarm`'s key is `swarm-that-walks`), with the plain mythic colour when there's no path.
  - The key comes from the engine's types the UI already reads. If the UI has no path key at a site, add the smallest engine accessor and test it in `rules`.
- **Marcellus, self-hosted.**
  - Download the latin subset woff2 from Google Fonts (fetch the css2 URL for `family=Marcellus` with a modern browser User-Agent, then the woff2 it names) into `apps/frontend/public/fonts/marcellus.woff2`.
  - Add the licence beside it, `apps/frontend/public/fonts/OFL.txt` (the SIL Open Font License 1.1 text with Marcellus's copyright line).
  - `@font-face` in `main.css`, with `font-display: swap` and a URL relative to the stylesheet. It's served at `./pkg/wotr-planner.css`, so the font is at `../fonts/marcellus.woff2`, which also works under the Pages sub-path.
  - Headings use `Marcellus, serif` as in C.
  - `scripts/build-site.sh` already copies `apps/frontend/public/`. Make it fail if a `url(...)` in the stylesheet names a file the site lacks, as it does for the page's files.
- **The link preview image** stays as it is: it already uses a dark palette, and updating it is a separate change.
- **Glossary:** no new terms.

**Done when:**
- **Browser check** at 375 px and at desktop width, on a full Build (the Enforcer link in `.scratch/wotr-planner/issues/88-qa-full-builds.md`), compared against the prototype's C (`?variant=c` on the prototype branch). Check:
  - the Levels grid;
  - the Mythic track, with Azata in pink;
  - a second path in its own colour: switch the path or use another Build;
  - an open picker, its filters, and a locked list;
  - the Build start and its fields;
  - the alignment wheel;
  - no console errors.
- **The font loads** from the site itself (`read_network_requests`), and no request goes to Google.
- **`scripts/build-site.sh`** builds the site with the font in it, and the site served under `/wotr-planner/` (`python3 -m http.server`) loads the font.
- **`scripts/check.sh`** passes, and `--full` once at the end.

## Choices settled

1. **No engine accessor.** Every site already reads a path key from the engine's types:
   - a rank row and its path name: `RankSheet::path`;
   - the first ascension chip: the same `RankSheet::path` at MR 1;
   - the path and switch chips: `Build::mythic_path` and `Build::path_switch`;
   - the grid's Mythic badge: `LevelSheet::state.mythic_path`, the path at the end of that level. So the badges before MR 3 have no path and take the plain mythic colour.

   The UI writes `key.as_str()` as `data-path`. An empty path chip has no key. CSS resets its `--path` to the plain mythic colour (`.path-chip:not([data-path])`), so an empty switch chip doesn't take its rank's path colour.
2. **The row cards keep the chip width.** C's card takes 12 px across (5 px padding and a 1 px border each side). On the Enforcer at 375 px, the tightest line had 8 px to spare. So `.grid` reaches 6 px into the gutter on each side (`margin: 0 -6px`). A row's content keeps today's width (221 px of chips) and still lines up with the summary above. The prototype's C lost those 12 px (209 px of chips), and its level 1 wrapped onto a fourth line.
3. **Marcellus in one grouped rule** in the base section: `.top h1`, `.section-title`, `.tabs button`, `.row-number`, `.picker-title h2`, `.alignment-name`. Those rules lost their own `font-weight`, so C's 400 holds. The title is 20 px at every width, so the 18 px desktop override is gone.
4. **`backdrop-filter` only on the large surfaces:** the Build start, the sticky tabs and the picker. C also blurred every button. On a full Build that is a hundred or more blurred chips while scrolling on a phone, and over a nearly flat background the blur is invisible. Buttons keep C's 6 % white fill.
5. **`<meta name="color-scheme">` is `dark`** (was `light dark`), so form controls and scrollbars match the one theme.
6. **Two layout fixes found by the browser check:**
   - "MR 10" in 17 px Marcellus is 54.6 px wide and wrapped in the track's 50 px number column on a phone. The column is now 58 px, and the number doesn't wrap.
   - At 600 px and up, a rank's features take a whole row, so its chips fell into the 60 px number column (on `main` and in the prototype alike). The features and chips now run from the level column to the end.
7. **The font:** the latin subset (14,272 B, Google Fonts v14) as `apps/frontend/public/fonts/marcellus.woff2`. `OFL.txt` beside it has the copyright line from the font's own name table, then the SIL OFL 1.1 text. `@font-face` has `font-display: swap` and `url("../fonts/marcellus.woff2")`, with no `unicode-range`. Glyphs the subset lacks fall back to `serif`.
8. **`build-site.sh` checks the stylesheet's `url()`s.** Each must name a file inside the site, relative to `pkg/wotr-planner.css`. `data:` URIs are skipped. A path from the host's root, or any URL with a scheme (another host, Google included), fails the build. Tested against fake sites: a good font and a data URI pass; a missing file, `/fonts/…`, `https://fonts.gstatic.com/…` and `../../../…` out of the site fail.

## Progress log

- 2026-11-08: started in a worktree fast-forwarded to 1331e25. Read `CLAUDE.md`, the map's Notes, this ticket, the prototype's C block and `main.css`.
- 2026-11-08: baseline on the Enforcer (`#b=AdVNPUhC…`, ticket 88) at 375 px, before any change. Level rows were `151,151,55,103,55,151,151,103,103,55,103,151,199,103,103,55,103,55,103,103` px tall, with `3,3,1,2,1,3,3,2,2,1,2,3,4,2,2,1,2,1,2,2` lines of chips. The chips column was 221 px.
- 2026-11-08: downloaded the Marcellus css2 (Chrome User-Agent) and its latin woff2. Read the copyright from the font's `name` table.
- 2026-11-08: built it: the tokens, the folded rules, the `data-path` markup, the `@font-face`, the `build-site.sh` check.
- 2026-11-08: browser check (own tab, dev server on 3940):
  - At 375 px the rows are `148,148,52,100,52,148,148,100,100,52,100,148,196,100,100,52,100,52,100,100` px, with the same lines of chips and the same 221 px. Each card is 3 px shorter than the old row; with the 6 px gap, each row takes 3 px more height. The prototype's C on the same Build: 209 px, level 1 on four lines (196 px).
  - Mythic track: MR 1–10 and every path chip in Azata pink (`#f28fd2`). The empty switch chip has the dashed gold border.
  - A second path: the first ascension set to Trickster's Bit of Fun gives MR 1–2 in Trickster violet (`#b78cff`), MR 3 on in Azata pink. Path switch to Legend: its chip in Legend (`#d8c49a`).
  - Grid badges: plain mythic (`#b9a2ff`) at levels 6–7, Azata from level 9.
  - Also checked: the Build start and its fields, the alignment wheel, the level 19 feat picker (12 filters, 77 open and 184 locked) and a centred picker at 1280 px.
  - The font comes from `127.0.0.1:3940/fonts/marcellus.woff2` (`document.fonts`: Marcellus loaded). No request went to Google or gstatic. No console errors.
- 2026-11-08: `scripts/build-site.sh` built a 3.1 MB site with `fonts/marcellus.woff2` and `fonts/OFL.txt`. Its stylesheet names `url(../fonts/marcellus.woff2)`, and its page says `color-scheme: dark`. Served by `python3 -m http.server` under `/wotr-planner/`, the Enforcer loads: every request is a 200, including `/wotr-planner/fonts/marcellus.woff2`; Marcellus loaded, the badges are Azata pink, and there are no console errors.
- 2026-11-08: `scripts/check.sh` ALL-OK; `scripts/check.sh --full` ALL-OK.

## Answer

Done 2026-11-08. The Mythic sigil is `main.css`'s one theme, dark only. C's tokens replace both old palettes, and its rules sit in the sections they style. Marcellus is self-hosted beside its licence, and no request goes to Google.

Each Mythic path's colour comes from real markup: `data-path="<key>"` on the rank rows, their path names, the path chips and the grid's Mythic badge. The keys come from engine types the UI already read, so `rules` gained nothing.

At 375 px on the Enforcer, no level row wraps another line of chips. Each card is 3 px shorter than the old row, and the 6 px gap between cards makes each row take 3 px more height. In the prototype, level 1 wrapped onto a fourth line.

`build-site.sh` now fails on a stylesheet `url()` the site lacks, or on one from the host's root or another host. The site built and loads its font under `/wotr-planner/`.

Two layout fixes came out of the check: "MR 10" no longer wraps on a phone, and the Mythic track's chips no longer fall into the number column from 600 px up. The second was already broken on `main`.
