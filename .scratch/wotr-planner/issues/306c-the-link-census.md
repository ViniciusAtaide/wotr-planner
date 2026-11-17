# Importer S2, 306c: the link census in the form proof

Type: build (the importer, stage S2)
Status: resolved
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as the section of [slices.md](../research/306/slices.md) says, with the differences below. No game fact changed: no data or rules file is touched, so there is no game citation; the proof is a record and compare on one tree.

**What was built**
- `tools/corpus/src/links.rs` (new, `pub`): `carried`, `heirs`, `assert_same_picks` lifted from `tests/links.rs`, and `fate(engine, data, keys, link, old_picks)` returning `Same`, `Heirs`, `Refused` (the failing picks) or `Changed`.
- `tests/form_proof.rs` `compare`: opens each old link that opened with this tree's data and prints `links: N the same, N through an heir, N refused, N changed`; `Changed` is a difference. `FORM_PROOF_LINKS_OUT=<file>` writes `name<TAB>slot<TAB>key<TAB>kind` (kind `heir`, key `old=>new`; or `refused`).
- `tests/links.rs`: `Stage` (a base file and a refused file) and `problems`; fixture `tests/links/fixture-base.tsv` (the corpus's `bench start`, `bench fighter20`, `bench legend40` links) and `fixture-refused.txt`; S2's `306-base.tsv` (2,000 links of the corpus's open Builds at base `5f32a115`) and `306-refused.txt` (empty).

**Decisions**
- Heir vs same is judged against the picks the link held when made (the record's `picks` text), not against `decode_build`: decoding already carries a pick whose old key has one heir (`codec.rs:367`), so the decoded Build alone would call it "same" (found by the first fixture run). The links test takes the old link keys for that (user's design, §4.2; main session's decision 2 for the committed links; this builder's call for the old-picks comparison).
- The base links file is 611,670 bytes, not the estimated 160 KB (links are about 300 bytes each): flagged for the main session.
- The fixture's bench links are the corpus's re-encoded ones: the raw bench links are alignment-without-faith, which open stale (ticket 94).

**Tests** (`tools/corpus/tests/links.rs`): `a_base_link_opens_the_same_through_an_heir_or_is_a_listed_refusal` (same 3/0/0; renamed with alias 2/1/0; renamed without alias, listed, 2/0/1; unlisted fails), `a_listed_refusal_that_opens_fails`, `every_s2_base_link_opens_the_same_or_through_an_heir_or_is_listed`.

**Proof:** `record` then `compare` on this tree: `links: 2000 the same, 0 through an heir, 0 refused, 0 changed`; 2161 Builds compared (2000 open), 0 differences. Checks: `cargo test -p corpus`, clippy `--all-targets -D warnings`, fmt, `scripts/test-modules.sh` all pass.

**Review fixes.** `moved_since` matches an old map entry only after `{` or `, `, so `Selection { .. }: ` no longer matches inside `OptionSelection { .. }: ` (test `a_selection_slot_is_not_matched_inside_an_option_selection_slot`, in `src/links.rs`). The S2 test's doc says its heir count is meaningful only in `compare`, since it reads the current data's keys.
