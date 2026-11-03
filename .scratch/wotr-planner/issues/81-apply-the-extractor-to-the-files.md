# Bring what only the extractor knows into the data files

Type: task
Status: open
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
