# A light mode for the proof: every Build, without the battery

Type: task
Status: resolved
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

## Answer

Done 2026-11-12 (branch `ticket-162`).

- **`--light`** is a record option, passed through `scripts/proof.sh` like the others (and accepted, ignored, by `export`). `fingerprint` takes a `Mode`: light skips `controls`, `battery`, the `allows` calls, `consistency_checks` and `shortcut_checks`, drops the `allows` field and adds `"light": true`. Every other field is as before, so a full record is byte-for-byte what it was (the maps are ordered). `compare` needs no change beyond refusing a light record against a full one: "A is a light record and B a full one: a light record leaves out `allows`, so it only compares with another light one (record both with `--light`, or neither)". A test in `tools/proof/tests/round_trip.rs` holds that and that a changed Build still shows in a light record.
- **Docs:** `scripts/proof.sh`'s header, `tools/proof`'s crate docs and a bullet in `CLAUDE.md`'s "Behaviour checks".
- **Measured** (`proof record` of side B, 2,163 Builds, 8 threads, three alternating runs each; the machine was not quiet, `uptime` load 20 to 26 throughout, but the times were steady): light 29.1, 29.3, 29.1 s (median 29.1 s, 212 CPU-seconds, a 17 MB record); full 45.4, 45.0, 45.2 s (median 45.2 s). **The ticket's estimate of a few seconds a side was wrong:** the kept fields are most of the cost today. A sample of a light run puts about 41% in the slot pickers (`Engine::options`) and 36% in the class pickers, which the ticket keeps; the battery and the checks were about a third of a full record. A light run through `scripts/proof.sh` is about 2.5 minutes in all, nearly all of it the two release builds.
- **Readable values: not done, hashes kept.** The projection (`snapshots.rs`, 1,300 lines) lives in a test crate on current engine calls (`choice_label`, `pick_name`, `class_value_name`, `path_name`…), so the harness could not run it against an older `rules`, and moving it would bind the harness to the new engine, which `--light` must not. And since the pickers are most of the light cost, writing the values in full would add to a run that is not cheap already. Not worth this ticket.
- **What it catches:** a throwaway change in `class_options` (Barbarian's level-1 option locked): `scripts/proof.sh rules <out> HEAD -- --light` reported `differing: 634`, in `classes` (381) and `class_reasons, classes` (253).
- **What it misses:** a change only the battery's edits see. Throwaway: `Engine::allows` answering false for `Edit::RemoveLevel` on a 20-level Build. Light: `differing: 0`. Full: `differing: 1179`, all in `allows`. So does a disagreement the checks panic on (a shortcut against its full walk; `removable` against `is_removable`/`withdrawal`; a picker status against `allows`).
- **Older refs:** `scripts/proof.sh both <out> e150fa2 -- --light` records and compares (`differing: 0`). `rules` against `e150fa2` can't, as before this ticket: the ref's reader doesn't know `gear.ron`.
- **Full proof unchanged:** `scripts/proof.sh rules <out> 87890ea` (the commit before this ticket), full: `A: 2163 Builds, B: 2163; only in A: 0, only in B: 0, differing: 0`, `compare.txt` empty.
- **Found, not fixed:** `scripts/proof.sh` reuses side A's target folder across refs, and cargo took a stale build of an older ref for a newer one (a run against `87890ea` after one against `e150fa2` failed on `gear.ron`); deleting `$CARGO_TARGET_DIR/A` fixed it. Ticket 157's fixed source folder is the place for it.
