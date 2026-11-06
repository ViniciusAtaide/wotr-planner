# Index the engine's hot lookups

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

A read-only review (2026-11-04) found lookups in `libs/rules`'s engine that scan a list, or allocate, on every call along its hot paths. A picker runs one recheck walk per candidate (about 240 when the class picker opens, about 40 for a selection picker), and each walk calls `recompute_tables`, `grant_selections`, `advance_holdings` and `check_pick` at every level, so a per-level cost is paid that many times. The user asked to build the first five (2026-11-06), measured before and after. File:line references are from 2026-11-04; re-find each site.

1. **Class rows** (verified by the lead):
   - `effective_row` (`engine.rs` ~173) finds a level's row by scanning `class.levels`, and the archetype's rows likewise.
   - `effective_rows` calls it once per row, so `Catalog::casts` (`engine/catalog.rs` ~303) costs rows².
   - `recompute_tables` (`walk/level.rs` ~462) calls `casts` and `row` for each held class at every level of every walk; `take_class_level`, `level_slots` and `check_advances` call them too.
   - **Index:** casts per class and per archetype computed once in `Catalog::new`; a row read by position (the data check already rejects tables that aren't contiguous; keep a guard), archetype rows found without a scan.
2. **`FeatureSelection::offers`** (`game_data.rs` ~1053, verified) scans up to 38 options by key on every call: per selection pick in every walk (`check_pick` → `Catalog::offers`), per candidate in a selection picker, and in `slot_lists`, `pools`, `holding_level`, `replace_row`, `grant_option_row`.
   - **Index:** `Catalog`'s `indexes.options` already maps each option to the selection that defines it; add a `Catalog` method (own option by that map, or a shared option) and route the engine's calls through it. `GameData`'s own `offers` stays for code without a `Catalog`.
3. **`Catalog::single_option`** (`engine/slots.rs` ~130, verified) builds a `BTreeSet` and a `Vec` on every call, from game data alone; it runs for every selection slot at every level of every walk (`grant_without_slot`), and `only_without_option` builds `offered()` too.
   - **Index:** selection → its single option, computed once in `Catalog::new`.
4. **`slot_lists`** (`engine/slots.rs` ~58) is rebuilt for each feat candidate in `options()` and twice more in `check_pick` (`slot_accepts_in`, `slot_ignores_in`), though it depends only on the slot and the state, fixed for one picker. It affects `OptionList`/`OneOf` slots (the sorcerer's bloodline feats, ranger, bloodrager, slayer, Dragon Disciple).
   - **Index:** compute the lists once per picker and pass them into the per-candidate check.
5. **`Catalog::pools`** (`engine/slots.rs` ~32) builds a `BTreeSet` on each check of a pooled pick (oracle, Magic Deceiver), scanning every held option.
   - **Index:** each pooled option's members, precomputed in `Catalog::new` (filtered as `has_option` filters them today).

Out of scope: the review's other candidates (`Walker::holds`, the ascension lookups, `features_granting_row`, `named_feats`, …) and the per-candidate cloning of the paused walk and the Build (`engine/removable.rs`); note in the Answer anything the timings point at.

## Build

Test-first (`tdd`) for each new `Catalog` index: a test that pins what it answers, against the small world in `libs/rules/tests/engine/support` where it fits.

**Measure first.** Before changing anything, write a native timing harness in the scratch dir, outside the repo: a small crate with path dependencies on `libs/rules` and `libs/seeds` that loads `libs/game-data/`, decodes the six Builds in `libs/proof/bench-links.tsv` (`blank`, `start`, `fighter20`, `fighter20_nopath`, `multi_prestige20`, `legend40`), and times in a release build, as the median of enough runs to be steady:
- `class_options` at level 7 of `multi_prestige20` and at level 1 of `legend40`;
- `options` for the level-1 feat slot and for a selection slot early in `legend40`;
- `removable_classes` and one `apply` + `is_possible` on `legend40`;
- one `options` call on an `OptionList` slot (a sorcerer's bloodline feat) and on a pooled pick (oracle or Magic Deceiver) — build such Builds if the six don't have them.

Point it at a `git archive` of `HEAD` for the "before" numbers and at the working tree for "after", with separate target dirs. Commit nothing from the harness.

**Then build 1–5**, one at a time, checking the proof after each.

**Done when:**
- Each finding is built, or the Answer says why not.
- `scripts/proof.sh rules <out-dir>` against `HEAD` reports no difference.
- `scripts/check.sh` passes.
- The Answer has a before → after table of the timings, per call, and says which finding moved which number.

## Progress log

- 2026-11-06: baseline is HEAD `15dc31e`, archived to the scratchpad (`t83/head-src`, `git archive`). The timing harness is `t83/harness-src/main.rs`, compiled twice in release: `t83/harness-before` (path dependencies on `head-src`'s `rules` and `seeds`, target `t83/target-before`) and `t83/harness-after` (on the working tree, `t83/target-after`). It loads `libs/game-data/`, opens the bench Builds of `libs/proof/bench-links.tsv` plus three "full" Builds from `proof export` (`full sorcerer`, `full oracle`, as `sorcerer20` and `oracle20`; the Magic Deceiver's full Build took no Living Deity, so it has no pooled slot), and times each call as the median of at least 31 runs and 1.5 s, three rounds (`t83/links.tsv`). The machine had just booted (load average ~500 from Spotlight and RustRover indexing), so the first numbers were taken once it had settled to a load of ~4.
- 2026-11-06: tests first. `libs/rules/tests/engine/indexed_lookups.rs` (7 tests) pins, through `Engine`, an answer each index could get wrong where today's scans don't. They pass on HEAD's code, as they must for a change that keeps every answer; each was then shown to catch a wrong index by breaking the new code on purpose (below).
- 2026-11-06: finding 1 built. `class_row` reads a class row by position, with the guard that the row there has that level (else a search, as before); `canonical_archetype_row` finds an archetype's row by halving its rows, which `GameData::canonical` sorts by level; `merged_row` is the one merge, shared by `effective_row` (public, unchanged for callers) and `Catalog::row`/`rows`. `Catalog::casts` reads `Casting`, built in `Catalog::new` for every class and every base class with each archetype (with the old fold as a fallback for a class it lacks). Mutations: no guard → `a_class_level_gets_the_row_of_its_own_number_when_the_table_has_a_gap` red; archetype rows read by position → `an_archetypes_rows_stored_out_of_level_order_apply_at_their_own_levels` red; casts ignoring the archetype → `whether_a_class_casts_follows_its_archetype` red.
- 2026-11-06: proof after finding 1 (`t83/proof-f1`): 1,999 Builds a side, `compare.txt` empty, `A.jsonl` and `B.jsonl` byte-identical (md5 `e0351685…`). Harness timings at that point (`t83/after-f1.tsv`): `is_possible` 0.071 → 0.065 ms, `class_options` at 7 of multi_prestige20 8.1 → 7.6 ms; the rest within noise.
- 2026-11-06: findings 2–5 built while the first proof recorded (its working-tree harness was already compiled, so the edits didn't reach it); the proof was then run once over all five (below) instead of after each, and per-finding effects were measured by leaving one finding out at a time (`t83/var-no-f2` … `var-no-f5`: HEAD's archive with the working tree's `libs/rules/src` less that finding, harnesses `t83/harness-no-f*`, shared target `t83/target-var`).
  - **2:** `Catalog::offers(selection, option)`: one it shares, or one of its own, which `indexes.options` answers (the option's defining selection is this one). Equal to `FeatureSelection::offers` while option keys are unique, which the data check requires (`DuplicateKey` per kind). Every engine call goes through it: `check_pick` (now `offers_now`, with pools), `pools`, both `slot_lists` arms, `holding_level`, `replace_row`, `grant_option_row`. `GameData`'s `offers` is unchanged for code without a `Catalog`. Mutation: the index answering any defined option → `an_option_list_slot_reads_only_the_lists_of_the_options_its_selection_offers` red (and three older tests).
  - **3:** `Catalog::single_option` reads a map built in `Catalog::new` by running the old computation (`find_single_option`) once for every indexed selection; it now returns a reference. `grant_without_slot`'s `only_without_option` check chains own and shared keys instead of building `offered()`. Mutation: singles counted from own options only → `a_selection_sharing_exactly_one_option_grants_it_with_no_pick` red (and `a_selection_offering_one_option_is_granted_with_no_pick`).
  - **4:** `check_pick` works the lists out once (it was twice: accepts, then ignores) and hands them to `check_pick_with`; `options` works them out once per picker and passes them to the candidate filter (`slot_accepts_with`) and to `check_pick_with` for every candidate. `slot_accepts_in`/`slot_ignores_in` are gone. Mutation: the picker passing only the first list → `an_option_list_pickers_statuses_are_what_the_walk_finds_of_each_pick` red (it needed the Crossblooded slot, with two lists, to catch it; added).
  - **5:** `Catalog::pool(option)`: each option's pool members (its pool selection's `offered()`, filtered by `has_option`), built in `Catalog::new` for options whose pool selection the data has. `held_pools` yields the held options' pools that the source selection offers; `offers_now` asks them without building a set, and `pools` (the picker's list) flattens them. Mutation: members not filtered by `has_option` → `a_pool_offers_only_the_options_the_data_has` red.
- 2026-11-06: the seven new tests also pass against HEAD's `rules` (`t83/head-tests`, the archive with the test file added): they pin answers, they don't change them.
- 2026-11-06: an IDE call that had timed out (RustRover's `create_new_file`, while it was indexing) landed minutes later, overwrote `indexed_lookups.rs` with its first draft and `git add`ed it. The file was rewritten and unstaged; the IDE's edit tools kept timing out, so the edits were made with the built-in tools.
- 2026-11-06: final timings, run back to back with nothing else heavy running (load average 5–9, falling; RustRover idle after indexing): HEAD, the working tree, the four leave-one-out variants, then HEAD and the working tree again (`t83/timings-final.txt`, table `t83/table.tsv`). Run-to-run noise is up to ±5 % (HEAD's two runs of the multi_prestige20 class picker: 8.05 and 8.50 ms).
- 2026-11-06: profile of the class picker at level 7 of multi_prestige20 (`t83/harness-prof`, release with debug info, macOS `sample` for 7 s, `t83/sample-class-options.txt`, symbolicated with a dSYM): see the Answer's last section.
- 2026-11-06: final proof (`t83/proof-final`): no difference, and the same records as the first run. `scripts/check.sh`: ALL-OK. Nothing committed or staged.

## Answer

**Done.** All five lookups are indexed, the engine's answers are byte-identical to HEAD's over the proof's 1,999 Builds, and `scripts/check.sh` passes. The gain is modest: 5–16 % per call, since the walks' time goes mostly to allocation and hashing of `String` keys, not to the scans (last section).

### What was built

1. **Class rows.**
   - `class_row` reads a class's row by position, guarded: when the row there isn't that level (a table that doesn't run 1, 2, … n, which the data check rejects), it searches as before.
   - `canonical_archetype_row` finds an archetype's row by halving (`partition_point`); `GameData::canonical` has sorted archetype rows by level, and the first row of a level is the one the old search found.
   - `merged_row` is the one merge, used by the public `effective_row` (same answers; its base row also uses the guarded position read) and by `Catalog::row`/`Catalog::rows` (`count_toward_features` reads `rows`).
   - `Catalog::casts` reads `Casting`, built in `Catalog::new`: every class alone, and every base class with each of its archetypes. The old fold stays as the fallback for a class it lacks.
2. **`offers`.** `Catalog::offers(selection, option)`: shared, or own by `indexes.options` (the option's defining selection is this one). It equals `FeatureSelection::offers` while option keys are unique, which the data check requires (`DuplicateKey`). The engine's seven calls go through it: `check_pick` (via `offers_now`), `pools`, both arms of `slot_lists`, `holding_level`, `replace_row` and `grant_option_row`. `GameData`'s `offers` is unchanged.
3. **`single_option`.** A map built in `Catalog::new` by running the old computation (`find_single_option`) once per selection; `single_option` returns a reference. `grant_without_slot`'s `only_without_option` check chains the own and shared keys instead of building `offered()`.
4. **`slot_lists`.** `check_pick` works the lists out once (it was twice, for accepts and for ignores) and passes them to `check_pick_with`. `options` works them out once per picker, for the candidate filter (`slot_accepts_with`) and for every candidate's `check_pick_with`. `slot_accepts_in` and `slot_ignores_in` are gone.
5. **`pools`.** `Catalog::pool(option)`: each option's pool members (its pool selection's offered options that the data has), built in `Catalog::new`. `held_pools` yields the pools of the options held that the source selection offers; `offers_now` asks them without building a set, and `pools` (the picker's list) flattens them.

Settled without asking: the archetype rows are found by halving rather than a per-archetype position table (≤ 20 rows; no extra state), and `effective_row` keeps its signature and its own search for archetype rows, since it's public and may be handed data that isn't canonical.

### Tests

`libs/rules/tests/engine/indexed_lookups.rs`, 7 tests through `Engine` on the test world. Each pins an answer an index could get wrong; each passes on HEAD's code too (`t83/head-tests`), as it must for a change that keeps every answer. Each was shown to catch a wrong index by breaking the new code on purpose:

| test | finding | turned red by |
|---|---|---|
| `a_class_level_gets_the_row_of_its_own_number_when_the_table_has_a_gap` | 1 | the row read by position without the guard |
| `an_archetypes_rows_stored_out_of_level_order_apply_at_their_own_levels` | 1 | archetype rows read by position |
| `whether_a_class_casts_follows_its_archetype` | 1 | `casts` ignoring the archetype |
| `an_option_list_slot_reads_only_the_lists_of_the_options_its_selection_offers` | 2 | `offers` answering any option the data defines |
| `a_selection_sharing_exactly_one_option_grants_it_with_no_pick` | 3 | single options counted from own options only |
| `an_option_list_pickers_statuses_are_what_the_walk_finds_of_each_pick` | 4 | the picker passing only the first list |
| `a_pool_offers_only_the_options_the_data_has` | 5 | pool members not filtered by `has_option` |

### Timings (release, native; ms, median of ≥ 31 runs and ≥ 1.5 s, the median of 3 rounds; HEAD and the working tree each run twice, back to back, and averaged)

| call | before | after | change | moved by |
|---|---|---|---|---|
| `class_options` at 7, multi_prestige20 | 8.27 | 7.35 | −11 % | 1 (−0.4 alone), 3 (−0.4 left out); 4 and 5 within noise |
| `class_options` at 1, legend40 | 2.48 | 2.32 | −6 % | spread over 1–5, each within noise |
| `options`, legend40's level-1 feat slot | 1.02 | 0.94 | −8 % | spread; this slot has no lists or pools |
| `options`, legend40's first selection slot (level 22) | 0.80 | 0.68 | −15 % | 3 (−0.055); 2, 4 and 5 about −0.02 each; 1 −0.015 |
| `removable_classes`, legend40 | 0.77 | 0.74 | −5 % | within noise |
| `apply` + `is_possible`, legend40 | 0.079 | 0.071 | −9 % | 1 (−0.007), 3 (−0.003) |
| `options`, sorcerer20's bloodline feat slot (`OptionList`) | 0.214 | 0.179 | −16 % | 4 (−0.030), 1 (−0.015) |
| `options`, oracle20's revelation slot (pooled) | 0.102 | 0.087 | −15 % | 5 (−0.013), 1 (−0.004) |
| `is_possible`, legend40 (extra) | 0.071 | 0.062 | −13 % | 1 (−0.006), 3 (−0.003) |
| `evaluate`, legend40 (extra) | 0.152 | 0.139 | −9 % | 1, 3 |

"Moved by": finding 1 from its own run (HEAD vs finding 1 alone, `t83/after-f1.tsv`); 2–5 from leaving each out of the final code (`t83/var-no-f*`). Noise between runs is up to ±5 %, so differences under about 0.3 ms (class picker) or 0.03 ms (the slot pickers) aren't attributable. The level-22 selection slot is legend40's first; it has no earlier one.

### Proof and checks

- `scripts/proof.sh rules t83/proof-f1` after finding 1, and `scripts/proof.sh rules t83/proof-final` after all five (findings 2–5 were built while the first proof recorded, so they share one proof run): both report "1999 Builds, differing: 0" with an empty `compare.txt`; `A.jsonl` and `B.jsonl` are byte-identical in both runs, and the same file each time (16.6 MB, md5 `e0351685…`).
- `scripts/check.sh`: ALL-OK (fmt, clippy, wasm clippy, the workspace tests with `wotr-planner-postgres` on 5434 up, the rules wasm build, the seed's check). No schema, query, game-data, link or UI change, so `--schema` wasn't needed and the database wasn't touched.
- `cargo test -p rules -- --list` lists the seven new tests (rules: 374 integration tests, 7 new, + 4 unit).

### What the timings point at beyond these five

Profile of the class picker at level 7 of multi_prestige20 (240 candidates):

- 81 % of it is the candidates' recheck walks (`Recheck::walks`; 71 % in `Walker::holds` itself). The per-level steps share that: the Mythic ascensions, `skill_ranks`, filling slots, `take_class_level`, and `recompute_tables` (still ~8 %, about a third of it `advancing_levels`, which scans each prestige class's rows).
- By the innermost frame: malloc/free 43 %, SipHash 14 %, `memcmp` 9 %, `memmove` 8 %. The keys are `String`s: each one held is cloned into a `BTreeMap` entry (`skill_ranks` clones each skill key every level even when it's there: `entry(skill.clone())`), every `Catalog` lookup hashes a `String` with the default SipHash, and `BTreeMap` lookups compare strings.
- Per candidate, `Recheck::keeps` clones the whole Build (~9 %), and the paused walk is cloned and later dropped (~13 %, `BTreeMap` clone and drop glue).

So the next gains are in the representation, not in more indexes: a faster hasher for the `Catalog`'s maps (FxHash or similar; no behaviour change), interned keys (`Arc<str>` or numeric ids) so cloning a state or a key doesn't allocate, and the per-candidate Build and walk clones the ticket left out of scope.
