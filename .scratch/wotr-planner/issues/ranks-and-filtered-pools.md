# Ranked features and prerequisite-filtered pools, as the game models them

Type: design, then build
Status: open (number to assign: the numbers from 243 on are another agent's until its difference tickets are filed)
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

## Done when

- Each group in 237's Answer of these two shapes is one entry in the data, holding its blueprint.
- The engine models ranked features, repeated selections and prerequisite-filtered pools. Each has a rules test on hand-made data and a real-data test whose expected values cite the game.
- Every existing link still opens with the same picks: a test over the corpus's links.
- `scripts/check.sh` passes.
