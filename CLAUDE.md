# wotr-planner

The plan for this project lives in `.scratch/wotr-planner/map.md`. Read its Notes before starting any work, and its tickets under `issues/` for the task at hand.

## Database (prototype stage)

The database is one SQLite file per checkout, `wotr-planner.db` at the checkout's root (`DATABASE_URL=sqlite://wotr-planner.db` in `.env`, a relative path; git-ignored). Its data is only seed data. The schema is the migrations in `libs/seeds/migrations/`, which the seed runs in order on every new file it builds.

**A schema change adds a new migration**, `libs/seeds/migrations/<YYYYMMDDHHMMSS>_<ticket>-<what>.sql` (for example `20261109120000_121-spells.sql`), timestamped when you write it. Never edit a migration that is already on `main`, since another branch may build on it. The database is always rebuilt from scratch, so a migration may create, alter, or drop and recreate tables. When no other branch or agent is open, the main session may squash the migrations into one, in a commit of its own.

After a schema change, rebuild your checkout's database, from its root:

1. Delete it: `rm -f wotr-planner.db`.
2. Run `sqlx database create && sqlx migrate run --source libs/seeds/migrations`, so the query macros compile against the new schema.
3. Reseed: `cargo run -p seeds --bin seed` (it builds a new file from the migrations and renames it over this one).
4. Run `(cd libs/seeds && cargo sqlx prepare -D "sqlite://$PWD/../../wotr-planner.db")`.

**Each checkout has its own database**, so agents in separate worktrees rebuild theirs at the same time without touching each other's:
- In a new worktree, copy `.env.example` to `.env` first; its relative `DATABASE_URL` then names that worktree's own file.
- Build in the worktree's own `target/` (the default). Never share `CARGO_TARGET_DIR` between worktrees: the seed and the server find the workspace root, and with it the database file and `libs/game-data/`, from the path they were compiled in, so a shared build can write another worktree's database.
- Agents sharing one checkout share its database. There, rebuild it only when you are the only agent in that checkout; otherwise leave it as it is and report the schema change you need.

`scripts/check.sh --schema` doesn't need the dev database: it checks the queries against a new file migrated in a temporary folder.

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
