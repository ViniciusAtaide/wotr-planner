# Equipment: how to introduce gear

Type: design
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The user, 2026-11-11, on the gold builds ([160](160-gold-builds.md)): "for that we need to introduce gear support so before implementing I have to plan how to introduce equipment".

What raised it: the Enforcer write-up takes Greater Two-Weapon Fighting at level 19. The feat needs Dex 19 and the Build's own Dex is 17. In the game a belt covers the gap, so the build works there and is refused here.

This reopens decisions on the map, which the plan has to revisit one by one:

- **Out of scope, "Gear and companions"** (user, 2026-10-31 and 2026-11-01): gear stays out of the Build, its link and every prerequisite; prerequisites check the character's own scores, never a gear bonus.
- **Assumed gear on the computed numbers** (user, 2026-11-10, [ticket 148](148-naked-numbers.md), reversing [ticket 122](122-assumed-gear-and-buffs.md)): the numbers are naked, and `assume` and `gear.ron` were removed.
- **Won't do:** [145](145-assumed-gear-in-the-web-app.md), assumed gear in the web app.

The user plans this before anything is built. Questions the plan has to answer, none decided yet:

- What gear is for: only to satisfy prerequisites (a belt's Dex for a feat), or also the computed numbers (attack, AC, DCs)?
- Whether gear is part of the Build and its link, or an assumption beside it. The link's format and its v1 keys are pinned today.
- How much is modelled: a bonus to a score from a named slot (belt, headband), or items as game data with their own catalogue.
- When a gear bonus counts: from the level it is equipped, or as a flat assumption for the whole Build. A prerequisite met by gear at level 19 depends on this.
- How the app and the MCP say that a pick is open only because of gear.
- What the gold builds assert once gear exists: the write-ups name no items.

## Done when

The user has decided each question, the decisions are in the Answer and on the map (its Out of scope entry rewritten), and the build is ticketed.

## Answer

**Decided** (user, 2026-11-11, in a grilling session; where not quoted, the user took the recommendation).

What gear is:

- **Named items from the guide,** one per slot, two rings. "Named items but with only a summary of the bonuses, no descriptions, I like how races had a small summary bellow telling its main characteristics. equipments should be the same."
- **Part of the Build and its link,** as a last section written only when the Build has gear. A Build has no gear unless the user adds it; the planner never assumes any.
- **Independent of level,** on a tab of its own. "Gear bonus count independant on level. We can select any gear we want since we know we will not get it at level 1. It's like another screen/tab overall independant on the level screen."
- **Two texts per item:** a summary generated from the modelled bonuses, and the other effects in our own words, in the item's detail. "we can leave an extra section on the equipment when you hover it you can see "On Crit: ..." since those effects never change the overall scores". On touch the detail opens by tapping (map, Mobile: no hover-only interactions). The guide's text is not copied.

What gear does:

- **It reaches every computed number** (Q1: "c"), through an item's always-on bonuses to numbers the planner already computes. Conditional and activated effects are text only. No new numbers (armour class) in the first stage.
- **A score bonus counts at every level,** for prerequisites and for everything derived from the score. The planner doesn't check when an item could be had.
- **Skill points per level never see gear,** so ranks stay valid whatever is worn.
- **Stacking:** bonuses carry a type; the highest of each type per target, and different types add. This is ticket 144's rule, applied to gear first.
- **Gear counts before "highest score" features** (Q23: "b"), so a belt can move Perfect Body's bonus.
- **Both numbers are kept.** "we can have both numbers one with gear and one without." The summary and the MCP's `sheet` show both when they differ. `compare` and `improve` rank by the naked number unless asked.
- **A pick open only because of gear** is marked with the item's name. If the item is removed, the pick stays and is refused with the usual reason.

Scope and order:

- **First stage:** the items that give an ability score bonus, about 77, from the base game and both DLCs. **Second stage:** weapons, armour and the rest, planned in its own design ticket.
- **Companions stay out of scope.**
- **The Enforcer** (ticket 160) wears the weakest item that makes its level 19 legal, a belt with Dex +2.
- **The text export** gains gear lines with new keys, only when the Build has gear.

The questions this ticket listed, as answered: gear is for prerequisites and the numbers; it is part of the Build and its link; items are a catalogue; a bonus counts from level 1; a gear-dependent pick is marked; the gold builds assume the smallest item that fits.

Facts the plan rests on: one function writes the ability scores every reader uses (`Walker::refresh_ability_scores`); the engine knows no bonus types today; the link can take a trailing optional section (ticket 91's precedent); no item data is extracted; the engine computes no armour class; the saved guide pages don't say whether item bonuses count for prerequisites, and the user's knowledge that the Enforcer works in the game is the evidence.

Tickets, all waiting for ticket 158's agent ("don't change any code yet", user):

- [165](165-gear-data.md): the data.
- [166](166-gear-in-the-engine.md): the engine. After 165, since both edit the game data's types and the codec.
- [167](167-gear-tab.md): the web app's Gear tab.
- [168](168-gear-in-the-mcp.md): the MCP, the skill, the map's Out of scope entry, and the Enforcer's belt.
- [169](169-gear-second-stage.md): the second stage's design.
