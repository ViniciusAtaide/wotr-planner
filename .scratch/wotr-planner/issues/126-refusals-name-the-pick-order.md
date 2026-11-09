# Refusals name the pick order within a level

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

In the Max DC run (2026-11-09, see [119](119-caster-line.md)) one `choose` batch put Spell Focus (Enchantment) in `slot.feat.1.general.0` and Greater Spell Focus (Enchantment) in `slot.feat.1.class.0`, the sorcerer's level-1 Magic feat. The second was refused:

> `unmet`: "Needs Spell Focus in the same school"

Spell Focus *was* held at level 1. The refusal is right: ticket 32 set the pick order within a level (class selections, class feat slots, race slots, then the general slot), so the class slot is checked before the general slot's pick exists. But nothing in the reason says so, and the agent guessed the cause and worked around it by moving Greater Spell Focus to level 3. The fix it missed was to swap the two slots.

## Scope

1. **The reason:** when an `unmet` prerequisite is met by a pick in a later slot of the same level, the reason says so and names that slot: "Needs Spell Focus in the same school: Spell Focus (Enchantment) is in slot.feat.1.general.0, which comes after this slot in the level's pick order; swap them." In the engine's reason text, so the web app's picker says it too.
2. **The instructions:** one bullet in "Before you plan": within a level, class selections and class feat slots come before race slots and the general slot, so a feat that needs another taken at the same level goes in the later slot.

## Open questions (with recommendations)

1. **Only feats?** *Recommend:* any prerequisite leaf a later same-level pick would meet (a feat, a class feature from a later selection), found by checking the level as if that slot came first; no new rule.

## Done when

A rules test (the sorcerer's level-1 case) and a server test give the new reason; the instructions bullet is in, for the user's review as in ticket 117; `scripts/proof.sh` shows only reason text changed; `scripts/check.sh` passes.
