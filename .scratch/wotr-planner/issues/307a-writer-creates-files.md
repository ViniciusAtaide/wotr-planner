# Importer S3, 307a: the writer creates new files

Type: build (the importer, stage S3)
Status: resolved
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [307](307-importer-carriers.md), the importer's stage S3 (the carrier entries), cut to about 15–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/307/slices.md](../research/307/slices.md). The decisions are in [research/307/decisions.md](../research/307/decisions.md).

## Done when

Its section in `slices.md` is built and proved as written there. S3 is a form change, so its census must be empty. The scoped checks pass.

## Answer

Built as the section of [slices.md](../research/307/slices.md) says. No data changed, so no note file and no form proof (the section's Proof is none).

**What was built** (`tools/game-check/`)
- `src/write/mod.rs`: `NewFile { at, file, text, kind }`; `apply`, `apply_with` and `plan` take `&[NewFile]` beside `&[Change]`. `fn creatable(file)` is true for `features/<key>.ron` only. Guards before any write, in path order: `WriteError::Planner` for a `planner/` path, new `WriteError::Exists` for a path that exists (a symlink counts) or that two new files share, `WriteError::NotOwned { field: "(new file)" }` for a path `creatable` refuses. Files are written after the field changes, their folders created first, each opened with `create_new` (not a rename), so a path taken after the plan fails with `Exists` and is neither replaced nor removed by the undo (review fix). Read-back per file: the disk text equals `text`, it parses as `rules::Feature` (`IMPLICIT_SOME`, `deny_unknown_fields`), its `key` is the path's stem and its `blueprint` is set (else `ReadBack`). On any failure (write, read-back, validate) every created file and every folder the run made are removed and field files put back as before; one that can't be removed is named in `RestoreFailed`. `Applied.created`; `Outcome.created` (empty until 307b makes the first plan one).
- `src/write/log.rs` `tsv` takes the created files: one row each, field `(new file)`, old `-`, new `<key> <blueprint>`.
- `src/main.rs` `write_run`: prints `<area>: N files to create|created` and each path for an area's new files; "nothing to write" counts them.

**Decisions** (builder, from the section): `Exists` also covers two new files with one path, compared lowercased (macOS); folders are removed only if the run made them.

**Tests** (`tests/fixture/write.rs`): `a_new_file_is_written_whole_and_read_back`, `an_existing_file_is_never_replaced_by_a_new_one`, `a_new_file_under_planner_or_outside_features_is_refused`, `a_failed_validation_removes_every_new_file` (also a file that is no feature, a key that is not its stem), `a_dry_run_lists_the_new_files_and_writes_none` (through `plan`, since no area plans a file yet), `a_file_planted_after_the_plan_is_neither_replaced_nor_removed`, `two_new_files_a_case_insensitive_disk_would_merge_are_refused`. The existing `apply` tests pass `&[]`. There is no game fact: the tests use the fixture, and the new tests were written with the code (the API did not exist, so there was no earlier red run).
