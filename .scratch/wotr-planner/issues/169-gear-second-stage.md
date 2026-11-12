# Gear, second stage: weapons, armour and the remaining items

Type: design
Status: open
Blocked by: 165, 166, 167, 168
Model: no agent. A design ticket: the user decides, in a grilling session with the main session.
Map: [WotR Build Planner](../map.md)

## Question

The equipment plan ([163](163-equipment.md), user, 2026-11-11) takes gear in two stages. The first covers the items that give an ability score bonus. The user wants gear to reach every computed number (163, Q1: option c), so this ticket plans the rest once the first stage has shown how the structure holds.

What the first stage leaves out, each a question here:

- **Weapons:** enhancement bonuses and the unique weapons. The attack line uses a base weapon today (`weapons.ron`, the Build's earliest Weapon Focus weapon). Does a worn weapon replace that default, and how do two-weapon builds pick two?
- **Armour and shields:** the engine computes no armour class. Is AC added as a new number (163, Q10 said not in the first stage), or do armour and shields carry only their other bonuses?
- **Items with no ability score bonus:** the caster-level and DC items ticket 122 extracted (`git show 4b1f164:libs/game-data/gear.ron`), the skill-boosting items, the magic books, rods.
- **Circumstance bonuses:** a bonus against one kind of enemy or for one school of spells. Ticket 122 modelled spell circumstances; ticket [143](143-switched-on-abilities.md) plans abilities switched on in a fight. Which of these become numbers, and which stay other effects?
- **Stacking with feats and features:** ticket [144](144-bonus-types-and-stacking.md) is still waiting on the user's list of bonus types. Gear's types come from the guide's wording; does that list now settle 144?
- **Inherent bonuses from books:** permanent, not worn. Are they gear, or a part of the character?

## Done when

The user has decided each question, the decisions are in the Answer and on the map, and the build is ticketed.
