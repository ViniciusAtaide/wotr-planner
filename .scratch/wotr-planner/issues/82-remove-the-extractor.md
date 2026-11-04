# Remove the extractor

Type: task
Status: resolved
Blocked by: none
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

## Progress log

Scratch dir (outside the repo): `/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-7437-4fe9-8ef8-ebb8716f6fbe/scratchpad/t82/` (below, `t82/`).

- 2026-11-04: the ticket's `git grep` at `HEAD` (dc23204) found, outside the tickets: `CLAUDE.md`'s sentence, the root `Cargo.toml` member, `libs/extract` itself, `scripts/extraction-diff.sh`, 24 `extract::…` mentions in `libs/game-data/README.md`, and `apps/backend/src/game_data.rs`'s `use axum::extract::Path` (Axum's extractors, not ours). A wider `git grep -i -w 'extract|extractor|extraction'` also found the map's Notes, `libs/seeds/Cargo.toml`'s comment, the root `README.md`'s "`libs/game-data/` is empty until the guide is extracted", one README module mention without the `extract::` prefix (`option_class_skills::cleanup`, ticket 38) and "the module asserts this" (ticket 36's druid note). Nothing in `scripts/check.sh`, `scripts/proof.sh`, `.githooks/` or another crate's `Cargo.toml` names the crate; no workspace dependency was used only by it.
- 2026-11-04: `git rm -r libs/extract scripts/extraction-diff.sh` (108 + 1 files); dropped the workspace member. `Cargo.lock` lost `extract` and its 26 dependencies only it used (`scraper`, `html5ever`, `selectors`, …).
- 2026-11-04: rewrote the docs (see the Answer). Open tickets 77 and 78 said "ticket 82 deletes them"; made it "deleted", so their `libs/extract` mention reads as history.
- 2026-11-04: `scripts/proof.sh rules t82/proof` against `HEAD`: 1999 Builds each side, 0 only in A, 0 only in B, 0 differing; `compare.txt` empty (log `t82/proof.log`). `scripts/check.sh` with `wotr-planner-postgres` up: fmt, clippy, wasm-clippy, test (the 28 Postgres tests included), rules-wasm and seed-check OK, ALL-OK (log `t82/check.log`).

## Answer

**The extractor is gone.** No data, schema or database change; no reseed needed.

- **Deleted:** `libs/extract` (the crate, its 108 tracked files: sources and the `tests/extract/` crate) and `scripts/extraction-diff.sh`, by `git rm`; the `libs/extract` workspace member in the root `Cargo.toml`. `Cargo.lock` dropped `extract` and the 26 packages only it pulled in (`scraper`, `html5ever`, `selectors`, `cssparser`, …). No workspace dependency was the extractor's alone, so `[workspace.dependencies]` is unchanged.
- **Docs changed:**
  - `CLAUDE.md`, "Behaviour proof": the `extraction-diff.sh` sentence is gone.
  - `libs/seeds/Cargo.toml`: "used only by `server` and `extract`" is now "`server` and `proof`" (`libs/proof` depends on `seeds`).
  - `libs/game-data/README.md`: the opening says the files are the master (ticket 75), were extracted once and are hand-corrected, the extractor is gone (ticket 82), and how data changes now: edit the `.ron` files, add a note under Layout, add a real-data test in `libs/seeds/tests/game_data/`, check and reseed. `EXTRACTION_REPORT.md` is described as the hand-kept list of open data gaps that began as the extractor's leftovers. The 24 `extract::…` mentions, plus ticket 38's `option_class_skills::cleanup`, lost the module name and kept their ticket and decision text; the backgrounds row names ticket 34 instead; the paragraph that ended "`extract::option_levels` reads all this from the guide" ends "All this was read from the guide"; ticket 36's "the module asserts this" is now "a real-data test asserts this" (`no_druid_can_take_impossible_domain_whatever_its_domain` in `libs/seeds/tests/game_data/option_families.rs`). "Which the extraction passed over" (tickets 62, 63) is history and stays.
  - The map's Notes: "Game data" says the extractor was removed and how a data change is made; "Extraction in stages" is in the past tense, marked as history since this ticket; "Architecture review" says 81 checked and 82 removed; "Skills to use" drops "the guide extraction" from `tdd` and names the real-data test. A decision-log line for this ticket is added; the existing lines are untouched.
  - Root `README.md` (decided here, small): "`libs/game-data/` is empty until the guide is extracted, so for now seed the hand-written fixture" was stale, so the run step seeds `libs/game-data/` and the fixture is described as an example you can seed instead.
  - Open tickets 77 and 78: "ticket 82 deletes them" became "deleted".
- **Stayed:** `guide-cache/` (agents still read the saved pages); `libs/game-data/EXTRACTION_REPORT.md` unchanged, name included: the README calls it the hand-kept list of open data gaps and explains the name, so nothing calls for a rename; every `.ron` file. `git diff HEAD -- libs/game-data` touches only `README.md`.
- **`git grep -n -e 'extract::' -e '-p extract' -e 'extraction-diff' -e 'libs/extract'`** finds, outside the tickets, only the map's decision-log line for this ticket and `apps/backend/src/game_data.rs`'s `use axum::extract::Path` (Axum's extractor module, unrelated). In the tickets: resolved ones (history), this ticket, closed ticket 79 (history), and open 77 and 78, whose one mention now says ticket 82 deleted the copies (history).
- **Checks:** `scripts/proof.sh rules` against `HEAD`: no difference (1999 Builds, 0 differing, empty `compare.txt`). `scripts/check.sh`: ALL-OK.
