# Assumed gear and buffs for the numbers

Type: design
Status: open
Blocked by: a scope decision for the user (gear is on the map's Out of scope). The guide's item pages are saved (2026-11-09).
Map: [WotR Build Planner](../map.md)

## Question

The map's Out of scope keeps gear out of a Build, gear bonuses toward prerequisites included (user, 2026-10-31). That rule is about legality, and it stays. But the numbers the planner now computes ([114](114-attack-line-and-damage.md)'s attack line, [119](119-caster-line.md)'s caster line if built) are always without gear, so they understate a level 20 character by a lot:
- The Max DC run (2026-11-09) ended at Charisma 25. The agent guessed "a +6 headband and inherent bonuses" add 3–5 DC, from memory. The saved item pages show the guide lists +8 headbands, so the guess was low.
- The same holds for an attack line without a weapon's enhancement or a belt.

## What the saved pages hold

The guide's three item pages were added to `PAGES` in `scripts/save-guide-pages.sh`, and the user saved them on 2026-11-09 (`guide-cache/SAVED_AT`). Read the same day:

- **`items`:** mundane weapons, armour and shields; unique items by slot (belts, boots, bracers, cloaks, glasses, gloves, headwear, necklaces, rings, rods, robes), each with its properties, cost and location; artifacts and relics; magic books; skill boosting items. For example:
  - Charisma: enhancement bonuses from +2 up to +8 (Headband of Mental Perfection +6 and +8, Razmir's Mask +8, Darkness Caress +8: Intelligence, Wisdom and Charisma; Crushing Presence +6: Charisma);
  - inherent bonuses: the manuals and tomes at +2 per ability, and items giving +1, +2 or +4 to all ability scores.
- **A "Caster Level and Spell DC Boosting Items" section** in `items`, two tables of item, bonus, circumstance and location:
  - CL boosts (Sin Mage's Staff +3, Robe of the Seven Sins +3, Phantasmal Guide +3 with illusion, ...);
  - DC boosts, about 30 of them. Examples: Heartstone of the Night Hag Queen +3; Rod of Power Source +4 with conjuration; Cloak of the Ancient Herald +2 with compulsion; Mindmaster's Eyes, Twisted Temptation and Ring of Chaotic Fascination +2 with enchantment; Quarterstaff of the War Mage +2; Robe of Determination +2 against Fortitude.
- **`items-inevitable-excess` and `items-lord-of-nothing`:** the DLC items, by the same slots (Flawless Headband of Mental Perfection +8 again, Headband of Confidence +4).

## Options

- **A. Nothing:** the numbers stay without gear; the agent adds gear in words.
- **B. An "assumed" overlay on the numbers only:** `sheet` and `compare` take an optional `assume` argument (`{"cha": 6, "inherent_cha": 5, "weapon_enhancement": 5}`), applied to the attack and caster lines and shown beside the unassisted numbers. It's not part of the Build or its link and never touches a prerequisite.
- **C. Gear in the Build:** slots and items in the link. Against the map's scope rule; not recommended.

## Open questions (with recommendations)

1. **Which option?** *Recommend:* B. It keeps the scope rule for legality and lets the numbers answer "how high at the end of the game", and the agent says what it assumed.
2. **Which assumptions?** *Recommend:* a fixed list of generic bonuses (enhancement to each ability score, inherent to each, weapon enhancement, a resistance bonus to saves), with the agent supplying the numbers, so no item catalogue is needed. Each generic bonus's highest value in the guide (+8 enhancement to Charisma, for one) is given in the tool's description, from the saved pages, so the agent assumes what the game has.
3. **Named DC and CL items?** The DC and CL tables are small (about 30 and 10 rows) and their bonuses are conditional (a school, a subschool, a save), like [119](119-caster-line.md)'s sources. *Recommend:* `assume` takes items from those two tables by name, and the caster line applies each where its circumstance matches; no other named items.
4. **The web app too?** *Recommend:* not yet; the MCP server first, where the run showed the need.
5. **The map's Out of scope entry:** today it reads "Gear and companions: not part of a character's build choices", including gear bonuses toward prerequisites. *Recommend:* whichever option is taken, the answer rewrites that entry so the map says what's now covered. With B: gear stays out of the Build, its link and every prerequisite, and assumed bonuses on the computed numbers are in scope (this ticket); companions stay out. With C: gear leaves Out of scope for Notes. With A: the entry stays and names this ticket as the decision.

## Done when

The user decides. If B: `sheet` and `compare` with `assume` give both numbers, the assumed ones labelled, with server tests; a prerequisite never sees an assumed bonus (a test); each highest value and named item cites its saved page; `scripts/check.sh` passes.

Whatever the option, the map is updated with it:
- its Out of scope entry on gear is rewritten as in question 5;
- a Decisions line is added for this ticket;
- if B, the attack line's fixed `leaves_out` list (from [114](114-attack-line-and-damage.md)) and the MCP instructions' honesty boundary say that gear is left out unless assumed.
