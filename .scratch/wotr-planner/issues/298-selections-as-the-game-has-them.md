# Selections as the game has them

Type: build (data form and engine)
Status: open
Blocked by: 297
Model: Opus to build: slots, lists and repeats change shape, and every link must keep opening.
Map: [WotR Build Planner](../map.md)

## Problem

The staged component plan (user, 2026-11-15; [295](295-the-component-vocabulary.md)) ends its form changes here.

**What the planner holds today** that the game doesn't:
- feat slots and feat lists, where the game lists every option explicitly (`BasicFeatSelection` names 276 feats);
- `extra_pick_of`, `MythicExtraPick` and a mythic ability's own selections;
- `Repeat`;
- `option_levels` and its kin;
- five held-fact maps in the state.

**What [244](244-the-game-data-laid-out-by-blueprint.md)'s layout left here** (main session, 2026-11-15): `tools/game-check/known.txt`'s former 244 lines, now under 298. They are:
- the per-class selections the game keeps separate: bard, skald, slayer 6 and 10, Dark Lurker, Instinctual Warrior, Exploiter, Provocateur, Hexcrafter, Order of the Paw, Elemental Witch, Nine-Tailed Heir, the third elemental focus;
- entries holding several blueprints: Blessing and Second Blessing, the Aeon gaze tiers, the kineticist element feat lists, Advanced Weapon Training, Extra Channel, Extra Hex, Magical Tail, Abundant Smite;
- the oracle mystery copies (Divine Herbalist, Enlightened Philosopher) and the revelations' prerequisites on them, and `shaman/hex`;
- the 11 differing selection groups, which need research in the game;
- the ranger archery feat-list pairs.

The domains are [294](294-the-domain-split-with-a-slot-aware-link-migration.md)'s. The design is in [research/component-vocabulary-2026-11-15.md](../research/component-vocabulary-2026-11-15.md), §3 and §4.

## Change

- **The game's shapes:**
  - feat slots and lists become selections;
  - nested selections replace `extra_pick_of`, `MythicExtraPick` and a mythic ability's selections;
  - `Repeat` is derived from the header (`Ranks`, selection or parametrised, `PrerequisiteCondition`);
  - progression header fields replace `option_levels` and its kin;
  - the held-fact maps become one, with the kinds kept as views for links and the sheet.
- **244's leftovers above** are split or remapped to the game's own selections, each read from its blueprint. Where a split changes an option key a link names, the old key reads through an alias or a slot-aware migration (as 243 and 294 do).

## Done when

- Every link opens with the same picks: a test over the corpus links and the bench links.
- The proof is clean: the hashes, the self-checks and the gold builds, with no real-data expected value changed but where the game says so, each cited.
- `known.txt`'s 298 lines are gone, and the game check exits 0.
- The scoped checks pass.
