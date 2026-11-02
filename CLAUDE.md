# wotr-planner

The plan for this project lives in `.scratch/wotr-planner/map.md`. Read its Notes before starting any work, and its tickets under `issues/` for the task at hand.

## Database (prototype stage)

The schema lives in one initial migration, `libs/seeds/migrations/20261031000000_game_data.sql`, and the data is only seed data. A schema change edits that file. Then reset the dev database:

1. Drop and recreate `wotr_planner` in `wotr-planner-postgres` (port 5434).
2. Run `sqlx migrate run --source libs/seeds/migrations`.
3. Reseed.
4. Run `cd libs/seeds && cargo sqlx prepare`.

Do this only when you are the only agent working in the repo. If another agent is running, leave the database as it is and report the schema change you need.

## Rust tests

Cargo compiles only the test modules that are declared:

- **Folder test crates** (`<crate>/tests/<name>/`) compile as one crate rooted at `main.rs`. Each new file in the folder gets a `mod <file>;` line in that `main.rs`, and shared helpers get `mod support;`.
- **Top-level test files** (`<crate>/tests/*.rs`) are each their own crate. Each one that uses the shared helpers in `tests/common/mod.rs` declares `mod common;`.

A new test is done when its name shows up in `cargo test -p <crate> -- --list`.

## Formatting

Rust is formatted with rustfmt, the official formatter, through `cargo fmt`. Its style is pinned in `rustfmt.toml` (`style_edition = "2024"`), so `cargo fmt`, the rustfmt hook and RustRover's Reformat Code all produce the same output.

- **After an edit:** a hook runs rustfmt on every `.rs` file an agent writes.
- **Before you finish:** `cargo fmt --all --check` passes.
- **RustRover's `reformat_file` tool** uses the IDE's built-in formatter rather than rustfmt, so Rust formatting goes through `cargo fmt`.
