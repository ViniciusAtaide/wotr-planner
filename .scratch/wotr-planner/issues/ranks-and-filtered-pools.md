# Ranked features and prerequisite-filtered pools, as the game models them

Type: design, then build
Status: resolved (number to assign: the numbers from 243 on are another agent's until its difference tickets are filed)
Blocked by: 237. Comes before [the game data laid out by blueprint](game-data-by-blueprint.md) (user, 2026-11-15).
Model: Opus to research and build: each split is read from the game and the engine learns two shapes it lacks.
Map: [WotR Build Planner](../map.md)

## Problem

Found while mapping each game-data entry to its blueprint ([237](237-a-blueprint-guid-per-entry.md), 2026-11-15). The planner cuts some of the game's objects into several entries, so many entries of one kind hold one blueprint. The user: "creating a blueprint-sharings.ron still makes the old architecture leak into the new", and these splits get "a ticket on its own". Two shapes account for most of them.

- **Rank splits.** The game grants one feature again at a later level, which adds a rank, or one selection again. The planner made one feature per step and put each in its own progression row:
  - `oracle/curse-2`, `-3`, `-4` (at levels 5, 10 and 15), and the same three under every oracle archetype: 27 entries. 237's review left them without a blueprint, because `OracleCurse` is only the class's signature display and `OracleProgression` grants nothing at those levels. How the game advances a curse is this ticket's research (the curse's own progression, most likely);
  - `wizard/school/transmuter/physical-enhancement-2` … `-5`: one `TransmutationPhysicalEnhancementFeature`;
  - `magus/spell-dancer/spell-dance` … `-iv`: one `SpellDanceFeature`;
  - `druid/feyspeaker/fey-magic-i` … `-viii`: one `FeyspeakerMagicSelection`, taken again at each step;
  - Change Shape I and II, the Assassin's poison save steps, and Martyr Stigmata (237's Answer, group b).
- **Context splits.** The game has one selection and filters its options by their prerequisites (a revelation requires its mystery). The planner made a pool per context:
  - `oracle/revelations/<mystery>`: 12 entries, pools and archetype copies, all on `OracleRevelationSelection`. The oracle's `revelation` selection names the pool its mystery opens.
  - Combat Healer, listed under both the Battle and the Life mysteries.

  The per-mystery final revelations are not a split: each has its own blueprint (237).

237's Answer lists every group left after its review, sorted into these shapes and the rest (archetype copies, which a layout by blueprint removes, and entries holding several blueprints).

## Change

**Research first, from the game.** For each group:
- **The blueprint:** how the game expresses the steps. Options include a feature's `Ranks` and its repeated `LevelEntries`, a selection repeated in a progression, a progression of its own, and a selection's `m_AllFeatures` with each option's `Prerequisite*` components.
- **The code:** how the rules apply them, by file and method at WrathDecompiled `6ce4a6b`. Examples: how a rank is counted, and how a selection hides an option whose prerequisite is unmet.

**The model**, as the game has it:
- **A ranked feature:** one entry, granted by several progression rows. The Build knows its rank at each level, and a prerequisite or a number can read the rank.
- **A repeated selection:** one selection entry named by several rows, each taking another option.
- **A filtered pool:** one selection whose options carry the prerequisite that admits them (the mystery, the curse). The pickers offer only the options it admits, so the planner's per-context pools go.

**The data** collapses each group onto its one entry, with the blueprint of 237.

## To decide

1. **Keys and links.** Features and selections never appear in a link (`libs/rules/src/codec.rs`, the `KeyKind`s), so collapsing `curse-2/3/4` breaks none. Pool options are selection options, which links name.
   - **Recommended:** keep every option's key (`oracle/revelations/battle/battlecry`) and change only what offers it, so every link still opens.
2. **What the sheet says of a rank.** Options are "Curse (stage 2)", a rank number, or the step's own summary.
   - **Recommended:** one summary per rank where the game's steps differ, read from the blueprint.
3. **A rank the planner's numbers read.** Physical Enhancement's bonus is one example; how many read a rank today needs counting.
   - **Recommended:** the engine reads the rank, and no number is stored per step (map Notes: no derived values in the data).

## Answer

**Built** (2026-11-15). The research (the main session's scratch `splits/design.md`, from the game's blueprints and WrathDecompiled `6ce4a6b`) found the groups' game shapes; the main session's answers (`splits/decisions.md`) settled the model. Each group is now one entry with its blueprint, and the engine models the three shapes. Data note: `libs/game-data/notes/ranks-and-filtered-pools.md`.

- **Ranked features.** A feature granted again gains a rank (`UnitLogic/FeatureCollection.cs` `PrepareFactForAttach`, `UnitLogic/Feature.cs` `AddRank`): the engine's grant count already was that rank, with no cap at the blueprint's `Ranks`. Collapsed: `assassin/poison-use-saves-required` (`AssassinCreatePoisonIncreasedSavingThrows`, at 5 and 9), `druid/feyspeaker/fey-magic` ("Fey Magic (Feyspeaker)", `FeyspeakerMagicSelection`, at 4 to 18). Physical Enhancement's `-2` to `-4` went (its bonus scales by wizard level, uncomputed as before) and `-5` is `physical-enhancement-capstone` (`TransmutationPhysicalEnhancementCapstone`). The MCP sheet's gained features say `rank` from 2 on a level line (`Gained::rank`).
- **Repeated selections** already existed (one key named by several rows); a rules test pins them.
- **The oracle's curse is a progression.** Each curse option has rows at curse levels 1, 5, 10 and 15 naming 38 new stage features with their blueprints and summaries in our own words (Powerless Prophecy's rows name its own Uncanny Dodge and Improved Uncanny Dodge copies; Demonic's and Hellbound's first rows make Persuasion a class skill). The curse level mirrors `BlueprintProgression.CalcLevel` with two new `FeatureSelection` fields, `option_level_archetypes` and `half_other_classes` (`option_levels: ["oracle", "witch"]`: a class with a listed archetype counts only taken with it). The Stigmatized Witch gets `witch/stigmatized-witch/curse`; the Possessed Oracle `oracle/possessed-oracle/curse` (Powerless Prophecy, granted with no pick); the Dual-Cursed Oracle's second curse `oracle/second-curse` and the Living Deity's `…/living-deity/curse` share the nine curses and never advance (`skipped_levels: [5, 10, 15]`). The 27 placeholders went. `bonuses.ron`'s initiative bonuses moved from the curse options (with `ClassLevel(oracle, 5)`) to the stages.
- **A filtered pool.** `pool`/`pool_of` are gone; `FeatureSelection::filtered_by` names the selection whose held options admit the pool's options. The nine pools merged into `oracle/revelations` (`OracleRevelationSelection`, 62 options), which the oracle's rows name; each revelation's prerequisite names its mystery. The picker (`Catalog::offered_now`) leaves out an option whose prerequisite names options of the filter selection none of which is held; the walk takes any option the selection offers and the prerequisite refuses an unadmitted one, with the reason. Combat Healer is one option (Battle or Life); Fortune and Misfortune moved in (`oracle/revelations/fortune`, `/misfortune`, needing the Dual-Cursed Oracle). The Dual-Cursed Oracle's and Living Deity's revelation selections keep their keys and share the 62.
- **Links.** Every option key stays. `SelectionOption::aliases` holds the three old keys (`oracle/revelations/life/combat-healer-life`, `oracle/dual-cursed-oracle/revelation/fortune`, `/misfortune`); only `LinkKeys` reads them (an alias's hash reads as the option's key, and counts in the collision test); `GameData::check` refuses an alias that is a key or another alias, and nothing else may name one. New selections were keyed so every row's selections keep their sorted slot indexes (`witch/stigmatized-witch/curse` after `witch/hex`, `oracle/second-curse` after `oracle/oracles-curse`, the Living Deity's curse after its revelation).

**Decisions:**
- The main session, under the user's "do everything as planned" (2026-11-15), decisions.md items 1 to 9: keys stay and three aliases; "rank n" on the sheet, no per-rank summaries; the rank is the grant count, no cap; `filtered_by` stored, not derived for every picker; the Living Deity's revelation copy stays until the layout ticket; stage summaries in our own words; Physical Enhancement; the curse slots; the out-of-scope items below.
- Ours: `filtered_by` sits on the selection and names the filter selection, the admitting options being those the option's prerequisite names, so nothing is stored twice; the filter is the picker's only, the prerequisite the walk's (the simplify review's point). The curse-level fields are copied on the three selections a curse advances through, as the game has them on each progression. The Possessed Oracle's curse selection names `PowerlessProphecyCurseProgression`'s GUID (an inference: the game has no selection there). The NoProgression copies hold the progression's level-1 features (two differ in a number; their summaries say so).
- Ours, a difference left for 238: Fortune needs the Dual-Cursed Oracle but not its level 5 (`FortuneRevelationFeature`'s `PrerequisiteArchetypeLevel`), because 25 corpus links that open today take it at 1 or 3; adding the level broke them.

**Tests** (each expected value cited by blueprint and GUID or by file and method):
- Rules, hand-made data, `libs/rules/tests/engine/ranks_and_pools.rs`: `a_feature_granted_again_gains_a_rank_that_a_prerequisite_reads`, `a_selection_named_by_two_rows_takes_another_option_at_each`, `a_filtered_pool_offers_what_the_held_option_admits_and_checks_the_rest`, `an_option_level_counts_half_the_other_classes_and_a_class_only_through_its_named_archetype` (`BlueprintProgression.CalcLevel`), `a_link_naming_an_options_old_key_reads_as_the_option`; the cleric's test-world pool became a filtered pool, its tests renamed.
- Real data, `libs/seeds/tests/game_data/ranks_and_pools.rs`: the assassin's saves (`AssassinProgression` `a02e1f0e…`, `0cbb2201…`), Fey Magic (`FeyspeakerArchetype` `da69747a…`, `237efc6a…`), Physical Enhancement (`b6a604da…`, `6aa7d349…`), Blackened's stages (`BlackenedCurseProgression` `ca1e4627…` and its five features), the curse level with a fighter and a plain witch (`CalcLevel`; `PowerlessProphecyCurseFeatureLevel5` `abfd1dea…` +4 initiative; Pranked's -4, `cfba3d40…`), the Stigmatized Witch (`AccursedWitchArchetype` `c5f6e53e…`, `LameCurseProgression` `3d319da9…`), the curses that never advance (`SecondCurseSelection` `cc6fda79…`, `MysteryGiftFeatureCurseSelection` `4e7265c0…`, `LameCurseNoProgression`, `PlaguedCurseNoProgression`), the Possessed Oracle (`PossessedOracleArchetype` `09035bb5…` and its five stages), the 62 revelations (`OracleRevelationSelection` `60008a10…`), the revelation slot's offer (`OracleRevelationBattlecry`, `OracleRevelationCombatHealer` `db1d9829…`, Fortune `16935c46…`, Misfortune `8121fe70…`), and `links_made_before_the_splits_open_with_the_same_picks` over the fixture `links-before-the-splits.tsv` (seven corpus links recorded at the base commit, of oracles, a Stigmatized Witch, a Living Deity and a Feyspeaker, and one made there with Combat Healer (Life)'s old key). Changed to the new model: `class_skills.rs` (Demonic, Hellbound), `blueprints.rs`, `archetype_losses.ron`, `option_families.rs`, `option_follow_ups.rs`, `scaling_features.rs`; the MCP's `gained.rs` (`real_data_the_sheet_says_the_rank_of_a_feature_gained_again`) and `compact.rs` (the label "Revelation").
- The corpus: a throwaway run recorded the 2,163 corpus Builds' links (the bench's among them) at `3c6b49d`; after the change every one decodes to the same picks (the three aliases mapped), and the same 160 fail to open as at the base (deliberately impossible Builds).

**Out of scope, for later tickets** (decisions.md item 9): Fey Magic's spell parameter; the Elemental Rampager's element progressions; Enchanting Smile's class-level number; Strength of the Abyss's `AddLevel2` (a 238 note). Also: Fortune's level 5 (above); the test world's revelations still sit in per-domain selections the cleric's slot shares.

## Done when

- Each group in 237's Answer of these two shapes is one entry in the data, holding its blueprint.
- The engine models ranked features, repeated selections and prerequisite-filtered pools. Each has a rules test on hand-made data and a real-data test whose expected values cite the game.
- Every existing link still opens with the same picks: a test over the corpus's links.
- `scripts/check.sh` passes.
