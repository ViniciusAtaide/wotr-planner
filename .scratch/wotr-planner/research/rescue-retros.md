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
