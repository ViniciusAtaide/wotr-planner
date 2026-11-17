# Ticket 308, the importer's stage S4 (prerequisites from the game): the slices (research, 2026-11-17)

Read-only research at `main` 2e044a3c. Game: the pinned dump (236,660 blueprints) through `target/debug/game-check`
(main's build; its findings are byte-identical to lane-tests' build, whose `known.txt` is mid-edit by 306aa, so
main's binary gives the clean exit); code WrathDecompiled 6ce4a6b. No game text here. Blueprints are cited by asset
name and GUID, each resolved through the entry or grantor whose finding names it.

Scope (user, 2026-11-17, `research/queue-scope-2026-11-17.md`): rank 1 = the **feat** prerequisites; rank 7 = the
rest of S4 that needs no S3 carrier (mythic, class features, classes and archetypes). Anything naming a blueprint
with no entry stays unwritten and listed. S5, S3's data slices (307c–307o), 306t–z, 306o, 306p, 306ab are deferred.

Evidence beside this file (kept in `research/308/`: `prereq.tsv`, `comps2.tsv`, `entry-owner2.tsv`,
`prereq-status2.tsv`, `unm-owner.tsv`, `pin-feats.txt`, `ignored.txt`; the rest stayed in the research's scratchpad,
and a fresh `game-check` run gives `run-main.txt` again). The main session's answers are in
[decisions.md](decisions.md). `run-main.txt` (fresh check), `prereq.tsv` (the 1,585 prerequisite findings: section, kind, entry kind, key,
blueprint, game, planner), `prereqs.jsonl` (every compared blueprint's `Prerequisite*` components, from one
`game-check show` of 1,421 GUIDs), `comps2.tsv` (each component classified), `entry-owner2.tsv` (each of the 1,433
compared entries: WRITABLE or BLOCKED and why), `prereq-status2.tsv` (each finding with its entry's status),
`unm-owner.tsv` (each unmapped blueprint's owner), `pin-feats.txt` (`scripts/pinning-tests.sh` for the feat run),
`ignored.txt` (the ignored real-data tests of these areas). Tools: `jq` and `awk` only (no Python).

---

## 1. Today's count, and what the writer can express

**Fresh run** (`game-check --data <main>/libs/game-data`): 4,096 differences in 197 groups, all known, exit 0.
Prerequisites: 1,433 entries compared, 883 agree, 550 differ; **1,585 findings** (the design's 1,569 + S2's moves):

| Kind | Findings |
| --- | --- |
| missing in planner | 517 |
| missing in planner, a blueprint it doesn't map | 555 |
| not in game | 501 |
| value differs / grouping differs | 9 / 3 |

By section and family (W = on entries the writer can write whole; B = blocked; entries/findings):

| Section | Family | W | B |
| --- | --- | --- | --- |
| 263 | feats | 55 / 88 | 28 / 93 |
| 264 | Extra feats | 9 / 102 | 6 / 32 |
| 259, 265, 293 | mythic abilities and feats | 12 / 29 | 20 / 63 |
| 266 | mythic path options (aeon) | 9 / 9 | 7 / 24 |
| 267 | deities: classes, archetypes, options | 5 / 6 | 69 / 125 |
| 269 | barbarian rage powers | 24 / 158 | 4 / 28 |
| 270, 271 | kineticist | 16 / 19 | 34 / 179 |
| 272, 281, 294 | oracle, bloodlines, domains | 0 / 0 | 75 / 124 |
| 274, 275, 279 | paladin, ranger/hunter, monk | 6 / 6 | 0 / 0 |
| 276 | rogue and slayer talents | 30 / 60 | 1 / 1 |
| 278 | witch and shaman hexes | 36 / 49 | 17 / 37 |
| 280 | alchemist | 22 / 35 | 0 / 0 |
| 282, 283 | arcanist, wizard, magus | 13 / 18 | 12 / 79 |
| 287 | prestige classes and their options | 11 / 25 | 6 / 21 |
| 293 (options) | mystery/domain copies | 5 / 10 | 69 / 165 |
| **all** | | **614 findings** | **971 findings** |

Of all 1,433 compared entries, 1,083 are writable whole and 350 blocked (feats 214 / 49; options 558 / 258; mythic
abilities 54 / 9; mythic feats 42 / 14; classes and prestige 29 / 10; archetypes 186 / 10). A writable entry with no
finding is written only where the compiled tree differs (§1.3); most are untouched.

### 1.1 The game's components → `Requires` (`libs/rules/src/component.rs`, `Condition`)

One `Requires` per game component, its `group` from the component's `Group` (`All`; anything else `Any`, as
`BlueprintFeature.MeetsPrerequisites` counts it). GUID → fact through `Planner::entries_of`: one entry its fact;
**several entries** (90 shared blueprints, 305's note; 34 named by prerequisites: Weapon Focus feat and rogue-talent
option `1e1f627d26ad36f43bbd26cc2bf8ac7e`, Stunning Fist feat and monk feature `a29a582c3daa4c24bb0e991c596ccb28`,
Evasion option and feature `576933720c440aa4d8d42b0c54b77e80`, the per-archetype selection copies, 302's nested
selections) `Any` of `Has` each (`HasNot`: `Not(Any(…))`), the game's rule that a blueprint is held once.

| Game component | Writes |
| --- | --- |
| `PrerequisiteFeature` | `Has(fact)` (shared: `Any([Has…])`) |
| `PrerequisiteNoFeature` | `HasNot(fact)`; **own** NoFeature: omitted on a non-selection with `Ranks` ≤ 1 and one entry (as the check); on a selection, `HasNot(self)` (303); on a **shared** GUID, `HasNot` of the other entries (306f's Evasion gap: `options/rogue/rogue-talents/evasion` gets `HasNot(Feature("evasion"))`) |
| `PrerequisiteFeaturesFromList` | `AtLeast(max(Amount,1), facts)`; `m_RestrictIfNot` → `Not(AtLeast…)`; Amount ≥ 2 over a shared GUID → gap |
| `PrerequisiteStatValue` / `FullStatValue` | `Stat{stat, value, full}`: abilities (≤ 5 omitted, as the check), `BaseAttackBonus` → `BaseAttack`, `Skill*` → `Skill(game_skill)`; full of `SneakAttack` → gap **314** (3) |
| `PrerequisiteClassLevel` | `ClassLevel(key, n)`; `Not` → `Not(ClassLevel)`; `Not` of a pet class omitted; a pet class (`AnimalClass` / `AnimalCompanionClass`) → gap **174** (the barding feats) |
| `PrerequisiteNoClassLevel` | `NoClassLevel(key)`; of a pet/animal class omitted |
| `PrerequisiteArchetypeLevel` | `Level ≤ 1` → `Has(Archetype(key))` (the data's form; `ArchetypeLevel(_,1)` would compile to an extra `ClassLevel 1` the check reports), else `ArchetypeLevel(key, n)` |
| `PrerequisiteNoArchetype` | `NoArchetype(key)` |
| `PrerequisiteAlignment` | `Alignment(set)` from the mask (`alignment_mask`) |
| `PrerequisiteCasterTypeSpellLevel` / `CasterType` / `CharacterLevel` / `MythicLevel` | `CasterTypeSpellLevel{…}` / `CasterType` / `CharacterLevel` / `MythicLevel` |
| `PrerequisiteProficiency` / `NotProficient` | armour groups → `Proficient` / `NotProficient` (`LightShield`, `HeavyShield`, `Buckler` → `Shield`); a weapon proficiency → gap **313** (7: Simple, Martial, Exotic, Shield Bash, Swordlord, Dueling Mastery); barding → gap **174** |
| `PrerequisiteParametrizedFeature` | `ParametrizedFeature(feat, SpellSchool(..))` or `Weapon(kebab key)` if that weapon entry exists, else gap 313 |
| `PrerequisitePlayerHasFeature` | `Has(fact)`: the Build is the main character (`Blueprints/Classes/Prerequisites/PrerequisitePlayerHasFeature.cs` `CheckInternal`: `GameHelper.GetPlayerCharacter().Progression.Features.HasFact`) |
| `PrerequisiteIsPet` (`Not`) / creature-only `NoFeature` (`SubtypeElemental`, `BlizzardServantLock`) | omitted (always hold) |
| `PrerequisiteCondition`, exactly `Not(ContextConditionBuffRank(self, 3))` | omitted when the entry's `ranks` is 3 (all 10 Extra caps are, 303's header), else gap |
| `PrerequisiteCondition`, other | gap **"condition tree"** (left under 308; the check doesn't read them: 29 of the 65 entries would need new `known.txt` lines; with S2's bloodlines) |
| `PrerequisiteParametrizedWeaponSubcategory`, `ConditionForWeaponCategory` | gap **313** |
| `PrerequisiteAnySpellsInSpellbook` | gap **317** |
| `PrerequisiteSelectionPossible` (Expanded Defense) | gap, listed (no vocabulary) |
| `PrerequisiteIsPet` (not `Not`), `PrerequisitePet`, `Etude`, `MainCharacter` | gap 174 / story |
| a class's `ForcedTrue` group | omitted (the check skips it) |

### 1.2 What blocks an entry (all or nothing per entry; decision 0.1)

Blocked entries by reason (findings on them; an entry may have several):

| Reason | → ticket | Findings |
| --- | --- | --- |
| a deity's fact (`…DomainAllowed`, deity features, Atheism) | 267 | 265 |
| a carrier S3 makes | **307** (307e kineticist 136, 307j druid/shifter/shaman 67, 307f bloodlines 57, 307l rest 44, 307g mythic 23, 307k monk/bard 16, 307i hunters 9) | 352 |
| a blueprint S3's set lacks (`MajorFormWererat/Werewolf`, `WarpriestChannelEnergyFeature`, `TricksterLoreReligionTier2Parametrized`) | **307** (new carriers) | 27 |
| oracle archetype mysteries | 306v | 135 |
| domains | 294 | 132 |
| a hand `prerequisite` field (62 entries: bloodlines' `NoOtherOptionOfGroup`, `FeatForSameValue`/`FeatIfValue`, `Worships`, `DeityOffers`, …) | 315, 267, 281/S2 | 103 |
| second elements, Kinetic Knight foci | 306t | 43 |
| crossblooded/Seeker/second bloodlines, Dragonheir | 306x/y | 33 |
| a mythic class level (`DemonMythicClass`, `LichMythicClass`, `GoldenDragonClass`, …), the Lich's upgrades | 265/266 | 41 |
| the excluded copies (Sage/Sylvan/Empyreal bloodline progressions) | 316 | 26 |
| `WinterWitch…HexSelection`, `EldritchMagusArcanaSelection`, `LoremasterTricksterCombatFeatSelection` | 306o/p, 287 | 24 |
| per-value features (`SkillFocus…`, `ArmorFocus…`, `ElementalFocus…`) | 315 | 23 |
| pets, barding, `AnimalCompanionRank` | 174 | 22 |
| weapon categories | 313 | 9 |
| a feature's rank as a stat (Accomplished Sneak Attacker, Arcane Trickster, Sneak Attacker mythic) | 314 | 5 |
| curses, opposition schools | 306w, 306p | 18 |
| a condition tree | 308 (later) | 3 |
| spellbook | 317 | 1 |

The 22 unholdable carriers of 307's decision 3 (13 kineticist blasts → 270, 9 wandering-spirit → 321) aren't
entries yet, so their prerequisites fall under "a carrier S3 makes" today and keep `-> 270` / `-> 321` lines.

**Blocked feats (34, 125 findings),** with their line's new ticket: barding ×3 and Boon Companion → 174;
Accomplished Sneak Attacker → 314; Fencing Grace, Slashing Grace, Point-Blank Master, Shield Bash → 313; the three
Trickster critical feats (hand trees) → 315; Spell Specialization → 307 and 317; Diehard, Improved Elven Immunities,
Steel Soul → 307 (307l, race features); Discordant Voice, Lingering Performance, Extra Ki, Extra Performance → 307
(307k); Natural Spell, Raking Claws, Shifter's Edge, Selective Channel → 307 (307j/l); Energized Wild Shape,
Frightful Shape, Shifter's Rush → 307 (new carriers); Wings → 307 (307f); Extra Slayer Talent → 307 (307i);
Disruptive → 306x; Extended Aspects → 308 (condition tree); Extra Arcana, Extra Hex, Extra Hex (Shaman) → 306o/p or Q2.

---

## 0. Decisions taken here (say if you disagree)

1. **All or nothing per entry.** An entry with any gap keeps its file and its findings. Dropping one leaf is unsafe
   both ways: an `All` leaf dropped offers what the game refuses; an `Any` member dropped refuses what the game
   allows, where the planner approximates the missing blueprint with another entry (Selective Channel's warpriest:
   the game's `WarpriestChannelEnergyFeature` bd588bc544d2f8547a02bb82ad9f466a has no entry; the planner's list
   names `WarpriestChannelEnergySelection`). Q2 is the one exception asked about.
2. **Equality is the compiled tree.** The writer compares `compile_requires(current)` and `compile_requires(target)`
   after `Prerequisite::canonicalize` (`component.rs`). Equal: the file is untouched, whatever its `Requires`
   shape. So most of the 883 agreeing entries stay byte-identical, and a second run writes nothing.
3. **The check is the oracle.** The writer reads the game components itself, since the check's `Node` form drops
   parameters, `full` and proficiency. After a run, every written entry has no prerequisite finding. That's asserted
   in the fixture and checked in every data run.
4. **Shared blueprints → `Any`.** 308's "a fact held once" is applied, not split. More permissive than one `Has`,
   never less, so it refuses nothing.
5. **The writer owns only `components`' `Requires`.** An entry with a hand `prerequisite` field (62) is never
   written. The check refuses an entry holding both, and `prerequisite` isn't owned. Those findings go to 315, 267
   or S2.

---

## 2. The slices

Fixed cost per data batch is about 15 minutes: `record` on the base with `FORM_PROOF_OFFERS=1` (the lane's warm
`target/`, about 1.5 min), `compare` with `FORM_PROOF_CENSUS`, the one full `seeds`/`corpus` run (`game_data`
about 3 min, `links` 45 s), the game check and check-data. So the batches are sized at about an hour. Lanes:
**W** = `tools/game-check/`; **C** = `tools/corpus/tests/form_proof.rs`; **D** = data, `known.txt`, seeds tests,
corpus generators. Two builders at most. Every **D** slice also edits `known.txt` and the seeds tests, as every 306
slice does, so D runs serial with 306s/q/r/n/m/aa. Each D brief carries 306's rules from `research/306/decisions.md`:
- the generator follows the game: the Build count holds, with before and after in the Answer;
- refusals are counted in link lines;
- a moved option keeps its gate.

### 308a. The writer's prerequisites target (Lane W, Opus) — about 90 min

**Goal:** `game-check --write prerequisites [--only <entry>…]` writes each writable entry's `Requires` from the
game. The check agrees with every form it writes.

**Edits:**
- **New `tools/game-check/src/write/prerequisites.rs`:**
  - `pub const KIND: &str = "requires"`.
  - `pub(crate) fn changes(ctx: &Ctx, only: Option<&Only>) -> (Vec<Change>, Vec<Unexpressed>)`. It runs over
    feats, options, mythic abilities, mythic feats, `all_classes()` (prestige too) and archetypes.
  - Skip: no blueprint, the blueprint missing from the dump, or `prerequisite.is_some()`. A hand tree is
    `Unexpressed` "a hand prerequisite".
  - `fn condition(ctx, own: &BlueprintGuid, json: &Value, ranks: u8, part: &Value) -> Result<Option<(Condition, Group)>, Gap>`
    holds §1.1's table. `Gap { what: String, ticket: &'static str }`. An unmapped GUID → ticket `"307"` (or the
    owner 267/294/316/174/315 by `carriers-left.txt` and §1.2), and the `what` names the blueprint and GUID.
  - `fn facts_of(ctx, guid) -> Vec<Fact>` reuses `carriers.rs`'s `fact_of` match. Make it `pub(super)` and generalise
    it to all entries.
  - Target: `Requires` in game component order, then the entry's non-`Requires` components in their order.
  - Change: `field: "components"`, `kind: KIND`. `old` and `new` are the compiled trees' `Debug`.
- **`write/mod.rs`:**
  - `WriteArea::Prerequisites` ("prerequisites"). Its `area()` is `Area::Prerequisites` and its `kinds()` is
    `[prerequisites::KIND]`. `run()`'s match gets an arm.
  - `--only` for this area takes the report's entry names (`feat extra-hex`, `selection option rogue/*`). A
    trailing `*` matches a prefix. One that matches nothing → `WriteError::Usage`. Refuse `--only` with any
    other area.
  - `owned()`/`declared()`: `mythic/abilities` and `mythic/feats` (read the second path component) own
    `components`. Today `folder()` returns `mythic` and owns nothing.
- **`write/span.rs`:** `MYTHIC_ABILITY_FIELDS` and `MYTHIC_FEAT_FIELDS`, in the order `rules::MythicAbility` and
  `rules::MythicFeat` declare them.
- **`write/log.rs` `tsv`:** the field column is `prerequisite` for a change whose `kind` is `prerequisites::KIND`.
  That's 308b's census row, kept apart from 307m's carrier-grant `components` rows.
- **`areas/prerequisites.rs`** (the check agrees with the new forms):
  - `game_component`: `PrerequisitePlayerHasFeature` → `has("m_Feature")`.
  - `PrerequisiteProficiency` and `NotProficient` with only armour groups (no barding, no weapons) → `Any` /
    `Not` of `Has(<guid of ArmourProficiency::feat()>)`, through `ctx.planner.guid(DataAt::Feat(..))`. The others
    stay "not compared".
  - `game_side`: skip the own `NoFeature` only when `ctx.planner.entries_of(own).len() <= 1`.
  - `planner_side`: arms for `Prerequisite::FullAbilityScore` (→ `Leaf::Ability`) and `Prerequisite::CasterType`
    (→ `SpellLevel{arcane, false, 1}`, the game side's reading). Today both fall to "no game form".
- **`main.rs`:** `USAGE` and the module doc name `prerequisites` and its `--only`.
- **`user-facts.txt`:** a `# 276 (the user, 2026-11-15) …` block with
  `prerequisites / * / selection option rogue/rogue-talents/uncanny-dodge -> applied 276` and the same for
  `improved-uncanny-dodge`. Q1's answer may add `… / mythic ability impossible-domain -> held`.
- **`known.txt`:** the check's changes move findings. Expected: the `not in game: TricksterPerceptionTier2Feature`
  lines go:
  - `prerequisites / not in game / feat improved-improved-critical`, `…-improved-improved-critical`,
    `…-critical-improved` (263);
  - `* / feat metamagic-completely-normal-spell`, `* / feat stat-focus`.

  Shield Focus, Shield Wall and Shield Bash lose `not in game: ShieldsProficiency`; drop their lines if empty. New
  findings (Shield Master, Bashing Finish's proficiency, the five proficiency feats' own `NotProficient`, Hellknight,
  Stalwart Defender, `armor-focus`, the Evasion option's `not Evasion`) get lines `-> 308` in a new "308" section,
  or the blocked entry's ticket (`armor-focus` → 315). Run the check and copy its unknown list. Don't guess.
- **Tests:** a new `tools/game-check/tests/fixture/prerequisites_write.rs`, with `mod prerequisites_write;` in
  `main.rs`. It plants records with `support::Record` the way `carriers_world` does:
  - `the_prerequisites_target_writes_each_component_as_a_requires`: feature, class level, archetype level 1 and
    3, stat, alignment and an `Any` group;
  - `an_entry_naming_an_unmapped_blueprint_is_left_whole_and_listed`;
  - `a_shared_blueprint_becomes_an_any_of_its_entries`;
  - `an_own_no_feature_is_omitted_unless_the_blueprint_is_shared`;
  - `an_extra_feats_own_cap_is_the_headers_ranks`;
  - `a_hand_prerequisite_is_never_written`;
  - `an_equal_compiled_tree_is_not_rewritten`;
  - `a_second_prerequisites_run_writes_nothing`;
  - `the_check_reports_no_prerequisite_finding_for_a_written_entry`;
  - `only_names_entries_and_a_prefix`;
  - `a_held_prerequisite_fact_goes_to_the_user`;
  - `a_mythic_abilitys_components_are_owned`.

  The fixture data gains one mythic ability and one option file.
- **Proof:** a dry run on real data (`--write prerequisites --dry-run`). Its counts go in the Answer: changes per
  kind and `Unexpressed` per ticket, against §1's W column. Check: `cargo test -p game-check`, fmt, clippy (hook).
- **Blockers:** none (307m merged). **Shares:** `known.txt` (a few lines, so it merges after any D slice in
  flight), `user-facts.txt`. **Group:** P1 with 308b.

### 308b. The census row `prerequisite` (Lane C, Sonnet) — about 30 min

**Goal:** `compare` attributes a prerequisite change's lines.

**Edits:** `tools/corpus/tests/form_proof.rs`:
- `FIELDS` gains a `prerequisite` row: kinds all nine `LineKind`s, `regenerates: true`. A refused or newly allowed
  pick can move every line of a Build that holds the entry.
- The census's held set becomes classes, archetypes and the pick keys of the base and the tip.
  - Add `fn pick_keys(picks: &str) -> BTreeSet<String>`, looping `pick_key`'s scan over the picks' `Debug` text.
  - In `compare`, `held = n.held ∪ pick_keys(&b.picks) ∪ pick_keys(&n.picks)`. Do the same for the reopened
    census.
  - `record_entry` keeps classes and archetypes.
- `read_change_log`: `rsplit_once(' ')`, so `selection option a/b` and `mythic ability x` read as their keys.
  Today `split_once` reads `option a/b`.

The header doc names the row.

**Tests:** unit tests in the file's `tests` module:
- `a_prerequisite_row_attributes_every_line_of_a_build_holding_the_entry_at_the_base`;
- `a_change_log_entry_with_a_two_word_kind_reads_as_its_key`.

**Checks:** `cargo test -p corpus --test form_proof` (the unit tests), fmt. **Blockers:** none.
**Group:** P1 with 308a.

### 308c. The feat run (Lane D, Opus) — about 75 min

**Blocked by:** 308a, 308b, 306aa (it edits the Extra feats' slots and `extra_picks.rs`). Run it before 306q/306s,
or have them run `--write prerequisites --only` on their new feats.

**Goal:** the 64 writable feats' prerequisites are the game's. That clears 190 findings in 263/264, less what 308a
already moved.

**Edits:**
- Record the base in the lane first.
- `game-check --write prerequisites --only 'feat *' --log <scratch>/308c.tsv`.
- `known.txt` sections 263 and 264: delete the written feats' lines. Re-home each blocked feat's line to its ticket
  from §1.2's list: `prerequisites / * / feat <k> -> 307`, 313, 314, 315, 174, 306x. A header comment notes "263
  and 264 superseded by 293; the lines left name their owner".
- The generators that the game now refuses, keeping the Build count:
  - `tools/corpus/src/builds/talents.rs` ~line 754, "bard 3 extra rogue talent". The game's
    `ExtraRogueTalentSelection` 14d9089df87a43b696fa9451ca2f0a12 needs the rogue's, the Dark Lurker's or Fey Tricks'
    selection, not the bard's.
  - `builds/feats.rs` ~line 53, "Underground Chemist 11 extra-discovery". `ExtraDiscoverySelection`
    537965879fc24ad3948aaffa7a1a3a66 needs `DiscoverySelection`.
  - `compare`'s "opened on the base, not on the tip" names any others.
- A data note: `libs/game-data/notes/308c-feat-prerequisites.md`.
- The stale comment in `options/rogue/rogue-talents/improved-uncanny-dodge.ron` waits for 308e.

**Expected refusals** (toward the game):
- Advanced Weapon Training for the Sohei and the Disciple of the Pike (`AdvancedWeaponTraining1`
  3aa4cbdd4af5ba54888b0dc7f07f80c4: fighter's `WeaponTrainingSelection` and fighter 5);
- Extra Rogue Talent for bards;
- Extra Discovery for the Underground Chemist;
- Extra Rage Power for a Battle Scion with no rage-power selection.

Removals that loosen: Manyshot's and Rapid Shot's Dexterity and Point-Blank Shot, Critical Mastery's two, the wild
shape alternatives.

**Tests:** un-ignore and run filtered first:
- `class_skills::advanced_weapon_training_needs_the_fighters_weapon_training`;
- `borrowed_selections::extra_rogue_talent_offers_the_rogue_talents_and_evasion`;
- `extra_picks::{the_extra_feats_that_give_a_pick_name_the_games_selections, the_extra_feats_the_game_refuses_to_an_underground_chemist_a_bard_a_skald_and_a_hexcrafter, a_hexcrafter_5_meets_extra_arcana}`.
  The Hexcrafter half waits on Q2. Re-point it to 306o/p if Q2 is B.
- `skald_rage_powers::extra_rage_power_is_closed_to_a_battle_scion_with_no_other_rage_power_selection`.

Pinning tests (`pin-feats.txt`, `scripts/pinning-tests.sh <64 keys> Requires`):
- `prerequisites_as_components::{a_blueprint_entry_holds_its_prerequisite_as_requires_components, crane_styles_groups_are_the_games, power_attacks_strength_13_is_a_requires_of_the_stat, picked_blueprint_entries, has_ranked_at_least}`;
- `parameterised_feats::{the_extra_talent_feats_stop_at_3_and_extra_rage_power_doesnt, the_extra_feats_repeat_up_to_their_blueprints_ranks, prerequisites_name_the_values_the_guide_names}`;
- `nested_selections::{each_extra_feat_grants_a_selection_with_its_blueprint, the_own_list_extra_feats_list_the_selection_they_stand_for, the_first_had_extra_feats_name_the_selections_they_reach}`;
- `extra_picks::*`, `skald_rage_powers::*`, `wild_talents::extra_wild_talent_needs_kineticist_level_2`;
- `bonus_feat_choices::the_bonus_feat_lists_are_the_guides`, `mythic::*extra_feat*`;
- `option_follow_ups::a_slayer_who_picked_archery_cant_take_two_weapon_fighting_through_a_combat_style_talent`;
- `gold::*`;
- rules: `engine/options.rs::fighter_bonus_slots_offer_only_combat_feats`, `removable::cleave_chain`, `combat_styles::*`.

Then the one full run. Each changed fact gets a real-data test citing its blueprint, one per pattern (263's
"a real-data test per pattern"):
- the repeated tier leaves (Strength 13 on Cleave's chain);
- the Combat Expertise or Kinetic Warrior grouping;
- the races;
- the BAB leaves;
- the Extra feats naming selections.

**Proof:**
- `compare` with `FORM_PROOF_CENSUS` gives 0 unattributed, with counts per file.
- `FORM_PROOF_OFFERS=1`: each changed option, explained in the Answer.
- Refusals are counted in link lines and listed by Build.
- Build count before and after.
- The game check exits 0 with 263/264's written lines gone.
- A second run writes 0.
- Run the ignored tests above with `-- --ignored`.

### 308d. Rank 7, batch 1: mythic, classes and archetypes (Lane D, Opus) — about 50 min

**Blocked by:** 308c. **Goal:** 12 mythic abilities (29 findings) and 14 classes and archetypes (24) written:
- the mythic abilities: `abundant-smite-smite-chaos`, `any-means-necessary`, `beneficial-curse`, `defensive-study`,
  `greater-enduring-spells`, `impossible-domain` (Q1), `kinetic-overcharge`, `limitless-rage`,
  `over-infused-blasts`, `second-bloodrager-bloodline`, `second-mystery`, `witch-wandering-hex`;
- the classes: assassin, duelist, eldritch-knight, hellknight, hellknight-signifer, mystic-theurge, oracle,
  stalwart-defender, winter-witch;
- the archetypes: magic-deceiver, dark-elementalist, rageshaper, mantis-zealot, hag-of-gyronna.

**Run:** `--only 'mythic ability *,mythic feat *,class *,archetype *'`.

**known.txt:**
- 259, 265, 266's mythic lines;
- 267's writable class and archetype lines;
- 282 and 287's class and archetype lines;
- the 308 lines 308a added for Hellknight and Stalwart Defender;
- re-home the blocked (mythic feats → 315/307g, mythical-beast → 174, second-bloodline → 306x).

**Coordinate:** ticket 208 (another session, staged: Greater Enduring Spells needs Enduring Spells) writes the same
file. 308d's run supersedes it, so land one and close the other.

**Tests:** un-ignore 259's two, and 265's `mythic::{all_skilled_…, extra_feat_offers_…, extra_mythic_ability_leaves_out_…}`,
`option_follow_ups::a_hellknight_3_with_pentamic_faiths_domain_cant_take_impossible_domain` (Q1),
`class_rules::domain_zealot_is_closed_to_a_druids_domain`, `option_families::domain_zealot_isnt_open_to_a_druids_domain`
(if blocked by 267, rename the reason), `pick_values::favorite_metamagic_offers_ten_metamagics_each_needing_its_feat`.
Pinning: `scripts/pinning-tests.sh` on the entry keys (`mythic.rs`, `class_rules.rs`, `option_follow_ups.rs`,
`gold.rs` expected). **Proof:** as 308c.

### 308e. Rank 7, batch 2: rogue, slayer, alchemist, magus, arcanist and the small families (Lane D, Opus) — about 70 min

**Blocked by:** 308d. **Goal:** about 78 options and about 120 findings:
- rogue 17/39, slayer 13/21 (276);
- alchemist 22/35 (280);
- magus 10/14 (283);
- arcanist 2/2 (282);
- ranger 5/10, hunter 2/2, paladin 3/3, monk 1/1 (274, 275, 279);
- stalwart-defender 3/9 (287).

**Run:** `--only 'selection option rogue/*,selection option slayer/*,…'`. The user's applied facts write Uncanny
Dodge (an `Any` of Eldritch Scoundrel rogue 4, bard, slayer, skald 4 and Feyform shifter 4: `UncannyDodgeTalent`
ca5274d057152fa45b7527cad0927840) and Improved Uncanny Dodge (class 12: `ImprovedUncannyDodgeTalent`
e821c61b2711cea4cb993725b910e7e8). The builder rewrites the stale comment in `improved-uncanny-dodge.ron`.

The combat-style talents become the game's:
- feat 2 is only "not holding `RangerStyleSelection2` c6d0da9124735a44f93ac31df803b9a9", over both entries;
- feats 6 and 10 need the previous style and ranger, slayer or Sanctified Slayer 6 or 10.

That clears 306g's lines under 276.

**known.txt:** 274, 275, 276, 279, 280, 282, 283 and 287's option lines; re-home the blocked.

**Gold:** the Enforcer (see §3) gains Uncanny Dodge at level 8. Pick it in `gold.rs`'s `enforcer()` and delete the
173 entry from `KNOWN_GAPS`, after the user sees it.

**Tests:**
- un-ignore 276's ten, the 308 one (`borrowed_selections::the_bard_slayer_and_feyform_shifter_are_offered_the_uncanny_dodge_talents`),
  280's two, 283's two, 282's two, 274's two, 275's two and 279's four, where their area is this batch's;
- pinning on the keys: `borrowed_selections.rs`, `combat_styles.rs`, `option_follow_ups.rs`, `repeating_talents.rs`,
  `gold.rs`.

**Proof:** as 308c.

### 308f. Rank 7, batch 3: barbarian, witch, shaman, kineticist, aeon and prestige options (Lane D, Opus) — about 70 min

**Blocked by:** 308e. **Goal:** about 85 options and about 235 findings:
- barbarian 24/158: the totems' 72 planner-only `HasNot`s go (the game has none), and the Primalist and other
  classes' level gates come in (269);
- witch 17/30, shaman 19/19 (278);
- kineticist 16/19 (270, 271);
- aeon 9/9 (266);
- 287's 3 options, and 293's 5 writable options.

**known.txt:** 266, 269, 270, 271, 278, 287 and 293's option lines. Re-home the blocked to 307, 270, 321, 306t or
306v. This is the last S4 slice of the queue. It writes a "308 left" summary into ticket 308: what's blocked, by
ticket, with counts.

**Tests:**
- un-ignore 269's four, 278's one, 270's five and 271's three where written;
- pinning: `skald_rage_powers.rs`, `instinctual_rage_powers.rs`, `infusions.rs`, `wild_talents.rs`, `follow_ups.rs`,
  `mythic.rs`.

**Proof:** as 308c, then the self-checks' long runs (main session, alone).

| Slice | Lane | Group | Estimate |
| --- | --- | --- | --- |
| 308a writer target + check alignment | W | P1 | 90 min |
| 308b census row | C | P1 | 30 min |
| 308c feats | D | after a, b, 306aa | 75 min |
| 308d mythic, classes, archetypes | D | after c | 50 min |
| 308e rogue/slayer/alchemist/magus/… | D | after d | 70 min |
| 308f barbarian/witch/shaman/kineticist/… | D | after e | 70 min |

D slices share `known.txt` with 306s/q/r/n/m. Interleave them; never run them side by side.

---

## 3. Gold builds (`libs/seeds/tests/game_data/gold.rs`)

Picks with prerequisite findings, and what writing them does:

- **Enforcer, level 8, Uncanny Dodge (308e): a gold change, goes to the user.** The `KNOWN_GAPS` entry (173,
  `WriteUpMistaken`) disagrees with the game, which offers the talent from slayer 4. The comment says the user
  approved the game's value on 2026-11-15, "the entry goes with the ticket that brings the talent to the slayer
  (276)". The Build picks it and the entry is deleted. Show it before the merge.
- **Enforcer MR 1, Defensive Study (308d), to verify:** the game's list drops the Sanctified, Swift and Tandem study
  features (`DefensiveStudy` ec041312142ba1f48a0044521f68fc0a). The Arcane Enforcer must hold the slayer's own
  Studied Target. If the gold test fails here, it goes to the user.
- **Enforcer, combat-style talents at 2, 6 and 10; Crippling Strike (rogue 1 at 16); Bashing Finish (17); Shield
  Focus and Shield Wall (308c/e): expected unchanged.**
  - The style chain holds.
  - Crippling Strike's `SneakAttack` and `AdvanceTalents` a33b99f95322d6741af83e9381b2391c come from
    `SlayerProgression` 1f35d11937a06624b8a60db68c7869a1 at 10.
  - Bashing Finish's BAB 11, Shield Bash, Two-Weapon Fighting and Shield Master are held by 17.
  - Proficiency reads the same feat.
- **Enforcer, Greater Enduring Spells (308d):** adds "needs Enduring Spells", which the Enforcer holds (MR 7).
  Unchanged.
- **Jack:** no writable pick changes. Spell Specialization (307l, 317), Second Bloodline (306x) and Mythical Beast
  (174, its existing known gap) stay blocked.

Known gaps stay as they are: 174 (Jack's companion) and 173 (deleted only by 308e). None is added.

---

## 4. Open questions

**Q1. Impossible Domain refuses a Pentamic Faith Hellknight (308d).** The game's `ExtraDomain`
213a8480d22206b45acbfa0619ca5aaf needs `DomainsSelection`, `DivineHunterDomainsSelection` or
`FakeDivineSparkSelection`. Pentamic Faith's own domain list (`HellknightDisciplinePentamicFaith`
b9750875e9d7454e85347d739a1bc894) is none of them. The planner's acceptance came from ticket 35 (guide-based,
"all recommended", user 2026-11-01). 242's test `a_hellknight_3_with_pentamic_faiths_domain_cant_take_impossible_domain`
already expects the game's rule.
- A (recommended): not a user fact. Write the game's (234).
- B: a `held` registry line, and the finding goes to the user.

**Q2. The three Extra feats blocked only by an S2 split.**
- Extra Hex: `WinterWitchWitchHexSelection` b921af3627142bd4d9cf3aefb5e2610a.
- Extra Hex (Shaman): `WinterWitchShamanHexSelection` 4e760c5034fafb2438993d8a192150b9.
- Extra Arcana: `EldritchMagusArcanaSelection` d4b54d9db4932454ab2899f931c2042c.

No Build can hold the missing member today. The Winter Witch's rows grant no hex selection, and the Eldritch Scion
holds `magus/magus-arcana`, which the game's list also names. So dropping it moves no Build away from the game, and
it fixes the Hexcrafter's Extra Hex (264, 306j's retro).
- A (recommended): 308c hand-writes the three entries' game `Requires`, less the member. The note cites the member.
  Its `known.txt` line narrows to that member, `-> 306o`.
- B: they stay blocked under 306o/p, with the Hexcrafter case.

**Q3. The condition trees.** `PrerequisiteCondition` is read by neither side today. 29 of the 65 entries holding it
have no `known.txt` line, mostly S2's bloodlines.
- A (recommended): defer it with S2's bloodline slices. Only Extended Aspects and Second Spirit wait.
- B: 308a reads it in the check and the writer (about 30 min more, plus about 29 new lines).
