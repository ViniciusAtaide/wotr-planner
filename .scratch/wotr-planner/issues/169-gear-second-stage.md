# Gear, second stage: weapons, armour and the remaining items

Type: design
Status: resolved
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
  Answered for the six manuals and tomes (user, 2026-11-11): they are gear, on a Tomes row.
- **Relics and artifacts** (found by ticket 165): the guide lists them in prose with upgrade tiers, not in table rows, so the first stage has none. Several give ability score bonuses: the Face-Stealer's Mask belts and headbands, the Stone of Ghostly Pathways circlets, Zaoris's Pin headbands. Are they in, and is each tier an item?

**Found again by the third martial run** (2026-11-14; see [212](212-athletics-iii-raises-bab.md)). Most of the gap between its two Builds was gear the planner can't hold. The guide-only agent's 1,970 damage per round against AC 45 counted:
- the Hasting Expanse ring (an extra attack at full bonus);
- Bracers of Slaughter (+3d10 force per attack, ignoring DR);
- Fencer's Gift (+3 damage with a two-hander, +2 Weapon Training);
- Goggles of Ferocious Pact (+4 damage);
- Boots of the Battle Herald (permanent Greater Heroism);
- an Elder Flaming +5 falchion (Defender's Nemesis).

With only a +5 falchion, the same Build came to about 775. The MCP agent found none of these in `search` and could only assume a +5 weapon and a +6 belt. A weapon's enhancement, items that add attacks, and flat damage per hit are the martial cases this stage most needs to answer.

## Answer

**Decided** (user, 2026-11-14, in a grilling session; recommendations taken but where noted):

1. **A held weapon:** the weapon in `gear.weapon` becomes the attack line's weapon, replacing the Weapon Focus default. Its enhancement counts to attack and damage. Its always-on properties count too: energy dice (Flaming's 1d6) as damage rows, Keen as threat range.
2. **Generic magic weapons and armour:** any base weapon or armour from the guide's tables, with an enhancement from +1 to +5, as a planner-made item named as such. The guide lists only its named items; these are shop stock.
3. **Two weapons:** an off-hand place, `gear.weapon.off`, read only by ticket [140](140-attack-line-trades-and-two-weapon-fighting.md)'s off-hand line; before 140 it is held but counts in no number.
4. **Armour class** becomes a computed number (armour, shield, natural, deflection, dodge, and Dexterity capped by the armour's max Dex), for the character and, after [174](174-animal-companions.md), the companion.
5. **The other items:** all of the guide's unique items (each slot's, rods, robes, usables) are in. An always-on bonus to a number the planner computes counts: an extra attack at full bonus (Hasting Expanse), flat damage (Goggles of Ferocious Pact), per-hit dice (Bracers of Slaughter), caster level and DCs, skills. What isn't a number stays text.
6. **Conditional bonuses** (user, against the recommendation: "b"): every condition counts, in a row that names it ("+2 vs undead", "+1 DC with fire spells"), as 174 counts the horse's "vs. Medium or smaller".
7. **Stacking:** settles [144](144-bonus-types-and-stacking.md) with the guide's own words. Feats and features take a bonus type where the guide names one and are untyped (always stacking) where it doesn't. The user no longer owes a list.
8. **Relics and artifacts** (user: "defer relics and artifacts for a subsequent ticket"): their own ticket, with the upgrade-tier question.
9. **Conditional rows and the headline:** a conditional bonus is shown beside the line, not summed into its headline number. Each line gains a second figure "with conditions" that names them. `compare` and `improve` rank by the headline unless asked for `with_conditions: true`, as they rank naked unless asked for gear. This also holds for 174's horse bonus.
10. **Build order**, one ticket each: weapons (1–3); the other items (5–6); stacking (7, closing 144); armour class (4).

## Done when

The user has decided each question, the decisions are in the Answer and on the map, and the build is ticketed.

Built by: [221](221-gear-weapons.md) weapons, [222](222-gear-other-items.md) the other items, [144](144-bonus-types-and-stacking.md) stacking, [223](223-armour-class.md) armour class, in that order; [224](224-relics-and-artifacts.md) relics and artifacts (design).
