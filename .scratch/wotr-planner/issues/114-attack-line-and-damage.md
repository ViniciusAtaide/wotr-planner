# An attack line and a damage estimate

Type: design
Status: open
Blocked by: none (option C waits on [113](113-effects-as-data.md) option C and a scope decision)
Map: [WotR Build Planner](../map.md)

## Question

The user asked an agent for "maximum attack damage" (2026-11-08, the Demon Greatsword run, see [112](112-effect-summaries.md)). The sheet gives BAB, saves and ability scores, but nothing about an attack:
- no attack bonus per attack and no attacks per round;
- no damage per hit, no crit range or multiplier.

So the agent couldn't measure "maximum", and `compare` can't show which of two branches hits harder. Any damage claim was the agent's own arithmetic.

What could be computed today without gear:
- iterative attacks from BAB;
- Str to attack, and to damage (×1.5 two-handed);
- Power Attack's trade;
- the flat bonuses of Weapon Focus and Weapon Specialization;
- crit range and multiplier from the weapon and Improved Critical.

`libs/game-data/weapons.ron` holds only `key`, `name` and `proficiency`. Damage dice, crit range, crit multiplier and handedness are missing.

This touches the map's **Out of scope**: gear and buffs. A greatsword's base stats are game data, not gear; its enhancement, the character's items and the party's buffs are gear and buffs.

## Options

- **A. Nothing:** damage stays the agent's own knowledge, labelled as such.
- **B. An attack line in `sheet`:** for one base weapon the agent names, at each level: the attack bonus of each iterative attack and the damage per hit (weapon dice + Str + flat feat bonuses + Power Attack), plus crit range and multiplier. Unconditional numbers only: no rage, no stances, no gear, no buffs. `compare` lists it.
- **C. A damage estimate:** expected damage per round against an AC, with toggles (rage, stances, Power Attack, mythic feats). It needs 113's option C and a decision on gear and buffs.

## Open questions (with recommendations)

1. **Which option?** *Recommend:* B. It is facts the engine can own, it lets `compare` rank two branches by an honest number, and it stays inside the current scope.
2. **Weapon stats in the data?** *Recommend:* yes, for option B: dice, crit range, crit multiplier and handedness in `weapons.ron`, from the saved pages, with a real-data test.
3. **In the web app too?** The attack line would be an engine computation in `rules`, so the web app could render it as well. Then `scripts/proof.sh` should show that nothing else changes. *Recommend:* yes, in the Build summary under the grid, with a weapon picker. The MCP and the web app then show the same numbers.
4. **The honesty boundary:** with B, the instructions add the attack line to what an agent may claim as fact, and say that rage, stances, gear and buffs are not in it.
