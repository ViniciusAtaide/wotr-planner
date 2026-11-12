# Finesse Training takes a weapon

Type: task
Status: open
Blocked by: 160
Model: Opus to build; review on another model.
Map: [WotR Build Planner](../map.md)

## Question

The gold test ([160](160-gold-builds.md)) lists "Finesse Training - Light Shield" at the Enforcer's level 18 as a known gap. The planner grants the rogue's Finesse Training with no weapon choice, by ticket [54](54-mercies-and-enforcer-exploits.md)'s rule that a choice which gates no pick stays a label.

The user, 2026-11-11: "make finesse training take a weapon". This reverses ticket 54's rule for Finesse Training only.

## Build

AFK. Game data and whatever the engine needs for a feature that takes a value; no schema change expected, but check.

- Find how the guide's rogue page (`guide-cache/rogue.html`) words Finesse Training: at which rogue levels a weapon is chosen, and from which weapons. The data follows the guide.
- Give Finesse Training a weapon value at each level the guide gives a choice, the way other parameterised picks take a weapon (`Parameter::Weapon`). Every archetype that keeps Finesse Training gets it.
- An old link, where Finesse Training has no weapon, must still open; its weapon shows as a choice still to make.
- The number it changes: Finesse Training lets Dexterity replace Strength on damage with the chosen weapon. Say in the Answer whether the attack line counts it now or it is left for the attack-line tickets (139, 140).
- Remove the Finesse Training entry from the gold test's known gaps and give the Enforcer the Light Shield. Regenerate the snapshots and review the diff: only Finesse Training's lines change.

## Done when

- "Finesse Training - Light Shield" agrees in the gold test, with its known-gap entry deleted.
- A real-data test pins the weapon choice at each level the guide gives one.
- An old link with Finesse Training still opens.
- `scripts/check.sh` passes.
