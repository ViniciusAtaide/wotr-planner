# Apply the normalization review

Type: task
Status: resolved
Blocked by: 14
Map: [WotR Build Planner](../map.md)

## Question

A read-only normalization review (2026-10-31) found that all 25 tables hold guide facts, and that none is a rule in disguise; `selection_option` is a real entity. The user approved its small fixes. AFK: driven by the agent, test-first (`tdd`).

### Fixes

1. **Drop `ability_modifier.kind`.** "Chosen" means `ability IS NULL`. The seed and loader match on `ability`; `rules::AbilityModifier` stays as it is.
2. **Move `level_cap` from `mythic_path` to `path_switch`.**
   - Rust: `PathSwitch.level_cap: Option<u8>`, and `MythicPath.level_cap` is removed.
   - Update the engine's switch handling, `GameData::check()`, and the fixtures `legend.ron` and `gold-dragon.ron`.
3. **Reference `mythic_ascension (rank)`** from `mythic_rank.rank` and `path_switch_kept_rank.rank`, replacing their copied `CHECK 1..10`.
4. **`prerequisite_node`:** merge the two kind CHECKs (the column count and the named columns) into one per-kind CHECK that names exactly the non-null columns for each kind, and document what `amount` means for each kind in it.
5. **Loader holes:** these become errors, with tests:
   - `class_skill` rows under an archetype with `overrides_class_skills = FALSE`. Today `load.rs` drops them with `then_some`.
   - Orphan or shared prerequisite nodes. Add a `Trees::finish()` that checks every node is reached exactly once, from exactly one owner, and that every root has `parent_id IS NULL`.
6. **Docs:**
   - the migration header: the truncate lives in `data/src/seed.rs`, and the surrogate-key claim is wrong
   - a dated superseded note on ticket 05's stale Answer (`progression.ron` contents, key example)
   - the stale "26 tables" line in `map.md`

### Deferred, waiting on the real guide data (ticket 10)

These stay out of this ticket; add them to the map's Not yet specified as one item:
- archetype level rows that repeat the base class (the user's earlier decision stands until real data shows how often archetypes differ)
- `kept_through_rank` replacing `path_switch_kept_rank`
- set vs multiset for `granted_*`
- whether options are shared across selections
- one_of feat slot vs feat-granting selection
- whether `level_cap` and `further_ascensions` are independent
- whether mythic `group_name` is a closed set

## Answer

Applied test-first and verified 2026-10-31 (uncommitted). All six fixes are done; none was skipped. The deferred
list is untouched and sits in the map's Not yet specified as "Schema questions for the real guide data".

### Fixes

1. **`ability_modifier.kind` dropped** (done). A NULL `ability` is the chosen kind. The seed writes
   `(race_key, subrace_key, ability, amount)`, and the loader maps `Some(ability)` to `Fixed` and `None` to `Chosen`.
   `rules::AbilityModifier` is unchanged. `ModifierKind` and its `text_column!` are gone from `data/src/columns.rs`.
   Test: `an_ability_modifier_without_an_ability_is_the_chosen_kind` (a row with no kind and no ability loads as
   `Chosen`).
2. **`level_cap` moved to `path_switch`** (done). `PathSwitch { kept_ranks, further_ascensions, level_cap:
   Option<u8> }`, and `MythicPath.level_cap` is removed. The engine's `switch_path` reads `switch.level_cap`, and
   `GameData::check` checks the 20–40 range inside the switch. `legend.ron` now says `switch: (kept_ranks: [1, 2],
   further_ascensions: false, level_cap: 40)`. `gold-dragon.ron` had no cap to move, so only its comment changed: its
   switch has no `level_cap`, so the cap stays 20. The engine's test world, `facts.rs` and `check.rs` use the new
   shape. The round-trip test covers the Legend cap through Postgres, and the schema test rejects
   `path_switch.level_cap = 41`.
3. **Ranks reference `mythic_ascension (rank)`** (done). `mythic_rank.rank` and `path_switch_kept_rank.rank` are
   deferred foreign keys to `mythic_ascension (rank)`, not copies of `CHECK (rank BETWEEN 1 AND 10)`. Test: the schema
   test inserts rank 11 into each and expects a foreign key error, where it used to get a check constraint error.
   "Kept ranks below the switch rank" stays in `GameData::check`.
4. **One per-kind CHECK on `prerequisite_node`** (done). The column-count CHECK and the named-columns CHECK are now
   one CHECK. It compares the names of the non-null columns, as an array, with the exact list for the kind (`'all'
   → '{}'`, `'class_level' → '{class_key, amount}'`, …). Its comment says what `amount` is for each kind: `at_least`'s
   n, the level of `class_level`, `spell_level`, `caster_level` and `character_level`, `feature`'s count (granted at
   least that many times), `skill_ranks`' ranks, `ability_score`'s score and `base_attack_bonus`'s BAB. The new CHECK
   accepts and rejects exactly what the two old ones did, so no test could go red first. The existing test gained
   four cases (a valid empty group, `class_level` without its level, `at_least` without n, a `feat` with a stray
   amount), and `every_prerequisite_variant_round_trips` still stores all 18 kinds.
5. **Loader holes are errors** (done).
   - `class_skill` rows under an archetype with `overrides_class_skills = FALSE` used to be dropped by `then_some`.
     They are now an error: `archetype:…: class_skill rows under an archetype that keeps its class's class skills`.
     Test: `class_skills_under_an_archetype_that_keeps_the_base_list_are_an_error`.
   - `Trees::get` (now `&mut self`) refuses a root that has a parent, and counts every node it reaches. The
     loader calls `Trees::finish()` last, which errors on a node reached 0 times ("belongs to no owner") or more
     than once ("belongs to N owners"). Refusing a parented root also means the walk can't loop: below a parentless
     root, every node has exactly one parent. Test: `every_prerequisite_node_belongs_to_exactly_one_owner` covers an
     orphan tree, one tree shared by a feat and a class, and an owner pointing into the middle of another tree.
6. **Docs** (done).
   - The migration header names `data::replace_game_data` in `data/src/seed.rs`, reached through `data::seed`, as the
     code that truncates. The surrogate-key bullet now says that multisets (granted features and selections, feat
     slots, ability modifiers) have only an `id`, that `class_level` and `mythic_rank` have an `id` for their grants
     and keep a UNIQUE natural key, and that `prerequisite_node.id` is the tree's link. Before, it called
     prerequisite children a keyless multiset and left out `class_level`/`mythic_rank`.
   - Ticket 05's Answer opens with a dated "superseded in part" note. It says `progression.ron` holds only the
     mythic ascension table, and that `selection:rogue/rogue-talents/<slug>` is a Selection option's key, not the
     selection's.
   - `map.md`'s line for 09 says "every game table" instead of "26 relational tables". Its note gives 25 now, 26 when
     09 closed.
   - Also updated: `game-data/README.md` (a path file's switch rule carries the level cap), the `prerequisite_node`
     table comment (a root has no parent, and each node is in exactly one owner's tree, checked by the loader) and
     the `load.rs` module docs.

### Columns

Still 25 tables.

| Table | Before | After |
| --- | --- | --- |
| `ability_modifier` | `kind TEXT NOT NULL CHECK (kind IN ('fixed', 'chosen'))`, `CHECK ((kind = 'fixed') = (ability IS NOT NULL))` | no `kind`; `ability` NULL means chosen |
| `mythic_path` | `level_cap SMALLINT CHECK (20–40)` | no `level_cap` |
| `path_switch` | `path_key`, `further_ascensions` | plus `level_cap SMALLINT CHECK (level_cap BETWEEN 20 AND 40)`, nullable |
| `mythic_rank` | `rank SMALLINT CHECK (rank BETWEEN 1 AND 10)` | `rank` FK → `mythic_ascension (rank)`, deferred |
| `path_switch_kept_rank` | `rank SMALLINT CHECK (rank BETWEEN 1 AND 10)` | `rank` FK → `mythic_ascension (rank)`, deferred |
| `prerequisite_node` | `num_nonnulls(…) = CASE kind …` plus `CASE kind WHEN … THEN col IS NOT NULL …` | one CHECK: the array of non-null column names equals the kind's list; `amount` documented per kind |

Rust: `PathSwitch.level_cap` added, `MythicPath.level_cap` removed, `data::columns::ModifierKind` removed,
`Trees::finish` added.

### Verified

```sh
cargo test --workspace                                 # rules 1 + 90, data files 10, data postgres 14: all pass
cargo clippy -p rules -p data -p server --all-targets  # clean
cargo fmt --all --check                                # clean
cargo build -p rules --target wasm32-unknown-unknown   # OK
SQLX_OFFLINE=true cargo leptos build                   # OK
cd data && cargo sqlx prepare --check                  # OK (52 queries)
cargo run -p data --bin seed -- data/tests/fixtures/game-data
#   seeded … (hash 7335210b…): 3 races, 4 skills, 12 feats, 3 classes, 3 archetypes, 2 prestige classes, 2 mythic abilities, 2 mythic feats, 3 mythic paths
```

The dev database was dropped, recreated, migrated and reseeded per `CLAUDE.md`, and `data/.sqlx` was regenerated.
The three new tests are in `data/tests/postgres.rs` and listed by `cargo test -p data -- --list`.
