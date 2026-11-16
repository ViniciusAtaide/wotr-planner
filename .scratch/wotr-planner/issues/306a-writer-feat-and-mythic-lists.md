# Importer S2, 306a: the selections target, feat and mythic lists, --only

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

Built as the slice's section says (`research/306/slices.md`, "306a"); `--write option-lists` brings a selection of feats or of mythic abilities, and its `ignores_prerequisites`, to its blueprint's `m_AllFeatures` and `IgnorePrerequisites`. It is not run on the real data: the data slices run it with `--only`.

**What was built** (`tools/game-check`)
- `src/areas/options.rs`: `game_list(json)` lifted from `list`; the check's output is unchanged (its report is byte-identical to the base's, `cmp`).
- `src/planner.rs`: `Planner::entries_of(guid)`.
- `src/write/selections.rs` (new): `changes(ctx, only)`. Kind by decision 3 (feats, then mythic abilities; a selection of options is 306b's and is skipped). All or nothing (decision 2): a game member with no entry of the kind, or with several, leaves the selection unwritten and listed; so does a game list with no `m_AllFeatures`, or an empty one (written empty, the selection would have no list). `feats`/`mythic_abilities` are written as sorted quoted keys; `ignores_prerequisites` only where `offers_feats` and it differs. `kind: "list"`; `old`/`new` are the keys removed/added.
- `src/write/span.rs`: `SELECTION_FIELDS`, in `FeatureSelection`'s declared order. `src/write/mod.rs`: `WriteArea::OptionLists` (`option-lists`), `owned`/`declared` for `selections`, `run(…, only, …)`; an `--only` key that is no selection is refused. `src/main.rs`: `--only <key>[,<key>…]`, only with `--write`.
- Fixture: three selections and five blueprints in `blueprints.json`, their data under `tests/fixture/data/` (`selections/`, `feats/`, `mythic/abilities/`). `planner.ron` is unchanged: the check fixture's counts stay.

**Decisions** (the slice's, none new except these two, the builder's): an empty game list is left unwritten; an unknown `--only` key is an error.

**Tests** (`tests/fixture/write.rs`): `writing_a_feat_list_brings_it_to_the_game`, `ignores_prerequisites_is_written_where_it_differs`, `a_member_with_no_feat_entry_leaves_the_selection_unwritten`, `only_writes_only_the_selections_named`, `a_second_option_lists_run_writes_nothing`. Expected values come from the fixture's blueprints (`TestBonusFeatSelection` ...b31, `TestShortFeatSelection` ...b32, `TestPathAbilitySelection` ...b33); the fixture is invented, so there is no game citation: no game fact changed.

**Dry run** (`game-check --write option-lists --dry-run` on main's data at 796aa5c3): 31 fields in 30 files would be written (the feat lists of the 267/268/275/279/285 families, `magic-feats` and `teamwork-feats`, and the `ignores_prerequisites` of `cleric/demonbane-priest/bonus-feats` and `rogue/master-of-all/skill-focus`); 19 selections are left unwritten for a feat or mythic ability with no entry (`feats`, `combat-feats`, `mounted-combat-feats`, the sorcerer bloodlines' Skill Focus members, `mythic/extra-mythic-ability`, ...), as the slice expected.

**Not done:** the test-first failing run was not recorded (the tests were written with the code); no data written.
