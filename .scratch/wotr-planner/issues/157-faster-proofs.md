# Faster proofs: cache side A, one full run per ticket, one proof at a time

Type: task
Status: open
Blocked by: 151, 154
Map: [WotR Build Planner](../map.md)

## Question

The user, 2026-11-11: "can we shave some time off running proof", then "let's do 1 2 and 3".

What a proof costs today (`scripts/proof.sh rules`, from the 2026-11-11 runs):

| Step | Time |
| --- | --- |
| Build side B's harness, release | cold in every new target folder; every agent today started cold |
| Export the Build list | seconds |
| Build side A's harness from the ref | cold the same way. `src-A` is extracted again into `<out>/src-A` per out dir, so the workspace crates rebuild every run |
| Record side A (2,163 Builds, 8 threads) | ~140 s quiet, ~355 s loaded |
| Record side B | ~140 s quiet, ~369 s loaded |
| Compare | seconds |

On 2026-11-11, every ticket proved against the same few `main` commits two to four times (after the build, the review and the fixes). Each time it rebuilt and re-recorded side A. Several proofs also ran at once on 8 cores, slowing each other down.

1. **Cache side A.** Side A's record is a function of:
   - the ref commit;
   - the harness source (`tools/proof`, from the working tree);
   - the game data it reads (the working tree's in `rules` mode, the ref's in `data` and `both`);
   - the Build list (`links.tsv` / `builds.tsv`);
   - the record options;
   - the mode.

   Key a cache on a hash of all of them, under `${TMPDIR:-/tmp}/wotr-proof-cache/`, and reuse `A.jsonl` and `A.err` on a hit. On a miss, record and store.
   - Extract the ref into a fixed folder per commit (`…/wotr-proof-cache/src-<commit>`, made once, never edited after its patches), and cache side A's harness binary by (commit, harness hash). A ref's binary is then built once, whatever checkout asks for it. Because the source path never varies for that key, a binary bakes in only its own `src-<commit>`, never another checkout's: `CLAUDE.md`'s rule against a shared target folder still holds.
   - Say "side A: cached (key …)" or "side A: recorded" on stderr.
   - Add `--fresh` to bypass the cache, and prune entries older than 7 days on each run.
2. **One full run per ticket.** `proof.sh` already passes `-- --only <name substring>` to the harness. Write the practice where agents read it:
   - in `CLAUDE.md`'s "Behaviour proof": while a ticket iterates, prove with `--only` over the Builds the change touches; run the full proof once, before the ticket is reported done;
   - in `proof.sh`'s header and `tools/proof`'s crate docs: list a few useful `--only` substrings, and how to see Build names (`proof export … | cut -f1`).

   Make `--only` cheap end to end: the export and both records honour it. Say on stderr when a run is partial ("partial: N of M Builds").
3. **One proof at a time.** Take an exclusive lock on `${TMPDIR:-/tmp}/wotr-proof.lock` for the whole run.
   - A second run waits, printing "waiting for the proof lock held by pid N (<checkout>)" once.
   - It doesn't fail, and a crashed run leaves no stale lock.
   - macOS has no `flock` command: use `/usr/bin/lockf` or perl's `Fcntl` `flock` (both present; the script already uses perl).
   - Add `--no-lock` for the rare deliberate case.

## Build

AFK. A tools change: no engine, data or schema change.

- **Land [154](154-proof-build-folder-per-checkout.md) first, in the same worktree,** as its own commits. Its branch `claude/beautiful-burnell-865ae9` (commit `954d285`, ticket file `issues/132-proof-build-folder-per-checkout.md`) was numbered 132 by another session.
  - Rename its ticket file to `154-proof-build-folder-per-checkout.md` and fix its number in its map line.
  - Bring it up to `main`: it conflicts with ticket 149's changes to `scripts/proof.sh` and `tools/proof/src/lib.rs`.
  - Then remove `CLAUDE.md`'s "until ticket 154 lands" line from "Parallel agents", and the matching bits in this ticket's text, since the folder per checkout now holds.
  - Then build this ticket on top. Check that the cache in item 1 and 154's per-checkout target folder fit together: side B builds in the checkout's own folder, and side A's binary comes from the shared cache keyed by commit.
- **Prove the cache with a test.**
  - Run `proof.sh rules` twice on an unchanged tree. The second run reuses side A: show the stderr line, and that `A.jsonl` is byte-identical to the first run's.
  - Change one `tools/proof` file (throwaway, reverted), and show the key misses.
  - Run two proofs at once from two shells, and show the second waits for the lock and then completes.
- Time a full `rules` proof twice, cold and then cached, on a quiet machine (`uptime`), and record the numbers in the Answer.

## Done when

- A repeat `proof.sh rules` against the same ref, with nothing side A depends on changed, records only side B and says so.
- `--fresh` and `--no-lock` work, and the cache prunes itself.
- Two proofs never run at once without `--no-lock`.
- `CLAUDE.md`'s "Behaviour proof" says how to prove while iterating and when to run the full proof. "Parallel agents" no longer has the "until ticket 154" line.
- 154 and 157 are both resolved, each with its Answer.
- `scripts/check.sh` passes.
