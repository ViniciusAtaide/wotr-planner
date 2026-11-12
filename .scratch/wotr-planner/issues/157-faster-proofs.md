# A faster proof: side A's source in a fixed folder

Type: task
Status: open
Blocked by: none
Model: Sonnet to build (a contained change to one script, proved by its own two runs); review on another model.
Map: [WotR Build Planner](../map.md)

## Question

The user, 2026-11-11: "can we shave some time off running proof", then "let's do 1 2 and 3". Those three were a cache for side A's record, one full run per ticket, and a lock so one proof runs at a time.

**Rewritten by ticket [161](161-proof-once-per-queue.md)** (user, 2026-11-11, in the grilling session on the cheap behaviour check). The proof now runs once per queue, by the main session (`CLAUDE.md`, "Behaviour checks"), so:

- the cache for side A's record is dropped: a repeat against the same ref is rare;
- "one full run per ticket" is replaced by the snapshot check;
- the lock is dropped: one session running one proof has nothing to lock against;
- ticket 154 (a target folder per checkout) is closed unbuilt for the same reason.

What is left is the one cheap change that still pays on every run. Today `scripts/proof.sh` deletes and re-extracts side A's source into `<out>/src-A` on every run and copies the harness in, so cargo rebuilds side A's workspace crates (`rules`, `seeds`, `proof`) each time, even against the same ref.

## Build

AFK. `scripts/proof.sh` only: no engine, data or schema change.

- Keep side A's source at `$CARGO_TARGET_DIR/src-A`, beside the `A/` and `B/` build folders.
- Beside it, a stamp file holding the ref's commit and a hash of `tools/proof` (the harness that is copied in) and of the mode. When the stamp matches, skip the extract, the copy and the `Cargo.toml` patches.
- On a mismatch, remove the folder and extract with `tar -x -m`, so the files take the current time. That closes the hazard the script warns about (ticket 86), where an archive's commit-time files count as up to date against an older build.
- Never share one `A/` build folder between source trees at different paths: the script's own comment describes that failure. One `CARGO_TARGET_DIR` holds one `src-A`.
- **A stale side A build, found by ticket 162:** the script keeps `$CARGO_TARGET_DIR/A` across refs, and cargo reused a build of an older ref for a newer one: a run against `87890ea` after one against `e150fa2` failed on `gear.ron` until `A/` was deleted. The stamp must cover the build folder too: when the ref changes, side A is rebuilt from clean.
- Say on stderr which happened: "side A's source: reused (<commit>)" or "side A's source: extracted (<commit>)".
- Update the script's header and `tools/proof`'s crate docs.
- Coordinate with tickets [162](162-proof-light-mode.md) and [164](164-cheap-key-clones.md), which edit the script or the harness: one at a time.

## Done when

- A second `scripts/proof.sh rules` against the same ref, with nothing changed, says the source was reused and rebuilds none of side A's crates, shown from cargo's output.
- Both runs' `A.jsonl` are byte-identical.
- Changing one file under `tools/proof` (throwaway, reverted) makes the next run extract again.
- The Answer has the wall time of the first and second run, on a quiet machine.
- `scripts/check.sh` passes.
