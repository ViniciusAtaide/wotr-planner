# The game data laid out by blueprint

Type: design (decided), then build
Status: built in part (2026-11-15): stages 1, 2 and 4 and stage 3's feature collapses; the rest of stage 3 is open (see "Built in part")
Blocked by: 237, [243, the rank and context splits](243-ranked-features-and-filtered-pools.md)
Model: Opus to build.
Map: [WotR Build Planner](../map.md)

## Question

Mapping each entry to its blueprint ([237](237-a-blueprint-guid-per-entry.md)) showed the planner's data cut the game's objects in ways the game doesn't:
- **Archetype and class copies:** Sneak Attack is 13 entries, one `SneakAttack`.
- **Rank splits:** `oracle/curse-2/3/4`, one `OracleCurse`.
- **Context splits:** per-mystery revelation pools, one `OracleRevelationSelection`.
- **Entries merging several blueprints:** the domains (Primary, Secondary, Separatist), Blessing and Second Blessing, Aeon Gaze tiers.
- **About 336 planner-made entries:** no blueprint behind them.

237 first planned a list of allowed sharings. The user: "creating a blueprint-sharings.ron still makes the old architecture leak into the new", and "why don't we create one file per blueprint in a blueprint folder?" Grilled on 2026-11-15.

**Facts found** (2026-11-15, read in the code at 0d2c2d8):
- **Files:** 160 `.ron` files; the folder decides the kind (`libs/seeds/src/files.rs:104-144`, a whitelist).
- **Keys:** a feature or selection key must start with its owner's key (`DataIssue::KeyOutsideItsOwner`, `check.rs:203`). The sheet's labels parse key prefixes for the holder (`engine/labels.rs:218-238`).
- **Lookups:** the engine finds entries by global key indexes (`engine/catalog.rs`), not by walking the owning class.
- **Cross-class references:** 78 progression entries already take another class's selection (skald and bard take `rogue/rogue-talents`). No feature is borrowed across classes; each class defines its own copy.
- **Links:** they name races, subraces, backgrounds, deities, skills, weapons, classes, archetypes, paths, feats, selection options, mythic abilities, mythic feats and gear, never features or selections (`codec.rs:15-27`).
- **The browser** receives the serialised `GameData` and checks it again (`apps/backend/src/game_data.rs`, `libs/ui/src/game_data.rs`).

## Answer

**Decided** (user, 2026-11-15). Round 1 was put to the user with recommendations; the user: "do everything as planned", so every recommendation below was taken. Each is open to revisit before the build starts.

1. **One file per blueprint, for every kind with a blueprint:** classes, archetypes, features, selections, feats, mythic paths, abilities and feats, races, subraces, backgrounds, deities, items, spells, spell lists and weapons. That is about 3,800 files, in folders by kind under `libs/game-data/`. Two data tickets then rarely touch one file.
2. **The model follows the data.**
   - `GameData` becomes flat catalogs keyed by key.
   - Classes, archetypes, paths and progression rows refer to features and selections by key, with no owned copies.
   - The engine's indexes already work this way. The structs, the loader, `GameData::check`, the MCP server's and the web app's walks of a class's own lists, and the browser's JSON (smaller without the copies) change with it.
3. **Order:** 237 (the GUIDs), then the rank and context splits ticket (the engine learns ranked features, repeated selections and filtered pools), then this layout. Each lands one shape at a time.
4. **Timing:** after the queue of 235–242. 238 reads through `GameData` and the GUIDs, so it needs little rework.
5. **Planner-made entries** go in a `planner/` folder of the same per-entry form. Each file says why no blueprint stands behind it. This ticket questions each one first: a placeholder that is really a rule becomes the rule. The 91 bloodline-spell placeholders, for instance, are "the held bloodline's spell at this level".
6. **File names and keys:**
   - Each file is named by its key: `features/sneak-attack.ron`. Its blueprint is inside.
   - A collapsed feature or selection takes one key without an owner (`sneak-attack`). Two game objects of one name are told apart by the game's own distinction (`sneak-attack`, `sneak-attack-vivisectionist`, if the game has two).
   - Keys that links name never change. A collapsed linked entry keeps the old keys as `aliases`, which only the link decoder reads.
7. **Owners:**
   - `KeyOutsideItsOwner` goes.
   - The sheet's holder comes from the row or selection that grants the entry, not from the key's prefix.
   - A feature granted by two classes is one entry with two holders.
8. **Selection options are features.** As in the game (a selection's `m_AllFeatures`), an option is a feature entry that the selection lists. The link codec's option kind looks its key up among the features. An option's level rows stay with the feature.
9. **Archetypes** are files of their own. Each one's added and removed rows refer to its class's features, as the game's `AddFeatures` and `RemoveFeatures` do.
10. **Entries merging several blueprints split** back into the game's objects: the Primary, Secondary and Separatist domain selections, Blessing and Second Blessing, the Aeon Gaze tiers. The web app may show them together; the data holds them as the game does.
11. **What an entry file holds:** the entry (key, blueprint, name, summary, guide link, prerequisites and its own fields). Progression rows stay in their class's or archetype's file. Data notes stay in `notes/`, one per ticket.
12. **The move is mechanical and proved.**
    - A script, kept outside the repo, writes the new layout from the current files and 237's mapping.
    - The proof is the corpus: its 2,163 Builds' names and `Debug` hash before and after, the way 235 proved its move. It is checked together with the self-checks, the gold builds and every real-data test, with no expected value changed.
    - Link compatibility: every bench and corpus link decodes to the same Build.
13. **One build ticket.** The structs, the loader, the check, the data move and the readers land together, since a half-moved layout doesn't load.

### Built in part (2026-11-15)

Data note: `libs/game-data/notes/244-the-game-data-laid-out-by-blueprint.md`.

**Decisions since the design:**
- **The user, 2026-11-15:** "Defer the domain split", to ticket 294.
  - The cleric's second domain, the Separatist's and the druid's and Blight Druid's stay one option per domain.
  - Their 46 `known.txt` lines move from 244 to a 294 section.
- **The main session, in the brief:**
  - Behaviour changes toward the game are accepted where collapsing copies makes the engine do what the game does, each with a real-data test.
  - Selection groups that differ are researched in the game, not collapsed blindly.
  - A feature and an option sharing a GUID merge only if the engine keeps "granted" and "picked" apart with a small change.
- **The earlier builder's plan, adopted by the main session:**
  - Flatten the model.
  - Move the files with a script kept outside the repo.
  - Collapse and split group by group.
  - Then the readers and docs.

**Stages 1 and 2, built together** (ours: a stage-1 loader reading the old nested files into the flat model needed a throwaway copy of every nested struct):

- **The model.** `GameData` is flat catalogs (`libs/rules/src/game_data.rs`).
  - It gains `subraces`, `archetypes`, `features`, `selections`, `options` and `feat_lists`.
  - Archetypes name their `class`, subraces their `race`.
  - `FeatureSelection::options` is the keys of the options it defines.
  - The lookups are `archetypes_of`, `subraces_of`, `selection`, `feature`, `selection_option` and `options_of`.
- **The layout.**
  - `libs/game-data/` holds 4,197 files.
  - Each entry is one file in a folder by kind, named by its key.
  - `planner/` holds the entries with no blueprint, each as `(why: "...", entry: ...)`.
- **The loader.** `libs/seeds/src/files.rs` refuses a file whose path isn't its key (`Misnamed`) and a planner file with no reason (`NoReason`).
- **The move script** is `move.py` and `ronspan.py`, kept outside the repo.
  - It keeps every entry's text and comments.
  - The files' header comments moved to the README.
  - It was re-run over main's files when 247, 249 and 250 merged.
- **`GameData::check`:**
  - **Gone:** `KeyOutsideItsOwner` and `OptionOutsideItsSelection`.
  - **New:** `OptionNotDefinedOnce`, for an option no selection or two define.
  - **Changed:** `SharesOwnOption` is an option in both a selection's own and shared lists. `MythicRanksOffAPath` means "no path's rows reach the selection". An archetype names a known class, a subrace a known race.
  - **Still read from keys:** `MisplacedBonusList` reads a race's or path's own list by its key prefix, the one rule that still does (the lists of races and paths keep their prefixes).
- **Readers.**
  - The engine's catalog indexes the flat catalogs.
  - The MCP server's `describe` lists the selections a class's, archetype's or path's rows name.
  - Search, validate and the game check read the flat catalogs.
  - The web app's race picker finds subraces through the engine.
  - The test world is written nested and flattened (`test_world/world/nested.rs`).
- **Proof:** the corpus's 2,163 Builds have the same names, `Debug` hash (`1cdd0522cf7234b1`) and build sheets (`4e430981c8be6e65`). Every link opens as before, and the game check prints the same report.

**Stage 3, the feature collapses:**
- **What was collapsed:** 68 groups of features that copied one game blueprint (351 copies) are one entry each, with an owner-less key (`sneak-attack`, `uncanny-dodge`, `wizard-cantrips`, …).
  - Every reference follows the new key, and a prerequisite's repeated children drop.
  - 149 archetype rows that became their class's rows went.
- **Left as two entries holding one GUID:**
  - The Martyr's Stigmata steps, because of three feats' stand-in prerequisites (tickets 263 and 264).
  - The school and Elemental Rampager powers, and the combined wild shapes.
  - The 39 feature/option pairs (decision 8): the engine keeps options and features as two kinds, so merging them is more than a small change. Listed in ticket 237's Answer.
- **Proof** against a baseline recorded after merging main: the 2,164 Builds have the same names, `Debug` hash (`b22f639f96336fe7`) and sheets (`02ab25fb5c6ab9f1`), and every link opens the same.

**Stage 4, the readers:**
- **Decision 7:** the sheet's holder of a feature or a selection comes from what grants it by its rows (`Catalog::feature_holders`, `selection_holders`, `GrantedBy`), not from its key.
- **The state:** it records the selection that defines each held option (`CharacterState::option_selections`), which `NoOptionFrom` reads, not the key prefix.
- **Docs:** `CLAUDE.md` "Game data" and the README's Layout describe the layout.

**Behaviour changes, each toward the game:**
- **Sneak Attack sums across classes.** A prerequisite reads Sneak Attack's rank summed across classes (ArcaneTricksterClass 9c935a076d4fe4d4999fd48d853e3cf3 asks `PrerequisiteFullStatValue` SneakAttack 2; SneakAttack 9b9eac6709e1c084cb18c3a366e0ec87 adds 1 per rank, `UnitLogic/FactLogic/AddStatBonus.cs` `OnTurnOn`, WrathDecompiled 6ce4a6b).
  - Test: `collapsed_features::the_arcane_trickster_reads_sneak_attack_summed_across_classes`.
  - The corpus has no Build this changes.
- **Display:**
  - Collapsed features show one name ("Needs Advanced Talents", not "… (Archaeologist)").
  - Mythical Beast's alternatives count 16 animal companions, not 17.
  - The Empyreal's Apotheosis and the Celestial's Ascension are one feature (BloodlineCelestialAscensionAbility d85e0396d8f68e047b7b67a1290f8dbc), held once.

**Tests changed to the new model, no game value changed:**
- **Rules engine tests:** `check.rs`'s owner tests became `a_key_needs_no_owner_but_an_archetype_and_a_subrace_name_theirs` and `an_option_is_one_selections_own`. `lookups.rs` became `the_selections_are_one_catalog_…` and `the_features_are_one_catalog_…`. `spells.rs` became `a_lists_key_needs_not_start_with_its_owners`.
- **Seeds:**
  - `files.rs` is rewritten to the new layout (the fixture with it), with tests of `Misnamed` and `NoReason`.
  - The real-data tests read the flat catalogs and the new keys.
  - `spells::each_spell_is_in_the_file_of_its_blocks_page_and_level` was deleted: its fact lived only in the old file paths.
  - `archetype_losses.ron` drops 185 losses that were a feature replaced by its own copy, and 7 name their levels.
  - `summaries` lowers the empty class-feature summaries from 104 to 28.
- **MCP:** `gained.rs`, `describe.rs` and `compact.rs` use the new keys.

**Left open, for the next build of this ticket, the importer (293) or a gap ticket:**
- **The selections the game keeps per class** (the `known.txt` group "a selection the game keeps separate", 199 findings): the bard's, skald's, slayer's 6th- and 10th-level, Dark Lurker's, Instinctual Warrior's, Exploiter's, Provocateur's, Hexcrafter's, Order of the Paw's, Elemental Witch's, Nine-Tailed Heir's and the third elemental focus. Each needs its game option list read and mapped.
- **Entries holding several blueprints:** Blessing and Second Blessing, the Aeon gaze tiers, the kineticist element feat lists, Advanced Weapon Training, Extra Channel, Extra Hex, Magical Tail and Abundant Smite.
- **The oracle copies:** the Divine Herbalist's and Enlightened Philosopher's mystery selections, the revelations' mystery prerequisites that name their mystery copies, and `shaman/hex`.
- **The groups that differ:**
  - 11 selection groups: the Armiger's and Hellknights' orders, Hag-Riven, the Stigmatized Witch's curse, the inquisitor's domains, the oracle archetypes' mysteries and revelations, the Elemental Specialist's school, the Feyform Shifter's talent, the sorcerer archetypes' bloodlines, the Blood Kineticist's element, the Wildland Shaman's spirit.
  - The 3 ranger archery feat-list pairs.
  - The 2 option pairs (Clear Mind, Skill Focus).
- **The planner-made entries** questioned and turned into rules: the 64 bloodline-spell placeholders.
- **The domains,** for ticket 294.

The `known.txt` lines under 244 stay until those are built; the game check exits 0.

## Done when (for the build)

- `libs/game-data/` holds one file per blueprint in folders by kind, plus `planner/`, and no entry is a copy of another.
- The game check's findings listed under 244 in `tools/game-check/known.txt` are gone, and their lines with them (2026-11-15: 323 at first, then 309 more assigned when tickets 247–291 were filed). Among them are the game's per-selection copies that the planner shares: the domain options' Secondary and Separatist exclusions, the revelations' Divine Herbalist and Enlightened Philosopher mystery prerequisites, the druid and Blight Druid domain lists, the archetype mystery selections, `shaman/hex`, and the cleric's secondary-domain rows. Splitting an entry keeps each copy's own prerequisites, so no behaviour moves.
- The corpus hash, the self-checks, the gold builds and every real-data test agree before and after, with every link still opening.
- `GameData::check`, the MCP server and the web app read the new model; `scripts/check.sh --full` passes.
- `CLAUDE.md`'s "Game data" and `libs/game-data/README.md` describe the layout.
