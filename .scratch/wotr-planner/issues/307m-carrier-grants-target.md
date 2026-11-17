# Importer S3, 307m: the carrier-grants writer target

Type: build (the importer, stage S3)
Status: resolved
Blocked by: 307b
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [307](307-importer-carriers.md), the importer's stage S3 (the carrier entries), cut to about 15–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/307/slices.md](../research/307/slices.md). The decisions are in [research/307/decisions.md](../research/307/decisions.md).

## Done when

Its section in `slices.md` is built and proved as written there. S3 is a form change, so its census must be empty. The scoped checks pass.

## Answer

Built as the section of [slices.md](../research/307/slices.md) says, with the differences named below. No data changed (the target is not run on the real data: 307n does), so no note file and no form proof; the census is untouched because the writer's output is not applied.

**What was built** (`tools/game-check/`)
- `src/write/carriers.rs`: `attachments(ctx, made, prospective, only) -> (Vec<Change>, Vec<Unexpressed>)`, kind `carrier grant`. For each existing feature, feat or option whose blueprint grants a carrier it adds, as `components`: `AddFacts` -> one `Grant` of the carriers among the facts; `AddFeatureOnApply` -> `GrantOnApply`; `AddFeatureIfHasFact` -> `GrantIf { has, not: Not, fact }`; `AddFeatureOnClassLevel` -> `GrantAtClassLevel { classes: m_Class + m_AdditionalClasses, archetypes, level: Level, before: BeforeThisLevel }`. For an option, a `LevelEntries` row naming a carrier at level L adds the carrier's key to that level's row in `levels` (a new row when none). It only adds: a carrier an entry's grant components already name, or its row at that level already holds, is not written again; the whole field is re-rendered with the entry's existing components or rows first.
- A grant whose checked fact, class or archetype has no single entry is left out and listed (`a grant of <name> if it has <name>: no single entry for the checked fact`).
- `src/write/mod.rs`: `WriteArea::CarrierGrants` (`"carrier-grants"`, area prerequisites); `owned()` gains `components` for `features`/`feats`/`options` and `levels` for `options`; `declared()` gains the three folders' field orders (`span.rs`: `FEATURE_FIELDS`, `FEAT_FIELDS`, `OPTION_FIELDS`) so a missing field inserts in place; `run` plans the grants after the carriers whatever the order of the areas, and `--only` takes GUIDs for it too (the grantor's blueprint).
- `src/main.rs`: `--only` allowed with `carrier-grants`; USAGE and the doc header.

**Decisions** (builder, each an inference where the section is silent)
- *What is a carrier.* The section says a grant is added "where its target is a carrier", and the carriers area leaves no mark in the file. A carrier is a `features/` entry with a blueprint, the template's shape (name = the key's words capitalised, no `summary`, no `guide`, no other field, only grant components), or a file made in the same run. A carrier is never a source here: its own grants are 307c's.
- *Dry run only:* with `--dry-run` the carriers the carriers area would make also count, so the plan shows what a run after it writes; a real run counts only existing carriers and this run's, so no change names a key no file holds.
- *Races are not written:* `owned()` has no `races` folder and the `Race` form is not extended here (slice: "if the `Race` form takes it"); the 3 race grantors stay for 307n to decide.
- *Row order:* a carrier is appended to a row; the loader sorts a row's keys, so the engine reads them sorted.
- *Existing components are re-rendered* (`ron_text` of each) when a grantor gains one; a file whose field holds a `//` comment is refused by the span writer, as for any field.

**Tests** (`tests/fixture/write.rs`, on a world built in the test): `an_option_s_row_names_the_carrier_its_progression_grants`, `a_feature_grants_the_carrier_its_blueprint_adds` (also the `GrantIf` with its `Not`, and the unmapped checked fact listed), `a_grant_already_named_is_not_written_again`, `a_dry_run_of_the_grants_counts_the_carriers_it_would_follow`. No game fact: the fixture's blueprints are invented.

**Dry run** (`game-check --write carrier-grants --dry-run` on main's data at `a5f20f39`): 76 fields in 75 entries (26 features and feats, 49 options), 103 grants listed unwritten (21 of them `SecondBloodlineEmptyFeature` if the option has the `SecondBloodline` selection, 5+ the wyrmshifter's three dragon-form level-ups per entry: the checked facts are unmapped or several). The section's 66 includes 3 races (not written here), so 63 compare with 75: the 12 more are grantors of carriers other tickets own (the animal-companion progressions and the like, which the carriers registry of 307c will leave out) because the dry run counts every carrier the carriers area would make. Not reconciled entry by entry: 307n's real run, after the carriers exist, is the count that matters, and its tests `an_abyssal_sorcerer_holds_its_bloodline_requisite_carrier` and `a_wyrmshifter_holds_its_dragon_form_level_up_carrier` need the checked-fact gap above settled first (a checked fact no entry maps, as the wyrmshifter's aspect features).

**Checks:** `cargo test -p game-check` 133 passed; `cargo clippy -p game-check --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean; `scripts/test-modules.sh` OK. The check's report is not touched (no `areas/`, `report.rs` or `libs/` change; the plain `game-check` run exits 0).
