# Rescue retros

The `subagent-rescue` skill appends one entry per rescue: the ticket, the minutes it took, what made it slow, and the change that would have prevented it, marked applied or proposed. A cause seen twice gets a mechanical fix.

## 2026-11-17: ticket 306i (exploits, arcana, mercies), about 65 minutes

- **Slow because:** it stalled 33 minutes, with no writes and no compiles, before its first full test run. Then that full `seeds` `game_data` run took about 5 minutes and showed 11 failures, which it iterated with full runs rather than filtered ones. It had also set `Status: resolved` before writing any Answer.
- **A move away from the game, caught by the rescue:** moving the six greater exploits into the ordinary exploit lists dropped the game's `ArcanistGreaterExploitsFeature` requirement. The brief didn't say that moving an option into a wider list carries over its owner's gating.
- **Changes:**
  - (proposed) `ticket-builder.md` step 3: iterate with `cargo test -p <crate> --test <bin> -- <filter>`, and run a crate's whole suite only at the end. The step says this already; repeat it in briefs for data slices, whose suites are slow.
  - (applied: research/306/decisions.md) Family-slice briefs: an option moved from a gated selection into a wider list keeps the gate as a `Requires` (cite it), or the census shows offers moving away from the game.
  - (applied) `ticket-builder.md` step 8: set Status only together with the Answer, in one edit.
  - (applied) The rescue helper and its 30-minute watchdog in the `ticket-queue` skill.

## 2026-11-17: ticket 306j (hexes, with the Sylvan Trickster's fey tricks), about 120 minutes

- **Verdict:** not stuck. It committed `421d1e28` at 03:24 with the full `seeds` and `corpus` suites passing and the game check at exit 0 (0 differences not known). The rescue had nothing to unblock.
- **Slow because the machine slept.** The build took about 33 minutes of work (01:25 to 01:58). Its final full run started at 01:58 and ended at 03:05, but the tests' own times are 45 s (`links`) and 183 s (`game_data`). `pmset -g log` shows the Mac, on battery, in "Maintenance Sleep" from 02:14:49, waking for a few seconds about every 15 minutes. A frozen process looks like a hung one: `ps` showed `links` at 16 minutes elapsed and load averages of 83 to 112 at each wake. The rescue's own read-only commands took 15 to 35 minutes each for the same reason.
- **Second cause, seen in 306i too:** the tests that pin a shape were found only by a full run. The first full run (cut at 01:52) showed 15 failures: about 10 pins of a count or a key (262 feats, 36 repeating feats, 634 `Requires`, the Hexcrafter's row 4, the Extra feat tables), and 5 real omissions (the new feat missing from `selections/feats.ron` and `gold-dragon/dragon-feats.ron`; `witch-wandering-hex`'s any-of and its planner selection missing the eleven copies and the channel). The section in `slices.md` named one test. All 15 were fixed in two minutes once seen. That log held no failure messages, since the run was cut before its summary.
- **Left for the reviewer** (none blocks the commit):
  - a Hexcrafter still takes `extra-hex` but gets no slot (the game refuses the feat: `ExtraWitchHexSelection` b6054088b4ab4be286724127cbf48b35 needs `WitchHexSelection` or `WinterWitchWitchHexSelection`; 264's `known.txt` line);
  - the corpus generators still build a shaman's and a Hexcrafter's Extra Hex with `extra-hex` (`tools/corpus/src/builds/feats.rs:47`, `talents.rs:798`), so a regenerated corpus has 2,155 Builds, not 2,159, and 3 that no longer open;
  - `witch/extra-hex` holds no list of its own (the writer skips a selection with none; the game's holds 27), which 306aa needs;
  - the Answer doesn't explain the census's `ok -> BreaksLater` groups (the 40 largest, in 24 to 28 Builds) or `witch-wandering-hex` going to Unmet in 21 Builds.
- **Changes:**
  - (applied) `ticket-queue` skill: keep the machine awake for the length of a queue (the app's keep-awake request, or `caffeinate -i` around it), and say so before dispatching on battery.
  - (applied) `subagent-rescue` skill, step 2: before judging a process hung, compare its elapsed time with the test's own "finished in" and read `pmset -g log` for `Sleep` lines since the builder started. When the machine slept, the verdict is "wait" and no rescue is needed.
  - (applied: `scripts/pinning-tests.sh`, a cause seen twice: a mechanical fix) a script, `scripts/pinning-tests.sh <key>…`, that lists the test functions in `libs/seeds/tests/game_data/` naming a selection, feat or option key the slice touches, or the totals it moves (the feat count, the repeating feats, the `Requires` count). The research puts its output in the section's **Tests**, and the builder runs those filtered before the one full run.
  - (queued on ticket 306ab) `game-check --write option-lists`: list a selection named in `--only` that it skipped for holding no list, rather than skipping it silently.
  - (proposed) the form proof's `compare`: when the regenerated corpus loses Builds or holds Builds that no longer open, name them and the generator's line, so the slice updates the generator.

## 2026-11-17: ticket 306k (the small splits: Aeon gaze, Elemental Witch, Nine-Tailed Heir, discoveries, angel), about 35 minutes at the check

- **Verdict:** progressing; the watchdog fired on a builder two steps from its commit. Nothing to unblock.
- **How the minutes split** (file times; the lane's base is 08:33, the Mac woke at 08:32 and didn't sleep after):
  - 08:33 to 08:38: the form proof's `record` on the base (49 s of run, the rest compile);
  - 08:38 to 08:53: the data edits and the writer (`lists.log`);
  - 08:53 to 09:00: tests and the game check (`gc.txt`);
  - 09:00 to 09:04: `record` on the tip, `compare`, the refused links listed;
  - 09:04 on: the one full `seeds`/`corpus` run (`links` 45 s; `game_data` about 3 minutes), then the Answer and the commit.

  All of it was work: no stall over a few minutes between writes, no sleep. Waiting was only the runs themselves, about 6 minutes of 35.
- **`pinning-tests.sh` and filtered runs did cut the time.** Ten test files under `libs/seeds/tests/game_data/` were edited before the first full run started at 31 minutes; 306i and 306j each found their pins (11 and 15 failures) from a full run and iterated from there. 306i took 65 minutes, 306j 33 of work with two full runs; 306k reached its first full run at 31. That run still showed one pin the script missed: `pick_order::the_selections_outside_the_features_band_are_the_games` lists every selection with `band: Path`, and the new `sorcerer/nine-tailed-heir/bloodline` (`band: Path`) is not in it. The script searches for the keys a slice touches; a test that lists all entries of a kind names no new key. One failure instead of 11 or 15, but it costs a second 4-minute run.
- **The estimate was the miss, not the builder:** `slices.md` says 20 minutes for five splits, a new feat, a new option, a census and a full run. The fixed costs alone (record, compare, the full run, the game check) are about 10 minutes.
- **Seen in the census, for the reviewer:**
  - 7 refused link lines where the section's Proof says 3 (2 Vivisectionist, 1 Underground Chemist): 6 Vivisectionist lines (t64 "discovery" and "talent only", sample and `full`) and 1 Underground Chemist (`full t49`). All are Extra Discovery, all listed in `306-refused.txt`; none is an Aeon link or a slot-order refusal (0 through an heir, 0 changed). The Answer should say why the count is 7.
  - the regenerated corpus has 2,156 Builds, not 2,158 (it lost `full t64 vivisectionist talent only` and its `/ filled`), and 1,990 open, not 1,994: the generator still gives a Vivisectionist `extra-discovery`. The same cause as 306j's shaman.
  - every offer status change (2,112 in 172 groups, all `-> BreaksLater`) sits in the 7 refused Builds; no other Build's offers changed status.
- **Changes:**
  - (declined: the user set the watchdog at 30 minutes; a medium-effort check is cheap) `slices.md` estimates for the remaining 306 slices: add 10 minutes of fixed cost per data slice, or move the watchdog to 45 minutes for a slice with a census, so it fires on a stall and not on a full run.
  - (applied: research/306/decisions.md; ticket 306af for compare) the corpus generator follows the split: a slice that gives a class its own Extra feat updates `tools/corpus/src/builds/` in the same diff, and `compare` names the Builds the regenerated corpus lost with the generator's line (306j's proposal, still open).
  - (applied: research/306/decisions.md) the research's Proof counts refusals in link lines (sample and `full`, each `/ filled` variant), the unit `306-refused.txt` and `compare` use, not in Builds.
  - (applied) `scripts/pinning-tests.sh`: also list the tests that enumerate every entry holding a field the slice's new entries set (`band: Path` → `pick_order::the_selections_outside_the_features_band_are_the_games`), from a small table of field → test in the script.
