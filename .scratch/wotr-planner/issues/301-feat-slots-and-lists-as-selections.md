# Feat slots and lists as selections

Type: build (data form and engine)
Status: open
Blocked by: 298
Model: Opus to build: every feat slot changes shape, and every link must keep opening.
Map: [WotR Build Planner](../map.md)

## Problem

Split from [298](298-selections-as-the-game-has-them.md) (user, 2026-11-16, "B: forms now, splits in 293"): one of the staged component plan's five form steps (user, 2026-11-15; [295](295-the-component-vocabulary.md)). The design is in [research/component-vocabulary-2026-11-15.md](../research/component-vocabulary-2026-11-15.md), §3 and §4. The per-class selection splits are [293](293-the-importer.md)'s, not this ticket's.

**What the planner holds today:** feat slots and named feat lists, where the game lists each option in a selection (`BasicFeatSelection` names 276 feats; the fighter's, the magus's and the other bonus-feat lists likewise).

## Change

- **Feat slots and feat lists become selections:** a selection with its options listed, as the game's are, granted on a progression row like any other selection.
- **A re-runnable rewrite script, kept in the main session's scratchpad (`game/301/`),** moves each from the current data. Where the planner's list and the game's differ, this ticket keeps the planner's list; 293 writes the game's.
- **The engine** offers a feat slot through the selection path. The slot ids a link names keep decoding (through 298's migration if an id changes).
- **The fields go** where nothing still needs them.

## Done when

- The form proof (`record` on the base in a temporary worktree, `compare` on the tip) shows 0 differences, or each difference is explained in the Answer and comes from the game, cited.
- Every corpus and bench link opens with the same picks (298's link test).
- The self-checks' short runs, the gold builds and every real-data test are unchanged.
- The scoped checks pass, and the game check exits 0.

