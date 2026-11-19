# The form proof records reasons

Type: build (tooling)
Status: open
Blocked by: —
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)
Design: [design-X.md §3 and §4 T0](../research/architecture/design-X.md), in the frame of [overall.md](../research/architecture/overall.md); decisions in [decisions.md](../research/architecture/decisions.md).

## Problem

The form proof (`tools/corpus/tests/form_proof.rs`, ticket 300) records status kinds, never texts. A form-only ticket can't prove no reason moved, and a wording ticket can't list what it changed. Tickets 332–336 of the six deepenings change or must keep reason texts.

## Change

An opt-in `FORM_PROOF_REASONS=1` on `record` streams each non-ok option's reason and each Failure's text, interned, beside the statuses. `compare` groups changed reasons by their edit (the texts with common prefix and suffix removed) and fails unless each edit is listed in `FORM_PROOF_REASONS_ACCEPT`. It combines with `FORM_PROOF_OFFERS` and `FORM_PROOF_CENSUS` as design-X §3 says. Reasons come from one function, `reason_text`, which ticket 336 swaps for the refusal module's text.

## Done when

- `record` and `compare` on one tree report `0 changed`.
- The pieces tests pass, with each new test in `cargo test -p corpus -- --list`.
- The test's header documents the variable, its size and its measured time.
- CLAUDE.md's form-proof bullet names `FORM_PROOF_REASONS=1` for a ticket that changes or must keep reason texts.

## Files

`tools/corpus/tests/form_proof.rs`, `CLAUDE.md`.

## Checks

`cargo test -p corpus`, `cargo clippy -p corpus --tests -- -D warnings`, `cargo fmt --all --check`, and the same-tree `record`/`compare` run twice (release, `--ignored`), timed.
