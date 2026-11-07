# Move the database from Postgres to SQLite

Type: task
Status: resolved
Blocked by: [84](84-precompressed-assets.md) (it edits `apps/backend/src/main.rs`)
Map: [WotR Build Planner](../map.md)

## Question

The user asked to replace Postgres with SQLite if it doesn't slow the app down (2026-11-06), to drop Docker, the container, port 5434 and a service that must be running (today `scripts/check.sh` failed because Docker wasn't up). A feasibility review (2026-11-06; scratchpad `sqlite-review/`: `translate.py`, `sqlite.sql`, `bench/`) found:

- **Performance:** negligible. The server's startup load is the same (Postgres `load_game_data` 32 ms median; the same reads on SQLite through sqlx ~23.6 vs ~21.9 ms). Seeding into a new SQLite file took 359 ms with the foreign-key check, against an estimated 2 s or more on Postgres.
- **The schema translates** (checked against sqlite3 3.54, with constraint, FK and error-message probes):
  - 16 `CREATE DOMAIN`s → the base type plus a named `CONSTRAINT <domain> CHECK` on each of the 59 columns (errors still name the domain, e.g. `CHECK constraint failed: u8`);
  - regex `~` → `GLOB`/`instr`;
  - 6 `GENERATED ALWAYS AS IDENTITY` → `INTEGER PRIMARY KEY`;
  - `TIMESTAMPTZ DEFAULT now()` → `TEXT DEFAULT CURRENT_TIMESTAMP`;
  - `num_nonnulls`/`starts_with` → sums of `IS NOT NULL` and `substr`;
  - the prerequisite column-set CHECK's arrays → `concat_ws` with `COALESCE` (a NULL must not pass the CHECK);
  - 9 `UNIQUE NULLS NOT DISTINCT` → unique expression indexes over `COALESCE`;
  - the 14 `ALTER TABLE … ADD FOREIGN KEY` → inline;
  - the 106 `DEFERRABLE INITIALLY DEFERRED` FKs, composite ones included, work as written.
- **Foreign keys stay the check for missing references** (the user's decision, 2026-11-03): sqlx-sqlite turns `foreign_keys` on by default (`sqlx-sqlite-0.9.0/src/options/mod.rs:187`). A failure at commit only says `FOREIGN KEY constraint failed`, so the seed runs `PRAGMA foreign_key_check` before `COMMIT` and reports every violation (table, row, parent table), which is more than Postgres's first one.
- **Code changes:** `libs/seeds/src` (pool and connection types; sqlx-sqlite infers `i64` for `SMALLINT`, so the 25 `i16` binds and `columns.rs`'s conversions change; `SET TRANSACTION … READ ONLY` goes; `TRUNCATE … RESTART IDENTITY` goes; nullability inference from `EXPLAIN` is weaker, so expect some `"col!"` overrides), `libs/seeds/tests/postgres.rs` (28 tests; ~24 message assertions change), `.sqlx/` (regenerate), the root `Cargo.toml` (sqlx feature), `.env`/`.env.example`, `docker-compose.yml`, the README, `CLAUDE.md`'s reset steps, `scripts/check.sh` and the map's Notes. The server's `main.rs` changes only in its messages.
- **Reseeding:** `DELETE`ing in place took 720 ms just for the deletes (SQLite scans child tables). Seed into a new file next to the database and rename it over the old one: faster, and atomic.

A no-database option (the server reads `libs/game-data/` directly; ~5.7k lines removed) was also sized, but it reverses the user's relational-schema decision and moves the FK checks into Rust; the user chose SQLite. Client-side SQLite was ruled out (a 3.3× bigger download than the JSON, and per-lookup queries 100–1000× slower than the engine's maps).

## Build

The user is awake: settle small choices yourself and record them here; bring a real open question back.

- **Driver:** sqlx's `sqlite` feature (it builds SQLite's C source once, through `libsqlite3-sys`; a C compiler is needed, which macOS has). A pure-Rust engine was checked and ruled out (2026-11-06): Turso (the Rust rewrite of SQLite, v0.8.3) has no first-party sqlx driver (sqlx issue #2674 is open); the only one, the third-party `sqlx-turso` 0.1.0-alpha.1, is a work in progress last updated 2026-05, with its own macros and no `#[sqlx::test]`; and Turso doesn't support `PRAGMA foreign_key_check`, which the seed needs.
- **The database file:** one file, path from `DATABASE_URL` (`sqlite://…`), defaulting to a git-ignored file in the repo (say where in the README). The server opens it read-only.
- **The migration:** still one initial migration (the map's "one migration while prototyping"), rewritten for SQLite from the review's `sqlite.sql` (start from it, then read it against the Postgres original line by line). Name every table-level CHECK (SQLite reported an unnamed one by a column's name). Use `STRICT` tables if the column types allow it, so a type is enforced without a CHECK; say in the Answer if not.
- **The seed:** into a new file in the same directory, `PRAGMA foreign_key_check` before `COMMIT` (every violation reported, the seed fails on any), then rename over the database. Keep the stored hash and the server's staleness warning.
- **Tests:** `#[sqlx::test]` works with SQLite (a database file per test, no server). Keep every test's meaning; change only what SQLite's messages and types force, and list those in the Answer.
- **Scope (user, 2026-11-06):** SQLite on the server only; the browser keeps getting the game data as JSON, exactly as today (`apps/backend/src/game_data.rs`'s route and its encodings, from ticket 84).
- **Remove Postgres and Docker** (user, 2026-11-06: "remove docker-compose since we won't have a db"): delete `docker-compose.yml` (it holds only this project's Postgres), the sqlx `postgres` feature, every Docker and Postgres step and the port 5434 in the README (its prerequisites list, the run steps, the note on the `seeds` tests) and in `.env.example`. Docker is no longer needed for this project. Update `CLAUDE.md`'s "Database" steps (delete the file, `sqlx migrate run`, seed, `cargo sqlx prepare`) and the map's Notes ("Stack", "Game data", the 2026-11-03 FK note now says the database's foreign keys).
- **Leave alone:** the `wotr-planner-postgres` container and its data (the user removes them when they're ready; say how in the Answer), and never touch the `postgres` container on 5432.

**Done when:**
- `scripts/check.sh --schema` passes with Docker stopped or the container down (show it).
- A fresh clone's steps work from the README: migrate, seed, `cargo leptos serve`, and the app loads the game data (browser pane: no console errors, a pick works).
- The loaded game data equals what the files read (`postgres.rs`'s round-trip test, renamed).
- `scripts/proof.sh rules <out-dir>` against `HEAD` reports no difference (the engine sees the same data).
- `scripts/check.sh --full` passes.
- The Answer has the startup load and seed times, before → after.

## Progress

- 2026-11-06: before timings taken read-only from `wotr-planner-postgres` (status + load, release); the loaded data saved as JSON for the after comparison. HEAD's seed timed against a throwaway `postgres:18` container on port 5499, stopped and removed afterwards (5434 and 5432 untouched).
- Migration rewritten for SQLite from the review's `sqlite.sql` (a new translation script, then read against the Postgres original table by table): STRICT tables, named CHECKs, unique indexes, inline foreign keys. Probed in `sqlite3` (keys, domains, the prerequisite column-set CHECK).
- `seeds`: SQLite pool and connection; `Database::open` (read-only, refuses another migration's database); `database_path()`; the seed builds `<db>.new`, checks the foreign keys, renames. sqlx-cli reinstalled with its `sqlite` feature (and `postgres`, kept for other projects).
- Tests: `tests/postgres.rs` → `tests/database.rs`, 29 pass. `.sqlx` regenerated. `scripts/check.sh --schema` passes against a temp migrated file.
- Docs: README, `CLAUDE.md`, `.env.example`, `.gitignore`, `scripts/check.sh`, the map's Notes. `docker-compose.yml` deleted.
- Fresh clone (the working tree copied to a scratch folder, no `.env`, no database): the README's steps (copy `.env`,
  `sqlx database create && sqlx migrate run`, seed, `cargo leptos serve` on port 3940) worked; browser pane: the game
  data loaded, no console messages, picking Dwarf as the race worked and wrote the link (`#b=AWOc_eo9AyMDEHCBAYjFAAA`).
  The game data route still answers brotli (100,862 B) or gzip (116,569 B) with the same headers. Server stopped.
- `scripts/proof.sh rules` against `6663281` (HEAD when it ran): 1999 Builds on each side, 0 differing.
  `scripts/check.sh --schema --full`: ALL-OK.

## Answer

Done: the database is one SQLite file, the server opens it read-only, and Docker and Postgres are gone from the
project. The browser still gets the same JSON (the loaded game data equals Postgres's, field for field).

**Small choices settled here:**
- The file is `wotr-planner.db` at the repo root, git-ignored (`/wotr-planner.db*` also covers the seed's
  `.new` file and journals). `DATABASE_URL=sqlite://wotr-planner.db` in `.env.example`; without `DATABASE_URL` the
  seed and server use the same file and the query macros the `.sqlx` metadata. A relative path is resolved against
  the repo root (`seeds::database_path()`), as rustc does for the macros, so running a binary from another folder
  can't pick a different file.
- The server never migrates (it opens the file read-only). `Database::open` refuses a database whose
  `_sqlx_migrations` (versions and checksums) aren't this build's, with "reseed it": the read-only equivalent of the
  checksum error the old startup migration gave after an edited migration.
- The seed reports the foreign key failures as `seeds::Error::MissingReferences`, one line each: table, rowid, the
  foreign key's columns and values, and the parent table and columns, e.g.
  `prerequisite_node row 12: feat_key 'no-such-feat' is not in feat key`.
- `scripts/check.sh --schema` checks the queries against a new file migrated in a temporary folder, so it needs
  neither the dev database nor a server; `cargo sqlx prepare` gets the database's absolute path (`-D`), because it
  runs in `libs/seeds` while the macros resolve paths from the repo root.

**Schema translation** (`libs/seeds/migrations/20261031000000_game_data.sql`, still one migration, 42 tables):

| Postgres | SQLite |
| --- | --- |
| 16 `CREATE DOMAIN`s | The base type and a `CONSTRAINT <domain> CHECK (...)` on each of the 59 columns of the type, so an error still names it (`CHECK constraint failed: u8`). A header comment lists them. |
| `text_key`'s regex `~` | Four GLOBs: starts and ends with `[a-z0-9]`, nothing outside `[a-z0-9/-]`, no two of `-`/`/` in a row (the same language as the regex; probed with the tests' bad keys). |
| `hash ~ '^[0-9a-f]{64}$'` | `length(hash) = 64 AND hash NOT GLOB '*[^0-9a-f]*'` |
| `SMALLINT`, `BIGINT` | `INTEGER` |
| `BOOLEAN` (15 columns) | `INTEGER` with `CONSTRAINT boolean CHECK (x IN (0, 1))`; the loader reads them `AS "x: bool"` |
| 6 `BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY` | `INTEGER PRIMARY KEY` (the rowid; an explicit id is no longer refused, and the seed never gives one) |
| `game_data_seed.singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton)` | `INTEGER NOT NULL PRIMARY KEY DEFAULT 1 CHECK (singleton = 1)` |
| `TIMESTAMPTZ NOT NULL DEFAULT now()` | `TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP` (UTC) |
| `num_nonnulls(a, b, ...) = 1` | `((a IS NOT NULL) + (b IS NOT NULL) + ...) = 1` |
| `starts_with(k, p)` | `substr(k, 1, length(p)) = p` |
| The prerequisite column-set CHECK: `array_remove(ARRAY[...], NULL) = CASE kind ... '{a, b}'` | `concat_ws(',', ...) IS CASE kind ... 'a,b'`; `feat_for`/`feat_if_value` build `'feat_key,' \|\| COALESCE(<the value column>, '')`. `IS`, not `=`, and the `''`, because a CHECK lets a NULL through. |
| 9 `UNIQUE NULLS NOT DISTINCT` | 9 unique expression indexes `<table>_unique`, with `COALESCE(col, '')` (text) or `COALESCE(col, 0)` (ids, which start at 1) on the nullable columns only |
| 14 `ALTER TABLE ... ADD FOREIGN KEY` | Inline `REFERENCES` on the column (SQLite allows forward references) |
| 106 `DEFERRABLE INITIALLY DEFERRED`, composite ones included | Kept as written; sqlx turns `foreign_keys` on |
| Unnamed table-level CHECKs (Postgres named them `<table>_check<n>`) | All 46 named `<table>_<rule>` (`feature_key_under_owner`, `feat_slot_one_owner`, ...) |
| Text `PRIMARY KEY`s | `NOT NULL PRIMARY KEY`, explicit (STRICT implies it too) |
| `CREATE INDEX prerequisite_node_parent_id` | Kept |

**STRICT: yes**, on all 42 tables. Every column is INTEGER or TEXT, so a type is enforced without a CHECK
(`cannot store TEXT value in INTEGER column`). The one cost is booleans: STRICT has no BOOLEAN, so they are INTEGER
with the `boolean` CHECK, and sqlx infers `i64` for them, hence the 15 `AS "x: bool"` overrides in `load.rs`. Five
`RETURNING id`s read `AS "id!"` (sqlx-sqlite can't tell they're non-null). The integer columns read as `i64`
(`to_u8`, `to_i8`, `repeat_from` and the stored prerequisite node's `amount` take `i64`); the `i16` binds stay.
`SET TRANSACTION ... READ ONLY` and `TRUNCATE ... RESTART IDENTITY` are gone (the file is read-only to the server,
and the seed always writes an empty file).

**The seed** (`seeds::replace_game_data`, `libs/seeds/src/seed.rs`): `GameData::check`, then build
`<database>.new` (any one a killed seed left behind is removed first): create the file, run the migration, insert
everything in one transaction, run `PRAGMA foreign_key_check` (through `pragma_foreign_key_check` and
`pragma_foreign_key_list` for the columns and values) before `COMMIT`; any failure rolls back and returns every one.
Then close the pool and rename the file over the database (atomic, same folder). On any error the `.new` file and
its journal are removed and the old database stays as it was. The stored hash and the server's staleness warning are
unchanged. A running server keeps reading the file it opened until it restarts.

**Tests changed** (`libs/seeds/tests/postgres.rs` → `tests/database.rs`, 28 → 29 tests, all pass):
- The round trip is `files_to_database_to_game_data_equals_files_to_game_data` (was `files_to_postgres_...`).
- The 26 tests that seed are `#[tokio::test]`s on a `TestDatabase`: a file in a temp folder, which the seed builds
  and renames into place. A pool opened on the old file would never see the new one, so a test that writes to the
  database takes `test.writer()` (one connection) after each seed. The 2 that need only the empty schema
  (`status_is_never_seeded_before_the_first_seed`, `a_prerequisite_node_must_fill_exactly_its_own_columns`) stay
  `#[sqlx::test]`, now on a `SqlitePool` (a file under `libs/seeds/target/sqlx/test-dbs`).
- Messages: SQLite says `CHECK constraint failed: <name>`, `FOREIGN KEY constraint failed`, `UNIQUE constraint failed:
  index '<name>'`. The 9 copies of the `rejected` closure became one `rejected(&pool, sql)` helper that lowercases
  the error, and two direct assertions lowercase too, so every expected substring (`check constraint`, `foreign key`,
  `unique`, the domain names) is unchanged.
- `the_seed_refuses_data_that_fails_the_data_check` ended by asking the status (`NeverSeeded`); now it asserts no
  database file exists (nothing was written, and there is no database to ask).
- New: `a_failed_seed_names_every_missing_key_and_leaves_no_new_file` (two missing keys in two tables both reported,
  the database and the `.new` file absent).

**Removed:** `docker-compose.yml`; the sqlx `postgres` feature (`cargo tree -i sqlx-postgres` prints nothing:
nothing in the build can talk to Postgres); the Docker/Postgres/5434 lines in the README and `.env.example`; 15 stale
`.sqlx` files (13 new, 75 rewritten).

**Docs updated:** README (summary, prerequisites: sqlx-cli with `sqlite` and a C compiler instead of Docker, run
steps, the game data paragraph, the layout table, the tests note, queries and offline builds), `CLAUDE.md`'s
Database steps, `.env.example`, `.gitignore` (`target/` anywhere for the sqlx test files, `/wotr-planner.db*`),
`scripts/check.sh`'s header and `--schema`, the map's Notes (Stack, Game data's seed step and one-migration line, the
foreign keys note), and the migration's, `seeds`' and the seed binary's docs.

**Times** (this machine, release builds):

| | Postgres (before) | SQLite (after) |
| --- | --- | --- |
| Startup: connect / open | 53 ms | 0.9 ms |
| Startup: staleness check + load, median of 9 (3 runs) | 38–39 ms | 41–43 ms |
| Seed, whole process | 4.4–4.6 s (5.0 s with the migration; HEAD's seed against a throwaway `postgres:18` on port 5499) | 0.72–0.73 s (0.69 s in `seeds::seed`; 2.0 s in a debug build) |

The load is about 3 ms slower, and startup as a whole about 50 ms faster (no connection, no migration run). The
seed is about 6× faster. Before was measured read-only on `wotr-planner-postgres`; the throwaway container was
stopped and removed.

**Checks:** `scripts/check.sh --schema` passes with nothing pointing at Postgres (`.env` and `.env.example` name the
SQLite file, and the build has no Postgres driver); `scripts/check.sh --schema --full`: ALL-OK; `scripts/proof.sh
rules` against `6663281`: 1999 Builds each side, 0 differing; the loaded real data equals the files and Postgres's
load.

**Removing the old container and volume** (when you're ready; `docker compose down -v` no longer works, since the
compose file is gone):

```sh
docker stop wotr-planner-postgres && docker rm wotr-planner-postgres
docker volume rm wotr-planner-pgdata
docker image rm postgres:18   # optional: only that container uses this image
```
