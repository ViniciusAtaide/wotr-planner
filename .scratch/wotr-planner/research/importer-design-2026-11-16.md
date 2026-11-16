# Ticket 293, the importer: design (2026-11-16)

Research only: no repo edit, no build. Read at `main` 1387bbeb (after 301; 302–305 open, designed against their tickets
and `<scratchpad>/game/30{1,2,3}/decisions.md`). Game: BubblePrintsData `2.7.0x_0` (the check's pinned dump, 236,660
blueprints); code WrathDecompiled 6ce4a6b. Every blueprint below is cited by asset name and GUID; no game text.

**Evidence files beside this one** (`<scratchpad>/game/293/`):
- `run.txt`: a fresh game-check run (lane-tests' debug build, `--data` the main checkout's data = main): 4,413
  differences in 208 groups, all known, exit 0.
- `scripts/report.py` (the report as records), `scripts/planner.py` (the data's key → blueprint), `scripts/splits.py`
  (each "selection the game keeps separate" resolved to its GUID through the class or archetype that grants it),
  `scripts/lists.py` (the game's list of each split against the planner's), `scripts/corpus_removed.py` (corpus Builds
  holding a pick the game's list for that slot lacks, from 301's form-proof record `<scratchpad>/game/301/after/`,
  which is main's data), `corpus_removed.txt`, `splits.json`.
- They read `<scratchpad>/game/vocab/db.pkl` (the design's 34,225 blueprints) with `vocab/bp.py`.

---

## 0. What 293 is, in one paragraph

A `--write` mode of `game-check` that, for every entry naming a blueprint, computes the game's value of each field the
check compares (through the same reader and the same GUID→key mapping the check uses), writes it into the entry's own
file by splicing only that field's text, and logs every change. What the planner's model can't say is left as it is,
reported, and becomes a gap ticket. The run is by hand, deterministic and idempotent. It lands in five stages (§4),
each its own branch, merge and review, because the areas change values on thousands of entries and depend on each
other in one order.

---

## 1. Where it lives and how it writes

### 1.1 A `--write` mode of `game-check` (the ticket's recommendation: taken)

`tools/game-check` already has the reader (`binz.rs`, `dump.rs`, `lookup.rs`), the planner side (`planner.rs`: every
entry by GUID) and one comparison per area (`areas/*.rs`). Today each area builds a *normal form* of both sides and
diffs them; the game side never exists in the planner's model. The importer adds that model-side value and makes the
check compare through it, so the two can never disagree on what a field means:

```
src/game/          the game's value of each compared field, in the planner's model (keys, not GUIDs)
  resolve.rs       GUID -> key by expected kind (feature, option, feat, selection, class, archetype, …), and the
                   keys planned for blueprints with no entry yet (S2/S3)
  tables.rs        class/archetype: skill points, class skills, BAB and save tables
  requires.rs      Prerequisite* -> Vec<Component::Requires> (the vocabulary of 295/296), plus what has no condition
  selections.rs    a selection's header and option list; the selection a row grants, by GUID
  rows.rs          a class's, archetype's or path's rows (archetypes as ProgressionData.CalculateLevelEntries)
  gear.rs          an item's GearStat components
  Target { value, unexpressed: Vec<Gap> }   each Gap names what was left out and its gap ticket
src/areas/*.rs     compare = normal form of the planner's field vs normal form of Target.value; finding kinds and
                   texts unchanged, so known.txt keeps matching; each Target.unexpressed item is the finding's cause
src/write/
  mod.rs           plan (targets for the chosen areas) -> change set -> guard -> apply -> read back
  span.rs          field_span / ron_text / list style, lifted from the 296/297/301 rewrite crates
  guard.rs         owned fields per kind; planner/ and blueprint-less entries; the user's facts (§3)
  log.rs           changes.tsv
```

- `game-check --write <area>[,<area>…] [--dry-run] [--log <file>] [--data <dir>] [--binz <file>]`; areas are the
  report's (`class-tables`, `items`, `option-lists`, `prerequisites`, `rows`, `mythic-ranks`) plus `carriers` (S3).
  Exit 0 when it wrote, or had nothing to write; 2 when a guard refuses (nothing written) or the dump is refused.
- The writer reads the files with `seeds::read_game_data_files` (the unchecked model: `Requires` still in
  `components`), never through `GameData::checked` (which compiles `Requires` into `prerequisite`, 296). After the run
  it loads the whole tree through `seeds::check_game_data_files`: `GameData::check` must pass, or the run restores
  every file it touched and exits 2.
- **The prerequisites area moves to the components** (S4): it compares the planner's `Requires` (uncompiled) with
  `requires.rs`'s, as sorted multisets per entry (vocabulary design §5). The hand normal form in
  `areas/prerequisites.rs` stays only for planner entries' hand `prerequisite` trees.

### 1.2 How it writes RON

The data's text is the planner's: keys, names, summaries, guide links, `aliases`, comments, field order and line
style. The writer never re-serialises a file. Per entry:

1. **Owned fields only.** A table per kind (`guard.rs`) lists what the importer may write; anything else differing
   between the file and its target is a bug, refused before anything is written:

   | Kind | Owned (written) |
   | --- | --- |
   | `classes/`, `prestige/` | `skill_points`, `bab`, `fort`, `reflex`, `will`, `save_exceptions` (only emptied), `class_skills`, `levels` (each row's `features`, `feats`, `feats_for`, `selections`), `components`' `Requires` |
   | `archetypes/` | `skill_points`, `class_skills`, `fort`/`reflex`/`will`, `bab` (new field, S1), `levels` (replacement rows, §2.5), `components`' `Requires` |
   | `selections/` | `options`, `shared_options`, `feats`, `ignores_prerequisites`, and the header fields 303/304 settle (`ranks`, mode, the progression header) |
   | `feats/`, `options/`, `mythic/abilities/`, `mythic/feats/` | `components`' `Requires`; 303's header |
   | `mythic/paths/` | `ranks` rows' `features` and `selections` |
   | `gear/` | `components` (`GearStat`) |
   | new files (S2, S3) | the whole entry, from a template per kind (§2.7) |

   **Never:** `key`, `name`, `summary`, `guide`, existing `aliases`, `interactions.ron`, `notes/`, `planner/` (any
   file), `MythicPath::switch`, `Progression::mythic_ascensions`, `daily_choice`, `option_row`, `level_from`,
   `spontaneous`, `casting_ability`, `advances`, `counts_as`, `spell_levels`, the `Story` leaves, any comment. An entry
   with no `blueprint` is never written.
2. **Splice, don't serialise.** `span.rs::field_span` (296's: a top-level `name: value` outside strings and comments,
   depth-aware) finds the field; the new value is written with `ron_text` (one line, `implicit_some`, 296's
   `PrettyConfig`), as a one-line list in a one-line file and one element a line (the field's indent plus four) in a
   pretty one (296's `components_text`, generalised). A missing owned field is inserted after the last field that
   precedes it in the struct's declared order. Text outside the replaced span is byte-identical, so comments survive.
   A span that *contains* a comment is not replaced: the run lists it and refuses (one file today has a comment inside
   an owned field: `archetypes/fighter/titan-fighter.ron` line 12, inside `levels`).
3. **Semantic no-op first (idempotence).** The target and the file's current value are compared as the model
   compares them (sets as sets: "order means nothing anywhere in the data", README Layout; rows by level; components
   as a multiset). Equal: the file is not touched, whatever order a person wrote it in. Different: the field is
   written in canonical order (keys sorted, rows by level, components `Requires` first then by kind, the order 296/297
   wrote). So a second run writes nothing, and a person's ordering of an unchanged field is kept.
4. **Read back.** Every written file is parsed again with the `rules` types; the owned fields must equal the target
   and every other field must equal the file before (asserted per entry, as 296's and 301's rewrites did).
5. **Determinism.** Entries are visited in key order; targets come only from the pinned dump and the data; new keys
   come only from the blueprint (§2.7). Two runs on two copies of one tree give byte-identical files and logs (a fixture
   test, and once on real data per stage).
6. **The change log** (`--log`): one TSV row per changed field: entry, field, old (normal form), new, the report
   finding(s) it removes, `user-fact` flag. It is the review's input (§4.3), kept in the scratchpad, not the repo.

### 1.3 Tests (fixture, in the workspace's test run)

The check's fixture (`tests/fixture/`, its own small dump and `planner.ron`) gains a write fixture: one entry per
owned field kind, one per refusal (a planner field that would change, a comment inside a span, a user fact without an
approval, a `planner/` file, an ambiguous GUID). Tests: the run writes the expected files; the check reports nothing
after it; a second run writes nothing; two runs on two copies are byte-identical; each refusal exits 2 and writes
nothing.

---

## 2. Per area: today's count, what the importer writes, what it can't

Counts are the fresh run's findings (`run.txt`); the check compares 4,050 entries with a blueprint (of 4,205). By
`known.txt` section: 293's own 638, 294's 205, the rest 261–291 and 255–260.

| Area | Entries compared / differ | Findings |
| --- | --- | --- |
| prerequisites | 1,392 / 535 | 1,569 |
| class tables | 235 / 56 | 63 |
| progression rows | 235 / 202 | 1,753 |
| option lists | 202 / 106 | 981 |
| mythic ranks | 10 / 7 | 39 |
| items | 82 / 6 | 8 |
| **total** | | **4,413** |

### 2.1 Class tables (63) — stage S1

| Kind | n | Writes |
| --- | --- | --- |
| Will / Reflex / Fortitude differ | 33 / 11 / 3 | the save's table from the blueprint's `m_WillSave` etc. Most are guide `save_exceptions` the game doesn't have (`PaladinClass` bfa11238e7ae3544bbeb4d0b92e897ec: `SavesHigh`, +12 at 20, the data's exception +20; `HunterClass` 34ecd1b5e1b90b9498795791b0855239 Will `SavesLow`, exception +3 at 7; `ShifterClass` a406d6ebea5c46bba3160246be03e96f Reflex `SavesHigh`; `WarpriestClass` 30b5e47d47a0e37438cc5a80c96cfb99): the exception goes. Archetype overrides (`DarkElementalistArchetype` f12f18ae8842425489d29f302e69134c, `TraditionalMonk` 8f3242d209994884bbb6c0999660f4df, `PriestOfBalanceArchetype` a4560e3fb5d247d68fb1a2738fcc0855) set `fort`/`reflex`/`will`. `StalwartDefenderClass` d5917881586ff1d4d96d5b7cebda9464 is `SavesPrestigeHigh`/`Low`/`High`, the data Poor/Good/Poor. |
| BAB differs | 1 | `FeyspeakerArchetype` da69747aa3dd0044ebff5f3d701cdde3 is `BABLow` on a druid: archetypes have no `bab` field ("Archetypes never change it", `game_data.rs` `Class::bab`). S1 adds `Archetype::bab: Option<BabProgression>` (a small form addition with a rules test), then writes it. |
| skill points (the user's facts) | 2 | `ArcaneTricksterClass` 9c935a076d4fe4d4999fd48d853e3cf3 3 (user, 261); `DefenderOfTheHearthArchetype` 6c96dc1f5573487399336cd351ef8f2c 3 (user, 262). |
| class skills (the user's facts) | 13 | the 13 rows of 262 (user: "Apply the game's"). The game's list maps `StatType` to skills (`areas::skill_stat`); `StalwartDefenderClass` lists `CheckIntimidate`, which no planner skill is: written without it, the rest of its finding stays (gap G4). 262's caution: a skill the archetype's own feature gives back (the Empyreal's, the Sylvan Sorcerer's bloodline class skill, the Sohei's) must not vanish from the sheet: S1 writes that feature's `ClassSkill` component (295's kind) where the game has `AddClassSkill` on it. |

The five game tables met (`BABFull`/`Medium`/`Low`, `SavesHigh`/`Low`/`PrestigeHigh`/`PrestigeLow`, `ZeroProgression`
for pets only) map onto today's enums if a base class may name a prestige table and a save may name a BAB table (one
case: a `FortitudeSave` of `BABLow`, a pet's). Writable: 62 of 63; 1 left (G4).

### 2.2 Items (8) — stage S1

- Writes `GearStat` value and descriptor: `SarkorisDirgeItem` cffc4037c224475ca2a0c0b6bfda8688 and
  `HalfplateOfVigorItem` 053873b9eca18854696dfc0e49d260fe are `Enhancement`, not untyped (2); `BeltOfVeneratedChampionItem`
  87c5f3e1b5a01d0448d550b727f45cb4 adds Fortitude +2 (1); `TokenOfHonor` 905ec00744cd4627b0e00f187e215d4b has no
  always-on Charisma or Persuasion bonus (2, removed).
- Can't: 3 "slot differs" (`DLC3_BrokenTricksterGlassesArtifactItem` 692b59ae0433471aa675b5b1b80db863 and `RazmirMask`
  9338510a8d5c4b43b1284470b706128d are Glasses, `TokenOfHonor` a usable): the planner has no Glasses place or usable
  item: **288's model part** (open), not a new ticket.
- Not a finding but a user fact: the **Headband of Perfection +8** (`HeadbandOfPerfection8`
  f94d69b19fe07ca4a8ade7993a1a7510) is missing from `gear/` (the Belt, `BeltOfPerfection8` dc2f2613634c2f947b306fa3379232c2,
  is `gear/belt-of-physical-perfection-8.ron`). The check compares only entries that exist, so no finding shows it. S1's
  builder adds the entry by hand (user, 288), its `GearStat` then written by the run.

### 2.3 Selection option lists (981) and the per-class splits — stage S2

| Kind | n | Writes |
| --- | --- | --- |
| in game only | 667 | 346 are list membership of an entry that exists (197 feats or mythic picks, 149 options): written. 316 name a blueprint no entry maps (143 features, 128 progressions, 37 selections, 7 spellbook replacements, 1 parametrised): S2 creates the pickable ones as entries (§2.7); 57 of the progressions are 294's domains (left to 294); the 7 `BlueprintFeatureReplaceSpellbook` wait on G6. |
| in planner only | 296 | the planner offers what the game's list lacks: written (removed). Many disappear instead through a split or a remap (below), not as a removal. |
| a choice the game doesn't offer | 11 | the planner makes a selection of a blueprint that is a plain `BlueprintFeature` (`JudgmentFeature` 981def910b98200499c0c8f85a78bde8, `StudentOfWarKnowYourEnemyFeature` 846c2094392076a4f9f3c84f0ecfe180, `Geomancy` 30493b3bd627435aa6b1cf3011dc762c, `ImitationFeature` 35336c21b12f40ef9bf0ef7207107f4b, `ShifterDragonFormFeatureFinal` c676ba5eb744492583d989244c81f127, `FeralChampionBlessing` 6c52c34b462be9047b83ae7f2a3b5822, `ArcanistGreaterExploitsFeature` c7536b93f17c70d4fa3a8cf9aa76bfb7, …): can't, a model question (G5). Greater Exploits dissolves here: its six exploits join the exploit lists (260). |
| ignores prerequisites or not | 7 | the header's `IgnorePrerequisites`: written (Disenchanter ×3, `DevilbanePriestTeamworkFeatSelection` cf2ca457ffb585a4995fd79441167a72, `MasterOfAllSkillFocus` f2d2c1702d8a4cc6adfcbd4ebff8eee4, `SwordSaintChosenWeaponSelection` 3130e3083845b7b4fa2ac5b6f1a1ae6e, `Adaptability` 26a668c5a8c22354bac67bcd42e09a3f). |

**The splits, identified by what grants them** (298's correction). The rows area pairs "a game selection and a
planner selection of the same group at one level" and reports it (193 row findings + 6 mythic); but it pairs a grant of
a *feat* selection as "a pick of its group" on both sides (`areas/rows.rs`, `FEAT_POOLS`) and reports nothing. So the
universe is not the report's: `scripts/splits.py` walks every mapped class's progression `LevelEntries`, every
archetype's `AddFeatures` and every mapped progression, and lists each granted visible selection no entry maps: **73**.
Leaving out the two the rules build (`MythicAbilitySelection` ba0e5a900b775be4a99702f1ed08914d,
`MythicFeatSelection` 9ee0f6745f555484299b0a1563b99d81), 294's `SecondDomainsSelection` 43281c3d7fe18cc4d91928395837cd1e
and the pets' mounts (174), S2 writes about 65 selections. The ones that change offers or links:

| Game selection (grantor) | Today | Game list vs today |
| --- | --- | --- |
| `BardTalentSelection` **94e2cd84bf3a8e04f8609fe502892f4f** (`BardProgression` 8127f5ff40f5b484b8be98609358b9d2; the other blueprint of that name, 40f85fbe8cc35ef4fa96c66e06eeafe8, only units reference) | `rogue/rogue-talents` (38) | 24: **14 removed** (hunters-surprise, jump-up, mystery-seeker, the three skill-masteries, slippery-mind, survivalist, trailblazer, unbalancing-trick, underhanded-trick, weakening-wound, weapon-snatcher, wearying-strike) |
| `SkaldTalentSelection` d2a8fde8985691045b90e1ec57e3cc57 (`SkaldClass`) | `rogue/rogue-talents` | 21: **17 removed** (the bard's 14 less weakening-wound, plus emboldening-strike, petrifying-strike, uncanny-dodge, improved-uncanny-dodge) |
| `ProvocateurTalentSelection` aa88cb5263a54791b8d1e471bc192ad4 (`ProvocateurArchetype`, the Inciter) | `rogue/rogue-talents` | 24: **14 removed** |
| `DarkLurkerRogueTalentSelection` 1edbb68422c046f98a86f4726f6721b2 (`DarkLurkerArchetype`) | `rogue/rogue-talents` (inherited rows) | 39: none removed; adds `Evasion` 576933720c440aa4d8d42b0c54b77e80 (a feature today: needs an option entry) |
| `SlayerTalentSelection2` 04430ad24988baa4daa0bcd4f1c7d118 | `slayer/slayer-talents` (41) | 37: **8 removed** (emboldening-strike, intimidating-prowess, petrifying-strike, skill-focus, weapon-focus, weapon-snatcher, combat-style-feat-6, -10); adds uncanny-dodge and improved-uncanny-dodge (user, 276) |
| `SlayerTalentSelection6` 43d1b15873e926848be2abf0ea3ad9a8 (`SlayerClass`, `SanctifiedSlayerArchetype`, `BloodseekerArchetype`) | `slayer/slayer-talents` | 38: **7 removed**; same two added |
| `SlayerTalentSelection10` 913b9cf25c9536949b43a2651b7ffb66 | `slayer/slayer-talents` | 39: **6 removed**; same two added |
| `ExtraSlayerTalentSelection` 53f7237f5b1447bb851ba68045a00e41 (the feat) | first list reached | 41: **4 removed** (intimidating-prowess, skill-focus, weapon-focus, weapon-snatcher) |
| `RagePowerSelection` 28710502f46848d48b3f0d6132817c4e | `barbarian/rage-power` (38) | 33: **5 removed**, the Instinctual Warrior's powers |
| `SkaldRagePowerSelection` 2476514e31791394fa140f1a07941c96 | `barbarian/rage-power` | 33: **5 removed**, the same |
| `InstinctualWarriorRagePowerSelection` 609f0e5336084442a0dafa3abd4d31c5 | `barbarian/rage-power` | 38: none removed (it is the planner's list) |
| `ExtraRagePowerSelection` 0c7f01fbbe687bb4baff8195cb02fe6a, `ExtraRogueTalentSelection` 14d9089df87a43b696fa9451ca2f0a12, `ExtraDiscoverySelection` 537965879fc24ad3948aaffa7a1a3a66, `ExtraWitchHexSelection` b6054088b4ab4be286724127cbf48b35 | 302's lists | none removed (Extra Rogue Talent adds Evasion) |
| `ExploiterExploitSelection` 2ba8a0040e0149e9ae9bfcb01a8ff01d, `ExtraArcanistExploitSelection` 0a7065e13f4449fa81bfb33199bf7f6a | `arcanist/arcanist-exploits` | none removed; add the six greater exploits (260) |
| `HexcrafterMagusHexMagusSelection` a18b8c3d6251d8641a8094e5c2a7bc78 | `witch/hex` | none removed |
| `SorcererBonusFeat` d6dd06f454b34014ab0903cb1ed2ade3 (sorcerer 1), `SeekerFeatSelection` c6b609279cc3174478624182ac1ad812 | `magic-feats` (35) | 21 and 26 (user, 281). **No finding shows this**: the rows pair it by group. |
| `MagusFeatSelection` 66befe7b24c42dd458952e3c47c93563 (181), `ArcaneRiderFeatSelection` 8e627812dc034b9db12fa396fdc9ec75 (182), `CavalierBonusFeatSelection` dd17090d14958ef48ba601688b611970 (146), `GendarmeFeatSelection` ef1cd58e0b7fc7f45baedb09407a1cd1 (155), `WarpriestFeatSelection` 303fd456ddb14437946e344bad9a893b, `EldritchKnightFeatSelection` da03141df23f3fe45b0c7c323a8e5a0e, `StudentOfWarCombatFeatSelection` 78fffe8e5d5bc574a9fd5efbbb364a03, `SelfSuffisientBonusFeats` db3c26e791cb4b70a82203b28563da02, `SkaldFeatSelection` 0a1999535b4f77b4d89f689a385e5ec9, `CavalierTacticianFeatSelection` 7bc55b5e381358c45b42153b8b2603a6, the Vanguard's `TeamworkFeat` 90b882830b3988446ae681c6596460cc (a second blueprint of that name besides 301's d87e2f6a9278ac04caeb0f93eff95fcb), `WarDomainGreaterFeatSelection` 79c6421dbdb028c4fa0c31b8eea95f16, `GriffonheartShifterFeatSelection` f1add10c87fa4563ad5f71779eecde19 | `feats`, `combat-feats`, `teamwork-feats` | the class's own lists. The planner's `mounted-combat-feats` names `CavalierBonusFeatSelection` (301) with the 5 mounted-combat feats; the game's is the cavalier's 146-feat bonus list, so the 141 "in game only" findings there are this split, not 141 offers. |
| `ThirdElementalFocusSelection` e2c1718828fc843479f18ab4d75ded86, `SecondatyElementalFocusSelection` 4204bc10b3d5db440b1f52f0c375848b | `kineticist/expanded-element` naming the element options | a remap, not a removal: the game's options are `SecondaryElement*` and `ThirdElement*` progressions (`ThirdElementAir` 651570c873e22b84f893f146ce2de502 … with `PrerequisiteNoFeature` of the matching secondary): new options with `aliases` naming the element keys. That `HasNot` makes `once_per_option` unnecessary there (302's Q5). |
| `SecondBlessingSelection` b7ce4a67287cda746a59b31c042305cf; `AeonGazeFirstSelection` 7e468a5266235454e9dcefee315ee6d5 / `Second` 7d498e034d18ca94baea19d7edee7403 / `Third` 0bec49f67ecb49a5826fcfefb9408a35; `NineTailedHeirBloodlineSelection` 7c813fb495d74246918a690ba86f9c86; `OrderOfThePawMountSelection` 7b8b6474cc374a23830341d2a40eecaf (pets: 174); `ElementaWitchEnergySelection` 9ec1bf2087874390a9d9be62884a0027 | one shared selection | per tier/archetype selections; the options keep their keys |

**Rules for a split** (applies to all ~65):
- The selection's key: the grantor's key plus the shared selection's last segment (`bard/rogue-talents`,
  `skald/rogue-talents`, `skald/inciter/rogue-talents`, `rogue/dark-lurker/rogue-talents`, `slayer/slayer-talents-6`,
  `-10`, `skald/rage-power`, `barbarian/instinctual-warrior/rage-power`, `magus/bonus-feats`, …); the shared one keeps
  its key and takes its own blueprint's list. Name, summary and guide link are copied from the shared selection (the
  planner's words), never from the game.
- **Options keep their keys.** A selection's options are `shared_options` of the copies; an option is defined in the
  selection whose game list holds it, and since 244 a key says nothing of its owner, so moving where an option is
  defined (the five Instinctual Warrior powers move from `barbarian/rage-power` to the IW copy) changes no key and
  needs no alias.
- **An alias only where a key changes**, read by 298's slot-aware migration (`Engine::open_link`, `LinkKeys::heirs`):
  - entries holding several blueprints become one entry per blueprint: Advanced Weapon Training
    (`AdvancedWeaponTraining2`…`4`), Extra Channel's six per-class feats (`ExtraChannelOracle`, `…Shaman`,
    `…Hospitaler`, `…HexChanneler`, `…EmpyrealSorcerer`), Extra Hex (Shaman) (`ExtraShamanHexSelection`
    d0b4c8245d504b8c9c6d3fccc1f8c5b6, beside `ExtraWitchHexSelection`), Extra Discovery (Vivisectionist)
    (`ExtraVivsectionistDiscoverySelection` 10287e7b8cee479e82ea88bd6d2d4dae, new), Magical Tail (`MagicalTail1`…`8`;
    303's planner `ranks` go), Abundant Smite (`AbundantSmiteChaos`, …), the kineticist element feat lists. The entry
    most like the old one keeps its key; each other gets `aliases: [<old key>]`. Feats have `aliases` since 298.
  - copies with their own options: the oracle archetypes' mysteries (Divine Herbalist, Enlightened Philosopher: the
    revelations' prerequisites name `DivineHerbalist*MysteryFeature`/`EnlightenedPhilosopher*MysteryFeature`, 120 of
    the "blueprint it doesn't map" prerequisite findings), the shaman's hexes (`shaman/hex` naming `witch/hex/*`), the
    second curse's `<X>CurseNoProgression` options, the expanded/third element. Each new option carries
    `aliases: [<the key the old slot named>]`.
  - 298's migration takes the one heir that holds at the pick's slot (each slot once; none or several: stale). Checked
    shapes: an old key two copies name opens on the copy its slot offers (298's
    `an_old_key_two_copies_name_opens_each_slot_on_the_copy_it_offers`); Magical Tail's eight heirs resolve one at a
    time because each needs the one before (only one holds at each successive slot).
- **Rows follow the grant**: every class, archetype and path row that grants the shared selection where the game grants
  the copy names the copy (an archetype that inherits the class row needs no row; the Dark Lurker needs new rows at 4,
  6, 10, 12, 16 and 18, since its talents come from inherited rows today). These are the 193 + 6 "selection the game
  keeps separate" findings and the 56 "bonus pick of another group".
- `extra_pick_of`/302's nested selections: each Extra feat's own game selection (302 kept marked first-reached lists
  for Extra Rogue Talent, Hex, Arcana and Discovery "until 293 writes the game's").

### 2.4 Prerequisites (1,569) — stage S4

| Kind | n | Writes |
| --- | --- | --- |
| missing in planner | 504 | the conditions (class level, archetype, `HasNot` archetype, stat, BAB, feature): `Requires` from `requires.rs` |
| missing in planner, a blueprint it doesn't map | 538 | 120 name the oracle archetype mysteries (S2 makes them entries), 88 the kineticist blast carriers (`FireBlastFeature`, `BlueFlameBlastFeature`, …), 85 domains (294), 75 deities' facts (267); the rest name other carriers (162 features, 147 progressions, 12 selections, …: 247 distinct blueprints). Written once S3 has made the carriers entries and held; 294's and 267's stay theirs. |
| not in game | 515 | removed (class levels and character levels the game doesn't ask, the totems' `HasNot`s, …) |
| value differs / grouping differs | 9 / 3 | written |

Also on the 62 blueprint entries 296 left with a hand `prerequisite` field:
- `NoOtherOptionOfGroup` (28 bloodlines) and `NoOptionFrom` (Dragon Disciple): the game's `PrerequisiteCondition`
  trees (`BloodlineDraconicRedProgression` 8c6e5b3cf12f71e43949f52c41ae70a8, `BloodlineElementalFireProgression`
  17cc794d47408bc4986c55265475c06f, `DragonDiscipleClass` 72051275b1dbb2d42ba9118237794f7c): written as `Condition`
  trees (295 has `All`/`Any`/`Not`/`Has`).
- `FeatForSameValue`, `FeatIfValue` (Armor Focus): G3.
- `Worships`, `Atheist`, `DeityOffers`, `PickedThrough`: the deity's `…DomainAllowed` facts and `AtheismFeature`
  92c0d2da0a836ce418a267093c09ca54: **267's model part and 294**, not new tickets.
- the Arcane Trickster's Sneak Attack: G2. `Story` (the Gold Dragon breaths): stays, planner logic.

Not compared today and not written (gaps): `PrerequisiteNotProficient` 12, `PrerequisiteProficiency` 11,
`PrerequisiteParametrizedWeaponSubcategory` 3 (G1); `PrerequisiteAnySpellsInSpellbook` 1 (G6); a full `SneakAttack`
3 (G2); `PrerequisiteSelectionPossible` 1 (302/303's nested selections; written if 303 builds `SelectionPossible`);
`PrerequisiteCondition` 63 (written: the vocabulary has the tree; the check's new comparison compares it). Pets,
story, DLC and "a feature's own NoFeature" (303's `HasNot(self)`) stay out as today.

### 2.5 Progression rows (1,753) and mythic ranks (39) — stage S5

| Kind | n | Writes |
| --- | --- | --- |
| a selection the game keeps separate / a bonus pick of another group | 193 / 56 | in S2 (the splits) |
| in game only / a further rank in game only / at another level | 606 / 160 / 9 | 415 name an entry that exists: written. 360 name 148 distinct blueprints with none (`DetectMagic`, `SmiteEvilAdditionalUse`, `JudgmentAdditionalUse`, `FavoriteEnemySelection`, `WeaponTrainingRankUpSelection` 5f3cc7b9a46b880448275763fe70c0b0, …): written after S3 made them entries |
| a further rank in planner only | 557 (107 features) | the planner's step grants where the game grants once (`cavalier/challenge` 54, `warpriest/fervor` 48, `wild-shape-wolf` 41, `smite-evil` 27 against the game's 27 `SmiteEvilAdditionalUse`, …): removed. **Q2.** |
| in planner only | 172 | removed, one by one: some are the planner's model of a game shape (the monk's `ki-power` 17, a feat slot), which the change log flags for the review |
| mythic: a further rank / in planner only | 7 / 3 | removed |
| mythic: in game only | 23 | 17 distinct unmapped carriers (`AeonGazeFeature`, the immunities, `LichDeityFeature`, `LocustClone*`, …) after S3; Azata's `LifeBondingFriendshipSelection1` 69a33d6ced23446e819667149d088898 at 4–10 (303's Q2: row-shaped selections go to 293) |

**Archetypes.** The game stores `RemoveFeatures`/`AddFeatures` (`ProgressionData.CalculateLevelEntries`); the
planner stores replacement rows (README: "A row's … replace the base row's in full"). The importer keeps the
planner's form: it computes the game's effective row per archetype level (what the check already compares) and writes
an archetype row exactly where it differs from the class row, and deletes an archetype row equal to the class's (the
data check refuses those). Changing the form to adds/removes is a form change with no value change: not 293's (G8,
optional).

**Hidden grants** (787 rows, 213 mythic "a hidden grant the planner doesn't map") are not compared today and stay out
of S5; they come in with the carriers (S3) only as far as the rows need them to hold a carrier a prerequisite reads.

### 2.6 The kinds of 297 and 303's ranks

- **297's kinds**: 297 moved the numbers into components *in the planner's descriptors* ("The game's own descriptors
  come with 293"; Legend's and the Swarm's are untyped in the game). No area compares components yet, so 293's stages
  write only `Requires` (S4) and `GearStat` (S1). The rest of the vocabulary's import (design §5: every reachable entry's
  components, the game's descriptors, `AddFactContextActions` patterns, the property subset) is G7, its own ticket,
  after the check learns to compare components (Q3).
- **303's ranks**: 303 derives repeats from the header (`ranks`, selection, parametrised, `PrerequisiteCondition`) and
  gave four merged entries a planner `ranks` with `known.txt` lines under 293 (kineticist elements 2, Magical Tail 8,
  Primalist rage powers 5, Beneficial Curse once). S2 splits Magical Tail and the elements (above) and writes each
  entry's `Ranks` from its blueprint; the Primalist's and Beneficial Curse's go when S5 writes their rows. 303's
  game-derived repeats (Extra Feat (mythic) e10c4f18a6c8b4342afe6954bde0587b, Extra Mythic Ability
  8a6a511c55e67d04db328cc49aaad2b8, New Strain's Ranks 4) are already the game's; the importer writes `ranks` for every
  entry it touches from `BlueprintFeature.Ranks`, so 249's Weapon Focus per weapon (a user fact) stays as the game has
  it.

### 2.7 New entries (S2, S3)

- **Key**: kebab-case of the blueprint's asset name, without a trailing `Feature`/`Selection`/`Progression`, prefixed
  by the grantor's key where it is a per-class copy; a clash adds the stripped suffix back. Derived once: a later run
  finds the entry by its `blueprint` first, so a key never changes after it is written.
- **File**: in the kind's folder, from a template per kind: `key`, `blueprint`, `name`, `summary: ""`, the header and
  the owned fields. No `guide`.
- **Name**: never the game's localized text. Q1.
- **Ambiguous GUIDs**: 90 blueprints are held by several entries today (39 feature/option pairs; 21 with several
  entries of one kind, e.g. one GUID behind 8 options). The resolver picks by expected kind, then by the context's
  selection (an option of the list being written); anything left ambiguous is refused and logged, never guessed.

### 2.8 The gap tickets (306 on), one line each

- **G1** Weapon categories in prerequisites: `Proficient`/`NotProficient` over weapon categories and
  `PrerequisiteParametrizedWeaponSubcategory` (26 uses; 295 built armour groups only).
- **G2** A feature's rank as a stat: `PrerequisiteFullStatValue` on `SneakAttack` (the Arcane Trickster and 2 more);
  the engine already sums Sneak Attack across classes (244), the vocabulary's `Stat` can't name it.
- **G3** Parametrised feats the game keeps per value: Skill Focus as `SkillFocusSelection` c9629ef9eebb88b479b2fbc5e836656a's
  11 features, Armor Focus as `ArmorFocusSelection` 76d4885a395976547a13c5d6bf95b482's, the mythic Armor Focus
  variants; `FeatIfValue`, `FeatForSameValue`; a value-aware link alias (key + parameter → key), which 298's migration
  can't do.
- **G4** The game's sub-skills (`CheckIntimidate`, `CheckBluff`, `CheckDiplomacy`) in class skills and bonuses (the
  Stalwart Defender; items and curse features).
- **G5** The planner's choices that are a plain feature in the game (the 11 "a choice the game doesn't offer"): a
  `planner/` entry with its `why`, or gone.
- **G6** The spellbook kinds: `AddSpellbookLevel`, spell lists, `BlueprintFeatureReplaceSpellbook` options,
  `PrerequisiteAnySpellsInSpellbook`, Fey Magic's spell, `spell_levels` from the game (290's area; design step 6).
- **G7** The components import: every reachable entry's components (design §5; 3,458 entries, 5,048 components), the
  game's descriptors, the check comparing components to components (Q3).
- **G8** (optional) Archetype rows as the game's adds and removes, a form change with the form proof.
- **G9** (only if Q2 = A) Steps on the sheet from the game: a feature's step from its `Scales`/rank config or its
  `…AdditionalUse` features, where 557 step grants go.

Not new tickets (open already): 267 (deities and domains: `…DomainAllowed`, atheism), 270, 288 (Glasses, usables),
289 (the weapon table, shield bashes: no area reads weapons), 294 (domains), 174 (pets and mounts), 245.

---

## 3. The user's facts

### 3.1 The facts the importer applies, and where each is recorded

| Fact | Recorded | Stage |
| --- | --- | --- |
| Uncanny Dodge is a slayer talent from slayer 4 (the game's `Any`) | 276 line 12; 242 Answer line 42 | S2 (lists), S4 (prereqs) |
| Improved Uncanny Dodge needs Eldritch Scoundrel, bard, slayer or skald 12 (reverses 190) | 276 Status and line 24; queue record 2026-11-15; the stale comment in `options/rogue/rogue-talents/improved-uncanny-dodge.ron` | S4 (and the builder updates that comment) |
| Incredible Heft replaces Armor Training at 3, 7, 11, 15, 19 (Titan Fighter) | 285 line 13; supersedes the comment in `archetypes/fighter/titan-fighter.ron` (2026-11-09) | S5 (the comment sits inside `levels`: the builder moves it first) |
| Belt and Headband of Perfection +8 | 288 line 15 | S1 (the headband is missing) |
| The sorcerer's level-1 bonus feat is `SorcererBonusFeat` (21), the Seeker's `SeekerFeatSelection` (26) | 281 line 12 | S2 |
| Shield bashes 1d6/1d4, unarmed finessable | 289 line 12 | not the importer's (no area reads the weapon table): 289 |
| The Arcane Trickster's 3 skill points | 261 line 14 | S1 |
| The rogue talent Weapon Focus per weapon | 249 line 14 | built by 249; the importer keeps it (`ranks` from the header) |
| 262: the 13 class-skill rows and the Defender of the Hearth's 3 skill points | 262 Status and table; queue record | S1 |

(242's Answer line 42 lists the seven; 262 and 276's Improved Uncanny Dodge are the queue record's other two.)

### 3.2 How a run refuses any other user fact

The user's facts in the importer's fields aren't marked in the data, so S1 adds a registry,
`tools/game-check/user-facts.txt`, in `known.txt`'s line grammar (`<area> / <kind> / <entry> [/ <value>] ->`
`applied <ticket>` or `-> held`), seeded from:
- every class's and archetype's class skills and skill points (`research/skills-from-user.md`, map Notes): the kinds
  the check already tags "(the user's facts)" (`areas/class_tables.rs`);
- the user's 22 plain stat belts and headbands (the `gear/` entries with no guide link: README's `gear/` row);
- the backgrounds (`research/backgrounds-from-user.md`) and the weapon table (2026-11-08), for G7 and 289;
- facts the data's comments credit to the user (`grep -rn "user" libs/game-data --include=*.ron`: the Crusader's bonus
  feats ignoring prerequisites, `selections/cleric/crusader/bonus-feats-{1,10,20}.ron`, ticket 21 R5, which agree with
  the game today; the two superseded ones above);
- the tickets' "the user's decision" lines that name a fact in an owned field (a grep over `issues/`, the stage's
  builder confirming each: 21, 27-1, 30-1, 32-2, 92, 131, 163, 172, 188, 190, …).

The guard (`guard.rs`): a change to a field the registry names is written only under an `applied` line; otherwise it
is not written, the run prints it under "to the user", the finding stays, and the stage files it under a to-the-user
ticket. The nine facts above are seeded `applied`. A registry line that matches no field fails the run (as a stale
`known.txt` line fails the check).

---

## 4. Staging

### 4.1 Five stages, in this order (recommended), each one branch, one review, one merge

| Stage | Writes | Findings it removes (of 4,413) | Depends on | Estimate |
| --- | --- | --- | --- | --- |
| **S1** the writer | `--write`, `span.rs`, guard, registry, log, fixture tests; class tables (incl. `Archetype::bab`), items (incl. the +8 headband), the features' `ClassSkill` 262 needs | ~67 | 302–305 merged | one Opus build (about 297's size) |
| **S2** selections | option lists, ~65 per-class splits and their rows, the Extra feats' lists, split entries with aliases, new option entries | ~1,200 (981 lists + 193 + 6 + 56 rows), less the gaps and 294's | S1 | two builds |
| **S3** carriers | an entry for each blueprint a prerequisite or a row of S4/S5 names (≈400 distinct: 247 + 148 + 17, plus their grant chains), with only their grant components (`Grant`, `GrantAtClassLevel`, `GrantIf`, `GrantOnApply`) | 0 (a form change: nothing reads them yet) | S2 | one build |
| **S4** prerequisites | every `Requires` from the game; the 62 hand trees where §2.4 says | ~1,350 | S2, S3 | one and a half builds |
| **S5** rows and ranks | class, archetype and path rows | ~1,500 | S2, S3 (Q2 answered) | two builds |

Why this order: rows and prerequisites name the split selections and the new entries, so S2 comes first; prerequisites
and rows name carriers, so S3 comes before S4 and S5; S4 before S5 because S5's removed step grants and row changes
move many more Builds, and a prerequisite fix should be seen on its own. S1 first because it is small and proves the
writer on ~67 values. One ticket per stage (293 keeps S1; the main session numbers S2–S5 after or before the gap
tickets), since each is a full Opus build with its own review and the proof below.

### 4.2 Each stage's proof (the form proof can't pass: values change)

1. **The game check**: exit 0 with the stage's `known.txt` lines gone (deleted by the builder from the run's list of
   resolved findings) and each remaining finding under a gap ticket or 294/267/270/288/289/174; never a new line under
   261–291 or 293.
2. **Idempotence and determinism**: a second `--write` writes 0 files; two runs on two copies byte-identical (the
   counts in the Answer).
3. **The change census** (the form proof run as a census, not a pass): `record` on the stage's base in a temporary
   worktree, `compare` on the tip; it fails, and a classifier in the scratchpad (301's `classify.py` shape) attributes
   every differing line of `builds.tsv`, `slots.txt`, `sheets.txt` and `numbers.txt` to a change-log row (the slot's
   selection, the pick's key, the sheet line's feature, the number's part). The Answer gives the counts per kind and
   **0 unattributed**. S3's census must be empty (it is a form change; any difference is a bug).
4. **Links**: `compare`'s reopen of every old link, counted three ways: opens with the same picks; opens with a pick
   moved to an alias heir (298's migration: S2's expected kind); stale or a pick refused. Each refused one is
   attributed to a change-log row (a removed offer, a new prerequisite, a removed row grant) and listed by Build in the
   Answer. 298's `every_corpus_and_bench_link_opens_with_the_same_picks` changes from "the same picks" to "the same picks
   or an alias heir, or a refusal the stage lists" (a test change in S2, its expected list from the census).
5. **A sampled hand check**: 100 changed fields per stage (S1: all ~67), stratified by area and kind in proportion with
   at least 5 per kind, picked by the FNV hash of entry key and field (reproducible); the reviewer compares each with
   `game-check show <guid>` and the cited code; one mismatch stops the merge.
6. **The self-checks' long runs**, once per stage, alone on the machine: `cargo test --release -p corpus -- --ignored`
   and `cargo test --release -p seeds --test game_data agreement -- --ignored`. A disagreement is the engine's to fix
   (map Notes).
7. **The gold builds**: every change in a gold build's gains goes to the user before the merge (map Notes; the
   Enforcer is a slayer archetype, so S2's slayer lists and S4's talent prerequisites may touch it). A known gap is
   deleted when its fix lands, never added.
8. **The ignored tests**: 129 real-data tests wait on 255–293 today (128 `waits on ticket N`, 1 `waits on tickets …`
   in `archetype_losses.rs`; 242 counted 141 before 296–301 took some off). Each stage runs those of its areas with
   `-- --ignored`; each that passes comes off ignore; each that fails gets its reason rewritten to name the gap ticket
   (or the to-the-user ticket). By the waiting ticket: 276 20, 267 14, 264 7, 265 7, 288 7, 270 5, 281 5, 289 5,
   261 4, 266 4, 269 4, 277 4, 279 4, 257 3, 259 3, 268 3, 271 3, 286 3, and 1–2 each for 260, 263, 272–275, 278, 280,
   282, 283, 285, 287, 290, 293.
9. The scoped checks for `libs/game-data/` and `tools/game-check` (`CLAUDE.md`), and the queue-end checks after S5.

---

## 5. Offers the game's lists remove (S2)

Counted with each selection identified by its grant (§2.3), against the planner's list at main 1387bbeb:

| Game list | Removed offers |
| --- | --- |
| `BardTalentSelection` 94e2cd84bf3a8e04f8609fe502892f4f | 14 |
| `SkaldTalentSelection` d2a8fde8985691045b90e1ec57e3cc57 | 17 |
| `ProvocateurTalentSelection` aa88cb5263a54791b8d1e471bc192ad4 | 14 |
| `DarkLurkerRogueTalentSelection` 1edbb68422c046f98a86f4726f6721b2, `RogueTalentSelection` c074a5d615200494b8f2a9c845799d93, `ExtraRogueTalentSelection` 14d9089df87a43b696fa9451ca2f0a12 | 0 |
| `SlayerTalentSelection2` / `6` / `10` | 8 / 7 / 6 (each adds Uncanny Dodge and Improved: user, 276) |
| `ExtraSlayerTalentSelection` 53f7237f5b1447bb851ba68045a00e41 | 4 |
| `RagePowerSelection` 28710502f46848d48b3f0d6132817c4e, `SkaldRagePowerSelection` 2476514e31791394fa140f1a07941c96 | 5 each (the Instinctual Warrior's) |
| `InstinctualWarriorRagePowerSelection` 609f0e5336084442a0dafa3abd4d31c5, `ExtraRagePowerSelection` 0c7f01fbbe687bb4baff8195cb02fe6a | 0 |

298's Build notes said the bard loses 19 (9 + 10 advanced); with the bard's own blueprint it is 14, and the bard keeps
Terrain Mastery, Familiar, Blinding Strike, Emboldening Strike and Petrifying Strike.

**Corpus Builds holding a removed pick** (301's record of main's data, 2,162 Builds; each slot's class read from the
sheet's `Level N: Class (Archetype)` line, its class level counted, its game selection resolved from the rows'
findings or the selection's own blueprint; an Extra feat's slot from that feat's selection): **67 Builds, 62 of them
open today** (the 5 others already open nothing).
- slayer lists (2, 6, 10 and Extra Slayer Talent): 39 (38 open); weapon-focus and skill-focus most of them;
- bard: 14 (weakening-wound, survivalist, trailblazer, unbalancing-trick, underhanded-trick, skill-mastery-lore);
- skald: 8 (jump-up, underhanded-trick, mystery-seeker); Inciter: 2 (jump-up);
- `RagePowerSelection`'s Instinctual Warrior powers outside an Instinctual Warrior: 4 (none open today).

After S2 these 62 open with the pick refused (234; as 251's game-correct changes did). Beyond the three asked about,
the feat-group lists remove offers too: Builds holding a pick the game's `FighterFeatSelection`,
`WizardFeatSelection` and the inquisitor's `TeamworkFeat` lack: 34, 34 and 19 (Toughness and the save feats from the
fighter's list; Brew Potions, Scribe Scrolls, Arcane Strike, … from the wizard's; Spell Chain): upper bounds, since
the magus, cavalier, warpriest and other class lists of §2.3 replace those selections first. S2's census gives the
exact numbers.

Not removals but remaps (the link migrates through an alias): the expanded element (88 Builds name one), the second
curse (41), the Aeon gaze tiers (95), the shaman's hexes (21), the oracle archetype mysteries (298's count: 92 Builds
name a mystery), Extra Hex (21), Extra Channel (7).

---

## 6. Risks and open questions

**Q1. Names of new pickable entries.** S2 offers entries no file has (the Dark Lurker's Evasion, the Vanguard's extra
talent `VanguardTacticianExtraUseTalent` c67db0b6ede802b43a6cf00136b1a5e6, the elemental witch's rays, Extra Hex
(Shaman), AWT 2–4, and the rest of the ~316 list members): a picker shows a name and a summary, and the game's text
can't be used.
- A (recommended): the S2 builder writes a name and a one-line summary in our own words, from the blueprint's
  mechanics, for every new *pickable* entry (an option, a feat, a mythic pick: roughly 150–250); carriers (S3) get a
  name from the asset name and no summary, since no picker shows them.
- B: every new entry named from its asset name, summaries empty, a later ticket writes them.
- C: new pickable entries wait, unoffered, until named (the findings stay).

**Q2. The planner's step grants.** 557 row findings across 107 features are the planner granting a feature again at
each step (tickets 170, 177, 182, 188 — the user's "option A" for the cavalier's Challenge and Tactician — 194), where
the game grants it once and counts steps another way (a rank config, or its own `…AdditionalUse` features:
`SmiteEvilAdditionalUse` at the 27 levels the data steps Smite Evil). Writing the game's rows drops "rank n" from those
features on the sheet and in `describe`.
- A (recommended): the game's rows (234); the step display comes back from the game's own shape in G9.
- B: keep the step grants as a planner layer; their 557 findings stay known for good.

**Q3. Where "every reachable feature an entry" lands.** The user chose it (2026-11-15); the design put it in the
importer (§5). The check doesn't compare components, so nothing would verify 3,458 entries and 5,048 components.
- A (recommended): 293 makes only the carriers its own areas need (S3, ≈400, grant components only, proved by an empty
  census); the full import with the check comparing components to components is G7, the next ticket.
- B: all 3,458 in 293 (one more stage, about two builds, verified by hand samples only until G7's check exists).

Risks (no decision needed):
- **Ambiguous GUIDs** (90 blueprints behind several entries): refused, never guessed; S2's splits remove most.
- **302–305 change the forms** the writer targets (nested selections, `ranks`, the progression header replacing
  `option_levels`): S1 starts after 305 merges and reads the forms then; §1.2's owned-field table is updated to them.
- **Silent pairing** in the check (feat selections paired by group) hid the sorcerer's and 13 other classes' lists;
  S2 resolves every row's selection by GUID, and its rows comparison reports a grant of a different feat selection
  as a difference from then on.
- **Gold builds** (the Enforcer's slayer talents) and **corpus coverage** (62 more Builds stop opening after S2, on top
  of 251's 829; 245 regenerates them).
- **CPU**: each stage's census, long self-check runs and two-run idempotence check are heavy runs: one at a time.

## The user's answers (2026-11-16)

- Q1 names: "Own words now". The builder writes a name and a one-line summary in our own words for each new pickable entry; hidden carriers are named from their asset name.
- Q2 step grants: "The game's rows". Write the game's single grant; the sheet's "rank n" on those 107 features comes back the game's way through gap ticket G9. This revises the user's ticket 188 choice for the cavalier's steps.
- Q3 every reachable feature: "Carriers now, rest later". 293 makes only the ~400 carriers (S3); the full import of ~3,458 entries is gap ticket G7, once the check compares components. This revises the user's 2026-11-15 "every reachable feature" decision's timing, not its goal.

## Main session's numbering

- S1 stays 293. S2–S5 are tickets 306–309 (option lists and splits; carriers; prerequisites; rows). The gap tickets G1–G9 are 310–318, filed at the queue's close.
- Hand work beside S1: the +8 headband the user approved is missing from gear/ (S1 adds it, cited); 289's shield-bash dice stay with 289.
