#!/usr/bin/env python3
"""Expected damage per round of one full attack, against one or more armour classes.

Usage: python3 -I dpr.py PLAN.json   (or the JSON on stdin)

PLAN.json:
{
  "label": "Fighter, Demon",          # optional, echoed back
  "ac": [30, 45, 60],                 # the armour classes to price the round against
  "attacks": [42, 42, 42, 37, 32, 27],# each attack's bonus, in order; or objects, see below
  "damage": 95,                       # average damage per hit that a crit multiplies
  "extra": 0,                         # average damage per hit that a crit doesn't (sneak attack, energy dice)
  "threat": 15,                       # lowest natural roll that threatens (15 for 15-20)
  "multiplier": 3,                    # the crit multiplier
  "confirm_bonus": 0,                 # added to the attack's bonus on the confirmation roll (Critical Focus: 4)
  "auto_confirm": true,               # Weapon Mastery: every threat that hits is a crit
  "natural_one_hits": false           # Always a Chance: a natural 1 isn't an automatic miss
}

An attack may be an object instead of a number, to override the plan for that attack alone:
  {"bonus": 42, "damage": 82, "extra": 0}   (Backswing's first attack, say, with less damage)

The model, per attack: a natural 20 always hits, a natural 1 always misses (unless `natural_one_hits`), any other
roll hits when roll + bonus >= AC. A threat is a hit on a roll >= `threat`. It becomes a crit on a confirmation
roll made the same way with `confirm_bonus` added, or always with `auto_confirm`. Expected damage =
P(hit) * (damage + extra) + P(threat) * P(confirm) * (multiplier - 1) * damage.
"""

import json
import sys


def p_hit(bonus: int, ac: int, natural_one_hits: bool) -> float:
    """Chance that one d20 roll + bonus hits the AC."""
    hits = 0
    for roll in range(1, 21):
        if roll == 20:
            hits += 1
        elif roll == 1 and not natural_one_hits:
            continue
        elif roll + bonus >= ac:
            hits += 1
    return hits / 20


def p_threat(bonus: int, ac: int, threat: int, natural_one_hits: bool) -> float:
    """Chance that one roll both hits and lands in the threat range."""
    count = 0
    for roll in range(max(threat, 1), 21):
        if roll == 1 and not natural_one_hits:
            continue
        if roll == 20 or roll + bonus >= ac:
            count += 1
    return count / 20


def attack_value(attack, plan: dict, ac: int) -> dict:
    if isinstance(attack, dict):
        bonus = attack["bonus"]
        damage = attack.get("damage", plan["damage"])
        extra = attack.get("extra", plan.get("extra", 0))
    else:
        bonus, damage, extra = attack, plan["damage"], plan.get("extra", 0)
    nat1 = plan.get("natural_one_hits", False)
    hit = p_hit(bonus, ac, nat1)
    threat = p_threat(bonus, ac, plan["threat"], nat1)
    if plan.get("auto_confirm", False):
        confirm = 1.0
    else:
        confirm = p_hit(bonus + plan.get("confirm_bonus", 0), ac, nat1)
    expected = hit * (damage + extra) + threat * confirm * (plan["multiplier"] - 1) * damage
    return {
        "bonus": bonus,
        "p_hit": round(hit, 3),
        "p_crit": round(threat * confirm, 3),
        "expected": round(expected, 1),
    }


def main() -> None:
    source = open(sys.argv[1]) if len(sys.argv) > 1 else sys.stdin
    plans = json.load(source)
    if isinstance(plans, dict):
        plans = [plans]
    for plan in plans:
        result = {"label": plan.get("label"), "rounds": []}
        for ac in plan["ac"]:
            attacks = [attack_value(attack, plan, ac) for attack in plan["attacks"]]
            result["rounds"].append(
                {
                    "ac": ac,
                    "per_round": round(sum(a["expected"] for a in attacks), 1),
                    "expected_hits": round(sum(a["p_hit"] for a in attacks), 2),
                    "attacks": attacks,
                }
            )
        print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
