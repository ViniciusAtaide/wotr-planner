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

## From the Legend QA (2026-11-17, `main` a129b11f)

A Legend to level 40 passed every check of levels 21–40 (caps, feats, increases, skill points, BAB, saves, the attack and caster lines by hand; the caster level 39 is the game's: WizardSpellLevels 78bb94ed2e75122428232950bb09e97b has 41 rows, so `Spellbook.MaxCasterLevel` is 40, WrathDecompiled 6ce4a6b `Spellbook.cs`). These texts, all older than the 308 queue, read wrongly:

6. **The Legend switch is never named when it is what breaks.** `options mythic.2.level` on a Legend caster refuses "Level 21, which moves … MR 8 to level 21" with "Level 18: Skill ranks, Level 19: Skill ranks, Level 20: Skill ranks and 2 more need MR 2 earlier than level 21"; `withdraw mythic.switch` reads "Would break Level 18: Skill ranks, … and 40 more", skill ranks ahead of the 20 levels past 20 (ticket 171's wording).
7. **`sheet` doesn't show the Mythic path choices.** `mythic.first-ascension`, `mythic.path` and `mythic.switch` never appear as picks with an address; the MR 8 row reads `{"rank":8,"status":"not_reached"}` though Legend was taken there.
8. **Legend without a path.** On a fighter 20 with no first ascension or path, `choose [mythic.switch: legend, level.21.class: barbarian]` is applied. Check against the game whether MR 8's switch needs a path, and whether open choices make this by design.
9. **A vague skill-rank refusal.** `choose level.40.skill.stealth 2` gives "At Stealth ranks at level 40, 1 → 2: there is no such step (…)" when the level's points are spent (Stealth at 23 of 40); it should name the spent points.
