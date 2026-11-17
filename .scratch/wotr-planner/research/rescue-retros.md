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
