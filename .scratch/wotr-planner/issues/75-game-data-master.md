# Decide whether the extractor or the data files are the master

Type: grilling
Status: open
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
