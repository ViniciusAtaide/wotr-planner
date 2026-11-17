# Importer S2, 306ae: the census attributes each line, and the form proof skips when unasked

Type: build (tools)
Status: open
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

Two findings from S1 (main session, 2026-11-16):
1. **The census attributes by Build, not by line.** 293l's census classifier (`scratchpad/293l/classify`) counted a changed Build as explained when it held a class the change log touched. It never checked that the changed line's quantity is one that field can move: a save field moves the save lines, `bab` the attack lines, `class_skills` the skill lines, `skill_points` the skill-point lines. An unrelated number change in such a Build would pass as "0 unattributed". Each importer stage proves itself by this census (design §4.2), and S2–S5 change far more lines.
2. **`cargo test --release -p corpus -- --ignored` fails.** CLAUDE.md's long-run command also runs the form proof's `record` and `compare`, which panic without `FORM_PROOF_OUT` or `FORM_PROOF_BEFORE` (`tools/corpus/tests/form_proof.rs`, around line 81). They've done so since ticket 300.

## Change

1. **A committed census in the form proof.** `compare` reads `FORM_PROOF_CENSUS=<change log>`, the TSV `game-check --write --log` writes, and attributes each differing line of `numbers.txt`, `sheets.txt`, `slots.txt` and `builds.tsv` to a log row. A line is attributed when the Build holds the row's entry and the line is one of the quantities that row's field moves. That table, field → line kinds, is written once in the test. `compare` prints the attributed counts per field and the unattributed lines, and fails on any unattributed line. It replaces the per-ticket classifiers.
2. **`record` and `compare` print a skip line and pass when their variable is unset,** so the long-run command passes. A test covers it.
3. **CLAUDE.md's form-proof line** names `FORM_PROOF_CENSUS` for a stage that changes values.

## Done when

- `cargo test -p corpus` passes, and `cargo test --release -p corpus -- --ignored` passes.
- 293l's base and tip, with its change log (`scratchpad/293l/`), give 0 unattributed lines under the new census.
- A planted unrelated number change in a Build holding a changed class is reported unattributed. It's a throwaway, not committed.
- clippy on corpus with `--all-targets -D warnings`, fmt and `scripts/test-modules.sh` pass.
