# The server reads the game data files; no database

Type: refactor
Status: resolved
Blocked by: none
Model: Opus to build (the seeds crate, the server, the scripts and the docs; the checked JSON compared byte for byte).
Map: [WotR Build Planner](../map.md)

## Problem

The `.ron` files in `libs/game-data/` are the source of truth, but the web server didn't read them. A seed copied them
into a SQLite file (`wotr-planner.db`, 12 migrations, sqlx with checked-in `.sqlx` query data), and the server loaded
that file at startup, only to serialise it to JSON for the browser. Nothing else read the database: the MCP server
already read the files (`seeds::check_game_data_files`), and so did every real-data test and the proof.

On 2026-11-13 this gave an empty class picker: the dev server restarted during a database rebuild and loaded the
empty tables. Beside that hazard the database cost a schema per data change (a migration, a reseed, `cargo sqlx
prepare`, `--schema` in the checks), per-checkout rebuild rules, a seed lock, a stale-data warning, and a C compiler
for SQLite.

The user asked why the files aren't turned into the browser's JSON directly (2026-11-13). That reverses their
2026-11-06 note "SQLite stays for now" (map, Stack), which they have now asked for.

## Answer

Done 2026-11-13.

1. **The server reads the files.** `apps/backend/src/main.rs` loads `seeds::check_game_data_files(&seeds::game_data_dir())`
   once at startup, the MCP server's own call, and serves its JSON as before (`GameDataFile`: the content-hash URL
   `/game-data/<hash>.json`, `Cache-Control: public, max-age=31536000, immutable`, never compressed). Data with a
   problem stops the server with the list. The MCP server it serves beside the site answers from the same data
   (`mcp::Planner::new`, now public), so the files are read once, not twice.
2. **The database is gone.** The migrations, sqlx (workspace dependency and its `sqlx-macros` profile), the
   `.sqlx` query data, the seed and load code (`seed.rs`, `load.rs`, `columns.rs`, `prerequisites.rs`,
   `bonuses.rs`, `gear.rs`), the `Database` type, the data hash (`game_data_hash`, only for the stale-data warning)
   and the seed's lock. The seeds crate is now `files.rs` and `lib.rs`: `read_game_data_files`,
   `check_game_data_files`, `game_data_dir` and an `Error` of two cases (the files, the check); its dependencies are
   `ron`, `rules` and `serde`. The crate keeps its name, which every test command, `CLAUDE.md` and the proof's side
   A use.
   - The `seed` binary is `check-data` (`cargo run -p seeds --bin check-data [DIR]`), the old `--check-only`:
     it reads and checks a folder and prints the counts or the problems. `scripts/check.sh` runs it as `data-check`.
   - `.env.example`: no `DATABASE_URL` or `SQLX_OFFLINE`; the server's `.env` is optional. `CLAUDE.md`'s Database
     section is a short Game data section; `--schema` is gone from it and from `scripts/check.sh`. `README.md`,
     `libs/game-data/README.md` (Seeding is Checking) and `EXTRACTION_REPORT.md` say the same.
   - `scripts/build-site.sh` seeds nothing; it sets `GAME_DATA_DIR` to `libs/game-data/` for the server, since a
     developer's `.env` could name another folder and dotenvy never overrides a variable already set (`unset`
     alone didn't stop that). `pages.yml`, `build-mcp.sh` and the pre-commit hook lose `SQLX_OFFLINE` and `.sqlx/`.
   - `scripts/proof.sh` keeps `SQLX_OFFLINE=true`: its side A may be a commit from before this ticket, which still
     compiles the query macros. `.gitignore` keeps `/wotr-planner.db*`, for the files older checkouts still hold.
3. **Tests.** `libs/seeds/tests/database.rs` tested the tables (round trips, the schema's constraints, seeding and
   the stale-data status) and is deleted. Two of its tests checked the data, not the tables, and moved to
   `tests/files.rs` against `check_game_data_files`: every missing key is named, and where each problem is. The
   two hash tests went with the hash. The real-data tests (`tests/game_data/`, the snapshot check and the gold
   builds among them) already read the files and are untouched.
4. **`GameData::check` is the only reference check** (it named every missing key before the foreign keys did since
   ticket 98; they were a backstop). An audit of the final schema's 157 foreign keys against the check: each one
   that holds a data reference is checked, of the right kind and often more narrowly (an option inside its
   selection, a domain among the domains); the rest follow from the types' nesting (an archetype's class) or are
   row ids. No gap.

   The schema's CHECK and UNIQUE constraints held eight rules that neither the types nor the check did. They are
   the check's now, each with a test in `libs/rules/tests/engine/check.rs`:
   - `MalformedKey`: every key is lowercase slugs (`a-z`, `0-9`, single `-` or `/` between them), the `text_key`
     column type;
   - `PoolOfItself`: a selection isn't a pool of itself;
   - `MythicRanksOffAPath`: only a path's selection is levelled by the Mythic ranks reached;
   - `AbilityBonusTwice`: a feature has one ability bonus of each kind (and ability, for a fixed one);
   - `EmptyText`: a slot's label, an interactions note, and a gear item's name and guide reference, when there,
     aren't empty.

   One wasn't carried over: the selection option and mythic feat tables took no `metamagic`, `stat` or `element`
   parameter. Ticket 125 widened the feat and mythic ability tables for its picks and left these two, which no data
   needed: a column's limit, not a rule of the game.

**Checks.**

- The browser's JSON is the same, byte for byte (`cmp`): the old server's, made before any change as
  `GameDataFile::new` made it (the seed into a new database, `load_game_data`, `serde_json::to_vec`; 2,035,984
  bytes, so `/game-data/ce253b55287c77b2.json`), against the new dev server's response. The page names the same URL,
  and the response keeps `Cache-Control: public, max-age=31536000, immutable`. The server reads, checks and
  serialises the files in about 320 ms at startup (a debug build).
- `scripts/build-site.sh` builds the static site without a seed, and its `game-data/ce253b55287c77b2.json` is the
  same file again.
- The snapshot files are untouched; `scripts/check.sh` and `scripts/check.sh --full` pass.
- The app works in its own dev server (port 3960): the page loads the data, "+ Level 1" opens the class picker
  with every class, and choosing Alchemist makes level 1 with its numbers, with no console error.
