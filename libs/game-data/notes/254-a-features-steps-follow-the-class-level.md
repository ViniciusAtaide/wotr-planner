# 254: a feature's steps follow the class level

**What the data says now.** A feature the game grants once and scales by a level carries `steps` (`Feature::steps`:
`on: Classes([...])` or `on: CharacterLevel`, and `at`, the levels it rises at) and sits on one row. Its step on the
sheet is its grants plus the step levels the level it reads has reached while it is held (`Engine::feature_step`;
`feature_count` stays the grants, the feature rank).

The worked example, in `classes/bloodrager.ron`:

- `bloodrager/bloodline/infernal/infernal-resistance`: granted on the Infernal bloodline's row 4 only (the row at 8
  no longer names it), `steps: (on: Classes(["bloodrager"]), at: [8])`. Game: `BloodragerInfernalBloodline`
  9aef64b53406f114cb43f898a3aec01e grants `BloodragerInfernalResistance` ac77b22f74aed034ca2b16ba6a4bfde9 at 4;
  `BloodragerInfernalResistanceBuff` 66c973ccad1293345ae6c5bfeced4a7b ranks it on `BloodragerClass` level, a custom
  table rising at 8 (`ContextRankConfig.GetBaseValue` ClassLevel, `ApplyProgression` Custom, WrathDecompiled 6ce4a6b).
- `bloodrager/bloodline/abyssal/abyssal-bloodrage`: granted on the Abyssal bloodline's row 12 only (rows 16 and 20 no
  longer name it), `steps: (on: Classes(["bloodrager"]), at: [16, 20])`. Game: `BloodragerAbyssalBloodline`
  55d5bbf4b5ae1744ab26c71be98067f9 grants `BloodragerAbyssalBloodrage` 29fc83f824111bb41829957486752c93 at 12;
  `BloodragerAbyssalBloodrageBonusProperty` 415f71e5a47f4cccb3dbd10bd7a0f8f8 multiplies a class-level getter on
  `BloodragerClass` (1 at 12–15, 2 at 16–19, 3 at 20) by the feature's rank, and `BloodragerStandartRageBuff`
  5eac31e457999334b98f98b60fc73b2f adds it to bloodrage's bonus (`ClassLevelGetter`, `FactRankGetter`,
  `PropertySettings.ApplyProgression`).

**What changes for a Build.** The Primalist (`PrimalistProgression` d8b8d1dd83393484cbacf6c8830080ae) who takes the
rage powers at 4 never holds Infernal Resistance, so it gets no step at 8 (it did before); one who takes them at 8
keeps the resistance from 4 and its step at 8 (it lost the step before). Likewise Abyssal Bloodrage: traded at 12,
never held and no steps; traded at 16 or 20, still rising there. Every other holder rises at the same levels as
before (the bloodrager and its archetypes, the Reformed Fiend, the second bloodrager bloodline at MR 1).

**Decisions.** The model is ticket 254's recommendation, taken by the main session under the user's instruction of
2026-11-15 to build tickets 247–291 in order. Ours: the step is a field of the feature (so it moves with the feature
into 244's per-blueprint files), and a step reads the level whatever rows are traded, including a step level reached
before the grant, which counts from the grant on (the game reads the level whenever the feature is held).

**Not modelled.** A step limited to an archetype's levels (`MaxClassLevelWithArchetype`: Caustic Ridicule, Inhibit
Courage) is written on the class: the feature is the archetype's own, so only that archetype holds it. The game check (`tools/game-check`) reads no `steps` and doesn't compare option rows, so it neither saw these two features' repeated grants nor checks their step levels: each `steps` is cited here and pinned by a real-data test. Steps change no
number the planner computes yet (no total reads these two features); the sheet and the MCP say them. The other
classes' repeated grants move to `steps` in their own tickets (268–287), with the code pass over the 17 features in
ticket 254's Answer.

**Tests.** `scaling_features.rs` `a_primalists_rage_powers_replace_the_row_and_its_step_with_it`, and the existing
`the_bloodragers_infernal_resistance_and_abyssal_bloodrage_step` and
`the_selections_sharing_a_stepped_option_hold_the_steps_its_rows_give` (unchanged values, now from steps); the MCP's
`real_data_the_sheet_says_the_step_a_level_raises_a_feature_to`.
