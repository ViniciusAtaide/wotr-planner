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

### Fix round: the narrowing guard (review's blocking finding)

The writer turned each game GUID into whichever entry maps it, so where the planner grants the game's blueprint under another entry, or a grantor's rows are still the planner's, a rewrite refused characters the game allows. The check compares GUIDs and can't see it.

**Built:**
- **The guard** (`write/prerequisites.rs`, `narrowings_of`, `Named`, `leaf_at`): for each entry it would rewrite, the writer walks the old and the new compiled tree and names each fact leaf by its entry (`feat power-attack`, `class cleric` for a class level, the feat of a `FeatFor`, the fact of a `Rank`), apart from counts, ranks and values.
  - **A dropped fact:** one the old tree accepts (under an even number of `Not`s, in an `All`, `Any` or `AtLeast`) that the new tree doesn't name.
  - **An added negation:** a fact the new tree refuses (an odd number of `Not`s) and the old one didn't. A refused leaf that is no fact is named by its `Debug`.
  - An entry with either is left whole. Each unallowed narrowing is one `Unexpressed` line, `narrows: drops <fact> (<its blueprint> <GUID>) -> 308` or `narrows: adds Not <fact> (…) -> 308`, all or nothing per entry. The blueprint is the fact's own, or "no blueprint, the planner's own entry".
- **The allowlist** (`tools/game-check/prerequisite-narrowing.txt`, `Narrowings::embedded`/`parse`): a narrowing passes only when the file names its entry and narrowing. Its header says the reason must cite the game blueprint and GUID and why no planner holder of the dropped fact gets it. Each narrowing has a line of its own. Its reason is its own (` -> <reason>`) or its entry's (`<entry> -> <reason>`, once above them), and one with neither is refused. `write::run_with` takes the list. `write::run` (the fixture tests) allows none, and `main.rs` passes the embedded list.

**Decisions:**
- **A negation of the entry's own blueprint is not a narrowing** (builder's): refusing itself, or another entry mapping its own GUID (Evasion, the twin fixture), is the game's NoFeature exactly. So the Extra caps, the selections' own NoFeature and the shared own NoFeature pass unlisted.
- **A dropped `All` requirement counts as a drop** (builder's): the brief's "a fact the old tree accepts in an `Any` or `Has` position" is read as any position outside a `Not`. Some flagged drops only widen (Manyshot's Point-Blank Shot); they're allowlisted as such rather than exempted.
- **The allowlist's reason may be the entry's** (builder's): 163 lines with the reason repeated came to 55 KB. Each line is still one entry and one fact.

**Triage** (dry run before the allowlist: 78 entries, 213 narrowings, against the research's "about 50"). The evidence: `game-check show` of each entry's blueprint for what the game names; `refs`/`granted-by` for who grants it; and the check's `progression rows / … game only` findings (`game: <Blueprint> at level N | planner: -`) for a grantor whose planner rows lack it.
- **Allowlisted: 59 entries, 163 narrowings,** each with its reason in the file:
  - a requirement the game doesn't state, dropped, which only widens (16: Manyshot, Rapid Shot, four kineticist options' class level, four wild talents' element, Sudden Speed, three mercies, the slayer's first style talent, Hag of Gyronna's deity);
  - a class or an archetype the game names or refuses, which maps one entry exactly (20: Witch Wandering Hex, Kinetic Overcharge, Mummification, Improved Uncanny Dodge (the user's 276), and the NoArchetype/NoClassLevel of the discoveries, Potent Magic, Increased DR, the three elements, the four arcana, Mystery Seeker, Survivalist, Magic Deceiver and Mystic Theurge);
  - a NoFeature of a blueprint one entry alone maps (5: Blinding Strike, the two Iceplants, Intimidating Display, the oracle's Apsu);
  - a member the game's list or feature doesn't name (5: Advanced Weapon Training, Critical Mastery, Pummeling Style (the archetypes' `RemoveFeatures` of `MonkFlurryOfBlowstUnlock`), Defensive Study, Winter Witch);
  - a selection the game names in place of its options or a class level, which no `game only` row lacks (13: Extra Arcanist Exploit, Extra Discovery (Vivisectionist), Extra Rage Power, Extra Wild Talent, Extended Range, the five ranger style-10 options, Beneficial Curse, Second Bloodrager Bloodline, Second Mystery).
- **Blocked: 19 entries, 50 narrowings,** each with its owner (`known.txt`'s line retargeted in place, and a note in its 308 section):
  - `feat extra-channel` -> **267**: drops `cleric/channel-energy` (`ChannelEnergySelection` d332c1748445e8f4f9e92763123e31bd), while `ClericProgression` grants the named `ChannelEnergyFeature` a79013ff4bcd4864cb669622a29ddafb. A plain cleric would be refused.
  - `feat favored-enemy-spellcasting` -> **275**: drops `ranger/favored-enemy`; the check's `class ranger | game: FavoriteEnemySelection at level 1…20 | planner: -`.
  - 13 `witch/hex/*` major and grand hexes -> **283**: drop the Hexcrafter magus 12/18 branch; the check's `archetype magus/hexcrafter | game: WitchMajorHex at level 12` / `WitchGrandHex at level 18 | planner: -`.
  - `archetype warpriest/mantis-zealot` -> **267** (its line was already 267): adds Not `slayer/deliverer/deity` for `DeitySelection` 59e7a76987fe3b547b9cce045f4db3e4.
  - `feat extra-mercy` (15 drops) -> **274**: `SelectionMercy` 02b187038a8dce545bb34bbfb346428d, which the check's `archetype paladin/divine-hunter-paladin` row grants in the game only. It has no `known.txt` line, since the check already agrees on its GUIDs, so the owner is recorded here and in the note.
  - `selection option shaman/hex/battle-master` -> **278** (unchanged): drops the Wildland Shaman; `ShamanBattleSpiritProgression` 26e81d83d13dbf6448b10019c802612d is `game only` on its rows.
  - **Open question,** `mythic ability impossible-domain` -> **294**: it drops the cleric, inquisitor and Divine Hunter levels, Living Deity and Pentamic Faith for the domain selections. Decision Q1 (the user: "as the game") covers Pentamic Faith. The question: do the cleric archetypes' planner rows keep `cleric/domain` where the game's keep `DomainsSelection` 48525e5da45c9c243a343fc6545dbdb9? The cleric's rows are open under 267/294. **308d can't write Impossible Domain** until that's answered and the five lines are added here.
- **The review's four cases** all end blocked: extra-channel (267), favored-enemy-spellcasting (275), the 13 hexes (283), Mantis Zealot (267).

**Answer correction (the form-only rewrites):** line 34's "form-only rewrites the check already agreed with" is wrong for 7 of them, which widen who qualifies, all toward the game. 308c should expect these offers to change:
- dazzling-display, penetrating-strike, shatter-defenses and weapon-focus-mythic accept the rogue-talent Weapon Focus (the shared GUID 1e1f627d26ad36f43bbd26cc2bf8ac7e);
- arcane-strike becomes `CasterType`, so a bloodrager at levels 1–3 qualifies;
- natural-charmer and perfect-strike read the full ability score.

The five ranger `combat-style-feats-6` options widen too. Extra Mercy, listed there, is now blocked (above).

**Dry run** (`--write prerequisites --dry-run`, `<scratchpad>/308a-fix/dry2.txt`), against the previous run's 271 changes and 351 left whole:
- **Changes: 252** (69 feats, 158 options, 11 mythic abilities, 1 mythic feat, 9 classes, 4 archetypes), so 19 fewer: 3 feats, 14 options, 1 mythic ability, 1 archetype.
- **Left whole: 370** (53 feats, 272 options, 10 MA, 14 MF, 10 classes, 11 archetypes). Per ticket: 307 68, hand 62, 306v 60, 267 57, **308 46** (27 before, plus the 19 narrowed), 294 33, 315 13, 313 12, 306x/y 12, 306t 11, 306w 9, 306p 9, 174 6, 316 5, 306o 5, 266 3, 314 2, 317 1.
- **Narrowings:** 213 found; 163 allowlisted, 50 blocked (on 19 entries).

**Tests** (`tests/fixture/prerequisites_write.rs`; the world gains `wide-feat`, `deny-feat`, `both-feat`, `shun-twin` and `kept-feat`):
- `a_rewrite_that_drops_an_old_alternative_is_left_whole_and_listed`, `a_rewrite_that_adds_a_negation_is_left_whole_and_listed` and `an_allowlisted_narrowing_is_written` (an entry with one of two narrowings allowed stays whole, listing the other; the entry-reason form too): these 3 failed before the guard (the rewrite was written).
- `an_entrys_non_requires_components_are_kept`: passed before the guard. It pins the `WeaponFocus` kept after the `Requires`.
- `a_no_feature_on_a_shared_guid_becomes_a_not_of_each_entry`: passed before the guard. It pins `Not(Any([Has(twin-feat), Has(twin-option)]))`, the game's `PrerequisiteNoFeature.CheckInternal` (`!unit.HasFact(Feature)`, `Blueprints/Classes/Prerequisites/PrerequisiteNoFeature.cs`, WrathDecompiled 6ce4a6b) with the blueprint held through either entry.

**Checks (fix round):**
- `cargo test -p game-check`: 151 passed.
- `cargo fmt --all --check`: exit 0.
- `scripts/test-modules.sh`: OK.
- The game check: `Differences: 3832 in 201 groups; 0 not known in 0 groups, 3832 known in 201 groups`, exit 0 (4 groups more from the retargeted lines).
