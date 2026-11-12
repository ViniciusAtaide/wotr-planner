# A faster proof: side A's source in a fixed folder

Type: task
Status: resolved
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

## Answer

Done 2026-11-12 (branch `ticket-157`). `scripts/proof.sh` (and the header of it and `tools/proof`'s crate docs) only.

- **Source and stamp:** side A's source is `$CARGO_TARGET_DIR/src-A`, with `src-A.stamp` beside it: a format number, the ref's commit, a hash of every file under `tools/proof` and the mode. A match prints `side A's source: reused (<commit>)` and skips the extract, the harness copy and the patches. Otherwise it prints `extracted (<commit>)`, removes the stamp, `src-A` **and `$CARGO_TARGET_DIR/A`** (so side A builds from clean: the stale build ticket 162 found), extracts with `tar -x -m` (files take the current time), patches, and writes the stamp last (an interrupted extract leaves none).
- **The stale build reproduced first,** on the script as it was (target folder of its own): `both <out> e150fa2` then `rules <out> 87890ea`: the second built side A in 11 seconds and failed with `Unexpected { path: ".../libs/game-data/gear.ron" }`. The same two runs after the change: the second says `extracted (87890ea...)`, builds side A anew and records `A: 2163 Builds, B: 2163; only in A: 0, only in B: 0, differing: 0`.
- **Reused, nothing rebuilt:** after `rules HEAD` twice (second: `side A's source: reused`), `cd $CARGO_TARGET_DIR/src-A && cargo build --release -p proof` against `A/` printed only `Finished release profile [optimized] target(s) in 0.29s`: no `Compiling` line, so no side A crate was rebuilt. (The script's own build is `--quiet`.)
- **Byte-identical `A.jsonl`:** `rules HEAD` fresh against reused (full records, 17,733,693 bytes): `cmp` silent. `both e150fa2` fresh against reused (light): `cmp` silent. `rules 87890ea` after a harness change and its revert (both extracted) against the first: `cmp` silent.
- **Harness change:** appending a comment to `tools/proof/src/lib.rs` (reverted after) made the next run print `extracted`; the run after the revert extracted again (the hash is of content).
- **Times** (`rules HEAD`, full, fresh target folder, so both builds from clean; the machine was not quiet, another agent building: `uptime` load 21 at the first run's start, 168 at its end, 175 for most of the second): first run 5:11 (side A recorded in 82 s), second run 2:26 (side A 53 s, side B 78 s). The saving is side A's workspace build, about 2.5 minutes under that load; the record times vary with the load and are not comparable. `both e150fa2 --light`: first 2:29 (load 149 to 132), reused 1:16 (load 132 to 123). Not on a quiet machine: the ticket's request for one could not be met while ticket 170 built beside it.
- **Mode in the stamp:** as the ticket says, so `rules` after `both` against the same ref extracts and rebuilds, though the source is the same; kept for the sake of the ticket's rule.
- **`export --only a`** does not narrow the Builds (1,729 of 2,163 contain an `a`), so those throwaway runs were not small.
- `scripts/check.sh` passes.
