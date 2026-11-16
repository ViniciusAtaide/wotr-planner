# Importer S1, 293e: the user-fact registry and guard

Type: build (the importer, stage S1)
Status: resolved
Blocked by: 293d
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [293](293-the-importer.md)'s stage S1, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/293/slices.md](../research/293/slices.md). The decisions are in [research/293/decisions.md](../research/293/decisions.md) and in that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as written in [research/293/slices.md](../research/293/slices.md) §293e, with 293d's review findings.

- **What:** `src/write/facts.rs`: `Facts` (`embedded`, `parse`, `decide(area, kind, entry) -> Decision { Free, Applied(ticket), Held }`, `stale(&data)`), in `known.txt`'s grammar with `-> applied <ticket>` or `-> held`; the most specific line wins (entry, then kind, then area). `tools/game-check/user-facts.txt`: the class skills and skill points held, the 13 class-skill entries and the Defender of the Hearth's skill points applied 262, the Arcane Trickster's skill points applied 261, and the 22 guideless gear files held. `src/write/mod.rs`: `run(dump, dir, areas, dry_run, &Facts, validate)` drops a held change into `Outcome.to_the_user`, records `applied <ticket>` by entry and field in `Outcome.facts`, and fails with `WriteError::StaleFacts` before planning when a line names an entry the data lacks. `src/write/log.rs`: `tsv` takes the facts for its `fact` column. `src/main.rs`: prints "to the user".
- **293d's findings:** `--log` is created before the dump is read, so a bad path fails first; `--write` with a lookup word is refused; `log.rs` documents that `findings removed` is per (`area / kind`, entry), so a finding whose text changed, or one of two in a group, shows `-`.
- **Decisions of mine:** the registry refuses a value in a line (a change has none). A change's registry area is the check's `Area` of its `WriteArea` (`WriteArea::area`). An `items` change's kind is `bonuses`, so the item lines use `*`. The headband +8's line waits for 293k.
- **Tests:** `a_held_fact_is_not_written_and_goes_to_the_user`, `an_applied_fact_is_written_and_logged_with_its_ticket`, `a_fact_naming_no_entry_fails_the_run_and_writes_nothing`, `the_registry_refuses_what_it_cannot_read`, `the_embedded_registry_names_only_entries_the_data_has` (real data), and in the new `tests/fixture/command.rs` (the binary): `a_log_that_cannot_be_written_stops_the_run_before_the_dump_is_read`, `write_with_a_stray_word_is_refused_and_runs_no_lookup`.
- **Proof:** the game check's stdout is byte-identical (`cmp`); `game-check --write items --dry-run` on main's data prints the same 4 items as in 293d, none held.

**Review fixes, second commit:** `user-facts.txt` gains `items / * / gear item belt-of-physical-perfection-8 -> applied 288` (ticket 288, line 15: the user's decision on the +8s; the headband's line waits for 293k). Two lines of one specificity for one thing (`area` and `area / *` count as one) are a parse error naming the first. `facts.rs` carries a `TODO(293h)`: when `ClassTables` becomes a `WriteArea`, a test must check that every non-`*` kind in the registry is one the check emits. Tests: `the_most_specific_line_wins` (kind over area; entry over kind, on the real class-table shape), `two_lines_of_one_specificity_are_a_parse_error`.
