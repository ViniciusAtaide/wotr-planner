# Importer S2, 306b: option lists and ownership

Type: build (the importer, stage S2)
Status: resolved
Blocked by: 306a
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [306](306-importer-selections.md), the importer's stage S2, cut to about 10–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof, the expected counts and the files it shares are under its heading in [research/306/slices.md](../research/306/slices.md). The decisions are in [research/306/decisions.md](../research/306/decisions.md) and that file's §0.

## Done when

Its section in `slices.md` is built and proved as written there, and the scoped checks pass.

## Answer

Built as the slice's section says (`research/306/slices.md`, "306b") and with 306a's review folded in. `--write option-lists` now also brings a selection of options to its blueprint's `m_AllFeatures`: its `options` are the options it owns, its `shared_options` the rest. Not run on the real data.

**What was built** (`tools/game-check/src/write/selections.rs`, rewritten around two phases)
- Phase 1 resolves every selection of options (not only those in `--only`): each game member to one option entry; a GUID several options name resolves to those the selection offers, else to the one of them some selection offers, else it is ambiguous and the selection is unwritten and listed (decision 3).
- Phase 2 plans ownership (decision 4): an option stays with its owner while the owner's game list holds it, else moves to the first selection in key order whose list holds it, else stays and is listed. An owner whose list can't be read keeps its options. A move is made only when both selections are in `--only`; otherwise the option stays and is listed. The lists are then written with the span writer (`kind: "list"`; an emptied field is removed).
- 306a's review: a selection with no `blueprint`, or whose GUID is not in the dump, is listed ("no blueprint", "blueprint not in the dump"). `--only` without `option-lists` among the areas is refused in `main.rs`; an `--only` key that is no selection is `WriteError::Usage`, printed as itself, not as a failed check.

**Decisions:** the slice's. The builder's: the "none offered and one holds it" case of decision 3 is read as the one of the options that some selection offers; an owner that is a selection of feats or whose list can't be resolved keeps its options.

**Tests** (`tests/fixture/write.rs`): `an_option_moves_to_the_selection_whose_game_list_holds_it`, `an_option_no_game_list_holds_stays_and_is_listed`, `a_guid_two_options_hold_resolves_to_the_offered_one`, `a_move_outside_only_is_not_made`, `the_written_lists_pass_the_data_check` (no `OptionNotDefinedOnce`), `a_selection_with_no_blueprint_or_none_in_the_dump_is_listed`, `an_only_key_that_is_no_selection_is_a_usage_error`; `tests/fixture/command.rs`: `only_without_the_option_lists_area_is_refused`. Expected values come from the fixture's invented blueprints (no game fact changed). The fixture gained seven blueprints and the options and selections around them.

**Dry run** (`--write option-lists --dry-run`, main's data at e5eaeb6b): 46 fields in 45 files: the 31 of 306a and 15 option lists, all `shared_options` (the copies: Empyreal, Sage and Sylvan bloodlines, Hag-Riven, Wind Whisperer, Lone Strider, the Elemental Specialist, Feyform Shifter and others); no option moves. Left unwritten: 88 lines, the shaman's hexes, 294's domains, the Lich's upgrades, Finesse Training, the G5 choices, selections with no blueprint, and 14 options no game list holds.
The check's report is byte-identical to the base's (`cmp`).

**Safety after review (the main session's decision).** A copy's blueprint is its base's, so a full run would widen the copy to the base's list (`sorcerer/sage-sorcerer/bloodline` from `[arcane]` to all 21 bloodlines; the same for Wind Whisperer, Empyreal, Sylvan, Lone Strider and the Elemental Specialist): those are G5, ticket 316. So `--write option-lists` without `--only` is a usage error ("option-lists writes only the selections `--only` names (copies share their base list's blueprint: ticket 316)"), and a selection whose blueprint another planner selection names too is listed as unexpressed ("its blueprint is another selection's too (a copy: 316)") unless both are in `--only`; it stays unresolved in the ownership plan, so its options stay. Tests: `option_lists_without_only_is_refused` (`command.rs`), `a_selection_whose_blueprint_is_another_selections_too_is_listed_unless_both_are_in_only` (a copy pair in the fixture). The full dry run above is therefore no longer a command; under `--only magic-feats,teamwork-feats` it writes 2 fields in 2 files.
