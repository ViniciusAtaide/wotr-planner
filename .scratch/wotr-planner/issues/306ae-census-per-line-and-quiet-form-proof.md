# Importer S2, 306ae: the census attributes each line, and the form proof skips when unasked

Type: build (tools)
Status: resolved
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

## Answer

Built in `tools/corpus/tests/form_proof.rs` and one clause of CLAUDE.md's form-proof line. No game fact changed: no data or rules file is touched, so there is no game citation; the proof is the census on 293l's records.

**What was built**
- `FORM_PROOF_CENSUS=<change log>`: `compare` reads the log (`read_change_log`) and attributes each differing line of `builds.tsv` (the link, picks and decoded picks as one line), `slots.txt`, `sheets.txt`, `numbers.txt`, and the reopened Builds' slots and sheets (`Attribution::lines`). The Build's classes and archetypes come from this tree's Build (`Entry::held`). A `numbers.txt` line's kind comes from its shape (`numbers_kind`: header, totals, skill, spell, attack, steps).
- The table `FIELDS`: field to the line kinds it moves, and whether it regenerates the corpus's full Builds. `bab`: header, attack, regenerates. `fort`, `reflex`, `will`, `save_exceptions`: header, totals. `class_skills`: skill lines. `skill_points`: skill lines, regenerates. A log field the table lacks panics, naming it, so a later stage adds its row (S2's option lists, S4's prerequisites and S5's rows may use `Slot`, `Sheet`, `Link`, or `regenerates`).
- `compare` prints `census <field>: N lines in N Builds (per file)`, each unattributed line, `census: N unattributed`, and fails on any.
- `record` and `compare` print `<var> is not set: skipped` and pass when their variable is unset.

**Decisions**
- Builder: `regenerates` exists because 293l's own log needs it. A `bab` or `skill_points` change alters the generator's draws (feat picks, ranks, ability increases), so those 4 Builds differ in every kind of line. For such a Build (its link or slots changed) every line goes to the regenerating field it holds; any other Build is judged by line kind. A changed link on a Build with no regenerating field is unattributed.
- Builder: the census replaces the Differences for link, picks, decoded picks, slots, sheet and numbers only when `FORM_PROOF_CENSUS` is set; without it `compare` is as before.

**Tests** (`pieces`, in `form_proof.rs`): `a_missing_variable_skips_the_mode_and_a_set_one_names_its_folder`, `a_numbers_line_is_classified_by_its_shape`, `the_change_log_reads_each_entry_key_with_its_fields`, `a_log_field_the_table_lacks_is_refused`, `a_line_a_held_field_moves_is_attributed_and_an_unrelated_one_is_not`, `a_regenerated_build_attributes_all_its_lines_to_the_field_that_regenerates`, `a_line_one_side_lacks_differs`. They were written with the code, not seen failing first (a test-only tool).

**Proof.** 293l's `proof-before` against this tree (it contains 293l) with 293l's log: bab 156 lines in 3 Builds, class_skills 529 in 19, fort 804 in 28, save_exceptions 440 in 220, skill_points 376 in 2, will 296 in 8; 0 unattributed; 2161 Builds compared, 0 differences. Planted throwaway: one attack line of the `wizard stalwart-defender` record changed (+0 to +90) in a copy of the record: `UNATTRIBUTED numbers.txt: wizard stalwart-defender: Attack`, 1 unattributed, `compare` failed. Not committed.
