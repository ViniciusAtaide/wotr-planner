# Importer S4, 308b: the form proof's `prerequisite` census row

Type: build (the importer, stage S4)
Status: resolved
Blocked by: —
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [308](308-importer-prerequisites.md), the importer's stage S4, scoped to ranks 1 and 7 of [the queue's scope](../research/queue-scope-2026-11-17.md) (user, 2026-11-17). The goal, the edit list, the tests by name, the proof and the expected counts are under its heading in [research/308/slices.md](../research/308/slices.md); that file's §0 and [research/308/decisions.md](../research/308/decisions.md) hold the decisions. A data slice also follows [research/306/decisions.md](../research/306/decisions.md): the generator follows the game with the Build count held, refusals are counted in link lines, a moved option keeps its gate.

## Done when

Its section in `slices.md` is built and proved as written there, as amended by `decisions.md`, and the scoped checks pass.

## Answer

Built in `tools/corpus/tests/form_proof.rs` only (no data or engine change, no form proof: the unit tests are the proof).

- `FIELDS` gains a `prerequisite` row: all nine `LineKind`s, `regenerates: true`.
- The census's held set is the Build's classes and archetypes (`record_entry` keeps them) plus the pick keys of the base and the tip: `fn pick_keys` repeats `pick_key`'s scan over the picks' `Debug` text, and `held_with_picks` unions them in `compare` and in the reopened census.
- `read_change_log` reads an entry with `rsplit_once(' ')`, so `selection option a/b` and `mythic ability x` read as `a/b` and `x`. The header doc names the row.
- Decisions: the slice's edit list (main session, 2026-11-17); no others.

Tests (in `pieces`): `a_prerequisite_row_attributes_every_line_of_a_build_holding_the_entry_at_the_base` (a Build whose base picks name the entry, with an empty tip, has all ten lines of every file attributed to `prerequisite`; a Build holding nothing of the row is not), `a_change_log_entry_with_a_two_word_kind_reads_as_its_key`. Both failed first (`held_with_picks` not found), then passed. Expected values come from the change log's own format (`game-check --write --log`: `entry<TAB>field`), not the game: this ticket changes no game fact.

Checks: `cargo test -p corpus` ok (last: 1 passed, 1 ignored), `cargo fmt --all --check` clean, `scripts/test-modules.sh` OK.
