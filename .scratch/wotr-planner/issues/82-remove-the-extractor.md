# Remove the extractor

Type: task
Status: open
Blocked by: [81](81-apply-the-extractor-to-the-files.md) (and the user's answers to its questions)
Map: [WotR Build Planner](../map.md)

## Question

The data files are the master ([ticket 75](75-game-data-master.md), user, 2026-11-03). Once ticket 81 has brought what only the extractor knew into the files, the extractor has no job left: the saved guide pages don't change, and every data fix is a hand edit with a real-data test.

## Build

**Goes:**
- `libs/extract` (the crate and its tests) and its workspace member in the root `Cargo.toml`.
- `scripts/extraction-diff.sh`, and its sentence in `CLAUDE.md`'s "Behaviour proof".
- Every other mention that treats the extractor as current:
  - the map's Notes: "Game data", "Skills to use" ("`tdd` for … the guide extraction"), "Extraction in stages" (keep it as history, in the past tense);
  - `libs/seeds/Cargo.toml`'s comment ("used only by `server` and `extract`");
  - `libs/game-data/README.md`: its `extract::…` mentions keep their ticket and decision text with the module name dropped, and its opening says how data changes now (edit the `.ron` files, add a README note and a real-data test).

  `git grep -n -e 'extract::' -e '-p extract' -e 'extraction-diff' -e 'libs/extract'` finds them; mentions in resolved tickets and the map's decision log are history and stay.

**Stays:**
- `guide-cache/`: agents still read the saved pages for facts.
- `libs/game-data/EXTRACTION_REPORT.md`, as the hand-kept list of open data gaps (the README already says to delete an item's line when it's fixed). Rename it only if the README's wording calls for it.

**Done when:**
- `libs/extract` and `scripts/extraction-diff.sh` are gone.
- The `git grep` above finds only history (resolved tickets, the map's decision log).
- `git diff HEAD -- libs/game-data` touches only `README.md`.
- `scripts/proof.sh rules <out-dir>` against `HEAD` reports no difference.
- `scripts/check.sh` passes.
