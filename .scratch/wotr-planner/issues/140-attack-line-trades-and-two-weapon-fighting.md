# The attack line: Deadly Aim, Piranha Strike and two-weapon fighting

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

`ATTACK_LINE_LEAVES_OUT` names Deadly Aim and Piranha Strike, and "a second weapon, off-hand attacks and two-weapon fighting". So ranged, finesse and two-weapon Builds get a line that misses the trade or the attacks that define them. `compare` and `improve` can't rank them on those.

Options:
- **A.** Deadly Aim (ranged weapons) and Piranha Strike (light and finesse weapons) as trades beside the numbers, not applied to them, as Power Attack is shown.
- **B.** A, plus an off-hand weapon: a second weapon argument on `sheet`, `compare` and `improve` and a second picker in the web app. The line then gives the two-weapon penalties and the off-hand attacks that the Two-Weapon Fighting feats add.

*Recommend:* A first, since it's small and follows Power Attack. B as a second step, if the user wants two-weapon Builds ranked.

## Done when

A Build with Deadly Aim and a longbow shows its trade, and one with Piranha Strike and a rapier shows its. With B, a two-weapon Build shows both hands' attacks. Real-data tests cover each case. `ATTACK_LINE_LEAVES_OUT` no longer names what is now counted. `scripts/check.sh` passes.

## Answer

**Recommendation taken** (user, 2026-11-10): A first (Deadly Aim and Piranha Strike as trades beside the numbers), then B (the off-hand line) as a second step.
