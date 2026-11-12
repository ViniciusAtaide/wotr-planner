# The proof runs once per queue; the snapshot check is each ticket's requirement

Type: task
Status: done
Blocked by: 158, 159
Model: the main session (Fable), not a spawned agent: it edits `CLAUDE.md`, the map and other tickets.
Map: [WotR Build Planner](../map.md)

## Question

With the snapshot check ([159](159-snapshot-check.md)) in place, the proof stops being every ticket's requirement. Decided in the grilling session (user, 2026-11-11):

- **Per ticket:** the snapshot check, which `scripts/check.sh` already runs. A ticket that means to change behaviour regenerates the snapshots and shows the diff.
- **Per queue:** the main session runs `scripts/proof.sh` once, before merging a queue of tickets, alongside `scripts/check.sh --full`.
- **[158](158-faster-engine-walk.md) stays** as written. An agent is live in its worktree (user, 2026-11-11): don't edit its ticket or its per-part proof requirement while it runs.
- **[157](157-faster-proofs.md) shrinks** to a fixed folder for side A's source. Its cache and its lock are dropped: repeats against the same ref are now rare, and one session running one proof has nothing to lock against.
- **[154](154-proof-build-folder-per-checkout.md) closes** unbuilt: it matters only when two checkouts run proofs at once.

## Build

Main session, in a commit of its own: it edits the plan and `CLAUDE.md`, which every agent reads. Start only when 158 is merged and no agent is live in any worktree.

- **`CLAUDE.md`:**
  - "Behaviour proof": the snapshot check is what a ticket runs and how it reads and regenerates it; the proof is run by the main session once per queue. Keep the pointer to `scripts/proof.sh`'s header.
  - "Checks": name the snapshot check among what `scripts/check.sh` runs.
  - "Parallel agents": remove the "until ticket 154 lands" line. Keep `scripts/proof.sh` under "Heavy runs one at a time".
- **Open tickets:** in each open ticket whose "Build" or "Done when" asks for an empty `scripts/proof.sh` comparison, ask for the snapshot check instead (unchanged snapshots, or a reviewed diff). Find them by searching `issues/` for `proof.sh`. Leave resolved tickets as they are.
- **Rewrite 157** down to the fixed side-A folder:
  - `scripts/proof.sh` keeps `src-A` at `$CARGO_TARGET_DIR/src-A`, beside `A/` and `B/`, with a stamp file holding the ref's commit and a hash of `tools/proof`. It skips the extract and the patches when the stamp matches.
  - On a mismatch it extracts with `tar -x -m`, so the files take the current time. That closes the ticket 86 hazard the script warns about, where an archive's commit-time files count as up to date.
  - Its "Done when": a second `proof.sh rules` against the same ref rebuilds none of side A's crates, shown from cargo's output; both runs' `A.jsonl` are byte-identical.
  - Its numbers are stale: a side records in about 60–70 s since `b573ef9`.
- **Close 154:** Status and a one-line Answer saying why. Its branch `claude/beautiful-burnell-865ae9` is left for the user to delete.
- **The map:** the Notes entry on faster proofs, and a Decisions line for the two tiers.
- `scripts/proof.sh`'s header and `tools/proof`'s crate docs: say when the proof runs now, and fix "A full record takes about 4 minutes".

## Done when

- No open ticket requires a per-ticket `scripts/proof.sh` run.
- `CLAUDE.md` describes the two tiers and no longer mentions ticket 154.
- 154 is closed, and 157 is rewritten and still open.
- The map's Notes and Decisions say the same as `CLAUDE.md`.

## Answer

Done 2026-11-12, by the main session, once tickets 159, 160 and 165 to 168 were merged and no agent was live.

- **`CLAUDE.md`:** "Behaviour proof" is now "Behaviour checks", with the two tiers: the snapshot check and the gold builds for every ticket, the proof once per queue by the main session. "Checks" names the snapshot check. "Parallel agents" lost the "until ticket 154 lands" line; the proof stays a heavy run.
- **Open tickets:** 137, 139, 141 and 156 ask for the snapshot check where they asked for a proof run. 139's snapshots stay untouched, since they hold no attack line. 164 is another session's and already says to follow this ticket's process, so it is left as written. 162 (the proof's light mode) and 157 are about the proof itself.
- **[157](157-faster-proofs.md)** is rewritten down to side A's source in a fixed folder. Its cache, its "one full run per ticket" and its lock are dropped. It is still open.
- **154** has no ticket file on `main` (its file is on branch `claude/beautiful-burnell-865ae9`). The map's Notes record it as closed unbuilt, with the branch left for the user to delete.
- **The map:** the Notes on 154, 157 and the two tiers, and a Decisions line.
- **`scripts/proof.sh`'s header and `tools/proof`'s crate docs** say when the proof runs, and that a record takes about 45 seconds a side (it was "about 4 minutes").
- **The glossary:** an item a choice needs can't be removed, "or replaced by an item that doesn't keep the choice allowed" (left over from ticket 166's review).

### This queue's proof

`scripts/proof.sh both <out> e150fa2`, the commit before the queue, against `main` with 159, 160 and 165 to 168 merged: 2,163 Builds a side, each record in about 45 seconds.

- **First run:** 2,161 Builds differed, in `totals` (all) and `caster_lines` (1,367) only. No sheet, slot, option or `allows` field differed. The cause was the one ticket 166 predicted: it rewrote the "leaves out" texts, and the harness hashed them with the numbers. So the run couldn't say whether a number had changed.
- **The harness change** (`tools/proof/src/fingerprint.rs`): `totals_hash` and `caster_lines_hash` blank `leaves_out` before hashing. Both sides have the field, so the harness still compiles against older crates.
- **Second run:** `A: 2163 Builds, B: 2163; only in A: 0, only in B: 0, differing: 0`. The queue changed no behaviour for any Build without gear, and the proof's Builds hold none.

What the proof doesn't cover: a Build with gear. The link's gear section, the gear tests in `libs/rules/tests/engine/gear.rs`, the real-data test and the Enforcer's snapshot cover those.

`scripts/check.sh` passed on `main` with 167 and 168 merged together, before this ticket's edits.
