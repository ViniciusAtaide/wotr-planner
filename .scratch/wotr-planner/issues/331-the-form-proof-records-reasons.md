# The form proof records reasons

Type: build (tooling)
Status: resolved
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

## Answer

Built in `tools/corpus/tests/form_proof.rs` and CLAUDE.md's form-proof bullet, as design-X §3 describes.

- **Record.** `FORM_PROOF_REASONS=1` on `record` implies the offers and shares their pass (`offers_pass` now yields a `Found`: the offers and, with reasons, each failed choice's text). It writes `reasons.bin` and `reasons-texts.txt`: per Build the name, each refused option (hash of slot and pick, as `offers.bin`, and an index into the interned texts), then each `Engine::failures` choice (hash of its `Debug` and an index); streamed, the writer keeps only the distinct texts. One function, `reason_text(engine, &Issue)`, makes every text (`issue_text`, `"BreaksLater"` for its `None`): ticket 336 swaps it.
- **Compare.** Options refused on both sides and failures by choice are compared; each changed text groups by its edit (common prefix and suffix removed); `reasons: N compared, N changed in N edits`, each edit with count, Builds and an example (`FORM_PROOF_REASONS_SHOWN`). It fails on an edit not in `FORM_PROOF_REASONS_ACCEPT` (`before<TAB>now`, escaped, `#` comments; documented in the header), on a stale accepted edit, and on a failure that appeared or went. With `FORM_PROOF_CENSUS`, changes on log-named keys or in refused-link Builds are counted apart and don't fail. `FORM_PROOF_REASONS=1` on `compare` fails on a record without reasons.
- **Decisions.** Opt-in, accept-file and `reason_text` as the brief set (main session). Mine: a failure that appeared or went always fails (it can't be accepted by an edit; with the census it is excused only for a log-named or refused-link Build); `compare` reads the reasons whenever the record has them, without the variable.
- **Size and time (same tree, release).** `record` with `FORM_PROOF_OFFERS=1 FORM_PROOF_REASONS=1`: `reasons.bin` 182,716,604 bytes (15,217,717 refused options, 220 failures), `reasons-texts.txt` 293,674 bytes (4,258 texts), `offers.bin` 164,014,763 bytes; the test ran 45 s (75 s with the build) at a load of 4 to 13. The offers-only run took 49 s at a load above 23 (another builder was running), so the reasons' added cost is not cleanly measured; no median of alternating runs was taken.
- **Compare output (same tree).** `reasons: 15217937 compared, 0 changed in 0 edits`, `failures: 0 appeared, 0 gone`, `offers: ... 0 statuses changed`, and `2157 Builds compared (1992 open), 2157 old links reopened, 0 differences, 0 offer differences, 0 reason problems`; 48 s.
- **Tests** (`pieces`, in the normal run, each in `--list`): `an_edit_is_the_texts_with_their_common_prefix_and_suffix_removed`, `the_accept_file_holds_edits_tab_separated_with_comments_and_blank_lines_skipped`, `the_texts_table_keeps_each_text_once_and_survives_newlines`, `an_ok_option_is_not_recorded`, `changed_reasons_group_by_their_edit_across_options_and_builds`, `an_edit_fails_unless_it_is_accepted_and_an_accepted_edit_nothing_used_fails`, `a_failure_is_compared_by_its_choice_and_one_that_appears_or_goes_fails`, `with_a_census_only_changes_the_log_and_refused_links_do_not_name_fail`, `the_reason_is_the_engines_text_and_breaks_later_has_none`. They check the record's format and the compare's logic, not a game fact: the texts are the engine's own, so no game citation applies (the proof's purpose is to list what a later ticket moves).
- **Left.** No median-of-alternating-runs timing on a quiet machine. The census split of reasons was tested on synthetic Builds only; no census ticket has run it on the corpus.
