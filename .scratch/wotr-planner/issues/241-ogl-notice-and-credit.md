# The Open Game License notice and credit

Type: build
Status: resolved
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

Found in [234](234-game-blueprints-as-a-source.md) (2026-11-14), the user taking the recommendation:
- **What's open.** Owlcat's OGL notice (https://owlcat.games/OGLPFWotR) designates the game's mechanics as Open Game Content. Proper nouns, characters, dialogue, locations and artwork are Product Identity.
- **What the OGL requires.** Republishing Open Game Content means:
  - a copy of the licence with it (section 10);
  - its Section 15 copyright notice, updated with ours (section 6);
  - marking which parts are Open Game Content (section 8);
  - not using Product Identity, or a contributor's name to market it (sections 7 and 11).
- **What's owed.** The planner already publishes mechanics (prerequisites, class tables, numbers) from the guide, which comes from the game, on GitHub Pages. It has no licence file, notice or credit, so this is owed whatever the source.

## Change

- **The licence file.** A file at the repo root, such as `OPEN-GAME-LICENSE.md`, with the OGL 1.0a text and the Section 15 notice as Owlcat's page gives it, the Wrath of the Righteous CRPG Reference Document's line and Paizo's lines included, plus a line for this project.
- **The designation.** A short statement, in `README.md` and the licence file, that the game mechanics in `libs/game-data/` are Open Game Content. The planner's own text (summaries in our own words, code) is not.
- **On the site:** a small footer or About line on the published page linking to the licence and crediting Paizo and Owlcat, without using their marks to market the planner.
- **The game's names.** Names of feats, spells and items are Product Identity under the notice's "proper nouns". The planner names picks as the game does, as every fan tool does; this ticket notes that and changes nothing.

## Answer

**Decided** (user, 2026-11-14, in [234](234-game-blueprints-as-a-source.md): the notice and credit are owed whatever the source). The builder's decisions:

1. **The licence text** is taken from Owlcat's own page (https://owlcat.games/OGLPFWotR, fetched as saved HTML and converted to text, not retyped): the OGL 1.0a and the whole Section 15 (Wizards of the Coast, Paizo, the Necromancer Games and other third-party entries, the Kingmaker and Wrath of the Righteous CRPG Reference Document lines), word for word, typos included. Only the layout changes: one paragraph per item, and a space after each licence item's heading colon (the page's markup hid it).
2. **Our Section 15 line:** "WotR Planner, a build planner for Pathfinder: Wrath of the Righteous, (c) 2026, the WotR Planner authors; the game mechanics in libs/game-data/, derived from the works above." The owner's name is generic because no author name is recorded in the repo; the user may replace it.
3. **The designation,** at the top of `OPEN-GAME-LICENSE.md` and in a README "Licence" section: the mechanics in `libs/game-data/` are Open Game Content; the planner's own text and code are not; Product Identity as Owlcat's notice states it, short and in our words; the game's names are used to refer to it (the ticket's "game's names" note, nothing changed).
4. **On the site:** a `Footer` component (`libs/ui/src/footer.rs`, rendered at the end of the sheet page, `.site-footer`): a credit naming Owlcat and Paizo (no logos, marks or artwork) and a link to the licence. It wraps at phone width and the link has a thumb-sized tap target.
5. **Serving the licence, not linking GitHub:** the licence must travel with the content it covers, and a link to GitHub would leave the published site without it. `scripts/build-site.sh` copies `OPEN-GAME-LICENSE.md` to `open-game-license.txt` in the site root (Pages would offer a `.md` as a download; `.txt` shows in the browser), and the script's existing "every file the page names is in the folder" check covers the footer's relative link (`./open-game-license.txt`, so it works under `/<repo>/`). That change is outside `libs/ui/`, so the full `scripts/check.sh` ran. The dev server (`cargo leptos watch`) does not serve that file, so the link 404s there; only the built site has it.

**Tests** (`cargo test -p ui footer`, in `libs/ui/src/footer.rs`; the first run failed to compile, the footer's items didn't exist): `the_footer_credits_paizo_and_owlcat_and_links_the_licence_relative_to_the_page`, `the_licence_file_holds_the_licence_and_section_15_with_owlcats_lines_and_ours`, `the_readme_and_the_site_build_carry_the_licence`. The ui crate has no render-to-HTML test harness (SSR is the backend's feature), so the footer's text is pinned through its constants.

**Review fixes:** the footer link's height is now the stylesheet's `--tap` (44px, `inline-flex`, `min-height`), which the first build's padding missed (about 37px). The first test now pins the contract: `LICENCE_PATH` without its `./` is the file `scripts/build-site.sh` writes, and `planner.rs` renders `<crate::footer::Footer/>` (a source-text check, no render harness).

**Snapshot message:** none; the snapshot check no longer exists (235) and no rule or data changed.

**For the queue's checks:** `scripts/build-site.sh <dir>` was not run here (a release build, the main session's `--full`); its output should list `open-game-license.txt` and the page should name `./open-game-license.txt`.

## Done when

- The licence file and its Section 15 notice are in the repo.
- The README and the published page link to them.
- `scripts/check.sh --ui` (or `scripts/check.sh`, if more than the UI changes) passes.
