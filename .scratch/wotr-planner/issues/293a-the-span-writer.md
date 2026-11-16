# Importer S1, 293a: the span writer

Type: build (the importer, stage S1)
Status: resolved
Blocked by: none (tools only; beside 305)
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [293](293-the-importer.md)'s stage S1, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/293/slices.md](../research/293/slices.md). The decisions are in [research/293/decisions.md](../research/293/decisions.md) and in that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as written in [research/293/slices.md](../research/293/slices.md) §293a (the main session's decisions; no new ones).

- **What:** `tools/game-check/src/write/span.rs` (new, lifted from 296's rewrite crate): `field_span`, `ron_text`, `Value`, `write_field`, `SpanError { CommentInside, NoAnchor }`, and the declared orders `CLASS_FIELDS`, `ARCHETYPE_FIELDS` (`bab` between `casting_ability` and `fort`), `GEAR_FIELDS`. `write/mod.rs` holds the module doc; `lib.rs` has `pub mod write;`; `Cargo.toml` takes `ron` and `serde` as dependencies (ron leaves the dev-dependencies).
- **Beyond 296's code:** the scanner works on bytes (no panic inside a multi-byte character) and an insert after a field with a trailing `//` comment goes after that comment's line.
- **Tests** (`tests/fixture/span.rs`, no game data, so no game citation): `replacing_a_field_keeps_every_other_byte`, `a_value_that_spans_lines_is_written_one_element_a_line`, `a_one_line_file_stays_one_line`, `a_missing_field_goes_after_the_last_field_before_it`, `removing_a_field_takes_its_line`, `a_comment_inside_the_field_is_refused`, `a_name_in_a_string_or_a_nested_struct_is_not_the_field`. The first run failed for the missing `mod span;` (0 tests found), not by assertion.
- **Proof:** the game check's stdout is byte-identical before and after (`cmp`).
