# Importer S3, 307b: the carriers target

Type: build (the importer, stage S3)
Status: resolved
Blocked by: 307a
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [307](307-importer-carriers.md), the importer's stage S3 (the carrier entries), cut to about 15–20 minutes of building (user, 2026-11-16). The goal, the edit list, the tests by name, the proof and the files it shares are under its heading in [research/307/slices.md](../research/307/slices.md). The decisions are in [research/307/decisions.md](../research/307/decisions.md).

## Done when

Its section in `slices.md` is built and proved as written there. S3 is a form change, so its census must be empty. The scoped checks pass.

## Answer

Built as the section of [slices.md](../research/307/slices.md) says, with the differences named below. No data changed (the target is not run on the real data), so no note file and no form proof.

**What was built** (`tools/game-check/`)
- `src/dump.rs`: `Dump::iter`.
- `src/write/carriers.rs` (new): `changes(ctx, only) -> (Vec<NewFile>, Vec<Unexpressed>)`. Candidates (a): every `!bp_` reference, at any depth, inside a `Prerequisite*` component of an entry the prerequisites area compares (feats, options, mythic abilities and feats, classes, archetypes), unmapped and of a feature type. (c): the reverse of the vocabulary's grants (`AddFacts`, `AddFeatureOnClassLevel`, `AddFeatureIfHasFact`, `AddFeatureOnApply`, `LevelEntries`, an archetype's `AddFeatures`) walked up from each (a) until a mapped blueprint, an archetype or a class's `m_Progression`. Keys (`short_key`, `words`): the asset name less one trailing `Feature`/`Selection`/`Progression`, kebab-case with digits as words; a key two candidates or an existing feature would share keeps its suffix; computed over the whole set before `--only`; a key still taken is listed in `unexpressed`. The file is `(key, blueprint, name)` through `span::ron_text`, the name the key's words capitalised.
- `src/write/mod.rs`: `WriteArea::Carriers` (`"carriers"`, area prerequisites, kind `carrier`); `run` routes it; `--only` for carriers takes 32 lowercase hex GUIDs, never together with option-lists (`WriteError::Usage`).
- `src/main.rs`: `--only` allowed with carriers; USAGE and the doc header.

**Decisions** (builder, where the section is silent, each an inference): a *pick* is a member of the list of a selection that has an entry or that a progression's or archetype's rows grant, and a selection those rows grant; a pick is never an (a) seed and nothing is walked up from it, but a chain can still reach one (the kineticist's `AirBlastProgression`, `ElectricBlastProgression`, `ColdBlastProgression`, `WaterBlastProgression` are listed by selections nothing models; ticket 307's research class `final-cited.tsv` counts them). A class's own progression and an archetype end a walk and are not candidates. The section's "any `m_AllFeatures`" would have dropped those four. A GUID in `--only` that is no candidate is ignored (a second run stays idempotent).

**Tests** (`tests/fixture/write.rs`, on a world built in the test): `a_prerequisite_naming_an_unmapped_feature_makes_a_carrier`, `a_chain_up_to_an_entry_makes_carriers`, `a_selection_member_is_never_a_carrier`, `a_member_of_a_list_no_entry_stands_for_is_still_a_carrier`, `keys_drop_the_suffix_unless_two_would_share_one`, `a_second_carriers_run_writes_nothing`, `two_carriers_runs_on_two_copies_make_the_same_files`, `only_takes_guids_and_a_dry_run_writes_none`. No game fact: the fixture's blueprints are invented; the real data's check is the dry run.

**Dry run** (`game-check --write carriers --dry-run` on main's data): 301 files. Against `final-cited.tsv`'s 70 prerequisite and 10 chain carriers, 77 are listed and the 3 missing are chain carriers above row seeds (`MentalOverflowProgression`, `AeonFifthLevelImmunities`, `AeonSeventhLevelImmunities`: 307c adds the row seeds). The other 224 are carriers the section's later pieces leave out or that other tickets own: 37 deity features (267) and 2 domain ones (294) the prerequisites name, the chains above them, the S2-pending members of selections that have no entry yet (bloodrager second bloodlines, animal-companion progressions: until 306 merges), and the 12 the registry of 307c leaves out. The research told these apart by the ticket of the finding that named them, which the writer cannot read: 307c's `carriers-left.txt` has to carry the 267/294 ones too (it is a count, not a defect of this slice).

**Checks:** the check's report is byte-identical to the base's (no `libs/` change; only the timing line on stderr differs).
