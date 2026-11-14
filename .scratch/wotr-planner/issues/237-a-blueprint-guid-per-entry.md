# A blueprint GUID on every game-data entry

Type: build
Status: open
Blocked by: 236
Model: Opus to build: the mapping needs judgement where names don't match.
Map: [WotR Build Planner](../map.md)

## Problem

Decided in [234](234-game-blueprints-as-a-source.md) (user, 2026-11-14): each entry in `libs/game-data/` names its blueprint by GUID. That makes the planner's data a mapped subset of the game's graph, which the game check ([238](238-the-game-check.md)) walks side by side.
- The user's question that led here: "aren't GUID's reference for you to search and create a tree of knowledge?"
- Blueprints refer to each other only by GUID. Names are not unique: "Mobility" is both an encyclopedia page and the feat.

## Change

- **The field.** An optional `blueprint` field (a 32-hex-digit GUID) on every kind of entry with a blueprint behind it: feats, classes, archetypes, prestige classes, features, selections, selection options, mythic paths and abilities, races and subraces, backgrounds, deities, items and spells.
  - **What it isn't.** The planner's keys stay as they are, since links depend on them. There is no per-fact source field (user: the recommendation): an exception, such as one of the user's facts or a code citation, goes in the data's note.
- **`GameData::check`.** It checks the format. It refuses one GUID on two entries of the same kind, unless a list of known sharings allows it: a parameterised feat's one entry, or a feature the planner splits, such as the Invulnerable Rager's Damage Reduction.
- **The mapping**, by a script kept outside the repo (as decided, no game data is committed) or by the game check's own code once [238](238-the-game-check.md) exists:
  - match each entry to its blueprint by type and name;
  - follow the GUID references from the class progressions and the feat selections (`BasicFeatSelection` and the class bonus-feat selections) to settle duplicates;
  - review the leftovers by hand.

  Entries the planner made up, with no blueprint, are left empty and listed in the ticket's Answer.

Data only, so no number or pick moves.

## Done when

- Every entry with a blueprint names it, and the Answer lists those left empty, with the reason for each.
- `GameData::check` refuses a malformed or wrongly shared GUID, with a test.
- `cargo run -p seeds --bin check-data` and `scripts/check.sh` pass.
