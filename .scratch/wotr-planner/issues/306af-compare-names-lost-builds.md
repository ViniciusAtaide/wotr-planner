# Importer S2, 306af: the form proof names the Builds the regenerated corpus lost

Type: build (tools)
Status: open
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

From the rescue retros of 306j and 306k (a cause seen twice, so a mechanical fix). A split that gives a class its own entry leaves the corpus generators (`tools/corpus/src/builds/`) naming the old one. The regenerated corpus then loses Builds, or holds Builds that no longer open, and only a reviewer reading counts noticed it.

## Change

`compare` in `tools/corpus/tests/form_proof.rs` lists each Build the record holds that the tip's corpus doesn't, and each Build that opened on the base but not on the tip, with the generator file and line that made it (`tools/corpus/src/builds/`, found by its name). The stage's Answer explains each or fixes the generator.

## Done when

A test with a planted lost Build shows its name and its generator line. `cargo test -p corpus` and clippy on corpus with `--all-targets -D warnings` pass, and so do fmt and `scripts/test-modules.sh`.
