# Importer S4, 308a: the writer's prerequisites target

Type: build (the importer, stage S4)
Status: resolved
Blocked by: —
Model: Opus to build: the writer and the check's matching, read by every data slice after it.
Map: [WotR Build Planner](../map.md)

## Problem and change

A slice of [308](308-importer-prerequisites.md), the importer's stage S4, scoped to ranks 1 and 7 of [the queue's scope](../research/queue-scope-2026-11-17.md) (user, 2026-11-17). The goal, the edit list, the tests by name, the proof and the expected counts are under its heading in [research/308/slices.md](../research/308/slices.md); that file's §0 and [research/308/decisions.md](../research/308/decisions.md) hold the decisions. A data slice also follows [research/306/decisions.md](../research/306/decisions.md): the generator follows the game with the Build count held, refusals are counted in link lines, a moved option keeps its gate.

## Done when

Its section in `slices.md` is built and proved as written there, as amended by `decisions.md`, and the scoped checks pass.

## Answer

**Built** (base `c4eea6bf`; no data written):

- **`tools/game-check/src/write/prerequisites.rs`** (new): `KIND = "requires"`; `changes(ctx, only)` over feats, options, mythic abilities and feats, `all_classes()` and their archetypes. It skips an entry with no blueprint or one missing from the dump; lists a hand `prerequisite` ("a hand prerequisite") and leaves it alone; and builds one `Requires` per game component through `condition`, following §1.1's table. Any `Gap` leaves the entry whole, with one `Unexpressed` line per gap, `<what> -> <ticket>` (all or nothing, decision 0.1). Equality is checked on `compile_requires` + `canonicalize` of the current and the target components (0.2). The target is the `Requires` in game order, then the entry's other components. A shared GUID becomes `Any` of each entry's `Has`, or `Not(Any…)` for a `HasNot` (0.4). `--only` takes report names, and a trailing `*` matches a prefix (`unmatched` → `WriteError::Usage`). An unmapped blueprint's owner comes from the asset name (my inference, §1.2): 267, 294, 316, 306x/y, 306v, 306t, 306w, 306p, 306o, 315, 174 or 266, and 307 otherwise. No `carriers-left.txt` exists yet.
- **`carriers.rs`**: `fact_of`'s match is now `fact_at`, and `facts_of(ctx, guid)` (every entry's fact) is new. Both are `pub(super)`, and so is `words`.
- **`write/mod.rs`**: `WriteArea::Prerequisites` added. `owned`/`declared` read `mythic/abilities` and `mythic/feats` (`subfolder`). `--only` with prerequisites and another area → `Usage`. **`span.rs`**: `MYTHIC_ABILITY_FIELDS` and `MYTHIC_FEAT_FIELDS`. **`log.rs`**: the field column reads `prerequisite` for `KIND`. **`main.rs`**: the usage text, the module doc and `--only`'s guard.
- **The check** (`areas/prerequisites.rs`): `PrerequisitePlayerHasFeature` → `has(m_Feature)`. An armour-only `Proficiency`/`NotProficient` → `Any` of `Has`, or `Any` of `Not(Has)`, of the proficiency feats (`armour_groups`, shared with the writer). The own NoFeature is skipped only when `entries_of(own).len() <= 1` (`is_own_no_feature`, shared). `FullAbilityScore` → `Ability`, and `CasterType` → spells of level 1.

**Decisions:**
- **The Extra feats' own cap** (builder's): §1.1 says "omitted when ranks is 3". All 10 Extra feats in the data hold `Not(FactRank(self, 3))`, and `Repeat::derive` reads it for a selection (303), so omitting it would turn their repeat into `WhileOptionLeft`. The writer therefore writes `Not(FactRank(self, ranks))` when the cap equals the header's `ranks`, and otherwise reports a gap (308). This is why the Extra feats compare equal.
- **Impossible Domain** (user, "as the game"): no `held` line.
- **Uncanny Dodge** (user, 276): two `applied 276` lines in `user-facts.txt`.

**Dry run** (`--write prerequisites --dry-run`; the CLI refuses `--log` with `--dry-run`, so the output went to `<scratchpad>/308a/dry.txt`):
- **Changes:** 271 entries (72 feats, 172 options, 12 mythic abilities, 1 mythic feat, 9 classes, 5 archetypes).
- **Left whole:** 351 entries (50 feats, 258 options, 9 MA, 14 MF, 10 classes, 10 archetypes). Per ticket, an entry counted under each of its tickets: 307 68, hand 62, 306v 60, 267 57, 294 33, 308 27 (26 condition trees, 1 SelectionPossible), 315 13, 313 12, 306x/y 12, 306t 11, 306w 9, 306p 9, 174 6, 316 5, 306o 5, 266 3, 314 2, 317 1.
- **Against §1:** 350 blocked there, 351 here. The extra one is `extra-channel-shaman`: `ShamanLifeSpiritChannelEnergyFeature` b8ec9dccc0e7ef74fb4072b0679c2aec has no entry now. My inference is that a slice after the research's base unmapped it. The writable feats with findings were 64 there; here 72 feats change, because 8 are form-only rewrites the check already agreed with (arcane-strike, dazzling-display, extra-arcane-pool, extra-mercy, natural-charmer, penetrating-strike, perfect-strike, shatter-defenses: `FullAbilityScore`, `CasterType`, shared Weapon Focus as `Any`, `AtLeast`). There are 19 such rewrites in all.
- **A real run on a copy** (`--data <copy> --log run.tsv`) passed `check_game_data_files`. After it, the check reports **no prerequisite finding on any of the 271 written entries**.

**306aa's feats and Evasion:**
- Extra Rogue Talent and Extra Discovery: equal, not rewritten.
- Extra Hex: blocked by `WinterWitchWitchHexSelection` b921af3627142bd4d9cf3aefb5e2610a (-> 306o), so it is not compared.
- `rogue/rogue-talents/evasion`: equal.

**`known.txt`:**
- **Stale lines removed:** 263's `metamagic-completely-normal-spell`, `shield-bash`, `shield-focus`, `shield-wall` and `stat-focus`; the three `not in game` improved-critical lines; and 306aa's `selection option rogue/rogue-talents/evasion -> 308`.
- **New "308" section at the end:** `missing in planner` lines for armor-focus (-> 315), the five proficiency feats and shield-master (-> 308).

**Tests** (`tests/fixture/prerequisites_write.rs`; 11 failed against a stub that planned nothing, and the equality guard passed):
- `the_prerequisites_target_writes_each_component_as_a_requires`
- `an_entry_naming_an_unmapped_blueprint_is_left_whole_and_listed`
- `a_shared_blueprint_becomes_an_any_of_its_entries`
- `an_own_no_feature_is_omitted_unless_the_blueprint_is_shared`
- `an_extra_feats_own_cap_is_the_headers_ranks`
- `a_hand_prerequisite_is_never_written`
- `an_equal_compiled_tree_is_not_rewritten`
- `a_second_prerequisites_run_writes_nothing`
- `the_check_reports_no_prerequisite_finding_for_a_written_entry`
- `only_names_entries_and_a_prefix`
- `a_held_prerequisite_fact_goes_to_the_user`
- `a_mythic_abilitys_components_are_owned`

The fixture's mythic ability and options are written by the test's world, not added to `tests/fixture/data/`.

**Game citations:**
- `ExtraWitchHexSelection` b6054088b4ab4be286724127cbf48b35: the cap's shape (`ContextConditionBuffRank`, `Not`, `UseFactInsteadBuff`, Value 3).
- `PrerequisiteNotProficient.cs` `CheckInternal` and `ArmorProficiencyGroupFlag.cs` (WrathDecompiled 6ce4a6b): the armour groups.
- Deity blueprints carry `ForbidSpellbookOnAlignmentDeviation` (`KabririFeature`, `PuluraFeature`, `BaphometFeature`).

**Checks:**
- `cargo test -p game-check`: 146 passed.
- `cargo fmt --all --check`: exit 0.
- `scripts/test-modules.sh`: OK.
- The game check: `Differences: 3832 in 197 groups; 0 not known`, exit 0.
