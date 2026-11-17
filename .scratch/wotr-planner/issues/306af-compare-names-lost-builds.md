# Importer S2, 306af: the form proof names the Builds the regenerated corpus lost

Type: build (tools)
Status: resolved
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

From the rescue retros of 306j and 306k (a cause seen twice, so a mechanical fix). A split that gives a class its own entry leaves the corpus generators (`tools/corpus/src/builds/`) naming the old one. The regenerated corpus then loses Builds, or holds Builds that no longer open, and only a reviewer reading counts noticed it.

## Change

`compare` in `tools/corpus/tests/form_proof.rs` lists each Build the record holds that the tip's corpus doesn't, and each Build that opened on the base but not on the tip, with the generator file and line that made it (`tools/corpus/src/builds/`, found by its name). The stage's Answer explains each or fixes the generator.

## Done when

A test with a planted lost Build shows its name and its generator line. `cargo test -p corpus` and clippy on corpus with `--all-targets -D warnings` pass, and so do fmt and `scripts/test-modules.sh`.

## Answer

Built in `tools/corpus/tests/form_proof.rs` only (Sonnet builder; the change and its scope are the ticket's).

- **`compare` matches Builds by name**, not by index (a lost Build shifted every later one). It lists each Build the record holds that the corpus doesn't (`lost Build`), each new one, and each that opened on the base but not on the tip (`opened on the base, not on the tip`), every one with its generator lines, and fails on any (they are differences).
- **Generator lines** (`generator_lines`): a string literal in `tools/corpus/src/builds/*.rs` or `generate.rs` whose fixed text around its `{…}` placeholders the name has, best fit (most fixed text) first, three at most, printed as `path:line: text`; a `full <name>` Build also names its base Build's line; a `bench <name>` Build is its line of `bench-links.tsv`. A name built from data may match none, and says so. The stage's Answer explains each lost Build or fixes the generator.
- **Tests** (`cargo test -p corpus`, all pass): `a_lost_build_is_named_with_its_generator_line` (a planted source: the lost Build's name and `planted.rs:3`; the opened-on-base case gives line 5), `a_full_build_names_the_generator_of_the_build_it_fills` (also the bench line), `the_real_generators_are_found_by_a_build_s_name` (`t55 fighter <key> / empty` finds `mythic.rs`'s literal). Their names print in `--list`. No game fact changes, so no game citation applies.
- **Duplicate names** (review): `record_all` and `read_entries` panic naming any Build name two Builds share, since `compare` matches by name. Test `two_builds_with_one_name_are_refused` (planted duplicate). `record` on the real corpus passed: no duplicate name exists today.
