# Ticket 307, the importer's stage S3 (the carrier entries): the slices (research, 2026-11-16)

Read-only research at `main` 56b09798 (S1 merged; S2 merged through 306e and 306ae; 306f is building in lane-game).
Game: the pinned dump (236,660 blueprints) through lane-tests' `game-check` build at 56b09798; code WrathDecompiled
6ce4a6b. No game text below: blueprints are cited by asset name and GUID, and each one was resolved to its GUID
through the entry or grantor whose finding names it (the prerequisite's own entry; the class, archetype or path of the
row), never by its name alone.

Evidence beside this file (scratchpad `307/`; copy what the queue keeps into `research/307/`):
- `run.txt`, `findings.tsv`: the check on main's data, 4,418 differences in 204 groups, exit 0.
- `carriers/`: the Rust tool (`carriers-307 index <binz> <dir>`, `carriers-307 carriers <dir>`; lz4_flex, serde_json).
  `bp.tsv` and `edges.tsv` are its index of the dump (every blueprint reference of a feature, progression, selection,
  class, archetype, race, buff or ability, with the component and field it sits in).
- `seeds.tsv` (every blueprint a finding names that no entry maps, with its owner), `chains.tsv` (every grant chain of
  each S3 seed up to an entry or a row, `vocab` when every link is a kind of 295's vocabulary), `carriers.tsv`,
  `final-cited.tsv` (the set below), `excluded.tsv`, `keys.tsv`, `down.tsv` (each carrier's own grants),
  `prereq-anchors.tsv` (the entries a prerequisite carrier's chain starts from).

## 0. Decisions taken here (say if you disagree)

1. **A carrier is a `features/` entry.** `SelectionOption` must be defined by exactly one selection
   (`OptionNotDefinedOnce`, `libs/rules/src/check.rs:1388`) and a selection of feats nothing names is refused
   (`UnusedFeatList`), so a hidden blueprint of any feature type (feature, progression, a hidden selection) is a
   `Feature`. Nothing checks that a feature is granted, so an unreferenced carrier passes `GameData::check`.
2. **New files only, and nothing names them.** The target never touches an existing file. A carrier's own
   `components` hold only its grants (`Grant`, `GrantAtClassLevel`, `GrantIf`, `GrantOnApply`) whose target is an
   entry (existing, or created in the same run); a grant of a blueprint with no entry is left out and listed (G7,
   318). With nothing naming a carrier, no Build holds one, so the census is empty by construction: the build sheet
   lists picks, not features (`libs/rules/src/engine/build_sheet.rs` `write_levels`), and a carrier has no number.
3. **Template** (design §2.7, user Q1): `(key, blueprint, name, components)`, no `summary` (the default empty), no
   `guide`. **Key:** kebab-case of the asset name less a trailing `Feature`/`Selection`/`Progression`; when two carriers
   or a carrier and an existing feature would share it, each of them keeps its suffix (`fire-blast-feature`,
   `fire-blast-progression`: order-free, so deterministic). 207 keys, 14 suffixed, no clash left (`keys.tsv`). Flat
   keys (244: "a key says nothing of an owner"). **Name:** the key's words, capitalised (`Smite Evil Additional Use`,
   `Divine Hunter Bond Plus 2`): the asset name, never the game's text.
4. **The writer computes the set; a registry leaves some out.** Candidates are (a) every `Prerequisite*` reference of a
   compared entry to an unmapped feature-type blueprint, (b) every unmapped non-pick grant the rows area places (its
   "in game only", "a further rank in game only" and "at another level" items, classes, archetypes and paths), (c) the
   unmapped feature-type blueprints on a vocabulary chain from (a) up to the first entry or row. A registry,
   `tools/game-check/carriers-left.txt` in `known.txt`'s line grammar (`<guid> -> <ticket>`, `#` comments), leaves out
   the blueprints another ticket owns; a line naming no candidate fails the run, as a stale `known.txt` line does.
5. **Slot order is untouched.** S3 makes no selection entry and no row, so 306's slot-order rule doesn't bite; S5 meets
   it when its rows name carriers (feature keys don't order slots, only `selections` do).
6. **The check moves, the census doesn't.** Mapping a blueprint changes the check's findings: 159 prerequisite findings
   on 76 entries turn from "missing in planner, a blueprint it doesn't map" into "missing in planner" (kineticist 88,
   bloodlines 22, shaman 19, misc 9, mythic 8, druid and shifter 5, bard 3, hunters 3, monk 2); a mapped wrapper the
   rows area used to unfold is now kept, its facts optional, so leaf findings go and a wrapper finding comes; a hidden
   carrier a row reaches becomes "in game only". Each data slice re-homes those `known.txt` lines: prerequisites to 308,
   rows and ranks to 309, and its stale lines out.

## 1. The carrier set (main 56b09798)

**Every blueprint a finding names that no entry maps: 403 distinct** (853 findings resolved to a GUID through their
entry or grantor; 7 left are pets' class levels and one `EldritchScionClass` level, not blueprints to make):
268 named by prerequisites (`missing in planner, a blueprint it doesn't map`), 139 by rows and ranks (4 by both).

| Owner | Distinct | What |
| --- | --- | --- |
| 294 (domains) | 74 | the domains' `…DomainProgressionSecondary`/`Separatist` |
| 267 (deities) | 39 | deity features (`AreshkagalFeature` …) and `AtheismFeature`; plus Groetus and Green Faith (Camelia), excluded below |
| S2 (306f–z, and its gaps) | 83 | members of a game selection's list or row-granted selections: the oracle archetype mysteries (18, 306v), the curses' `…CurseNoProgression` (306w), the second elements and Kinetic Knight foci (306t), the opposition schools (306p), the split selections themselves (Dark Lurker, Exploiter, Eldritch Magus, Skald and IW rage powers, Winter Witch, Nine-Tailed Heir, Seeker: 306f–y), Favored Enemy's demons and the exotic weapon proficiency (rows of 275/268 that are list members), Armor Focus (G3, 315), the Lich's skeletal upgrades |
| **S3** | **207** | 197 after the 10 left out below, plus 10 chain carriers: 207 made |

**S3 left out (10, `excluded.tsv`, each a `carriers-left.txt` line):** `GroetusFeature` c3e4d5681906d5246ab8b0637b98cbfe,
`GreenFaithCameliaFeature` ca763809e01f4247a3639965364c26cb (deities: 267); `EmpyrealBloodlineProgression`
8a95d80a3162d274896d50c2f18bb6b1, `SageBloodlineProgression` 7d990675841a7354c957689a6707c6c2,
`SylvanBloodlineProgression` a46d4bd93601427409d034a997673ece (the copies 306 re-homed to G5, 316);
`LichSkeletalBarbarianTierI` e2ae828e848a30d4d9471a0dd393435e, `LichSkeletalFighterTierI` b398a2ac41ea6c94683efd968b928595
(the Lich's upgrades, S2's gap); `AngelIncorporateSpellbook` e1fbb0e0e610a3a4d91e5e5284587939 (a
`BlueprintFeatureSelectMythicSpellbook`: G6, 317); `AnimalCompanionRank` 1670990255e4fe948a863bafd5dbda5d (pets, as the
check leaves them out); `MongrelmanRace` daca06a3f3355664bba1e87e67f3b5b3 (a race, never a feature: its prerequisite
stays S4's to list). And on the
chains, `AirBlastSelection` 49e55e8f24e1ad24e910fefc0258adba and `WaterBlastSelection` 53a8c2f3543147b4d913c6de0c57c7e8:
visible picks an element option's row grants (Air or Electric, Water or Cold blast) that the planner doesn't model:
270's model part, not carriers.

**The 207 S3 carriers** (full table with GUIDs, keys and one chain each in the appendix):

| | Count |
| --- | --- |
| By source | 127 rows only (S5 names them), 67 prerequisites only (S4 reads them), 3 both, 10 on a chain above a prerequisite carrier |
| By type | 198 `BlueprintFeature` (46 hidden), 9 `BlueprintProgression` (2 hidden) |
| By family | kineticist 28, bloodlines 25, hunters (ranger, hunter, slayer, inquisitor) 23, misc (races, channels, magus, prestige, oracle, cavalier, barbarian, alchemist) 23, mythic 20, warpriest 20, paladin 17, shaman 16, druid and shifter 14, monk 14, bard 7 |
| Their own grants | to an entry or carrier: 21 (6 `Grant`, 4 `GrantIf`, 11 a progression's rows); to a blueprint with no entry, left out (318): 46 `Grant`, 21 `GrantAtClassLevel`, 27 `GrantIf`, 20 progression rows, 5 through an action |

**How each prerequisite carrier is held in the game** (`prereq-anchors.tsv`, 70 carriers, every shortest chain):
- through a vocabulary chain from an **existing entry**, 66 entries: an option's progression rows (50 edges: the
  sorcerer's and bloodrager's bloodline options grant the 21 `…BloodlineRequisiteFeature`s; `options/kineticist/element/fire`
  and `earth` grant `FireBlastProgression` → `FireBlastFeature`), a feature's `AddFacts`/`AddFeatureIfHasFact`/
  `AddFeatureOnClassLevel` (10: Sensei's Advice → `BardicPerformanceResourcePrerequisite`, the wyrmshifter forms →
  `ShifterDragonForm…LevelUp`, `InquisitorBaneNormalFeatureAdd` → `InquisitorBaneNormalFeature`), a feat's
  `AddFeatureOnApply` (`SpellSpecializationFirst` → `SpellSpecializationProgression`), a race's features (4:
  `ElvenImmunities`, `HalfOrcFerocity`, `Hardy`), an option's `AddFacts` (2);
- through a **row** (36 chains: S5 writes it) or an **S2-made option** (52: the second, Seeker and crossblooded
  bloodlines, the second and third elements);
- **beyond the vocabulary** (13 kineticist blasts: the composite blasts come from an `ActivateTrigger` action of the
  `SecondaryElement*`/`ThirdElement*` progressions and from `CompositeBlastBuff`, Air and Electric through the Air
  pick above) or **from nothing a Build has** (14: the 8 shaman `…SpiritWanderingFeature`s and their progression, given
  by the daily spirit choice; `SpiritTotemLesserFeature`, `CorruptedGoldenDragonFeature`, granted only to units: their
  prerequisites are `HasNot`s, which hold).

Against the design's ≈400 (247 + 148 + 17 + chains): S2 makes most of the chains' options and 120 mysteries, 294 and
267 keep theirs, so S3 makes 207.

## 2. The slices, in order

Lanes (CLAUDE.md: two builders at most). **Lane W** (`tools/game-check/src/write/`, `main.rs`, the fixture): a → b → c,
then m. It shares no file with S2's Lane D (data, `known.txt`, seeds tests), so **a–c can start now, beside 306f–z**
(group P1: Lane W ∥ S2's Lane D); 306ab/306ac touch `areas/rows.rs` and `areas/mapping.rs`, so c (which adds a helper to
`areas/rows.rs`) waits for 306ab. **Lane D** (`libs/game-data/features/`, `known.txt`, `carriers-left.txt`,
`libs/seeds/tests/game_data/importer_carriers.rs`, notes): e → l, one at a time (each edits `known.txt`), after 306 is
merged and d is recorded; m (Lane W) may run beside them (group P2), n after it.

"Run" = `game-check --write carriers --only <the family's GUIDs> --log <scratchpad>/307x.tsv`. Every slice runs
`cargo fmt --all --check` and the CLAUDE.md "Checks" rows of what it touches; every data slice also runs `check-data`,
the game check (exit 0) and writes `libs/game-data/notes/307x-<slug>.md`. The shared test file is
`libs/seeds/tests/game_data/importer_carriers.rs` (new in 307e, `mod importer_carriers;` in `game_data/main.rs`), one
section per family: each test reads `DATA` and asserts, per carrier, its key, its blueprint and its components,
expected values cited by blueprint name and GUID.

---

### 307a. Writer: new files (Lane W, P1)

**Goal:** `apply` can create an entry's file whole, never over an existing one, and undo it.
**Edits:**
- `tools/game-check/src/write/mod.rs`: `pub struct NewFile { at: DataAt, file: PathBuf, text: String, kind: &'static str }`;
  `apply`/`apply_with`/`plan` take `&[NewFile]` beside `&[Change]`. Guards before any write: a path under `planner/`
  (`WriteError::Planner`), a path that exists (new `WriteError::Exists(PathBuf)`), a folder other than `features`
  (new `fn creatable(file) -> bool`; `WriteError::NotOwned { field: "(new file)" }`). Write through `atomic_write`
  after the field changes, creating the parent folder; read back: the disk text equals `text`, it parses as the
  folder's type (`rules::Feature`, `IMPLICIT_SOME`, its `deny_unknown_fields`), its `key` is the path's stem and its
  `blueprint` is set. On any failure every created file is removed (and every folder the run made), field files put
  back as today; `RestoreFailed` names one it couldn't remove. `Applied` gains `created: Vec<PathBuf>`; `run`'s
  `Outcome` gains `created: Vec<NewFile>`.
- `src/write/log.rs` `tsv`: one row per new file, field `(new file)`, old `-`, new `<key> <blueprint>`.
- `src/main.rs` `write_run`: prints `<area>: N files to create|created` with each path.
**Tests** (`tests/fixture/write.rs`): `a_new_file_is_written_whole_and_read_back`,
`an_existing_file_is_never_replaced_by_a_new_one`, `a_new_file_under_planner_or_outside_features_is_refused`,
`a_failed_validation_removes_every_new_file`, `a_dry_run_lists_the_new_files_and_writes_none`.
**Proof:** none of §4.2 (no data). **Blockers:** none (S1's writer is merged). **Shares:** `write/mod.rs`, `main.rs`,
`write.rs` with b, c, m. **Estimate:** 20 min.

### 307b. Writer: the carriers target, its candidates and template (Lane W, P1)

**Goal:** `--write carriers` makes a `features/` file for every unmapped blueprint a prerequisite names and every one
on its grant chain, keyed and named from its asset name.
**Edits:**
- `src/dump.rs`: `pub fn iter(&self) -> impl Iterator<Item = (&BlueprintGuid, &Blueprint)>`.
- `src/write/carriers.rs` (new): `pub fn changes(ctx: &Ctx, only: Option<&BTreeSet<BlueprintGuid>>) -> (Vec<NewFile>,
  Vec<Unexpressed>)`. Candidates: (a) every `!bp_` reference inside a `Prerequisite*` component (`PrerequisiteCondition`'s
  tree too) of the blueprint of an entry the prerequisites area compares (feats, options, mythic abilities and feats,
  classes, archetypes), unmapped and of a feature type (`dump::is_feature_kind`); (c) the reverse vocabulary edges built
  once from every feature-type blueprint's JSON (`AddFacts.m_Facts`, `AddFeatureOnClassLevel`/`AddFeatureIfHasFact`/
  `AddFeatureOnApply.m_Feature`, a progression's `LevelEntries`, an archetype's `AddFeatures`), walked up from each (a)
  until a mapped blueprint, a class's `m_Progression` or an archetype: every unmapped feature-type blueprint met is a
  candidate. Never a candidate: a member of any `m_AllFeatures` (a pick: S2's), or a GUID an entry maps (a second run
  finds it by its blueprint). `key_of`/`name_of` per §0.3 over the whole candidate set (the clash rule needs all of it),
  the file `features/<key>.ron` as one line through `span::ron_text` (as `features/evasion.ron`).
- `src/write/mod.rs`: `pub mod carriers;`; `WriteArea::Carriers` (`"carriers"`, area `Area::Prerequisites`, kind
  `"carrier"`); `run` routes it.
- `src/main.rs`: `--only` with `carriers` takes GUIDs (32 hex), with `option-lists` keys; both areas at once with
  `--only` refused; USAGE and the doc header.
- Fixture `blueprints.json`/`planner.ron`/`data/`: a hidden feature a fixture feat's prerequisite names, a wrapper
  feature granting it from a mapped feature (a chain of two), a feature that is a member of a fixture selection's list.
**Tests:** `a_prerequisite_naming_an_unmapped_feature_makes_a_carrier`, `a_chain_up_to_an_entry_makes_carriers`,
`a_selection_member_is_never_a_carrier`, `keys_drop_the_suffix_unless_two_would_share_one`,
`a_second_carriers_run_writes_nothing`, `two_carriers_runs_on_two_copies_make_the_same_files`.
**Proof:** `--write carriers --dry-run` on main's data lists the 70 prerequisite and 10 chain carriers of
`final-cited.tsv` (plus the S2-pending ones until 306 merges, and the 12 the registry of c will leave out).
**Blockers:** a. **Estimate:** 20 min.

### 307c. Writer: the rows' leaves, the registry, the grant components (Lane W, P1)

**Goal:** the target also makes the rows' unmapped leaves, leaves out what another ticket owns, and writes each
carrier's grants of entries.
**Edits:**
- `src/areas/rows.rs`: `pub(crate) fn unmapped_grants(ctx: &Ctx) -> BTreeSet<BlueprintGuid>`: the `Item::Guid`
  placements (`How::Grant`) of every class, archetype and mythic path the rows area builds, unmapped, no pick (it runs
  `grant`/`nested` with a scratch `Out`).
- `tools/game-check/carriers-left.txt` (new; `#` comments, `<guid> -> <ticket>`), embedded as `known.txt` is;
  `src/write/carriers.rs` reads it: a candidate it names is not made; a line naming no candidate fails the run (new
  `WriteError::StaleLeft(Vec<String>)`). The real lines come in 307e; the fixture has its own.
- `src/write/carriers.rs`: candidates gain (b) `unmapped_grants`. Components from the blueprint, in 296/297's order:
  `AddFacts` → one `Grant` of its feature-type facts; `AddFeatureOnClassLevel` → `GrantAtClassLevel { fact, classes:
  m_Class + m_AdditionalClasses, archetypes: m_Archetypes, level: Level, before: BeforeThisLevel }`;
  `AddFeatureIfHasFact` → `GrantIf { has: m_CheckedFact, not: Not, fact: m_Feature }`; `AddFeatureOnApply` →
  `GrantOnApply`; a progression's `LevelEntries` → one `GrantAtClassLevel` per (level, fact) over its `m_Classes` and
  `m_Archetypes` (Q2), unexpressed when an `AdditionalLevel` isn't 0, `ForAllOtherClasses` is set or the fact is a
  selection. Each fact's kind is its entry's (`Planner::entries_of`, the carrier's own key for one made in this run);
  none or several → `Unexpressed` (`a grant of <name>, no entry: 318`). `--only` is closed under carrier-to-carrier
  grants: a member's carrier targets join it, so no file names a key a later run makes.
- Fixture: a carrier with `AddFacts` of an entry and of an unmapped feature, a progression carrier, a hidden row leaf,
  a registry line.
**Tests:** `a_row_leaf_no_entry_maps_is_a_carrier`, `a_carrier_holds_its_grants_of_entries`,
`a_grant_of_a_blueprint_with_no_entry_is_left_out_and_listed`, `a_progression_carrier_grants_its_rows_at_class_levels`,
`a_left_out_line_keeps_a_candidate_out_and_a_stale_one_fails`, `only_takes_in_the_carriers_its_members_grant`.
**Proof:** dry run on main's data: the candidates equal `final-cited.tsv` plus `excluded.tsv` (and S2's pending), the
unexpressed list is `down.tsv`'s 119 grants of blueprints with no entry. **Blockers:** b; 306ab merged (it edits
`areas/rows.rs`). **Estimate:** 20 min.

### 307d. The census base (main session; heavy, alone)

After 306 (306ad) and 307c merge, before 307e: `FORM_PROOF_OUT=<dir> FORM_PROOF_OFFERS=1 cargo test --release -p corpus
--test form_proof record -- --ignored --nocapture` on that `main` (`CLAUDE.md`, "Behaviour checks"). Each data slice
compares its tip with it: `FORM_PROOF_BEFORE=<dir> … compare` must print no differing line (S3 is a form change), and
`compare`'s link census 2,000 same, 0 heirs, 0 refused. Also the dry run of c on that base, saved for e's registry.

---

### 307e. Kineticist carriers, and the registry (Lane D)

**Edits:**
- `tools/game-check/carriers-left.txt`: the 10 lines of `excluded.tsv` (pets, 267 ×2, 316 ×3, 317, the Lich ×2,
  Mongrelman → 308 to list), `AirBlastSelection` 49e55e8f24e1ad24e910fefc0258adba and `WaterBlastSelection`
  53a8c2f3543147b4d913c6de0c57c7e8 → 270, and any candidate d's dry run shows that S2 left (each under its ticket).
- Run `--only` the 28 kineticist GUIDs (appendix): 6 blasts, 11 composite blasts, 6 blast progressions,
  `MentalOverflowProgression` 42b8ce9dfb087c74db531bad426ad2f3 and the 4 kineticist row leaves.
- `known.txt`: the 88 kineticist prerequisite findings (270 87, 271 1) turn to "missing in planner": their lines stay
  under 270 (its model part holds the blasts: Q3), the 271 one to 308; rows lines the leaves move, to 309.
**Tests** (`importer_carriers.rs`, new): `the_kineticist_blast_carriers_are_their_blueprints` (each key, blueprint and
components; `fire-blast-progression` grants `fire-blast-feature` at kineticist 1: `FireBlastProgression`
fbed3ca8c0d89124ebb3299ccf68c439 `LevelEntries`), `no_vocabulary_grant_holds_a_composite_blast` (none grants
`blue-flame-blast`: its grant is the `ActivateTrigger` of `SecondaryElementFire` caa7edca64af1914d9e14785beb6a143).
**Proof:** census 0 lines; game check exit 0; a second run writes 0 files. **Blockers:** 306 merged, c, d.
**Estimate:** 20 min.

### 307f. Bloodline carriers (Lane D)

Run the 25 GUIDs: the 21 hidden `…BloodlineRequisiteFeature`s (the sorcerer's and bloodrager's bloodline options grant
them at bloodline level 1: `BloodlineAbyssalProgression` → `AbyssalBloodlineRequisiteFeature`
b09b58c7f8efff244a33269489abeac6), `FeatureWingsAngelSorcerer`, the Greenrager's three `GreenragerSummoningRager*`.
`known.txt`: 22 prerequisite findings (293) re-kinded → 308. **Tests:** `the_bloodline_requisite_carriers_are_their_blueprints`.
**Estimate:** 15 min.

### 307g. Mythic carriers (Lane D)

Run the 20: the Aeon's immunities and their two wrappers (`AeonFifthLevelImmunities`, `AeonSeventhLevelImmunities`,
whose `Grant`s name the immunities), gaze and demythication features, the Locust's clones, the Demon's forced rage,
`MythicLichLevel3`, `LifeBondingFriendshipFeature`, `FakeDivineSparkFeature`, `CorruptedGoldenDragonFeature`.
`known.txt`: 8 prerequisite findings (266, 265) → 308; the mythic ranks' "in game only" lines of the Aeon move from the
leaves to the wrappers → 309. **Tests:** `the_mythic_path_carriers_are_their_blueprints`. **Estimate:** 15 min.

### 307h. Warpriest and paladin carriers (Lane D)

Run the 37 (rows only): the sacred weapon and armour enchantment steps, the Proclaimer's aura limits, the Mantis
Zealot's focus; the paladin archetypes' auras, Divine Hunter bonds, Martyr and Power of Faith tiers,
`SmiteEvilAdditionalUse`, `StonebaneFeature`. `known.txt`: rows lines that move (leaf findings a mapped feature's facts
replace) → 309. **Tests:** `the_warpriest_and_paladin_row_carriers_are_their_blueprints`. **Estimate:** 20 min.

### 307i. Hunter, ranger, slayer and inquisitor carriers (Lane D)

Run the 23: `JudgmentAdditionalUse`, `JudgmentFaithHunterFeature`, `InquisitorBaneNormalFeature`, the slayer's
studied-spawn and advance features, `SanctifiedSlayerSlayerTalentFeature`, the hunters' row leaves, the Living
Grimoire's and Faith Hunter's. `known.txt`: 3 prerequisite findings → 308, rows → 309. **Tests:**
`the_hunter_family_carriers_are_their_blueprints`. **Estimate:** 15 min.

### 307j. Druid, shifter and shaman carriers (Lane D)

Run the 30: the shifter's `…LevelUp` forms and final aspects, the druid's alignment restriction and Rageshaper forms;
the shaman's weapon steps and the 8 `Shaman…SpiritWanderingFeature`s with `ShamanNatureSpiritWanderingProgression`
(given by the daily spirit choice: nothing in the vocabulary holds them, Q3). `known.txt`: 24 prerequisite findings
(278 19, 277/263 5) → 308, the wandering spirits' under the gap Q3 names. **Tests:**
`the_druid_shifter_and_shaman_carriers_are_their_blueprints`. **Estimate:** 20 min.

### 307k. Monk and bard carriers (Lane D)

Run the 21: the Scaled Fist's, Sohei's, Sensei's and Quarterstaff Master's row leaves, `ScaledFistPowerFeature`;
`BardCantripsFeature`, the Beast Tamer's three, `CrumblingBlastTier2Feature`, `BardicPerformanceResourcePrerequisite` and
`BardicPerformanceResourceFact` (its `Grant`). **Tests:** `the_monk_and_bard_carriers_are_their_blueprints`.
**Estimate:** 15 min.

### 307l. The rest (Lane D)

Run the 23: the races' (`ElvenImmunities`, `HalfOrcFerocity`, `Hardy`), the channel features (`LichChannelNegativeFeature`,
`OracleRevelationChannelFeature`), `SpellSpecializationProgression` (its rows grant a selection: unexpressed, 317),
`SpiritTotemLesserFeature`, the magus's, cavalier's, oracle's, alchemist's and prestige classes' row leaves.
**Tests:** `the_remaining_carriers_are_their_blueprints`. **Estimate:** 15 min.

### 307m. Writer: the carriers' grantors (Lane W, P2; only if Q1 = A)

**Goal:** an existing entry whose blueprint grants a carrier names it, so the game's holder holds it.
**Edits:** `src/write/carriers.rs` `attachments(ctx) -> Vec<Change>`; `WriteArea::CarrierGrants` (`"carrier-grants"`);
`owned()` gains, for `features`/`feats`/`options`, `components` (the grant kinds only: the read-back already refuses a
change beyond the field) and, for `options`, `levels`: a grant is added only where its target is a carrier and the
entry's blueprint grants it (`AddFacts`, `AddFeatureIfHasFact`, `AddFeatureOnClassLevel`, `AddFeatureOnApply`; an
option progression's `LevelEntries` row at its level), never removing anything. Races' `features` if the `Race` form
takes it (3 carriers). **Tests:** `an_option_s_row_names_the_carrier_its_progression_grants`,
`a_feature_grants_the_carrier_its_blueprint_adds`, `a_grant_already_named_is_not_written_again`.
**Blockers:** c. **Shares:** `write/mod.rs` (Lane W only). **Estimate:** 20 min.

### 307n. The grantors' run (Lane D; only if Q1 = A)

Run `--write carrier-grants`: 66 entries (50 option rows: the bloodlines and the fire and earth elements; 10 features'
grants; Spell Specialization; 3 races; 2 options' `Grant`s). The census must stay empty: a held carrier is read by
nothing yet. **Tests:** `an_abyssal_sorcerer_holds_its_bloodline_requisite_carrier`,
`a_wyrmshifter_holds_its_dragon_form_level_up_carrier`. **Blockers:** l, m. **Estimate:** 15 min.

### 307o. S3's proof (main session; heavy runs alone)

The design's §4.2 for a form change: the game check exits 0 (no line under 293 for S3's areas; every re-kinded line under
308, 309, 270 or a gap); `--write carriers` (and `carrier-grants`) a second time writes 0 files; two runs on two
copies byte-identical; the census against d: 0 lines, links 2,000 same; the hand check of 100 of the 207 by FNV of key
(each file against `game-check show <guid>`); the long self-checks; the gold builds (none expected); the ignored tests:
none waits on S3's areas (the carriers are 308's and 309's inputs).

## 3. Open questions

**Q1. Who makes a carrier held?** A carrier is held only when something names it. For 70 prerequisite carriers the
game's holder is an existing entry for 66 entries (50 option progression rows, 10 features' grants, a feat, 3 races, 2
options; `prereq-anchors.tsv`), a row (S5) or an S2 option. If S4 writes `Has(carrier)` before the carrier is held, every
pick needing it is refused (the picks whose prerequisites name a bloodline requisite, a wyrmshifter form step or
the inquisitor's bane carrier). The design's owned fields give no stage the option rows or the features' grant
components (S5 owns class, archetype and path rows).
- A (recommended): S3 does it, 307m–n: a second target adding only grants of carriers to the entries whose blueprints
  grant them. The census stays empty (a held carrier is read by nothing), so S4's census shows prerequisites alone.
- B: S4 writes each grant beside the prerequisite that reads it; its census attributes both.
- C: G7 (318) writes them; S4 leaves those prerequisites unwritten and listed until then.

**Q2. A hidden progression's rows on a `Feature`.** 7 carriers are progressions whose `LevelEntries` grant features (6
blast progressions, `MentalOverflowProgression`), 2 more grant a selection (`SpellSpecializationProgression`,
`ShamanNatureSpiritWanderingProgression`). A `Feature` has no rows.
- A (recommended): one `GrantAtClassLevel` per level over the progression's `m_Classes`/`m_Archetypes` (the same sum
  `BlueprintProgression.CalcLevel` makes with no `AdditionalLevel`, checked by the target); a selection in the rows, an
  offset or `ForAllOtherClasses` is unexpressed and listed (317, 318).
- B: `Feature` gains `header` and `levels` as `SelectionOption` has (a form change and its own ticket).
- C: leave their rows out (318); the blasts are held only once G7 lands.

**Q3. Carriers nothing in the vocabulary can hold.** 13 kineticist blasts (the 11 composite ones from an
`ActivateTrigger` action and `CompositeBlastBuff`; Air and Electric through the element's blast pick, which the planner
doesn't model) and the shaman's 9 wandering-spirit carriers (the daily spirit choice, `daily_choice` in the planner).
88 + 18 prerequisite findings name them; written in S4 they would refuse every infusion and wandering hex.
- A (recommended): make the entries in S3 (named, nothing grants them), and S4 leaves those prerequisites unwritten,
  listed under 270 (its model part: the blast pick and the composite trigger) and a new gap for the wandering spirits.
- B: don't make them; their findings stay "a blueprint it doesn't map" under 270 and the gap.

**Risks (no decision):** S5 names the 130 row carriers, and `summaries.rs`'s "class feature" count of empty summaries
(28) then grows with them (user Q1: carriers get none), so 309 updates `EMPTY` with its reason. A carrier that is the
game's copy of a feature 244 collapsed into one entry would split it again: the hand check (307o) compares each row
carrier's stripped key with the existing keys. The writer's runs are cheap (the dump reads in a second); only d and o
are heavy. Counts are at 56b09798 and move as 306f–z land: the target computes the set at its own base, and e's
registry takes what S2 left.

---

## Appendix: the 207 carriers by family

GUIDs for each data slice's `--only`. "Source": prerequisite (S4 reads it), row (S5 names it), chain (above a
prerequisite carrier). "Known under": the tickets of the findings that name it today. The chain is the shortest one
`chains.tsv` holds (`vocab`: every link a kind of 295's vocabulary; `beyond`: an action or a buff on the way).

**bard**

| Carrier | GUID | Source; known under | Hidden | Key | A grant chain (shortest) |
| --- | --- | --- | --- | --- | --- |
| BardCantripsFeature | 4f422e8490ec7d94592a8069cce47f98 | row; 286 | false | `bard-cantrips` | BardProgression -LevelEntries-> BardCantripsFeature (vocab) |
| BardicPerformanceResourceFact | b92bfc201c6a79e49afd0b5cfbfc269f | chain; - | true | `bardic-performance-resource-fact` | on a chain above another carrier (`chains.tsv`) |
| BardicPerformanceResourcePrerequisite | 019ada4530c41274a885dfaa0fbf6218 | prerequisite; 263,264 | true | `bardic-performance-resource-prerequisite` | SenseiAdvice -Grant-> BardicPerformanceResourcePrerequisite (vocab) |
| BeastTamerInspireFerocity10Feature | 8e1c039bebf4346408c02cc79026802d | row; 286 | false | `beast-tamer-inspire-ferocity-10` | BeastTamerArchetype -AddFeatures-> BeastTamerInspireFerocity10Feature (vocab) |
| BeastTamerInspireFerocity14Feature | f6f4bc35fcb61be4295ea290e8bfce4e | row; 286 | false | `beast-tamer-inspire-ferocity-14` | BeastTamerArchetype -AddFeatures-> BeastTamerInspireFerocity14Feature (vocab) |
| BeastTamerInspireFerocity18Feature | 0d72ff5398412064cbcec0af95b32286 | row; 286 | false | `beast-tamer-inspire-ferocity-18` | BeastTamerArchetype -AddFeatures-> BeastTamerInspireFerocity18Feature (vocab) |
| CrumblingBlastTier2Feature | 777f1610f18a3b947908bfcd01f49bc3 | row; 286 | false | `crumbling-blast-tier-2` | HeraldOfTheHornArchetype -AddFeatures-> CrumblingBlastTier2Feature (vocab) |

**bloodlines**

| Carrier | GUID | Source; known under | Hidden | Key | A grant chain (shortest) |
| --- | --- | --- | --- | --- | --- |
| AbyssalBloodlineRequisiteFeature | b09b58c7f8efff244a33269489abeac6 | prerequisite; 293 | true | `abyssal-bloodline-requisite` | BloodragerAbyssalBloodline -LevelEntries-> AbyssalBloodlineRequisiteFeature (vocab) |
| ArcaneBloodlineRequisiteFeature | 60d8632e96739a74dbac23dd078d205d | prerequisite; 293 | true | `arcane-bloodline-requisite` | BloodragerArcaneBloodline -LevelEntries-> ArcaneBloodlineRequisiteFeature (vocab) |
| CelestialBloodlineRequisiteFeature | 6906eafd622304841bb45c2601aec7a7 | prerequisite; 293 | true | `celestial-bloodline-requisite` | BloodragerCelestialBloodline -LevelEntries-> CelestialBloodlineRequisiteFeature (vocab) |
| DraconicBlackBloodlineRequisiteFeature | d89fb8ce9152ffa4dacd69390f3d7721 | prerequisite; 293 | true | `draconic-black-bloodline-requisite` | BloodragerDragonBlackBloodline -LevelEntries-> DraconicBlackBloodlineRequisiteFeature (vocab) |
| DraconicBlueBloodlineRequisiteFeature | 64e1f27147b642448842ab8dcbaca03f | prerequisite; 293 | true | `draconic-blue-bloodline-requisite` | BloodragerDragonBlueBloodline -LevelEntries-> DraconicBlueBloodlineRequisiteFeature (vocab) |
| DraconicBrassBloodlineRequisiteFeature | 12bb1056a5f3f9f4b9facdb78b8d8914 | prerequisite; 293 | true | `draconic-brass-bloodline-requisite` | BloodragerDragonBrassBloodline -LevelEntries-> DraconicBrassBloodlineRequisiteFeature (vocab) |
| DraconicBronzeBloodlineRequisiteFeature | 1d34d95ad4961e343b02db14690eb6d8 | prerequisite; 293 | true | `draconic-bronze-bloodline-requisite` | BloodragerDragonBronzeBloodline -LevelEntries-> DraconicBronzeBloodlineRequisiteFeature (vocab) |
| DraconicCopperBloodlineRequisiteFeature | eef664d1e4318f64cb2304d1628d29ae | prerequisite; 293 | true | `draconic-copper-bloodline-requisite` | BloodragerDragonCopperBloodline -LevelEntries-> DraconicCopperBloodlineRequisiteFeature (vocab) |
| DraconicGoldBloodlineRequisiteFeature | bef8d08ee3c20b246b404ce3ef948291 | prerequisite; 293 | true | `draconic-gold-bloodline-requisite` | BloodragerDragonGoldBloodline -LevelEntries-> DraconicGoldBloodlineRequisiteFeature (vocab) |
| DraconicGreenBloodlineRequisiteFeature | 49115e2147cd32841baa34c305171daa | prerequisite; 293 | true | `draconic-green-bloodline-requisite` | BloodragerDragonGreenBloodline -LevelEntries-> DraconicGreenBloodlineRequisiteFeature (vocab) |
| DraconicRedBloodlineRequisiteFeature | 9c5ed34089fedf54ba8d0f43565bcc91 | prerequisite; 293 | true | `draconic-red-bloodline-requisite` | BloodragerDragonRedBloodline -LevelEntries-> DraconicRedBloodlineRequisiteFeature (vocab) |
| DraconicSilverBloodlineRequisiteFeature | 01e7aab638d6a0b43bc4e9d5b49e68d9 | prerequisite; 293 | true | `draconic-silver-bloodline-requisite` | BloodragerDragonSilverBloodline -LevelEntries-> DraconicSilverBloodlineRequisiteFeature (vocab) |
| DraconicWhiteBloodlineRequisiteFeature | 3867419bf47841b428333808dfdf4ae0 | prerequisite; 293 | true | `draconic-white-bloodline-requisite` | BloodragerDragonWhiteBloodline -LevelEntries-> DraconicWhiteBloodlineRequisiteFeature (vocab) |
| ElementalAirBloodlineRequisiteFeature | 65bc595a8a27acb48a8758c4d4caa338 | prerequisite; 293 | true | `elemental-air-bloodline-requisite` | BloodlineElementalAirProgression -LevelEntries-> ElementalAirBloodlineRequisiteFeature (vocab) |
| ElementalEarthBloodlineRequisiteFeature | 96e15b7b08b65d44b90a3f0c3c7ed72b | prerequisite; 293 | true | `elemental-earth-bloodline-requisite` | BloodragerElementalAcidBloodline -LevelEntries-> ElementalEarthBloodlineRequisiteFeature (vocab) |
| ElementalFireBloodlineRequisiteFeature | 363737ca96b4a084b852688fd7430fa5 | prerequisite; 293 | true | `elemental-fire-bloodline-requisite` | BloodragerElementalFireBloodline -LevelEntries-> ElementalFireBloodlineRequisiteFeature (vocab) |
| ElementalWaterBloodlineRequisiteFeature | cfd48c078a14ee247aa7eabe263ad132 | prerequisite; 293 | true | `elemental-water-bloodline-requisite` | BloodragerElementalColdBloodline -LevelEntries-> ElementalWaterBloodlineRequisiteFeature (vocab) |
| FeatureWingsAngelSorcerer | 894d21ff523481d49a51ab1750fca3a0 | prerequisite; 263 | false | `feature-wings-angel-sorcerer` | BloodlineCelestialProgression -LevelEntries-> FeatureWingsAngelSorcerer (vocab) |
| FeyBloodlineRequisiteFeature | 2c82cecdcafb5c741bf4901e689a4844 | prerequisite; 293 | true | `fey-bloodline-requisite` | BloodragerFeyBloodline -LevelEntries-> FeyBloodlineRequisiteFeature (vocab) |
| GreenragerSummoningRager10 | c8d3a1f34b8c4a348a41fcfb213da466 | row; 281 | false | `greenrager-summoning-rager-10` | GreenragerArchetype -AddFeatures-> GreenragerSummoningRager10 (vocab) |
| GreenragerSummoningRager13 | d0167cb030647ae4ea0cb8d401699d80 | row; 281 | false | `greenrager-summoning-rager-13` | GreenragerArchetype -AddFeatures-> GreenragerSummoningRager13 (vocab) |
| GreenragerSummoningRager7 | 35a04536a5707ef4f876ccd2155d7bc4 | row; 281 | false | `greenrager-summoning-rager-7` | GreenragerArchetype -AddFeatures-> GreenragerSummoningRager7 (vocab) |
| InfernalBloodlineRequisiteFeature | e71d00a5aa1ed8f4591d021056a6dbe7 | prerequisite; 293 | true | `infernal-bloodline-requisite` | BloodragerInfernalBloodline -LevelEntries-> InfernalBloodlineRequisiteFeature (vocab) |
| SerpentineBloodlineRequisiteFeature | 233d0e5bdd25a7f44a56c09c8b5041e7 | prerequisite; 293 | true | `serpentine-bloodline-requisite` | BloodragerSerpentineBloodline -LevelEntries-> SerpentineBloodlineRequisiteFeature (vocab) |
| UndeadBloodlineRequisiteFeature | 8a59e4af9f32950418260034c8b477fa | prerequisite; 293 | true | `undead-bloodline-requisite` | BloodragerUndeadBloodline -LevelEntries-> UndeadBloodlineRequisiteFeature (vocab) |

**druid-shifter**

| Carrier | GUID | Source; known under | Hidden | Key | A grant chain (shortest) |
| --- | --- | --- | --- | --- | --- |
| DruidAlignmentRestriction | d00ff3791359311449c481126fbf71ce | row; 277 | false | `druid-alignment-restriction` | DruidProgression -LevelEntries-> DruidAlignmentRestriction (vocab) |
| ElementalRampagerClaws1d10 | bd273aafc6104aacb0e0bf2727a2ce4d | row; 277 | false | `elemental-rampager-claws-1d-10` | ElementalRampagerArchetype -AddFeatures-> ElementalRampagerClaws1d10 (vocab) |
| ElementalRampagerClaws1d8 | c88eb191d1dd4f5dafc9988822ae24fd | row; 277 | false | `elemental-rampager-claws-1d-8` | ElementalRampagerArchetype -AddFeatures-> ElementalRampagerClaws1d8 (vocab) |
| FiendflashShifterFinalAspect | 6b8a596ef20c4d79b539147424c056e4 | row; 277 | false | `fiendflash-shifter-final-aspect` | FiendfleshShifterShifterArchetype -AddFeatures-> FiendflashShifterFinalAspect (vocab) |
| FiendflashShifterInfernalClawsFeature | 2c8a19a9d84847e988d46882fd7bb0af | prerequisite; 263 | false | `fiendflash-shifter-infernal-claws` | FiendfleshShifterShifterArchetype -AddFeatures-> FiendflashShifterInfernalClawsFeature (vocab) |
| FinalWeretouchedShifterAspectFeature | 0de9ff05339a47d2bdce00c024b172a8 | row; 277 | false | `final-weretouched-shifter-aspect` | WereTouchedShifter -AddFeatures-> FinalWeretouchedShifterAspectFeature (vocab) |
| RageshaperDevastatingFormGreater | 16690665414d4675919c0747aeb16100 | row; 277 | false | `rageshaper-devastating-form-greater` | RageshaperShifterArchetype -AddFeatures-> RageshaperDevastatingFormGreater (vocab) |
| RageshaperDevastatingFormMighty | c62ceaf7706444a2ba93d1990ee6ea12 | row; 277 | false | `rageshaper-devastating-form-mighty` | RageshaperShifterArchetype -AddFeatures-> RageshaperDevastatingFormMighty (vocab) |
| ShifterAspectGriffonFree | 38f0034df27e40509320b28029d1169e | row; 277 | false | `shifter-aspect-griffon-free` | GriffonheartShifterArchetype -AddFeatures-> ShifterAspectGriffonFree (vocab) |
| ShifterDragonFormEvilFeatureLevelUp | 60e0aeb936a84576a44ea9b620bb12d9 | prerequisite; 263 | true | `shifter-dragon-form-evil-feature-level-up` | ShifterDragonFormFeature -GrantIf-> ShifterDragonFormEvilFeatureLevelUp (vocab) |
| ShifterDragonFormGoodFeatureLevelUp | fdd15b03962b41a5a0d9f9d19ba219fd | prerequisite; 263 | true | `shifter-dragon-form-good-feature-level-up` | ShifterDragonFormFeature -GrantIf-> ShifterDragonFormGoodFeatureLevelUp (vocab) |
| ShifterDragonFormNeutralFeatureLevelUp | 73290f79a79a4eb78e310a8bb166d3e8 | prerequisite; 263 | true | `shifter-dragon-form-neutral-feature-level-up` | ShifterDragonFormFeature -GrantIf-> ShifterDragonFormNeutralFeatureLevelUp (vocab) |
| ShifterWildShapeGriffonFeatureLevelUp | 84e9b89787854c37b2412d1a71229e53 | prerequisite; 263 | true | `shifter-wild-shape-griffon-feature-level-up` | ShifterAspectGriffonProgression -LevelEntries-> ShifterWildShapeGriffonFeatureLevelUp (vocab) |
| WinterChildCompanionTrirdUpgradeFeature | 45e4928aa4174aaf8b8297ea9f8f2108 | row; 277 | false | `winter-child-companion-trird-upgrade` | WinterChildArchetype -AddFeatures-> WinterChildCompanionTrirdUpgradeFeature (vocab) |

**hunters**

| Carrier | GUID | Source; known under | Hidden | Key | A grant chain (shortest) |
| --- | --- | --- | --- | --- | --- |
| DemonslayerExpandedSpellList | 6536c42a01f37624dae358153f107027 | row; 275 | false | `demonslayer-expanded-spell-list` | DemonslayerArchetype -AddFeatures-> DemonslayerExpandedSpellList (vocab) |
| FaithHunterSwornEnemySmiteExtraUse | 2fab74210ab73ba47b8bad213becce4f | row; 284 | false | `faith-hunter-sworn-enemy-smite-extra-use` | FaithHunterArchetype -AddFeatures-> FaithHunterSwornEnemySmiteExtraUse (vocab) |
| FlamewardenSpellcasting2 | 275d72a20f615b54eb89460cd3aa8e1b | row; 275 | false | `flamewarden-spellcasting-2` | FlamewardenArchetype -AddFeatures-> FlamewardenSpellcasting2 (vocab) |
| FlamewardenSpellcasting3 | 0dc70999341cdd644be0b972dd83900b | row; 275 | false | `flamewarden-spellcasting-3` | FlamewardenArchetype -AddFeatures-> FlamewardenSpellcasting3 (vocab) |
| FlamewardenSpellcasting4 | 891441bcd187d4d43abc46d3202e07a0 | row; 275 | false | `flamewarden-spellcasting-4` | FlamewardenArchetype -AddFeatures-> FlamewardenSpellcasting4 (vocab) |
| ForesterAnimalFocusFeature | f63fdfe05a7393b40b5cd60e270bb117 | row; 275 | false | `forester-animal-focus` | ForesterArchetype -AddFeatures-> ForesterAnimalFocusFeature (vocab) |
| FreebooterMasterHunter | 6ce95769148f40c4aada8c01ee39d9a1 | row; 275 | false | `freebooter-master-hunter` | FreebooterArchetype -AddFeatures-> FreebooterMasterHunter (vocab) |
| ImitatorMasterSlayerFeature | 97c7054442504513b0324c00db0290fc | row; 276 | false | `imitator-master-slayer` | ImitatorArchetype -AddFeatures-> ImitatorMasterSlayerFeature (vocab) |
| InquisitorBaneNormalFeature | 4a24abda0d564f94094e55ef117fc5b2 | prerequisite; 265 | true | `inquisitor-bane-normal` | InquisitorBaneNormalFeatureAdd -GrantAtClassLevel-> InquisitorBaneNormalFeature (vocab) |
| JudgmentAdditionalUse | ee50875819478774b8968701893b52f5 | row; 275,284 | false | `judgment-additional-use` | DivineHoundArchetype -AddFeatures-> JudgmentAdditionalUse (vocab) |
| JudgmentFaithHunterFeature | 19a36bd554ff4fd492d7458d4f922be4 | prerequisite,row; 265,284 | false | `judgment-faith-hunter` | FaithHunterArchetype -AddFeatures-> JudgmentFaithHunterFeature (vocab) |
| LivingGrimoireSacredWordPlus2 | 7c9e0727b5394dab8bd850420f5f18c8 | row; 284 | false | `living-grimoire-sacred-word-plus-2` | LivingGrimoireArchetype -AddFeatures-> LivingGrimoireSacredWordPlus2 (vocab) |
| LivingGrimoireSacredWordPlus3 | 820c37f616a44265b1e53b9455153a73 | row; 284 | false | `living-grimoire-sacred-word-plus-3` | LivingGrimoireArchetype -AddFeatures-> LivingGrimoireSacredWordPlus3 (vocab) |
| LivingGrimoireSacredWordPlus4 | 4e3449dd48764ff79bbbddc6ed10cb14 | row; 284 | false | `living-grimoire-sacred-word-plus-4` | LivingGrimoireArchetype -AddFeatures-> LivingGrimoireSacredWordPlus4 (vocab) |
| LivingGrimoireSacredWordPlus5 | 2b590a953e544acf94c7a9a1e2b90b64 | row; 284 | false | `living-grimoire-sacred-word-plus-5` | LivingGrimoireArchetype -AddFeatures-> LivingGrimoireSacredWordPlus5 (vocab) |
| SanctifiedSlayerSlayerTalentFeature | db3767ba22ce84942af56525cb9f9ce6 | prerequisite,row; 264,284 | false | `sanctified-slayer-slayer-talent` | SanctifiedSlayerArchetype -AddFeatures-> SanctifiedSlayerSlayerTalentFeature (vocab) |
| SharpTeehCriticalRangeIncreaseUI | 38eee152c88b463987e75893c948e294 | row; 276 | false | `sharp-teeh-critical-range-increase-ui` | BloodseekerArchetype -AddFeatures-> SharpTeehCriticalRangeIncreaseUI (vocab) |
| SlayersAdvanceIncreaseResourceFeature | f4df3231977753d4ebd2ceb2ef5901ed | row; 276 | false | `slayers-advance-increase-resource` | SlayerProgression -LevelEntries-> SlayersAdvanceIncreaseResourceFeature (vocab) |
| StudiedSpawnColossalSizeBonus | 596bf484e187cfd4b9f0a22d1727dbf3 | row; 276 | false | `studied-spawn-colossal-size-bonus` | SpawnSlayerArchetype -AddFeatures-> StudiedSpawnColossalSizeBonus (vocab) |
| StudiedSpawnGargantuanSizeBonus | 04c9063177cce8044bb7997252a6d29f | row; 276 | false | `studied-spawn-gargantuan-size-bonus` | SpawnSlayerArchetype -AddFeatures-> StudiedSpawnGargantuanSizeBonus (vocab) |
| StudiedSpawnHugeSizeBonus | bd9f8b8e2217ab74ab5d6a81eec34691 | row; 276 | false | `studied-spawn-huge-size-bonus` | SpawnSlayerArchetype -AddFeatures-> StudiedSpawnHugeSizeBonus (vocab) |
| TacticalLeaderFeatShareFeature | 93e78cad499b1b54c859a970cbe4f585 | row; 284 | false | `tactical-leader-feat-share` | TacticalLeaderArchetype -AddFeatures-> TacticalLeaderFeatShareFeature (vocab) |
| UrbanHunterCaptor | 6a7183b066616464b838b10e92cf7146 | row; 275 | false | `urban-hunter-captor` | UrbanHunterArchetype -AddFeatures-> UrbanHunterCaptor (vocab) |

**kineticist**

| Carrier | GUID | Source; known under | Hidden | Key | A grant chain (shortest) |
| --- | --- | --- | --- | --- | --- |
| AirBlastFeature | cb09e292ad9acc3428fa0dfdcbb83883 | prerequisite; 270 | true | `air-blast-feature` | SecondatyElementalFocusSelection -Option-> SecondaryElementAir -Action-> AirBlastFeature (beyond) |
| AirBlastProgression | 6f1d86ae43adf1049834457ce5264003 | chain; - | false | `air-blast-progression` | on a chain above another carrier (`chains.tsv`) |
| BlizzardBlastFeature | 52292a32bb5d0ab45a86621bac2c4c9a | prerequisite; 270 | true | `blizzard-blast` | ElementalFocusFire -LevelEntries-> FireBlastProgression -Grant-> CompositeBlastBuff -Action-> BlizzardBlastFeature (beyond) |
| BlueFlameBlastFeature | 89dfce413170db049b0386fff333e9e1 | prerequisite; 270 | true | `blue-flame-blast` | SecondatyElementalFocusSelection -Option-> SecondaryElementFire -Action-> BlueFlameBlastFeature (beyond) |
| ChargedWaterBlastFeature | 9b7bf2754e2012e4dac135fd6c782fac | prerequisite; 270 | true | `charged-water-blast` | ElementalFocusFire -LevelEntries-> FireBlastProgression -Grant-> CompositeBlastBuff -Action-> ChargedWaterBlastFeature (beyond) |
| ColdBlastFeature | ce625487d909b154c9305e60e4fc7d60 | prerequisite; 270 | true | `cold-blast-feature` | ElementalFocusWater -LevelEntries-> WaterBlastSelection -Option-> ColdBlastProgression -GrantIf-> ColdBlastFeature (vocab) |
| ColdBlastProgression | dbb1159b0e8137c4ea20434a854ae6a8 | chain; - | false | `cold-blast-progression` | on a chain above another carrier (`chains.tsv`) |
| EarthBlastFeature | 7f5f82c1108b961459c9884a0fa0f5c4 | prerequisite; 270 | true | `earth-blast-feature` | ElementalFocusEarth -LevelEntries-> EarthBlastProgression -LevelEntries-> EarthBlastFeature (vocab) |
| EarthBlastProgression | d945ac76fc6a06e44b890252824db30a | chain; - | false | `earth-blast-progression` | on a chain above another carrier (`chains.tsv`) |
| ElectricBlastFeature | c2c28b6f6f000314eb35fff49bb99920 | prerequisite; 270 | true | `electric-blast-feature` | SecondatyElementalFocusSelection -Option-> SecondaryElementAir -Action-> ElectricBlastFeature (beyond) |
| ElectricBlastProgression | ba7767cb03f7f3949ad08bd3ff8a646f | chain; - | false | `electric-blast-progression` | on a chain above another carrier (`chains.tsv`) |
| FireBlastFeature | cbc88c4c166a0ce4a95375a0a721bd01 | prerequisite; 270,271 | true | `fire-blast-feature` | ElementalFocusFire -LevelEntries-> FireBlastProgression -LevelEntries-> FireBlastFeature (vocab) |
| FireBlastProgression | fbed3ca8c0d89124ebb3299ccf68c439 | chain; - | false | `fire-blast-progression` | on a chain above another carrier (`chains.tsv`) |
| IceBlastFeature | a8cc34ca1a5e55a4e8aa5394efe2678e | prerequisite; 270 | true | `ice-blast` | SecondatyElementalFocusSelection -Option-> SecondaryElementWater -Action-> IceBlastFeature (beyond) |
| KineticSharpshooterCapstoneFeature | dc059bdc9229446fb24ab00089a3a7f1 | row; 271 | false | `kinetic-sharpshooter-capstone` | KineticSharpshooterArchetype -AddFeatures-> KineticSharpshooterCapstoneFeature (vocab) |
| MagmaBlastFeature | 408b25c6d9f223b41b935e6ec550e88d | prerequisite; 270 | true | `magma-blast` | ElementalFocusFire -LevelEntries-> FireBlastProgression -Grant-> CompositeBlastBuff -Action-> MagmaBlastFeature (beyond) |
| MentalOverflowEffectFeature | 5441bfcff6f1c4846ae103ca1e500619 | row; 271 | false | `mental-overflow-effect` | PsychokineticistArchetype -AddFeatures-> MentalOverflowProgression -LevelEntries-> MentalOverflowEffectFeature (vocab) |
| MentalOverflowProgression | 42b8ce9dfb087c74db531bad426ad2f3 | chain; - | true | `mental-overflow` | on a chain above another carrier (`chains.tsv`) |
| MetalBlastFeature | ad20bc4e586278c4996d4a81b2448998 | prerequisite; 270 | true | `metal-blast` | SecondatyElementalFocusSelection -Option-> SecondaryElementEarth -Action-> MetalBlastFeature (beyond) |
| MudBlastFeature | 6e33cde96209b5a4f9596a6e509de532 | prerequisite; 270 | true | `mud-blast` | ElementalFocusFire -LevelEntries-> FireBlastProgression -Grant-> CompositeBlastBuff -Action-> MudBlastFeature (beyond) |
| OverwhelmingSoulMentalProwessFeature | c07916b1fe7400c4c97472337b719e64 | row; 271 | false | `overwhelming-soul-mental-prowess` | OverwhelmingSoulArchetype -AddFeatures-> OverwhelmingSoulMentalProwessFeature (vocab) |
| PlasmaBlastFeature | 93d8bc401accfe6489ea3797e316e5d9 | prerequisite; 270 | true | `plasma-blast` | ElementalFocusFire -LevelEntries-> FireBlastProgression -Grant-> CompositeBlastBuff -Action-> PlasmaBlastFeature (beyond) |
| PsychokineticistBurnFeature | 2fa48527ba627254ba9bf4556330a4d4 | row; 271 | false | `psychokineticist-burn` | PsychokineticistArchetype -AddFeatures-> PsychokineticistBurnFeature (vocab) |
| SandstormBlastFeature | af70dce0745f91f4b8aa99a98620e45b | prerequisite; 270 | true | `sandstorm-blast` | ElementalFocusFire -LevelEntries-> FireBlastProgression -Grant-> CompositeBlastBuff -Action-> SandstormBlastFeature (beyond) |
| SteamBlastFeature | 29e4076127a404e4ab1cde7e967e1047 | prerequisite; 270 | true | `steam-blast` | ElementalFocusFire -LevelEntries-> FireBlastProgression -Grant-> CompositeBlastBuff -Action-> SteamBlastFeature (beyond) |
| ThunderstormBlastFeature | 295080cf4691df9438f58ff5ce79ee65 | prerequisite; 270 | true | `thunderstorm-blast` | SecondatyElementalFocusSelection -Option-> SecondaryElementAir -Action-> ThunderstormBlastFeature (beyond) |
| WaterBlastFeature | 560887b5187098b428364de03e628b53 | prerequisite; 270 | true | `water-blast-feature` | ElementalFocusWater -LevelEntries-> WaterBlastSelection -Option-> WaterBlastProgression -GrantIf-> WaterBlastFeature (vocab) |
| WaterBlastProgression | e4027e0fec48e8048a172c6627d4eba9 | chain; - | false | `water-blast-progression` | on a chain above another carrier (`chains.tsv`) |

**misc**

| Carrier | GUID | Source; known under | Hidden | Key | A grant chain (shortest) |
| --- | --- | --- | --- | --- | --- |
| ArcaneRiderMountedCombatFeature | f6021e6d04e14e65a90e416ef08dc521 | row; 283 | false | `arcane-rider-mounted-combat` | ArcaneRiderArchetype -AddFeatures-> ArcaneRiderMountedCombatFeature (vocab) |
| BladeboundTrueBondFeature | fc7d5f4108a84caf84d3574558ede229 | row; 283 | false | `bladebound-true-bond` | BladeboundArchetype -AddFeatures-> BladeboundTrueBondFeature (vocab) |
| CavalierPawChallenge | d67ed8bfadc8466c86a0779fdd18a2cf | row; 273 | false | `cavalier-paw-challenge` | OrderOfThePawCavalierArchetype -AddFeatures-> CavalierPawChallenge (vocab) |
| DragonDiscipleBiteFeatureSecond | 01971351119121d429ecf62c2ab94de3 | row; 287 | false | `dragon-disciple-bite-feature-second` | DragonDiscipleProgression -LevelEntries-> DragonDiscipleBiteFeatureSecond (vocab) |
| DragonDiscipleFormFeatureUltimate | 717699191d106eb46a7d820a872ed24d | row; 287 | false | `dragon-disciple-form-feature-ultimate` | DragonDiscipleProgression -LevelEntries-> DragonDiscipleFormFeatureUltimate (vocab) |
| EldritchScionCantripsFeature | d093b8dec70b5d144acb593a3029d830 | row; 283 | false | `eldritch-scion-cantrips` | EldritchScionArchetype -AddFeatures-> EldritchScionCantripsFeature (vocab) |
| ElvenImmunities | 2483a523984f44944a7cf157b21bf79c | prerequisite; 263 | false | `elven-immunities` | ElfRace -Race-> ElvenImmunities (vocab) |
| HalfOrcFerocity | c99f3405d1ef79049bd90678a666e1d7 | prerequisite; 263 | false | `half-orc-ferocity` | HalfOrcRace -Race-> HalfOrcFerocity (vocab) |
| Hardy | f75d3b6110f04d1409564b9d7647db60 | prerequisite; 263 | false | `hardy` | DwarfRace -Race-> Hardy (vocab) |
| HermitRevelationFadeFromMemory14 | 7bb526fcbbc826048a2a65c1b60fa36d | row; 272 | false | `hermit-revelation-fade-from-memory-14` | HermitArchetype -AddFeatures-> HermitRevelationFadeFromMemory14 (vocab) |
| ImpromptuSneakAttackAdditionalUse | d48de79ddcbc08f42a2b3afef9ae150f | row; 287 | false | `impromptu-sneak-attack-additional-use` | ArcaneTricksterProgression -LevelEntries-> ImpromptuSneakAttackAdditionalUse (vocab) |
| LichChannelNegativeFeature | 295dff380fb8ed743bd5c76a30a49a46 | prerequisite; 263 | false | `lich-channel-negative` | LichFirstAscension -Grant-> LichChannelNegativeFeature (vocab) |
| MadDogDamageReductionMaster | a0d4a3295224b8f4387464a4447c31d5 | row; 269 | false | `mad-dog-damage-reduction-master` | MadDogArchetype -AddFeatures-> MadDogDamageReductionMaster (vocab) |
| MagusCantripsFeature | fa5799fb32844e94e88d4cb3430610ff | row; 283 | false | `magus-cantrips` | MagusProgression -LevelEntries-> MagusCantripsFeature (vocab) |
| MetamorphShapechangerFeature | 606a807687ff26e4782fafaae5d10ce0 | row; 280 | false | `metamorph-shapechanger` | MetamorphArchetype -AddFeatures-> MetamorphShapechangerFeature (vocab) |
| OracleFinalRevelation | 0336dc22538ba5f42b73da4fb3f50849 | row; 272 | false | `oracle-final-revelation` | OracleProgression -LevelEntries-> OracleFinalRevelation (vocab) |
| OracleRevelationChannelFeature | 4bf9a9afadca5304e89bf52f2ac2d236 | prerequisite; 263 | false | `oracle-revelation-channel` | OracleRevelationChannel -Grant-> OracleRevelationChannelFeature (vocab) |
| PaladinDivineMount15Feature | e7bdf20ef01b4284d807e073b8bb8f8c | row; 283 | false | `paladin-divine-mount-15` | ArcaneRiderArchetype -AddFeatures-> ArcaneRiderMountProgression -LevelEntries-> PaladinDivineMount15Feature (vocab) |
| SpellSpecializationProgression | fe9220cdc16e5f444a84d85d5fa8e3d5 | prerequisite; 263 | true | `spell-specialization` | SpellSpecializationFirst -GrantOnApply-> SpellSpecializationProgression (vocab) |
| SpiritTotemLesserFeature | bd63d3ea0e030cf4c930db9ba5eaf931 | prerequisite; 269 | false | `spirit-totem-lesser` |  (-) |
| StudentOfWarKnowYourEnemyDefensiveStanceExtraFeature | 79b6a8e55fddeda44884e253389a68e4 | row; 287 | false | `student-of-war-know-your-enemy-defensive-stance-extra` | StudentOfWarProgression -LevelEntries-> StudentOfWarKnowYourEnemyDefensiveStanceExtraFeature (vocab) |
| StudentOfWarKnowYourEnemyMartialStanceExtraFeature | 712ce66020ddca34d96d3d0a946bb5ca | row; 287 | false | `student-of-war-know-your-enemy-martial-stance-extra` | StudentOfWarProgression -LevelEntries-> StudentOfWarKnowYourEnemyMartialStanceExtraFeature (vocab) |
| WinterWitchFreezingColdUpgradeFeature | b2552d80cdd24e17af922ec8ef669192 | row; 287 | false | `winter-witch-freezing-cold-upgrade` | WinterWitchProgression -LevelEntries-> WinterWitchFreezingColdUpgradeFeature (vocab) |

**monk**

| Carrier | GUID | Source; known under | Hidden | Key | A grant chain (shortest) |
| --- | --- | --- | --- | --- | --- |
| QuarterstaffMasterDamageBonus12 | 169a9c271ad74afb819fe3cb904fb8c7 | row; 279 | false | `quarterstaff-master-damage-bonus-12` | QuarterstaffMasterArchetype -AddFeatures-> QuarterstaffMasterDamageBonus12 (vocab) |
| QuarterstaffMasterDamageBonus20 | 4f0e077d20c3463ebfd18d14fc1c3663 | row; 279 | false | `quarterstaff-master-damage-bonus-20` | QuarterstaffMasterArchetype -AddFeatures-> QuarterstaffMasterDamageBonus20 (vocab) |
| ScaledFistACBonusUnlock | 2a8922e28b3eba54fa7a244f7b05bd9e | row; 279 | false | `scaled-fist-ac-bonus-unlock` | ScaledFistArchetype -AddFeatures-> ScaledFistACBonusUnlock (vocab) |
| ScaledFistPowerFeature | ae98ab7bda409ef4bb39149a212d6732 | prerequisite,row; 264,265,279 | false | `scaled-fist-power` | ScaledFistArchetype -AddFeatures-> ScaledFistPowerFeature (vocab) |
| ScaledFistStunningFistFatigueFeature | abcf396b95e3dbc4686c8547783a719c | row; 279 | false | `scaled-fist-stunning-fist-fatigue` | ScaledFistArchetype -AddFeatures-> ScaledFistStunningFistFatigueFeature (vocab) |
| ScaledFistStunningFistSickenedFeature | e754ea837ee7a6e438ff7ebf6da40b79 | row; 279 | false | `scaled-fist-stunning-fist-sickened` | ScaledFistArchetype -AddFeatures-> ScaledFistStunningFistSickenedFeature (vocab) |
| SenseiMysticPowersMass | a316044187ec61344ba33535f42f6a4d | row; 279 | false | `sensei-mystic-powers-mass` | SenseiArchetype -AddFeatures-> SenseiMysticPowersMass (vocab) |
| SoheiKiStrikeAdamantineUnlock | 707756f8da99563429d869ca17ae76d2 | row; 279 | false | `sohei-ki-strike-adamantine-unlock` | SoheiArchetype -AddFeatures-> SoheiKiStrikeAdamantineUnlock (vocab) |
| SoheiKiStrikeColdIronSilverUnlock | 06256d5184cba5641a0839c310b64b85 | row; 279 | false | `sohei-ki-strike-cold-iron-silver-unlock` | SoheiArchetype -AddFeatures-> SoheiKiStrikeColdIronSilverUnlock (vocab) |
| SoheiKiStrikeLawfulUnlock | c8dd874cf2fa8b244a1bc416e9bb66ad | row; 279 | false | `sohei-ki-strike-lawful-unlock` | SoheiArchetype -AddFeatures-> SoheiKiStrikeLawfulUnlock (vocab) |
| SoheiKiStrikeMagicUnlock | 17f744d48f014924f91534abc4d7ee98 | row; 279 | false | `sohei-ki-strike-magic-unlock` | SoheiArchetype -AddFeatures-> SoheiKiStrikeMagicUnlock (vocab) |
| SoheiMountedCombatFeatFeature | 64f12b8a5149e8e4386a767eaf2b62b4 | row; 279 | false | `sohei-mounted-combat-feat` | SoheiArchetype -AddFeatures-> SoheiMountedCombatFeatFeature (vocab) |
| SoheiUnarmedStrikeFeature | 60e81557bac3a11458657685515c1c16 | row; 279 | false | `sohei-unarmed-strike` | SoheiArchetype -AddFeatures-> SoheiUnarmedStrikeFeature (vocab) |
| StudentOfStoneBonusFeatFeature | 7eb3b215dc34db648a1150355cb6c621 | row; 279 | false | `student-of-stone-bonus-feat` | StudentOfStoneArchetype -AddFeatures-> StudentOfStoneBonusFeatFeature (vocab) |

**mythic**

| Carrier | GUID | Source; known under | Hidden | Key | A grant chain (shortest) |
| --- | --- | --- | --- | --- | --- |
| AeonDemythicationUpgrade1 | 60d4387ce45efa848a65c5a5838171c7 | row; 266 | false | `aeon-demythication-upgrade-1` | AeonProgression -LevelEntries-> AeonDemythicationUpgrade1 (vocab) |
| AeonDemythicationUpgrade2 | 0352a91d533acc64c936a3555d8d2a7b | row; 266 | false | `aeon-demythication-upgrade-2` | AeonProgression -LevelEntries-> AeonDemythicationUpgrade2 (vocab) |
| AeonFifthLevelImmunities | c52b48a922161fe45a258f0214d6501a | chain; - | false | `aeon-fifth-level-immunities` | on a chain above another carrier (`chains.tsv`) |
| AeonGazeFeature | 2689d57a889951241912f4bc7d6bb77e | row; 266 | false | `aeon-gaze` | AeonProgression -LevelEntries-> AeonGazeFeature (vocab) |
| AeonGazeFeatureExtraGaze2 | 7d827b9679f8427cb9c64bb934252261 | row; 266 | false | `aeon-gaze-feature-extra-gaze-2` | AeonProgression -LevelEntries-> AeonGazeFeatureExtraGaze2 (vocab) |
| AeonSeventhLevelImmunities | e4d96713f033cc347b43e941b6f5a743 | chain; - | false | `aeon-seventh-level-immunities` | on a chain above another carrier (`chains.tsv`) |
| AeonTenthLevelImmunities | 711f6abfab877d342af9743a11c8f3aa | row; 266 | false | `aeon-tenth-level-immunities` | AeonProgression -LevelEntries-> AeonTenthLevelImmunities (vocab) |
| CorruptedGoldenDragonFeature | 1e66cfb4f351466fb05636082bce50d6 | prerequisite; 266 | false | `corrupted-golden-dragon` |  (-) |
| DemonRageForcedRageFeature | 2a5d1de842d4c514495a195a808b14c9 | row; 266 | false | `demon-rage-forced-rage` | DemonProgression -LevelEntries-> DemonRageForcedRageFeature (vocab) |
| FakeDivineSparkFeature | bcf3ea648bd94612b8e8f1a69e9026e5 | prerequisite; 265 | false | `fake-divine-spark` | MagicDeceiverWayOfLivingGodProgression -LevelEntries-> FakeDivineSparkFeature (vocab) |
| ImmunityToAbilityDamage | fda40b9ba7644754f97cb51f04759a3e | row; 266 | false | `immunity-to-ability-damage` | AeonProgression -LevelEntries-> AeonFifthLevelImmunities -Grant-> ImmunityToAbilityDamage (vocab) |
| ImmunityToBleed | 52f8ef060a751a247964adae7fcb7e64 | row; 266 | false | `immunity-to-bleed` | FlurryFormFeature -Grant-> SubtypeElemental -Grant-> ImmunityToBleed (vocab) |
| ImmunityToCurseEffects | d64da5fbf9783b946ac7a0e94c9bccc1 | row; 266 | false | `immunity-to-curse-effects` | AeonProgression -LevelEntries-> AeonSeventhLevelImmunities -Grant-> ImmunityToCurseEffects (vocab) |
| ImmunityToEnergyDrain | efe0344bca1290244a277ed5c45d9ff2 | row; 266 | false | `immunity-to-energy-drain` | DragonLevel3Immunities -Grant-> ImmunityToEnergyDrain (vocab) |
| ImmunityToMindAffecting | 3eb606c0564d0814ea01a824dbe42fb0 | row; 266 | false | `immunity-to-mind-affecting` | TranquilWhispererApotheosis -Grant-> ImmunityToMindAffecting (vocab) |
| LifeBondingFriendshipFeature | 5b63a6690ebbaa4459df3e04fe094cbd | prerequisite; 266 | false | `life-bonding-friendship` | LifeBondingFriendshipSelection -Grant-> LifeBondingFriendshipFeature (vocab) |
| LocustCloneLevel08Feature | 9af0119d6e194f98b3ebd6732ac81816 | row; 266 | false | `locust-clone-level-08` | SwarmThatWalksProgression -LevelEntries-> LocustCloneLevel08Feature (vocab) |
| LocustCloneLevel09Feature | a4217bf42d8b41429f83663ef4d44a10 | row; 266 | false | `locust-clone-level-09` | SwarmThatWalksProgression -LevelEntries-> LocustCloneLevel09Feature (vocab) |
| LocustCloneLevel10Feature | 1e648f7a98d84d71bb28cb4bf7dc0865 | row; 266 | false | `locust-clone-level-10` | SwarmThatWalksProgression -LevelEntries-> LocustCloneLevel10Feature (vocab) |
| MythicLichLevel3 | d585ffb00ace4d389190d3cb23cff451 | prerequisite; 266 | true | `mythic-lich-level-3` | LichProgression -LevelEntries-> MythicLichLevel3 (vocab) |

**paladin**

| Carrier | GUID | Source; known under | Hidden | Key | A grant chain (shortest) |
| --- | --- | --- | --- | --- | --- |
| AllIsdarknessLvlUpFeature | a1cf87d341064810b032780ecd8df079 | row; 274 | false | `all-isdarkness-lvl-up` | TorturedCrusader -AddFeatures-> AllIsdarknessLvlUpFeature (vocab) |
| AloneInTheDarkFeatureDescription | ecbfbffc97544ef1892e37514ec6643f | row; 274 | false | `alone-in-the-dark-feature-description` | TorturedCrusader -AddFeatures-> AloneInTheDarkFeatureDescription (vocab) |
| AloneInTheDarkFeatureLevel14AuraOfFaithFeature | 672835d0cf244539aa89bf566a070ffb | row; 274 | false | `alone-in-the-dark-feature-level-14-aura-of-faith` | TorturedCrusader -AddFeatures-> AloneInTheDarkFeatureLevel14AuraOfFaithFeature (vocab) |
| AloneInTheDarkFeatureLevel17AuraOfRightFeature | 23711255f6ca4a9b8f0337ad2ffb2ac6 | row; 274 | false | `alone-in-the-dark-feature-level-17-aura-of-right` | TorturedCrusader -AddFeatures-> AloneInTheDarkFeatureLevel17AuraOfRightFeature (vocab) |
| AloneInTheDarkFeatureLevel3AuraOfCourageFeature | 25a015e8c4724a4699b96a4002a9711b | row; 274 | false | `alone-in-the-dark-feature-level-3-aura-of-courage` | TorturedCrusader -AddFeatures-> AloneInTheDarkFeatureLevel3AuraOfCourageFeature (vocab) |
| AloneInTheDarkFeatureLevel8AuraOfResolveFeature | ab0a392eb6884091845644de3985730c | row; 274 | false | `alone-in-the-dark-feature-level-8-aura-of-resolve` | TorturedCrusader -AddFeatures-> AloneInTheDarkFeatureLevel8AuraOfResolveFeature (vocab) |
| DivineHunterBondPlus2 | c5b169a910918e540a5494a1f24bd6ee | row; 274 | false | `divine-hunter-bond-plus-2` | DivineHunterArchetype -AddFeatures-> DivineHunterBondPlus2 (vocab) |
| DivineHunterBondPlus3 | a3b4881f0510423c82e9fbecebc53833 | row; 274 | false | `divine-hunter-bond-plus-3` | DivineHunterArchetype -AddFeatures-> DivineHunterBondPlus3 (vocab) |
| DivineHunterBondPlus4 | 36a6017145a742c0a0a20eb314b3a884 | row; 274 | false | `divine-hunter-bond-plus-4` | DivineHunterArchetype -AddFeatures-> DivineHunterBondPlus4 (vocab) |
| DivineHunterBondPlus6 | 6752ecab81984f05830764b321355eb0 | row; 274 | false | `divine-hunter-bond-plus-6` | DivineHunterArchetype -AddFeatures-> DivineHunterBondPlus6 (vocab) |
| MartyrAuraOfResolveFeature | 9d079fd1448ca1d4b9e009c5628cc132 | row; 274 | false | `martyr-aura-of-resolve` | MartyrArchetype -AddFeatures-> MartyrAuraOfResolveFeature (vocab) |
| MartyrAuraOfRighteousnessFeature | 1230b414cac49234fbcb00c46ba45503 | row; 274 | false | `martyr-aura-of-righteousness` | MartyrArchetype -AddFeatures-> MartyrAuraOfRighteousnessFeature (vocab) |
| PowerOfFaithTier2MainFeature | a83eff2f92b2e9147a43a7b8accaa46d | row; 274 | false | `power-of-faith-tier-2-main` | WarriorOfTheHolyLightArchetype -AddFeatures-> PowerOfFaithTier2MainFeature (vocab) |
| PowerOfFaithTier3MainFeature | 25c08230cdac4b14995b6c8d80bdf943 | row; 274 | false | `power-of-faith-tier-3-main` | WarriorOfTheHolyLightArchetype -AddFeatures-> PowerOfFaithTier3MainFeature (vocab) |
| PowerOfFaithTier4MainFeature | 5d5f11207836a374583ffffbd4d86c9c | row; 274 | false | `power-of-faith-tier-4-main` | WarriorOfTheHolyLightArchetype -AddFeatures-> PowerOfFaithTier4MainFeature (vocab) |
| SmiteEvilAdditionalUse | 0f5c99ffb9c084545bbbe960b825d137 | row; 274 | false | `smite-evil-additional-use` | PaladinProgression -LevelEntries-> SmiteEvilAdditionalUse (vocab) |
| StonebaneFeature | 0c3101a8ecac471ba9d98d0a02172dd9 | row; 274 | false | `stonebane` | StonelordArchetype -AddFeatures-> StonebaneFeature (vocab) |

**shaman**

| Carrier | GUID | Source; known under | Hidden | Key | A grant chain (shortest) |
| --- | --- | --- | --- | --- | --- |
| ShadowShamanStealthFeature | c4135465768c4119b2c7603c1d477ef3 | row; 278 | false | `shadow-shaman-stealth` | ShadowShamanArchetype -AddFeatures-> ShadowShamanStealthFeature (vocab) |
| ShamanBattleSpiritWanderingFeature | 82ec7a49f3ab65a44bbf4177dfdfe469 | prerequisite; 278 | false | `shaman-battle-spirit-wandering` |  (-) |
| ShamanBonesSpiritWanderingFeature | a3862152ae6010445bc25915ac58fc8e | prerequisite; 278 | false | `shaman-bones-spirit-wandering` |  (-) |
| ShamanFlameSpiritWanderingFeature | d12c230082825b945b92574df6f3caf1 | prerequisite; 278 | false | `shaman-flame-spirit-wandering` |  (-) |
| ShamanFrostSpiritWanderingFeature | 6aeab860502ca2643affcbf1a83187e0 | prerequisite; 278 | false | `shaman-frost-spirit-wandering` |  (-) |
| ShamanLifeSpiritChannelEnergyFeature | b8ec9dccc0e7ef74fb4072b0679c2aec | prerequisite; 263 | false | `shaman-life-spirit-channel-energy` | ShamanLifeSpiritBaseFeature -Grant-> ShamanLifeSpiritChannelEnergyFeature (vocab) |
| ShamanNatureSpiritWanderingFeature | 54943a544ec1b31438366aaf73e5f310 | prerequisite; 278 | false | `shaman-nature-spirit-wandering-feature` |  (-) |
| ShamanNatureSpiritWanderingProgression | f3e19b0d6d82e2a4d98957af591f5d36 | prerequisite; 278 | false | `shaman-nature-spirit-wandering-progression` |  (-) |
| ShamanStonesSpiritWanderingFeature | e6468a6fac9e1074897b2487a0659e96 | prerequisite; 278 | false | `shaman-stones-spirit-wandering` |  (-) |
| ShamanWavesSpiritWanderingFeature | 9d7c85f90f9d34845bd80ac09b34938c | prerequisite; 278 | false | `shaman-waves-spirit-wandering` |  (-) |
| ShamanWeaponPlus2 | ecdee76fb34d45f40988d06fe524129e | row; 278 | false | `shaman-weapon-plus-2` | SpiritHunterArchetype -AddFeatures-> ShamanWeaponPlus2 (vocab) |
| ShamanWeaponPlus3 | ba502d629f572d447b603a04ce536d4a | row; 278 | false | `shaman-weapon-plus-3` | SpiritHunterArchetype -AddFeatures-> ShamanWeaponPlus3 (vocab) |
| ShamanWeaponPlus4 | c47b2861f8e84504bbb25a413de98923 | row; 278 | false | `shaman-weapon-plus-4` | SpiritHunterArchetype -AddFeatures-> ShamanWeaponPlus4 (vocab) |
| ShamanWeaponPlus5 | 17527cfc59388bc4fbb959c5408db459 | row; 278 | false | `shaman-weapon-plus-5` | SpiritHunterArchetype -AddFeatures-> ShamanWeaponPlus5 (vocab) |
| ShamanWindSpiritWanderingFeature | cc05b2fff3e20c64e968f490a5160c08 | prerequisite; 278 | false | `shaman-wind-spirit-wandering` |  (-) |
| UnswornShamanWanderingSpiritFeature 1 | 4439ae8c6360cd544b69d9d80ed42d5e | row; 278 | false | `unsworn-shaman-wandering-spirit-feature-1` | UnswornShamanArchetype -AddFeatures-> UnswornShamanWanderingSpiritFeature 1 (vocab) |

**warpriest**

| Carrier | GUID | Source; known under | Hidden | Key | A grant chain (shortest) |
| --- | --- | --- | --- | --- | --- |
| MantisZealotSabreFightingFocus | 86892c35059f4c918e4ed5950f402833 | row; 268 | false | `mantis-zealot-sabre-fighting-focus` | MantisZealotArchetype -AddFeatures-> MantisZealotSabreFightingFocus (vocab) |
| ProclaimerCleanserOfEvil10FeetFeatureLimit | f236ddd506258794c9f1a18d0a1548d9 | row; 268 | false | `proclaimer-cleanser-of-evil-10-feet-feature-limit` | ProclaimerArchetype -AddFeatures-> ProclaimerCleanserOfEvil10FeetFeatureLimit (vocab) |
| ProclaimerCleanserOfEvil15FeetFeatureLimit | f0f07c512fa3bc44f8593154bf6b2a11 | row; 268 | false | `proclaimer-cleanser-of-evil-15-feet-feature-limit` | ProclaimerArchetype -AddFeatures-> ProclaimerCleanserOfEvil15FeetFeatureLimit (vocab) |
| ProclaimerCleanserOfEvil20FeetFeatureLimit | 0bb21b9818eac054a979be0fa82aade4 | row; 268 | false | `proclaimer-cleanser-of-evil-20-feet-feature-limit` | ProclaimerArchetype -AddFeatures-> ProclaimerCleanserOfEvil20FeetFeatureLimit (vocab) |
| ProclaimerZoneOfSanctification10FeetFeatureLimit | 90c3821c0fbc4644b9855580b16052e2 | row; 268 | false | `proclaimer-zone-of-sanctification-10-feet-feature-limit` | ProclaimerArchetype -AddFeatures-> ProclaimerZoneOfSanctification10FeetFeatureLimit (vocab) |
| ProclaimerZoneOfSanctification15FeetFeatureLimit | 723a3c89c44901c4191a92f6e1693a8b | row; 268 | false | `proclaimer-zone-of-sanctification-15-feet-feature-limit` | ProclaimerArchetype -AddFeatures-> ProclaimerZoneOfSanctification15FeetFeatureLimit (vocab) |
| ProclaimerZoneOfSanctification20FeetFeatureLimit | 25d0c15a1b8f94d4bb137fda7a3e293d | row; 268 | false | `proclaimer-zone-of-sanctification-20-feet-feature-limit` | ProclaimerArchetype -AddFeatures-> ProclaimerZoneOfSanctification20FeetFeatureLimit (vocab) |
| SacredArmorEnchantPlus2 | ec327c67f6a6b2f49a8ca218466a8818 | row; 268 | false | `sacred-armor-enchant-plus-2` | WarpriestProgression -LevelEntries-> SacredArmorEnchantPlus2 (vocab) |
| SacredArmorEnchantPlus3 | bd292463fa74d664086f0a3e4e425c47 | row; 268 | false | `sacred-armor-enchant-plus-3` | WarpriestProgression -LevelEntries-> SacredArmorEnchantPlus3 (vocab) |
| SacredArmorEnchantPlus4 | ee65ff63443ce42488515db6a43fee5e | row; 268 | false | `sacred-armor-enchant-plus-4` | WarpriestProgression -LevelEntries-> SacredArmorEnchantPlus4 (vocab) |
| SacredArmorEnchantPlus5 | 1e560784dfcb00f4da1415bbad3226c3 | row; 268 | false | `sacred-armor-enchant-plus-5` | WarpriestProgression -LevelEntries-> SacredArmorEnchantPlus5 (vocab) |
| SacredArmorShieldEnchantPlus2 | 9051a98e60cca2742ad5a80ab9c9a1f3 | row; 268 | false | `sacred-armor-shield-enchant-plus-2` | ShieldbearerArchetype -AddFeatures-> SacredArmorShieldEnchantPlus2 (vocab) |
| SacredArmorShieldEnchantPlus3 | 0f14e7ea4ae157248b7e6b29da64e0d7 | row; 268 | false | `sacred-armor-shield-enchant-plus-3` | ShieldbearerArchetype -AddFeatures-> SacredArmorShieldEnchantPlus3 (vocab) |
| SacredArmorShieldEnchantPlus4 | c4c8dd0b8c017044b851bf56c5376864 | row; 268 | false | `sacred-armor-shield-enchant-plus-4` | ShieldbearerArchetype -AddFeatures-> SacredArmorShieldEnchantPlus4 (vocab) |
| SacredArmorShieldEnchantPlus5 | a71f5193e7d0bb041a88dedc5737c07b | row; 268 | false | `sacred-armor-shield-enchant-plus-5` | ShieldbearerArchetype -AddFeatures-> SacredArmorShieldEnchantPlus5 (vocab) |
| SacredWeaponEnchantFeature | 6db4ccce643b7b2468717effb303afcf | row; 268 | false | `sacred-weapon-enchant` | WarpriestProgression -LevelEntries-> SacredWeaponEnchantFeature (vocab) |
| SacredWeaponEnchantPlus2 | 4ee3e74f79d5e4f47b7a0091984b2518 | row; 268 | false | `sacred-weapon-enchant-plus-2` | WarpriestProgression -LevelEntries-> SacredWeaponEnchantPlus2 (vocab) |
| SacredWeaponEnchantPlus3 | bc1a85932b7d82f49852d6ded3049837 | row; 268 | false | `sacred-weapon-enchant-plus-3` | WarpriestProgression -LevelEntries-> SacredWeaponEnchantPlus3 (vocab) |
| SacredWeaponEnchantPlus4 | b7295813f02c8be45a6e1fce541584df | row; 268 | false | `sacred-weapon-enchant-plus-4` | WarpriestProgression -LevelEntries-> SacredWeaponEnchantPlus4 (vocab) |
| SacredWeaponEnchantPlus5 | 5fffe6a74adb1b742a8ebf64b8bb90fb | row; 268 | false | `sacred-weapon-enchant-plus-5` | WarpriestProgression -LevelEntries-> SacredWeaponEnchantPlus5 (vocab) |
