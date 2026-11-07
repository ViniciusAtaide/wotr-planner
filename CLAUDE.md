# wotr-planner

The plan for this project lives in `.scratch/wotr-planner/map.md`. Read its Notes before starting any work, and its tickets under `issues/` for the task at hand.

## Database (prototype stage)

The database is one SQLite file, `wotr-planner.db` at the repo root (`DATABASE_URL` in `.env`; git-ignored). The schema lives in one initial migration, `libs/seeds/migrations/20261031000000_game_data.sql`, and the data is only seed data. A schema change edits that file. Then rebuild the dev database, from the repo root:

1. Delete it: `rm -f wotr-planner.db`.
2. Run `sqlx database create && sqlx migrate run --source libs/seeds/migrations`, so the query macros compile against the new schema.
3. Reseed: `cargo run -p seeds --bin seed` (it builds a new file from the migration and renames it over this one).
4. Run `(cd libs/seeds && cargo sqlx prepare -D "sqlite://$PWD/../../wotr-planner.db")`.

Do this only when you are the only agent working in the repo. If another agent is running, leave the database as it is and report the schema change you need. `scripts/check.sh --schema` doesn't need the dev database: it checks the queries against a new file migrated in a temporary folder.

## Rust tests

Cargo compiles only the test modules that are declared:

- **Folder test crates** (`<crate>/tests/<name>/`) compile as one crate rooted at `main.rs`. Each new file in the folder gets a `mod <file>;` line in that `main.rs`, and shared helpers get `mod support;`.
- **Top-level test files** (`<crate>/tests/*.rs`) are each their own crate. Each one that uses the shared helpers in `tests/common/mod.rs` declares `mod common;`.

A new test is done when its name shows up in `cargo test -p <crate> -- --list`.

## Checks

Before reporting a change done, run `scripts/check.sh` (add `--schema` after a migration or query change; `--full`, the cargo-leptos builds, only at the end of a queue of tickets). A pre-commit hook in `.githooks/` runs its fmt and clippy steps; enable it in a new clone with `git config core.hooksPath .githooks`.

## Behaviour proof

To prove a change to `rules` or `libs/game-data/` changes no behaviour but the intended one, run `scripts/proof.sh` (usage in its header and in `tools/proof`'s crate docs); a ticket filters its comparison for any special breakdown.

## Formatting

Rust is formatted with rustfmt, the official formatter, through `cargo fmt`. Its style is pinned in `rustfmt.toml` (`style_edition = "2024"`), so `cargo fmt`, the rustfmt hook and RustRover's Reformat Code all produce the same output.

- **After an edit:** a hook runs rustfmt on every `.rs` file an agent writes.
- **Before you finish:** `cargo fmt --all --check` passes.
- **RustRover's `reformat_file` tool** uses the IDE's built-in formatter rather than rustfmt, so Rust formatting goes through `cargo fmt`.
