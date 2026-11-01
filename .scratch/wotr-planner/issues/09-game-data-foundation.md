# Build the game data types, schema and seed

Type: task
Status: resolved
Blocked by:
Map: [WotR Build Planner](../map.md)

## Question

Implement the design in [Game data schema and seeding](05-game-data-schema.md) so the extractor and the rules engine have something to target.

- The `rules` crate types:
  - `GameData`, races, skills, feats with categories, classes, archetypes and prestige classes with per-level progression rows
  - feature selections, mythic abilities and mythic feats, mythic paths with their rank tables and switch rules
  - the `progression.ron` contents
  - the `Prerequisite` enum (`All`/`Any`/`AtLeast(n)`/leaf conditions)
  - text key newtypes
- Migrations for every game table, including `prerequisite_node` with its CHECK constraints and the `prerequisite_root_id` columns, plus a data-hash table.
- The `seed` binary in `data`: reads `game-data/**/*.ron`, then deletes and reloads all game tables in one transaction and stores the hash. Add the startup stale-hash warning in `server`.
- A loader that reads everything back from Postgres into `GameData`, and a server function that returns `GameData` to the browser.
- A small hand-written `game-data/` fixture: two or three races, a couple of classes and archetypes, a handful of feats with nested AND/OR/at-least prerequisites, and one mythic path. Also the round-trip test (files → Postgres → types equals files → types).
- Remove the placeholder `hello` table and `rules::Greeting`.

Work test-first (`tdd`), and keep the `rules` crate free of DB and Leptos code. AFK: driven by the agent.

## Answer

> **Superseded in part, 2026-10-31,** by [Apply the schema and layering review](14-review-fixes.md). The schema is
> now one migration, `data/migrations/20261031000000_game_data.sql`, edited in place while prototyping (the
> `…000100`, `…000200` and `…000300` files below no longer exist). No table has a `position` column (ticket 11
> removed them). `progression` and `combat_feat_category` are gone, path switches are a `path_switch` table, class
> and archetype rows use `class_key NOT NULL, archetype_key NULL`, and `data`'s API is `Database` + `data::Error`
> (no `PgPool`, no `sqlx::Error`). The server loads `GameData` once at startup. The rest of this answer is the state
> when 09 closed.

Built and verified 2026-10-31 (uncommitted). Files → Postgres → `GameData` equals files → `GameData` for the
fixture, the seed binary works, and the home page shows counts from `GameData`.

### What exists

| Path | What it is |
| --- | --- |
| `rules/src/{game_data,keys,prerequisite}.rs` | the types (serde only, builds for wasm). `rules::Greeting` is gone |
| `data/src/files.rs` | `read_game_data_files(dir)` (strict layout: a stray `.ron` is an error, a misspelt field names the file) and `game_data_hash(dir)` (SHA-256 over sorted relative paths and contents of the `.ron` files only) |
| `data/src/seed.rs` | `replace_game_data(pool, &GameData, hash)`: `TRUNCATE … RESTART IDENTITY` of every game table, then inserts, then the hash, in one transaction |
| `data/src/load.rs` | `load_game_data(pool)`: reads each table whole in `position` order (fixed number of queries) inside a `REPEATABLE READ, READ ONLY` transaction, groups rows under owners in memory |
| `data/src/prerequisites.rs` | trees in and out of `prerequisite_node`: pre-order insert (parent ids before children), read back by `parent_id` |
| `data/src/columns.rs` | enum ↔ snake_case text, `u8`/`i8` ↔ smallint |
| `data/src/lib.rs` | `seed(pool, dir) -> hash`, `game_data_status(pool, dir) -> Current / Stale { seeded, files } / NeverSeeded`, `game_data_dir()` (`$GAME_DATA_DIR`, else repo-root `game-data/`), `GameDataError` |
| `data/src/bin/seed.rs` | `cargo run -p data --bin seed [DIR]`: migrates, seeds, prints counts |
| `data/migrations/20261031000100_game_data.sql` | every game table (below) |
| `data/migrations/20261031000200_drop_hello.sql` | drops `hello` |
| `data/tests/postgres.rs` | 6 `#[sqlx::test]`s, each in its own throwaway database |
| `data/tests/fixtures/game-data/` | 3 races (one subrace), 4 skills, 11 feats with nested `All`/`Any`/`AtLeast` trees, fighter + rogue with an archetype each, arcane trickster, 2 mythic abilities, 2 mythic feats, Trickster and Legend |
| `app/src/lib.rs` | `#[server] get_game_data() -> GameData`; the home page lists counts (`GameDataCounts`) |
| `server/src/main.rs` | logs a warning at startup when the stored hash is missing or doesn't match `game_data_dir()`; never reseeds |
| `game-data/README.md` | the layout, key and prerequisite conventions, how to seed. No data yet |

### Types

`GameData { progression, races, skills, feats, classes, prestige_classes, mythic_abilities, mythic_feats, mythic_paths }`.
`Class` (base and prestige) holds `features`, `selections: Vec<FeatureSelection>` (each with `SelectionOption`s),
`levels: Vec<ClassLevelRow>` and `archetypes`. An `Archetype` has its own full `levels`, optional `skill_points` and
`class_skills` overrides. `ClassLevelRow` has BAB, saves, arcane/divine spell-level access, `advances_arcane/divine`,
granted features, `FeatSlot`s (`General`, `CombatFeat`, `Category`, `OneOf`) and selection slots. `MythicPath` has
`ranks: Vec<MythicRankRow>`, `switch: Option<PathSwitch>` and `level_caps: Option<LevelCaps>`. `Progression` has the
mythic ascension table, path choice/switch ranks, combat feat categories and `SkillRankCap::CharacterLevel`.
`Prerequisite` is `All`/`Any`/`AtLeast(n)` over 15 leaf conditions. Each key is a `String` newtype (`FeatKey`, …).

### Tables

- **Singletons:** `game_data_seed` (hash), `progression`.
- **Lists:** `mythic_ascension`, `combat_feat_category`, `skill`, `race`, `subrace`, `ability_modifier`, `feat`,
  `class` (`is_prestige`), `archetype`, `class_skill`, `feature`, `feature_selection`, `selection_option`,
  `class_level`, `mythic_ability`, `mythic_feat`, `mythic_path`, `path_switch_kept_rank`, `mythic_rank`.
- **Grants:** `granted_feature`, `granted_selection`, `feat_slot` + `feat_slot_option`, owned by a class level, a
  mythic rank or (feat slots only) a race.
- **`prerequisite_node`:** `parent_id`, `position`, `kind`, typed reference columns (feat, mythic feat/ability, class,
  archetype, feature, selection option, skill, race, all FKs), value columns (`ability`, `spell_type`, `alignment`,
  `amount`). Two CHECKs: the right columns are set for the kind, and `num_nonnulls(...)` equals the kind's count, so
  nothing else is. `feat`, `class`, `archetype`, `selection_option`, `mythic_ability`, `mythic_feat` have a nullable
  unique `prerequisite_root_id`.

### Commands (all verified)

```sh
docker compose up -d
cargo test --workspace                     # rules 2, data files 7, data postgres 6: all pass
cargo clippy -p rules -p data -p server    # clean (also with --all-targets)
cargo fmt --all --check                    # clean
cd data && cargo sqlx prepare [--check]    # 54 queries in data/.sqlx; the old greeting query is gone
SQLX_OFFLINE=true cargo leptos build       # OK
cargo run -p data --bin seed -- data/tests/fixtures/game-data
#   seeded … (hash e16bed58…): 3 races, 4 skills, 11 feats, 2 classes, 2 archetypes, 1 prestige classes, …
cargo leptos serve                         # page lists the counts and hydrates with no console messages
```

With the fixture seeded and the empty `game-data/`, the server logs
`warning: the game data tables are stale: seeded from data with hash e16bed58…, but the files in …/game-data hash to e3b0c442…`.
Set `GAME_DATA_DIR=data/tests/fixtures/game-data` to make the check pass.

### Gaps in the design, filled here

- **Several possible owners:** a row that can belong to different parents (feature, selection, level row, feat slot,
  grant, ability modifier, class skill) has one nullable column per owner, a `num_nonnulls(...) = 1` CHECK, and
  `UNIQUE NULLS NOT DISTINCT` (Postgres 15+) on owner + position.
- **Order:** every list has a `position` column so the round trip is exact, including top-level file order.
- **Insert order:** every FK is `DEFERRABLE INITIALLY DEFERRED`, so references (including cross-class ones like a
  prestige class granting `feature:rogue/sneak-attack`) are checked at commit. A dangling reference fails the whole
  seed and leaves the previous data and hash in place (tested).
- **Prestige classes** share the `class` table with an `is_prestige` flag, as they share the `Class` type.
- **Archetype class skills:** `overrides_class_skills` stores `Some(vec![])` apart from `None`. The old type comment
  said an empty override couldn't be stored; it now can.
- **Path switch and level caps** are nullable column groups with all-or-none CHECKs; kept ranks are a child table.
- **Enums** are snake_case text with CHECKs, not Postgres enum types, so `query!` needs no type overrides.
- **Leaf values** share one `amount` column (level, score, ranks, count, BAB, or `n` for `at_least`).
- **Data folder:** `$GAME_DATA_DIR` overrides the repo-root default for both the seed and the startup check.
- **The `hello` migration file stays.** It's already applied, and sqlx refuses to run when an applied migration is
  missing, so a new migration drops the table instead.

### Not done

- No CHECK stops a leaf node from having children; the loader never writes one.
- The seed binary and the startup warning are checked by hand, not by tests; the library functions they call are
  tested.
