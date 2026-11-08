# The Mythic sigil design

Type: task
Status: open
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
