# A caster line: spell DCs, caster level against spell resistance, concentration

Type: design
Status: open
Blocked by: none (option C waits on [113](113-effects-as-data.md) option C)
Map: [WotR Build Planner](../map.md)

## Question

The user asked an agent for "the strongest DC caster build possible: the maximum chance to debuff the maximum amount of enemies" (2026-11-09, the Max DC run: a Kitsune Sorcerer 20, Arcane bloodline with Fey as Second Bloodline, Azata). This is what [114](114-attack-line-and-damage.md) was for the martial run: the planner checked every pick but couldn't measure the goal.
- `sheet` gives Charisma 25 and caster level 20, but no save DC. The agent added up "about DC 35 for a 9th-level compulsion with metamagic" by hand and had to label it as its own knowledge.
- `compare` can't say which of two branches has the higher DC (Arcane against Crossblooded, Azata against Aeon).
- The same applies to the other two caster numbers the run cared about: caster level against spell resistance (Spell Penetration, Greater, Mythic) and concentration (Combat Casting).

The sources the run stacked, each one stated in a summary today:

| Source | Bonus | Applies to |
| --- | --- | --- |
| Casting ability modifier | Cha (sorcerer), Int (Sage), Wis (Empyreal) | every spell |
| Spell Focus, Greater Spell Focus, Spell Focus (Mythic) | +1 each | one school |
| Expanded Arsenal | the Spell Focus bonuses of one school | another school |
| Kitsune / Gnome / Oread | +1 | enchantment / illusion / acid spells |
| Arcane bloodline arcana | +1 | spells with metamagic |
| Arcane School Power | +2 | one school, from sorcerer 15 |
| Fey / Infernal bloodline arcana | +2 | compulsion / charm spells |
| Elemental Focus, Greater | +1 each | one energy type |
| Aeon's Save DC gaze | 1 + MR/3 | allies' spells, conditional on the gaze |

## Options

- **A. Nothing:** DCs stay the agent's arithmetic, labelled as such.
- **B. A caster line in `sheet` and `compare`**, as 114's option B did for one weapon: for one school (and optionally a subschool, an energy type and "with metamagic") the agent names, at each level, the DC of each spell level, broken down by source, plus caster level against spell resistance and the concentration bonus. Unconditional numbers only: no gaze, no gear, no buffs. A small, fixed table of DC sources in `rules` (the rows above, keyed by item and scope), with a fixed `leaves_out` list, like the attack line's.
- **C. Through a modifier model** ([113](113-effects-as-data.md) option C): DCs fall out of effects as data. Large; not needed for B.

## Open questions (with recommendations)

1. **Which option?** *Recommend:* B. It is the same shape as the attack line, it answers the run's goal, and the source table is small and every row comes from a summary the guide backs.
2. **Where does the DC source table live?** *Recommend:* in the game data, as a `dc_bonus` on the items that give one (value, scope: school, subschool, energy, metamagic, all), since it is a fact from the guide; the engine sums those held. The alternative, a table in `rules`, hides game facts in code.
3. **Subschools:** compulsion and charm need the spell's subschool, which only matters with spells. *Recommend:* the line takes the subschool as an input ("compulsion") and adds the arcana when it's named; no spell data needed ([121](121-spells-as-game-data.md)).
4. **Which spell levels?** *Recommend:* every spell level the Build can cast at that character level, plus a "heightened to the highest" row, since Heighten Spell is how a debuffer uses low-level spells.

## Done when

The run's Build shows its enchantment compulsion DC per spell level in `sheet` and `compare`, broken down by source, with a real-data test on that Build; `scripts/check.sh` passes.
