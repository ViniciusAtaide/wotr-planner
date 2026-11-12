# A faster engine walk: a fast hasher, cheap key clones, fewer state clones

Type: task
Status: done
Blocked by: 151
Map: [WotR Build Planner](../map.md)

## Question

The user, 2026-11-11: "let's do it all. write a ticket to be run after the current one finishes", after a profile of the proof's record step. Commit `b573ef9` already halved the record's wall time in the harness (work-stealing threads, no repeated walks) without touching the engine. What's left is the engine's own cost per walk. Every walk the planner makes pays it: the web app's pickers, the MCP server and the proof. This ticket cuts that cost without changing any behaviour.

The profile: macOS `sample`, 15 s over `proof record --only "full t6" --threads 4`, a release build with line tables, on `main` at `f413768`. Shares are of the busy samples (29,762, idle waits excluded):

| Where | Share | What |
| --- | --- | --- |
| malloc / free | 35 % | Allocation, mostly from clones (below) |
| `String::clone` under clones | ~14 % | The text keys (`keys.rs`: `pub struct FeatKey(pub String)` and the rest) copied each time a map holding them is cloned |
| `BTreeMap::clone` (`clone_subtree`) | ~8 % | `CharacterState`'s maps (`feats`, `skill_ranks`, `class_skills`, `feat_class_levels`, `deity_domains`…) cloned with the state |
| B-tree drops | ~7 % | The same maps freed again: `FeatureKey`, `SlotId → Pick`, `SkillKey`, `FeatKey`, `SelectionOptionKey`, `FeatureSelectionKey` |
| memmove / memcmp | 11 % | Copies and key comparisons |
| SipHash (`Hasher::write`, `hash_one`, `reserve_rehash`) | 10 % | std `HashMap` / `HashSet` with the default hasher, in `Catalog` (`archetype_of`, `selection`, `weapon`, `row`), `Walker::fill` and `hold`, `check_pick_with`, `Walker::level`, `Walker::advance`, `check_unreached`, `Engine::holder` |
| `Pick::clone`, `Vec<LevelUp>::clone`, `Prerequisite::clone`, `Vec<RankSheet>::clone` | ~5 % | Picks, a Build's levels (each edit's `Cursor::edited`), prerequisites and rank sheets cloned in the walk |

## Build

AFK. A rules change with no change to behaviour or data. Three parts, each its own commit and each proved on its own with `scripts/proof.sh rules` against the commit before it. Each comparison must be empty, since every part is meant to change nothing. Do them in this order, cheapest first, so a part that doesn't pay can be dropped without blocking the rest.

1. **A fast hasher.** Swap the default SipHash for `rustc-hash`'s `FxHashMap` / `FxHashSet` (already in `Cargo.lock` through other crates; or `foldhash`, also there, if it measures faster) in the rules crate's hot maps: `engine/catalog.rs`'s lookups (ticket 83's index) and the walk's maps keyed by `SlotId`. `check.rs`'s one-off checks and `codec.rs` can stay as they are.
   - No HashMap iteration order may reach an output. The default hasher is randomly seeded per process, so any order dependence would already make the records differ between runs. Still, grep for iteration over these maps and confirm each one is order-free (it is sorted, collected into a `BTreeMap`, or used only for lookups).
   - It must build for wasm (`scripts/check.sh` builds `rules` for `wasm32-unknown-unknown`).
2. **Cheap key clones.** Make a text key's clone a reference-count bump instead of a string copy: `pub struct FeatKey(pub Arc<str>)` in `keys.rs`'s `text_key!` macro, with serde's `rc` feature for `#[serde(transparent)]`.
   - `Arc`, not `Rc`, because the proof shares the engine across threads.
   - Keep `new`, `as_str`, `Display`, `Ord` and the serde form unchanged, so the data files, links, JSON and the database see no difference. `Ord` on `Arc<str>` compares the text, as `String` does, so every `BTreeMap` keeps its order.
   - About 58 places outside `keys.rs` construct keys with `Key(…)`, and some read `.0` as a `String`. Fix them, or make the field private behind `new`/`as_str` if that makes the diff smaller and clearer.
   - Optional, only if part 3 still shows key clones hot: intern the keys once in the catalog so equal keys share one allocation.
3. **Fewer state clones in the walk.** Find where the walk clones a `CharacterState`, a `Walker` or a `Build`, and cut the clones that only read. Look first at:
   - the cursor's paused walkers (`Cursor::paused`, `with_paused`, `until_slot`, `until_level`);
   - `Cursor::edited`, a whole `Build` per edit;
   - `Walker::level`;
   - `Vec<RankSheet>` and `Prerequisite` clones.

   Borrow where a reader only reads. Where a state must fork, consider putting its large, rarely-changed maps behind `Arc` with copy on write (`Arc::make_mut`), so a fork copies only what it changes. Re-profile after part 2: its cheap keys shrink the clones this part targets, so pick what's still hot then, not from this table.
4. **The release profile.** Try `codegen-units = 1` and `lto = "thin"` for the proof's build, through a `[profile.proof]` that inherits `release`, which `proof.sh` would pass. Keep it only if the record's speed-up outweighs the extra build time. After ticket 157, side A's binary is cached per commit, but side B still builds on every run. Leave `[profile.release]` alone: the server and the cargo-leptos builds use it.

### Measuring

- Before and after each part, time on a quiet machine (`uptime` load near 8 or under, with no other agent building), comparing medians of alternating runs on both sides (`CLAUDE.md`, "Parallel agents"):
  - a full `proof record` (8 threads), its wall time and its CPU time (`/usr/bin/time -p`, `user`);
  - natively, `Engine::evaluate` and one `Engine::allows` on the six bench Builds (`tools/proof/bench-links.tsv`), the way 151 times them.
- Profile with macOS `sample` on a release build that has line tables (`CARGO_PROFILE_RELEASE_DEBUG=line-tables-only`), as above. Record each part's before and after shares in the Answer.
- A part that doesn't make the record at least about 3 % faster in CPU time is dropped, and the Answer says why.

### Coordination

- **Order:** this runs after 151 (the user, 2026-11-11). It edits `engine/catalog.rs` and the walk, which 155 (weapon groups) also edits, so 155 runs after it, or rebases onto it. Agree the slot with the session running the queue (148–157) before starting.
- **CPU:** proofs, profiles and timings are heavy runs, so one at a time, never beside another agent's build. Use the proof's own target folder per checkout (or 154's, once it lands).
- **157:** if 157 has landed, its side-A cache saves a rebuild on each of this ticket's three proofs, and `--only` keeps the iteration proofs short. Run the full proof once per part, before that part's commit.

## Done when

- Each part kept has its own commit, and its `scripts/proof.sh rules` comparison against the commit before it is empty.
- The Answer gives, per part, the median timings before and after (record wall and CPU, `evaluate` and `allows` on the bench Builds) and the profile shares. A part that was dropped has the reason it was dropped.
- `scripts/check.sh` passes, with `--full` at the end since the keys reach the frontend and the MCP server.
- `cargo fmt --all --check` passes.

## Answer

Built 2026-11-11 on branch `ticket-158`, from `51c689d`. Kept: part 1 (`7ea244a`), part 3 (3a `c22284b`, 3b `4995d73`). Dropped: part 2 (kept on branch `ticket-158-arc-keys`, `04dae7e`, unproved), part 3c, and part 4 (157 isn't built).

The whole ticket, medians of alternating runs on a quiet machine: a full `proof record` (8 threads) went from **506 s CPU, 71.6 s wall** to **350 s CPU, 52.1 s wall** (−31 % CPU). `allows` on the bench Builds is 2.7 to 4 times faster; `evaluate` 10–15 % faster.

### How it was measured

- Release builds with line tables (`CARGO_PROFILE_RELEASE_DEBUG=line-tables-only`) of `proof` and of a timing example copied into `libs/seeds/examples/` and never committed. The example's medians over 201 runs: `Engine::evaluate`, and `Engine::allows` for clearing the last pick (a one-question cursor), on the bench Builds of `tools/proof/bench-links.tsv`. `blank` has no link; `start` has no pick, so no `allows`. The four with levels are below.
- Each comparison: 3 rounds, each the "before" binary then the "after" one: a full `proof record libs/game-data` (8 threads) under `/usr/bin/time -p` (`user` is the CPU time), then the timing example. Started only at a 1-minute load under 9 with no other proof or cargo build running.
- Profiles: macOS `sample`, 15 s of `proof record --only "full t6" --threads 4`. Shares are of the busy samples (idle waits excluded): self time by category, or inclusive for a function (the function and what it calls).

### Part 1, a fast hasher: kept

rustc-hash's `FxHashMap` / `FxHashSet` in `engine/catalog.rs` (every index, the feature facts, the bonus rows) and for the walk's `reached` set. foldhash measured slower: 68 µs per `evaluate` against Fx's 65 µs. No iteration order reaches an output. The walk only looks up and inserts in `reached`. The catalog's iterations (`selections.keys()`, `options.keys()`, `selections.values()`) each collect into another hash map. `check.rs`, `codec.rs` and the cursor's `named_feats` stay as they were. It builds for wasm.

| | Before (`51c689d`) | After |
| --- | --- | --- |
| Record CPU / wall | 506 s / 71.6 s | 442 s / 61.7 s (−12.8 % CPU) |
| fighter20 evaluate / allows | 64.1 / 88.3 µs | 55.2 / 79.0 µs |
| fighter20_nopath | 43.2 / 58.7 µs | 37.2 / 52.5 µs |
| multi_prestige20 | 100.9 / 137.4 µs | 86.5 / 121.5 µs |
| legend40 | 135.9 / 189.0 µs | 119.8 / 172.8 µs |
| Hashing (SipHash, `hash_one`, rehash), inclusive | 10.4 % | 0.8 % |

Proof against `51c689d`: `A: 2163 Builds, B: 2163; only in A: 0, only in B: 0, differing: 0`.

### Part 2, keys as `Arc<str>`: dropped

`text_key!` held `Arc<str>` behind a private field (`new`, `as_str`, `Display`, `Ord`, `Hash` and the serde form unchanged, serde's `rc` feature). The `Key(…)` constructions became `Key::new(…)`. It changes no other file in the frontend, the MCP server or the database.

Single-threaded it halves the walk. But the proof's record shares one `Engine` across 8 threads, and every key the walk clones comes from the shared catalog or from Builds built from it. The threads then contend on the same atomic reference counts, and the record got slower:

| | Before (part 1) | After |
| --- | --- | --- |
| Record CPU / wall, 8 threads | 445 s / 63.8 s | 515 s / 74.4 s (+15.8 % CPU) |
| `--only "full t6"`, 1 thread, CPU | 93.1 s | 56.2 s (−40 %) |
| `--only "full t6"`, 8 threads in one process, CPU | 159.1 s | 178.4 s (+12 %) |
| `--only "full t6"`, 8 single-thread processes at once, CPU each | 138.8 s | 83.9 s (−40 %) |
| fighter20 evaluate / allows | 57.0 / 80.6 µs | 31.8 / 42.9 µs |
| legend40 evaluate / allows | 119.8 / 171.0 µs | 58.6 / 82.6 µs |
| `String::clone`, inclusive | 24.1 % | 0 % |

So, by the ticket's measure (the 8-thread record), part 2 is dropped. It is kept as commit `04dae7e` on branch `ticket-158-arc-keys`, on top of part 1. It is not proved and not rebased onto part 3. **For the main session to decide:** the web app's walks are single-threaded (wasm), so part 2 would roughly halve what each picker and edit costs there. The MCP server shares one engine but rarely walks on two threads at once. The proof could keep the gain if each thread had its own engine and its own copies of the Builds. That is a harness change in `tools/proof`, outside this ticket. Interning the keys would share them even more, so it doesn't help.

### Part 3, fewer state clones: kept (3a and 3b), 3c dropped

Re-profiled on part 1. Walker clones were 17.6 % (the CharacterState's 12.5 % of that), and 11 % of it was in `Cursor::walk_to`: a cursor copied the walk at every level on the way to the level a question asked about. Most cursors answer one question (each `allows`, `withdrawal`, `is_removable` and picker), and the fingerprint asks about 150,000 of them.

- **3a** (`c22284b`): the cursor keeps only the paused walks asked for, in a `BTreeMap` by level. It walks a new one on from the closest kept before it, or from the Build start, without a copy at each level between.
- **3b** (`4995d73`): `Cursor::keeps_once`, for a cursor's only question (`allows`, `withdrawal`, `is_removable`), walks on from the paused walk itself rather than from a copy. `is_removable` finds its one held choice (`held_choices` holds each once) instead of filtering them all.
- **3c**, dropped: no copy of a slot's pick in a walk without a sheet, and no key copy for a key already in a state's map (`count_up`, `insert_new`). It measured 349 s against 348 s CPU, nothing to keep.

| | Part 1 | 3a | 3b |
| --- | --- | --- | --- |
| Record CPU / wall | 439 s / 68.3 s | 357 s / 55.5 s (−18.6 %) | 350 s / 52.1 s (−2.6 % on 3a; 360 s → 350 s in its own runs) |
| fighter20 evaluate / allows | 58.2 / 81.0 µs | 58.2 / 32.2 µs | 54.8 / 26.4 µs |
| fighter20_nopath | 37.9 / 53.2 µs | 37.5 / 20.6 µs | 37.8 / 18.0 µs |
| multi_prestige20 | 90.6 / 122.9 µs | 86.2 / 51.0 µs | 83.6 / 42.8 µs |
| legend40 | 125.1 / 184.0 µs | 119.8 / 49.2 µs | 120.4 / 45.3 µs |
| Walker clones, inclusive | 17.6 % | 7.4 % | 5.7 % |
| CharacterState clones, inclusive | 12.5 % | 6.0 % | 4.6 % |

3b alone is just under the 3 % bar. It is kept as part of part 3, which takes 20 % off part 1's record CPU, because it makes `allows` 10–16 % faster with a small change.

What is left, on 3b: `String::clone` 21 % inclusive (the text keys copied with the states and Builds), the `Build` copy per candidate in `Cursor::edited` (6.8 %), and the walk's own work in `Walker::level`. The keys are part 2's ground. Copy-on-write maps (`Arc<BTreeMap>`) in `CharacterState` would save little: a candidate's walk changes the large maps (`feats`, `features`, `skill_ranks`) at almost every level, and the per-level state copies in `evaluate` cost only 3 %.

Proofs: 3a against part 1, and 3b against 3a: `A: 2163 Builds, B: 2163; only in A: 0, only in B: 0, differing: 0`.

### Part 4, the release profile: skipped

Ticket 157 (the proof cache) isn't on `main`, so `scripts/proof.sh` is untouched.

### Checks

`cargo fmt --all --check` and `scripts/check.sh` pass on `4995d73`. `scripts/check.sh --full` (the cargo-leptos dev and release builds and `build-mcp.sh`) was left for the end of the queue, as the user asked (2026-11-11). With part 2 dropped, nothing here reaches the frontend or the MCP server beyond the rules crate's internals.
