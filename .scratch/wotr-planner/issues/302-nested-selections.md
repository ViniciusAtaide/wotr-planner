# Nested selections

Type: build (data form and engine)
Status: open
Blocked by: 301
Model: Opus to build: the extra picks and the mythic abilities' selections change shape.
Map: [WotR Build Planner](../map.md)

## Problem

Split from [298](298-selections-as-the-game-has-them.md) (user, 2026-11-16, "B: forms now, splits in 293"): one of the staged component plan's five form steps (user, 2026-11-15; [295](295-the-component-vocabulary.md)). The design is in [research/component-vocabulary-2026-11-15.md](../research/component-vocabulary-2026-11-15.md), §3 and §4. The per-class selection splits are [293](293-the-importer.md)'s, not this ticket's.

**What the planner holds today:** `extra_pick_of`, `MythicExtraPick` and a mythic ability's own selections, where the game nests a selection inside the feat or ability that grants it.

## Change

- **Nested selections replace them:** a feat or ability grants a selection, and that selection's pick is its slot.
- **The engine questions 298's first build found** are this ticket's:
  - `extra_pick_of` gives a slot of the first listed selection reached; the game's extra-talent selections (ExtraRogueTalentSelection 14d9089df87a43b696fa9451ca2f0a12, the slayer's) name their own lists;
  - `once_per_option` does not reach across selections, which the third elemental focus (ThirdElementalFocusSelection e2c1718828fc843479f18ab4d75ded86) needs.
- **A rewrite script in the scratchpad (`game/302/`)**, from the current data.

## Done when

- The form proof (`record` on the base in a temporary worktree, `compare` on the tip) shows 0 differences, or each difference is explained in the Answer and comes from the game, cited.
- Every corpus and bench link opens with the same picks (298's link test).
- The self-checks' short runs, the gold builds and every real-data test are unchanged.
- The scoped checks pass, and the game check exits 0.

