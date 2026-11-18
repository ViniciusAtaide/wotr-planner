# Older texts and a hint the 308 queue's QA found

Type: build (texts)
Status: open
Blocked by: —
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

The QA session after the 308 queue (2026-11-17, `main` a129b11f) found these, judged older than the queue (inferred: the code that shapes them isn't in the queue's tickets):

1. **`validate` drops the "take the rank later" hint.** A Living Deity arcanist 20 with `slot.mythic.9: impossible-domain` and no `mythic.9.level`: `validate` returns only `needs`, while `choose` says "… MR 9 taken at level 20 opens it, which moves MR 10 to level 20: `choose` `mythic.9.level` 20 first". `validate`'s description says the reason is left out only when it repeats the `needs`.
2. **A refused Extra feat's slot pick is a separate failure.** Instinctual Warrior 3 with `extra-rage-power` and a pick at `slot.feat-selection.3.0`: `validate` lists `{"issue":"not_reached","label":"Level 3: slot", …}` apart from the feat's refusal, not under it, and "slot" names nothing.
3. **"N more of" when none is held.** "Needs 1 more of: Bomb, Arcane Bombs" and "(2 more of: Diva Style, Feint)" read as if some were held.
4. **A nested label:** "Bloodline (Seeker (Sorcerer))".
5. **The Magic Deceiver's summary** says "spells to 6th level"; its rows give arcane spell level 9. Check which is the game's and fix the other.

## Done when

Each reads as its rule through `validate`, `options` and `choose`, with a test of the exact text; the Magic Deceiver's spell level is cited from the game.
