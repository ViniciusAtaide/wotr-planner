# A light mode for the proof: every Build, without the battery

Type: task
Status: open
Blocked by: 159, 160, 161
Model: Sonnet to build (a contained tools change that its own measurements prove); review on another model.
Map: [WotR Build Planner](../map.md)

## Question

The user, 2026-11-11, on a record mode without the `allows` battery and the consistency checks: "I want to do it in a subsequent ticket".

One full record makes 47,067 picker calls, against 700,625 `allows` calls for the battery of edits and 1,394,838 for the shortcut checks (from `$TMPDIR/wotr-proof-158-out/p1`). A record that keeps the sheet and every slot's options and leaves both out is estimated at 30–50 CPU-seconds for all 2,163 Builds, a few seconds per side on 8 threads. That estimate is derived, not measured.

It sits between the two tiers: wider than the snapshot check ([159](159-snapshot-check.md), a few dozen Builds, one side) and far cheaper than the full proof. It still needs both sides compiled, so it gains most once side A's source folder is fixed ([157](157-faster-proofs.md), as rewritten by [161](161-proof-once-per-queue.md)).

## Build

AFK. A tools change in `tools/proof` and `scripts/proof.sh`: no engine, data or schema change.

- **A record option,** `--light`, passed through `scripts/proof.sh` as its other record options are. It leaves out:
  - the `allows` battery (the record's `allows` field);
  - the consistency checks and the shortcut checks.
  
  It keeps every other field, so `compare` needs no change beyond accepting records without `allows`. Comparing a light record with a full one is refused, with a message.
- **Readable values, if cheap.** The record hashes the sheet and each slot's options. In light mode, see whether writing them in full (159's projection, moved where both can use it) keeps the record under about 100 MB a side and the run within the estimate. If so, `compare` prints the values that differ, grouped as 159's message is. If not, keep the hashes and say why in the Answer.
- **Measure** on a quiet machine (`uptime`): the light and full record of one side, three runs each, medians.
- **Show what it misses.** Take the difference ticket 150 found on its first run (140 Builds differing in one class picker) or a throwaway change like it, and show the light mode reports it. Then name, from the fields left out, a kind of change only the full proof would see.
- Document `--light` in `scripts/proof.sh`'s header, `tools/proof`'s crate docs and `CLAUDE.md`'s "Behaviour proof": when to reach for it between the snapshot check and the full proof.

## Done when

- `scripts/proof.sh rules --light` (or the spelling the script's options take) records both sides and compares them, and the Answer has the measured times beside the full proof's.
- A full proof's result is unchanged by this ticket: an empty `scripts/proof.sh rules` comparison against the commit before it.
- `scripts/check.sh` passes.
