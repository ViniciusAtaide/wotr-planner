# A cheap behaviour check: snapshots of a small corpus

Type: task
Status: open
Blocked by: 158
Model: Opus to build (it designs the projection and the message every later ticket reads); review on another model.
Map: [WotR Build Planner](../map.md)

## Question

The user, 2026-11-11: "my idea is to simplify the proof concept to be something very cheap to do instead of resource heavy. I was thinking about a simple tree diff simple comparison between builds and a generic message telling what's the difference between the builds."

What the proof costs (`scripts/proof.sh`): comparing takes seconds. The cost is producing two sides: two release builds of the workspace, and a record of 2,163 Builds per side (about 60–70 s each since commit `b573ef9`), most of it the `allows` battery and the consistency checks. Its record hashes most fields, so a difference names a Build and a field, never the values.

Decided in the grilling session (user, 2026-11-11):

- **Two tiers.** This cheap check runs on every change. The proof stays, and runs once at the end of a queue of tickets ([161](161-proof-once-per-queue.md)).
- **One side.** The check runs the current engine only. The other side is snapshot files committed in the repo, so an intended change shows up as a snapshot diff in the commit.
- **The claim is weaker than the proof's:** nothing changed in these Builds. It has no `allows` battery and no consistency asserts; those stay in the proof.

## Build

AFK. A new test and its files: no engine, data or schema change.

**Wait for [158](158-faster-engine-walk.md)** (an agent is live in its worktree, user 2026-11-11). This ticket adds files only, but its snapshots record engine output, and 158 must not change them: take the snapshots after 158 is merged.

- **Where:** `libs/seeds/tests/game_data/snapshots.rs`, declared with `mod snapshots;` in that folder's `main.rs`. It uses the real data and engine that `support.rs` loads once, and `scripts/check.sh` already runs it.
- **Corpus**, written in Rust with `support.rs`'s helpers, never as stored link codes (the bench links have gone stale before, ticket 86):
  - the 6 bench Builds (`support.rs`'s `bench()`);
  - one Build per class at level 20 with no picks;
  - one Build per mythic path at Mythic rank 10 with no picks beyond the path.
  
  Assert each one is possible: an impossible Build yields no options and would snapshot as empty.
- **The projection:** a small module in the test, about 150–200 lines, that turns a Build's engine output into flat `path = value` lines in tree order. It uses only public engine calls (`evaluate`, `options`, `totals`, `caster_line`, `choice_label`, `pick_name`, `feat_name`, `Address`'s `Display`). The engine's types stay serde-free.
  - **The sheet, for every Build:** the start picks; per level the class, BAB, saves, scores, skill points and ranks, and the feats and features *gained at that level* (not the whole state, so one new feature is one line); per Mythic rank the same; the last level's totals and caster lines. Copy the shape of the MCP's `SheetResult` (`apps/mcp/src/shapes.rs`); the test can't call it, since `mcp` depends on `seeds`.
  - **The options, for the pick-less class and path Builds only:** one line per slot, `<slot address>.open = key, key, …`, listing the keys `Engine::options` returns as open. A parameterised pick is `key(value)`. The bench Builds get the sheet only: all their slots are filled.
- **Snapshot files:** one per Build, `libs/seeds/tests/game_data/snapshots/<name>.txt`. Expect about 15–25 KB for a sheet and 45–80 KB with every slot's options, 2–3 MB in all.
- **Regenerating:** an environment variable, `SNAPSHOTS=write cargo test -p seeds --test game_data snapshots`, rewrites the files. No `insta`.
- **The message.** One `#[test]` compares every Build's stored and current lines, and on any difference panics with:
  - one line per changed path and value pair, with the number of Builds it affects and the first three names (`level.5.saves.will: 4 → 5 in 12 Builds: …`);
  - `.open` lists expanded to one path per key, so a new option reads `+ slot.feat.5.general.0.open/power-attack`;
  - a second grouping with numbers wildcarded (`slot.feat.*.general.0.open/power-attack: + in 31 Builds, 212 slots`), so one newly legal feat prints one line;
  - a missing or extra snapshot file named as such;
  - the regeneration command as its last line.
- **Test the message itself** on made-up line sets (a changed value, an added and a removed option, a missing file), so its grouping is pinned without the real data.
- Measure the test's wall time in a debug build on a quiet machine and record it in the Answer. The estimate from recorded timings is 2–4 s when the Builds run on threads. If it is over 10 s, say so and stop for a decision before trimming the corpus.
- Document it in `libs/game-data/README.md`, where the real-data tests are described: what the check covers, how to read its message, and how to regenerate.

## Done when

- `cargo test -p seeds --test game_data -- --list` shows the snapshot test and the message's own tests.
- The corpus covers every class and every mythic path in `libs/game-data/`, found from the data's own lists, so a new class without a snapshot fails the test.
- A throwaway change to one feat's prerequisite (reverted) fails the test with a grouped message that names the feat, and `SNAPSHOTS=write` then makes it pass.
- The Answer records the corpus size, the snapshots' total size and the measured run time.
- `scripts/check.sh` passes.
