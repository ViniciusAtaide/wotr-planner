# Bring what only the extractor knows into the data files

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The data files are the master ([ticket 75](75-game-data-master.md), user, 2026-11-03), and the extractor goes once the files hold everything it knows ([ticket 82](82-remove-the-extractor.md)). Today a fresh extraction of `guide-cache/` differs from `libs/game-data/` in 40 `.ron` files: 191 hunks, 264 lines only in the files and 253 only in the extraction. Ticket 75's progress log samples them.

## Build

1. **Extract into a scratch dir outside the repo** with `HEAD`'s extractor (`cargo run --release -p extract -- --guide guide-cache --out <scratch>/fresh`, with `SQLX_OFFLINE=true`), and `diff -r` it against `libs/game-data/`, leaving out `README.md` and `EXTRACTION_REPORT.md`. Never write the extraction into `libs/game-data/`.
2. **Give every hunk a verdict**, from git history: `git log -S` on each side's text, across the renames (`-- '*.ron'`), tells you which commit brought each side in.
   - **(a) The file wins.** A hand-fix stage (17–20) or a ticket changed the file deliberately and the extractor never learned it. Also any order-only or formatting-only hunk. Keep the file.
   - **(b) The extractor wins.** The extractor has knowledge the files never got: the file side is the older one, and no ticket chose it over the extractor's. Apply it to the file **by hand**, as the files are edited from now on.
   - **(c) Unclear.** Neither history settles it, or the guide would have to be re-read to choose; the oracle's "Wind" line ([ticket 69](69-option-families-where-owned.md)) is one. Leave the file and bring it back as a question, with both sides and what you found.
3. **A real-data test per applied fact** (or group of related ones) in `libs/seeds/tests/game_data/`, unless an existing test already covers it. Add a README note for the applied facts, like the earlier tickets'.
4. **Check the behaviour change:** `scripts/proof.sh data <out-dir>` against `HEAD`. Each difference in its `compare.txt` must trace to an applied (b) hunk.

Leave `libs/extract` alone; ticket 82 deletes it. No schema change. Don't reseed: report that the dev database needs a reseed, and the main session does it when no other agent is running.

**Done when:**
- Every one of the hunks has a verdict, listed in the Answer as a table (file, hunk, verdict, the commit or reason), or in a scratch-dir TSV the Answer points to, with the counts per verdict in the Answer.
- Every (b) hunk is applied and every (c) hunk is a question in the Answer.
- A fresh `diff -r` leaves only (a) and (c) hunks.
- `scripts/proof.sh data` differences are all accounted for in the Answer.
- `scripts/check.sh` passes.

## Progress log

Scratch dir (outside the repo): `/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-7437-4fe9-8ef8-ebb8716f6fbe/scratchpad/t81/` (below, `t81/`).

- 2026-11-03: extracted `guide-cache/` with `HEAD`'s (35b2bb3) extractor into `t81/fresh` (`SQLX_OFFLINE=true`; 23 report items, 0 check problems) and diffed it against `libs/game-data/` without `README.md` and `EXTRACTION_REPORT.md` (`t81/diff.txt`). Same counts as ticket 75: 191 hunks, 264 lines only in the files, 253 only in the extraction. They are in 39 `.ron` files, not 40, and no file exists on one side only.
- 2026-11-03: ran `git log -S` on every line of every hunk, across the renames (`-- '*.ron'`; `t81/tools/hunks.py`, output `t81/hunks.json` and `t81/summary.txt`). The hand-fix commit 2a2cdc3 (stages 17–20) touched exactly the 39 files in the diff. Every hunk except one is one of that commit's fixes, listed in the Answers of tickets 17–20, or a later ticket's change to a line it fixed (t23 sparse rows, t24 list names, t30/t31 selections). Each ticket from 23 on rewrote the hand-fixed files with a script and taught the extractor the same rewrite on its own unfixed output. That is how the extractor came to reproduce the tickets' work but not the hand-fixes. Checked by hand: t30 turned the bloodrager archetypes' "Bloodline" features into the `bloodrager/bloodline` selection, t31 turned the Divine Hunter's "Domain" feature into a selection, and the wizard and warpriest archetype rows give the same effective rows as the extraction's.
- 2026-11-03: verdicts in `t81/verdicts.tsv` (`t81/tools/verdicts.py`): 190 (a), 0 (b), 1 (c). Nothing is applied, so no `.ron` file, test or README note changed.
- 2026-11-03: read the saved `shaman` and `oracle` pages for the (c) hunk (see the Answer).
- 2026-11-03: `scripts/proof.sh data t81/proof` against `HEAD` reports 1999 Builds on each side and 0 differing, and `compare.txt` is empty. A second `diff -r` is byte-identical to `t81/diff.txt`. `scripts/check.sh` passed fmt, clippy and wasm-clippy, then stopped at `test`: all 28 `seeds --test postgres` tests failed after 120 s. Those `#[sqlx::test]`s need the Postgres server in `DATABASE_URL` (`localhost:5434`), and the `wotr-planner-postgres` container isn't running (`docker ps` shows only `postgres` on 5432 and `redis`). Nothing in this ticket changed code or data, and I left the database alone as instructed. I ran the later steps by hand, and they pass: every other workspace test, `seeds`' `files` and `game_data` tests, the rules wasm build and `seed --check-only`.

## Answer

**Nothing is applied.** No hunk is extractor knowledge that the files lack. The extractor reproduces every ticket's rewrite, from 23 on, but none of the hand-fixes from stages 17–20 (2a2cdc3) or later tickets' changes to those fixed lines. So the files win everywhere except one hunk, which stays a question.

- **Counts:** 191 hunks: **190 (a)**, **0 (b)**, **1 (c)**. Of the (a) hunks, 6 are order-only (the rank-2 or rank-1 feature order of Aeon, Angel, Azata, Demon, Lich and Trickster, left by t24's in-place rename). The rest trace to the hand-fix commit's items in tickets 17–20 or to a ticket's deliberate choice: t24's list names and anchors, t30's bloodrager bloodline selection and t31's Divine Hunter domain selection. Ticket 75's sampled hunks are all among them.
- **Table:** `/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-7437-4fe9-8ef8-ebb8716f6fbe/scratchpad/t81/verdicts.tsv`. It has one row per hunk: number, file, diff range, verdict, the commit and ticket item, and the first line of each side. Built by `t81/tools/verdicts.py` from `t81/hunks.json` (`git log -S` per line).
- **Applied facts and tests:** none, so no test or README note was added and no `.ron` file changed.
- **`scripts/proof.sh data`:** 0 differences (1999 Builds each side), as expected with no applied hunk.
- **Fresh `diff -r`:** unchanged, the same 190 (a) hunks and 1 (c) hunk.

### Question (c): hunk 66, `classes/oracle.ron`, the Wind Whisperer's "Wind" feature

- **Extraction side:** defines the feature `(key: "oracle/wind-whisperer/wind", name: "Wind", guide: "oracle#wind-whisperer")` and grants it on no row (`Pick::SelectionKeepingFeature`). It keeps the feature only because its raw shaman hexes Air Barrier, Vortex Spells and Wind Ward resolve their "Wind" line to it (hunks 102 and 108).
- **File side:** no such feature. 28117a0 (t31) removed it and the level-1 grant, and replaced them with the `oracle/wind-whisperer/mystery` selection, which offers only `oracle/mystery/wind`. The hand-fix (2a2cdc3, ticket 18) points the three hexes at `SelectionOption("shaman/spirit/wind")`. No file references the feature.
- **Guide (saved pages):**
  - `shaman.html`: Air Barrier, Vortex Spells and Wind Ward each list "Prerequisites: Wind" among the spirit hexes, alongside "Battle", "Flame" and so on, so it means the Wind spirit.
  - `oracle.html`: the Wind Whisperer "Must select the Wind Mystery". Its table's "Wind" cell at level 1 is that mystery.
- **History:** ticket 69 recorded the user's decision (2026-11-03): "the game data's hand fix is the answer".
- **Recommendation:** (a). Keep the file as it is, since the feature would be defined and never granted. This ticket names the line as (c), so I left it for you to decide. Ticket 82 deletes the extractor, and `Pick::SelectionKeepingFeature` goes with it.
- **Lead, 2026-11-03: settled as (a)** by the user's decision on ticket 69 ("the game data's hand fix is the answer"), so the final count is 191 (a), 0 (b), 0 (c). Lead check: a sample of (a) verdicts (the rogue archetypes' Advanced Talents, the Stalwart Defender's "+1" features, the Gold Dragon's Improved Dragon Breath) are removals in 2a2cdc3's diff, as the table says.

### What's left

- `scripts/check.sh` must pass with `wotr-planner-postgres` (port 5434) up: rerun it. Status stays open until then, and until the question above is answered.
- No reseed is needed, because the data is unchanged.
- **Lead, 2026-11-04: done.** With `wotr-planner-postgres` started (user), `scripts/check.sh` passes (ALL-OK, the 28 Postgres tests included). Ticket 82 is unblocked.
