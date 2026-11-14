# The Open Game License notice and credit

Type: build
Status: open
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

## Done when

- The licence file and its Section 15 notice are in the repo.
- The README and the published page link to them.
- `scripts/check.sh --ui` (or `scripts/check.sh`, if more than the UI changes) passes.
