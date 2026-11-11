# A faster engine walk: a fast hasher, cheap key clones, fewer state clones

Type: task
Status: open
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
