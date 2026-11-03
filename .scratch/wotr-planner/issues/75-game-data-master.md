# Decide whether the extractor or the data files are the master

Type: grilling
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The docs and the practice disagree about who owns the game data:
- **The docs** say the files are extracted once, then corrected by hand: the map's Notes ("Game data"), `libs/game-data/README.md` ("These files are the source of truth: fix data by editing them") and `libs/extract/src/lib.rs` ("It runs once … after that the data files are edited by hand").
- **The practice** keeps both in lockstep. Each data ticket from 59 to 67 edited `libs/game-data/` *and* added an extractor pass: ticket 67, for example, changed `classes/oracle.ron` and added `libs/extract/src/second_curse.rs` (138 lines) with its own `check()`, plus real-data tests in `libs/seeds/tests/game_data/`. Tickets 68–74 then refactored the extractor while keeping its output byte-identical.

So each fact is written about three times (file, extractor pass, test), and the extractor is maintained as if it were the master. The architecture review (2026-11-03) ranked this the largest simplification. Decide with the user (HITL, `grilling`), then write the build ticket and fix the three docs.

**First, the fact the decision hangs on** (find it before asking): does a fresh extraction of `guide-cache/` equal `libs/game-data/` today? Extract into a scratch dir outside the repo (never with `--out libs/game-data`) and `diff -r` it against `libs/game-data/`. List every difference, and whether each is a hand-fix the extractor lacks.

Options to put to the user:
- **The extractor is the master.** Files change only by re-running it; any remaining hand-fixes become extractor code. The real-data tests stay as the check.
- **The files are the master.** Freeze the extractor and delete the per-ticket passes (several thousand lines across `libs/extract/src`); new data tickets edit the files and add tests only. The extractor stays as a historical tool, or goes.
- **Keep both**, and write down why, so the docs match.

Blocks [ticket 77](77-one-prerequisite-walk.md) and [ticket 78](78-game-data-lookups.md): if the per-ticket passes go, most of their extractor sites go with them.

## Progress log

- 2026-11-03: a fresh extraction of `guide-cache/` (`HEAD` e7acd60, into a scratch dir) differs from `libs/game-data/` in 40 `.ron` files, 517 diff lines; the largest are `rogue` (57), `monk` (48), `shaman` (37), `hunter` (36) and `stalwart-defender` (34). Sampled differences are hand-fixes the extractor never learned:
  - from the hand-fix stages (2a2cdc3): the Eldritch Knight's spellcasting row and guide anchor;
  - from early tickets (24, 30, 32, and the key rename 0a9ec2d): Winter Witch's entry options, Icy Tomb's and Extra Channel's prerequisites, Destiny Beyond Birth's races, the sorcerer's bloodline feats at 7/13/19, the Lich's rank-2 feature order;
  - the oracle's "Wind" line, which the extractor writes and the files lack.
- From ticket 34 on, every data ticket changed both the extractor and the files; tickets 68–72 changed the extractor only. So the extractor reproduces the later tickets' work but not the earlier hand-fixes.
- Counted by direction: 191 hunks (27 only in the extraction, 24 only in the files, 140 changed); 264 lines only in the files, 253 only in the extraction. Nothing outside `libs/extract` uses it as a library.

## Answer

**The data files are the master** (user, 2026-11-03). The extractor's knowledge goes into the files as data, then the extractor is removed (the user's words: "apply the extractors declaratively on the files then later remove the extractors"). The rest took the recommendations (user, 2026-11-03):

1. **Applying** ([ticket 81](81-apply-the-extractor-to-the-files.md)): every hunk gets a verdict from git history. Where a hand-fix overrode the extractor, the file wins; extractor knowledge the files never got is applied by hand; anything unclear (the oracle's "Wind" line, for one) comes back as a question. `scripts/proof.sh data` lists the behaviour change.
2. **Removing** ([ticket 82](82-remove-the-extractor.md)): `libs/extract`, `scripts/extraction-diff.sh` and the docs that treat the extractor as current go. `guide-cache/` stays for agents to read, and `EXTRACTION_REPORT.md` stays as the hand-kept list of open data gaps. New data tickets edit the `.ron` files, add a README note and add real-data tests in `libs/seeds/tests/game_data/`.
3. **Re-scoping:** [77](77-one-prerequisite-walk.md), [78](78-game-data-lookups.md) and [80](80-pipeline-repeats-and-stale-notes.md) shrink to the code left after 82 and run after it; [79](79-game-files-without-sqlx.md) is closed. Order: 81, then 82, then 77, 78 and 80; 76 is independent.
