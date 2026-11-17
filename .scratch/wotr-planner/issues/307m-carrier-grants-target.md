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

**Tests** (`tests/fixture/write.rs`, on a world built in the test): `an_option_s_row_names_the_carrier_its_progression_grants`, `a_feature_grants_the_carrier_its_blueprint_adds` (also the `GrantIf` with its `Not`, and the unmapped checked fact listed), `a_grant_already_named_is_not_written_again`, `a_dry_run_of_the_grants_counts_only_the_carriers_only_names`, `a_real_run_never_writes_a_grant_toward_a_carrier_no_file_holds`. No game fact: the fixture's blueprints are invented.

**The guard (review).** `--only` of `carrier-grants` takes the GUIDs of the carriers, as `carriers` does. A grant is planned only toward a carrier that exists as an entry (the template's shape), was made in the same run, or, in a dry run, is one `--only` names and the carriers area would make. With no `--only` a dry run counts none, and a real run writes only toward carriers a file holds: no run writes a grant toward a carrier outside S3's researched set (the other tickets' carriers are never made without their GUID in `--only`, 307b's guard). Tests: `a_dry_run_of_the_grants_counts_only_the_carriers_only_names`, `a_real_run_never_writes_a_grant_toward_a_carrier_no_file_holds`.

**Dry run, reconciled** (`game-check --write carrier-grants --dry-run` on main's data at `a5f20f39`, scratch tables in the lane's scratchpad). Before the guard it planned 75 entries toward every carrier the carriers area could make (301). With `--only` the 207 GUIDs of `final-cited.tsv` it plans **52 entries**, none toward a carrier outside the set: 21 sorcerer and 21 bloodrager bloodline rows (their `...-bloodline-requisite`; the 10 bloodrager and 10 sorcerer draconic bloodlines also named a base feature the set does not hold, now left out), kineticist fire and earth (`fire-blast-progression`, `earth-blast-progression`), the arcanist's Ways of the Deceiver, the azata's Life Bonding Friendship and the oracle's Life Channel rows, and 5 entries by component (Spell Specialization, inquisitor Bane, lich Danse Macabre, monk Sensei Advice, shaman Life Channel). The section's 66 less its 3 races is 63; the 11 not planned:
- 3 races (`elven-immunities`, `half-orc-ferocity`, `hardy`: the `Race` form is not written here, decision above).
- 2 features, `shifter/dragonblood-shifter/wyrmshifter` and `improved-wyrmshifter`: the three dragon-form level-up grants are `GrantIf` on aspect features no entry maps, so they are listed (the gap for 307n's `a_wyrmshifter_holds_its_dragon_form_level_up_carrier`).
- 6 whose carrier the carriers area (307b) does not make yet, because it sits above a row seed, 307c's: `bard/tranquil-whisperer/tranquil-apotheosis` (`immunity-to-mind-affecting`), `gold-dragon/perfect-soul` (`immunity-to-energy-drain`), `druid/winter-child/flurry-form` (via `subtype-elemental` to `immunity-to-bleed`), the lich skeleton options' `bardic-performance-resource-fact` and `mythic-lich-level-3`, and `shifter/griffonheart-shifter/griffon-aspect`, a feature whose own `LevelEntries` grant `shifter-wild-shape-griffon-feature-level-up` (a feature has no rows: decision 2, 317/318). These six are read from the chains in `final-cited.tsv` and the carriers dry run (the last was not checked entry by entry).

The 23 entries planned before the guard that the set excludes (now never planned; none is written without their carrier's GUID in `--only`): 17 features and 1 option granting animal-companion, mount and companion progressions (`mad-dog`, cavalier mount, beast rider, ghost rider, the two animal domains, druid, drovier, primal druid, winter child's two, hunter, divine hound, urban hunter, sacred huntsmaster, nomad, sable marine, wildland shaman, sylvan sorcerer, the druid nature bond and the oracle's nature animal companion): S2's, members of selections whose entries 306 makes (`keys.tsv`, not `final-cited.tsv`); `subtype-elemental` (flurry form): above a row seed, 307c; the 2 lich skeleton options (`lich-skeletal-barbarian-tier-i`, `lich-skeletal-fighter-tier-i`): owned by the lich ticket 266 (`excluded.tsv`, `OUT:lich`); the shaman nature spirit (`shaman-animal-companion`, `animal-companion-rank`): the pets ticket (`OUT:pets`). None is a deity (267) or domain (294) carrier.

**Checks:** `cargo test -p game-check`, `cargo clippy -p game-check --all-targets -- -D warnings`, `cargo fmt --all --check` and `scripts/test-modules.sh` pass (last lines in the report). The check's report is byte-identical to the base's: `game-check --binz <dump> --data <lane's data>` on a temporary worktree of `a5f20f39` and on this tip, `cmp` finds no difference (676,408 bytes).
