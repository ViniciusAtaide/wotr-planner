# Numbers as components

Type: build (data form)
Status: open
Blocked by: 296
Model: Opus to build: every number source changes form, and the proof must show no number moves.
Map: [WotR Build Planner](../map.md)

## Problem

The staged component plan (user, 2026-11-15; [295](295-the-component-vocabulary.md)) moves the hand-shaped number fields onto the vocabulary's stat, value and line kinds. Those fields are:
- `bonuses.ron`;
- `ability_bonuses`;
- the BAB and skill floors (`bab_at_least_mythic_rank_plus`, Perfect Mind's skill floor);
- `steps` (as `Scales` on the game's `ContextRankConfig`);
- gear bonuses;
- the attack and caster lines' named feats.

The design is in the main session's scratch `vocab/design.md`, §3 kinds 10, 11, 13, 14 and 16, and §4.

## Change

- **A rewrite script, kept outside the repo,** moves each number source onto its component, from the current data, not the game.
- **The engine** reads the kinds instead of the named fields and feats. The attack and caster lines read kinds, not feat names.
- **The fields go** where nothing still needs them.

## Done when

- No number comes from a hand-shaped field the vocabulary covers.
- The proof is clean:
  - the corpus Builds' sheets, attack lines, caster lines and totals are identical before and after;
  - the hashes and links are identical;
  - the self-checks, the gold builds and every real-data test are unchanged.
- The scoped checks pass, and the game check exits 0.
