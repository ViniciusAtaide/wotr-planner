# Move the database from Postgres to SQLite

Type: task
Status: open
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
