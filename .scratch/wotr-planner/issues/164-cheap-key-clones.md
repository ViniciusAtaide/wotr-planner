# Keys that clone cheaply, with proof threads that don't share them

Type: task
Status: open
Blocked by: 158 (merged, `3bdc287`)
Map: [WotR Build Planner](../map.md)

## Question

The user, 2026-11-11, after ticket 158 dropped its part 2: "write a follow-up ticket".

Part 2 of [158](158-faster-engine-walk.md) made the text keys (`keys.rs`'s `text_key!`) hold `Arc<str>` instead of `String`, so cloning a key bumps a count instead of copying the text. It is commit `04dae7e` on branch `ticket-158-arc-keys`, on top of 158's part 1. It is not proved and not rebased onto 158's part 3. 158's Answer has the details. Its measurements, on 158's part 1:

| | `String` keys | `Arc<str>` keys |
| --- | --- | --- |
| `evaluate` / `allows`, fighter20 (µs) | 57.0 / 80.6 | 31.8 / 42.9 |
| `evaluate` / `allows`, legend40 (µs) | 119.8 / 171.0 | 58.6 / 82.6 |
| 1-thread `proof record`, CPU | | −40 % |
| 8 separate processes at once, CPU | | −40 % |
| 8-thread `proof record`, CPU | 445 s | 515 s (+15.8 %) |

Single-threaded, part 2 halves each walk. That is what the web app runs: wasm on one thread, where an `Arc`'s count is a plain increment. It loses only in the proof's 8-thread record, because the threads share one `Engine` and one list of Builds, so every key a walk clones is the same `Arc`. All eight threads then update the same atomic counts and contend for the same cache lines. 8 separate processes don't share keys, and get the full −40 %.

So the loss belongs to how the proof shares data across threads, not to the engine. This ticket fixes the harness first, then lands part 2.

## Build

AFK. Two parts, each its own commit.

1. **Proof threads that share no keys** (`tools/proof`). Give each record thread its own `Engine` and its own copies of the Builds it records, deep copies made without cloning keys:
   - The engine: read the game data files once per thread, or deserialise once per thread from one serialised copy. Don't use `GameData::clone()`, which shares the keys once they are `Arc`s.
   - The Builds: a serde round trip per Build, or per thread.
   - `in_parallel` (work-stealing since `b573ef9`) takes per-thread state: an init closure run once on each thread, handing that thread's engine to `work`.
   - The record must stay byte-identical: `cmp` a full record against the harness before the change, counters included.
   - With today's `String` keys this should cost about nothing: eight engine builds of well under a second each. Measure that it does.
   - The same goes for `export`'s and `generate`'s parallel loops if they walk.
2. **`Arc<str>` keys.** Rebase or re-apply `04dae7e` onto `main`. It touches `keys.rs`, `codec.rs`, the `Key(…)` constructions in `libs/seeds` and `apps/mcp`, and serde's `rc` feature, which 158's part 3 didn't touch, so conflicts should be few. Then:
   - `scripts/proof.sh rules` against `main` must come out empty (exit 0).
   - Measure, as medians of alternating runs on a quiet machine:
     - `evaluate` and `allows` on the six bench Builds (`tools/proof/bench-links.tsv`), single-threaded. This stands for the web app.
     - a 1-thread `proof record` (`--threads 1`, or `--only` with a big enough slice to take a minute);
     - the full 8-thread `proof record`, with part 1's harness.
   - Keep it if the single-threaded walk is at least about 3 % faster and the 8-thread record is no slower than on `main` beyond noise. If the 8-thread record still loses, profile where the shared counts are (the proof's own Builds? the engine's catalog? something left shared?) and fix that first, or report it.
   - Optional, if it's cheap to set up: time the web app itself. Build it with `cargo leptos build --release --lib-features timings`, and read `window.__wotrTimings` on a picker and an edit before and after (`libs/ui/src/timings.rs`).

### Notes

- The MCP server also shares one engine across Tokio's threads. Its requests rarely walk at the same moment, so contention there should be negligible. Say if a measurement shows otherwise.
- 158 found that interning the keys (one allocation per distinct key) doesn't help: it shares even more.
- **Coordination:** part 1 edits `tools/proof/src/lib.rs`, which tickets 154, 157 and 162 also edit (162 is the proof's light mode). Part 2 touches `apps/mcp` and `libs/seeds` where they build keys. Agree the slot with the session running the queue. If 161 has landed, follow its process: the snapshot check for the ticket, and the proof once per queue. Otherwise follow `CLAUDE.md`'s proof rules as written.
- **CPU:** proofs, timings and profiles are heavy runs, so one at a time, on a quiet machine (`CLAUDE.md`, "Parallel agents").
- **Worktree:** agent worktrees made with `isolation: "worktree"` start from `origin/main`. Push `main` first, or create the worktree by hand from local `main` and check its base.

## Done when

- Part 1's record is byte-identical to the harness before it, and its cost on today's keys is measured.
- Part 2 is kept with an empty `proof.sh rules` comparison and the timings above, or dropped with the measurements that say why.
- The Answer gives the medians: single-threaded `evaluate`/`allows`, the 1-thread record and the 8-thread record, before and after each part.
- `scripts/check.sh` passes. Part 2 reaches the frontend and the MCP server, so it also needs `--full`, now or at the end of the queue.
