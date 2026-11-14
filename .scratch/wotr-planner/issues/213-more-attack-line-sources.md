# More always-on sources for the attack line

Type: feature
Status: open
Blocked by: [139](139-attack-line-weapon-features-and-mythic-feats.md) (the `ItemBonus` shape with attack-line targets)
Model: Sonnet to build; Opus to review.
Map: [WotR Build Planner](../map.md)

## Problem

Found by the third martial run (2026-11-14; see [212](212-athletics-iii-raises-bab.md) for the run). Ticket 139 counts the weapon class features and the four mythic weapon feats. The guide-only Build also holds these always-on sources, which 139 doesn't name. The attack line leaves them out, so `compare` and `improve` can't see them:

| Source | Guide | What it changes |
|---|---|---|
| Flawless Attacks (mythic feat) | `mythic-paths#flawless-attacks`: iteratives at −4 rather than −5 | each iterative's bonus |
| Stat Focus (Trickster feat), parameter Attack or Damage | `trickster#stat-focus`: +1 mythic bonus | attack or damage per hit; its other parameters (Initiative, Fortitude, Reflex, Will) belong in `totals` |
| Focused Strike (feat) | `feats#focused-strike`: no Power Attack penalty on the first attack in a turn with a two-handed grip | Power Attack's trade on the first attack |
| Knowledge (World) II (Trickster trick) | `trickster#knowledge-world-ii`: a natural 1 becomes a 20 | no automatic miss; the line's threat chance gains the 1 |
| The Flawless Belt of Physical Perfection +8's "Critical threat range +1" | `items-inevitable-excess#belts` | the threat range; the item is in the planner today, with the line in `uncounted` |

The MCP agent spent about seven feats on picks that add no damage. One cause is that `improve` with `by: {attack_line: {number: "damage"}}` on its feat slots put Focused Strike in `no_change`. Its mythic slots put every mythic feat there (`slot.mythic.10` returned `options: []`). Once 139 and this ticket count them, `improve` ranks them.

Not in scope: Hammer the Gap (+1 per earlier consecutive hit this turn) depends on the dice; it stays in `attack_leaves_out`, or goes to [143](143-switched-on-abilities.md) if that ticket models per-turn effects.

## Build

- Each source as `ItemBonus` data with 139's attack-line targets, each its own `counts` row. Two need a target 139 may not add: an iterative-penalty target (Flawless Attacks), and a per-attack target for the first attack (Focused Strike). Add the target, not a hard-coded case.
- Knowledge (World) II: the line's `critical` already prints a range. Add the natural 1 as a note on the line (`"13–20 and natural 1"`) rather than a number; decide the wording with the reviewer.
- The Flawless belt's threat range counts with gear (`sheet` with the belt, `compare`/`improve` with `with_gear: true`). It moves out of the item's `uncounted`, and `naked` is unchanged.
- Stat Focus's Initiative, save and Fortitude/Reflex/Will parameters in `totals`, if they aren't counted there already: check first.
- `ATTACK_LINE_LEAVES_OUT` and `wotr://leaves-out` stop naming what is now counted.

## Done when

The guide-only Build from this run (the map's History entry for 2026-11-14) shows a `counts` row for each source it holds, and `improve` on its `slot.mythic.10` ranks Flawless Attacks above the mythic feats that change no number. Real-data tests cover each source. The snapshot check passes with the snapshot files untouched (they hold no attack line). `scripts/check.sh` passes.
