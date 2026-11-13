# The Invulnerable Rager's Damage Reduction carries the barbarian's summary

Type: bugfix
Status: resolved
Blocked by: none
Model: Sonnet to build.
Map: [WotR Build Planner](../map.md)

## Problem

Found by a QA session through the MCP server (2026-11-13), still so on `main` at `ce9cc3b`. The Invulnerable Rager's rows grant `barbarian/damage-reduction` at 2, 4, 6, …, 20, as its table in the guide does ("Damage Reduction" in every even row; the class overview's "10/- DR"). But the feature is the barbarian's own, so the sheet names it "Damage Reduction (Barbarian)" and its summary says "DR 1/- from 7th level, rising by 1 at 10th, 13th, 16th and 19th" (`libs/game-data/classes/barbarian.ron`), wrong for every grant the archetype gets.

## Build

- A feature of the archetype's own, `barbarian/invulnerable-rager/damage-reduction`, named "Damage Reduction (Invulnerable Rager)", with a summary from the guide's table and overview (DR 1/- at 2nd, +1 at each even level, 10/- at 20th), on the archetype's rows in place of the barbarian's.
- First check what reads `barbarian/damage-reduction` (prerequisites such as Increased Damage Reduction, `bonuses.ron`), and keep each reader working for the Invulnerable Rager as the guide intends.

## Done when

The Invulnerable Rager's sheet shows its own Damage Reduction at the even levels with a true summary; nothing that read the barbarian's feature changes for the archetype unless the guide says so; a real-data test pins it; `scripts/check.sh` passes.

## Answer

Done 2026-11-13.

1. **The premise was half right.** The archetype's rows granted `barbarian/damage-reduction` at 2, 4, 6, 8, 12, 14, 18 and 20 only. It had no rows at 10 and 16, so those two levels fell back to the base class's rows and took the barbarian's Damage Reduction from them (a row's absence is the base row). Swapping the key in the existing rows would have left the barbarian's feature at 10 and 16.
2. **The data.** `barbarian/invulnerable-rager/damage-reduction-invulnerable-rager`, named "Damage Reduction (Invulnerable Rager)", links to `barbarian#invulnerable-rager`, summary "DR 1/- at 2nd level, rising by 1 at every even level after that, to 10/- at 20th." The archetype's rows at 2, 4, 6, 8, 12, 14, 18 and 20 grant it in place of the barbarian's, and two new rows, 10 and 16, carry what the base rows give there (the new feature and the rage-power selection). It has the ten grants of the guide's table (every even row, 2 to 20) and the class overview's 10/-; the guide has no sentence for this feature, so the size of each step is read from those.
3. **Readers.** Nothing reads `barbarian/damage-reduction` but the barbarian's own rows and the `class-barbarian` snapshot (the base class, unchanged: the feature still reads "Damage Reduction (Barbarian)" since the skald and others share its name). Increased Damage Reduction needs barbarian level 8 only, so the Invulnerable Rager can take it as before. No prerequisite, bonus or interaction needed a change; no snapshot changed.
4. **Tests** (`libs/seeds/tests/game_data/scaling_features.rs`): `the_invulnerable_ragers_damage_reduction_is_its_own_at_every_even_level`, `the_barbarians_damage_reduction_is_not_the_invulnerable_ragers` (the class at 7, 10, 13, 16, 19; the Mad Dog at 10, 13, 16, 19; the Flesheater, Instinctual Warrior, Pack Rager and the Invulnerable Rager none), `the_two_damage_reductions_are_named_for_their_owners`, `the_invulnerable_rager_has_a_rage_power_at_every_even_level`.
5. **Decisions** (decided by the main session, 2026-11-13, under the user's "answer the easy ones"): the key follows the data's precedent of slugging the parenthesised name (`bloodrager/reformed-fiend/damage-reduction-reformed-fiend`, `barbarian/instinctual-warrior/greater-focused-rage-instinctual-warrior`, ...), not the shorter key this ticket first named; the summary is the one above; rows at 10 and 16 are added; the guide link is the archetype heading.
6. **For ticket 188:** the Invulnerable Rager has its own rows at every Danger Sense step level (6, 9, 12, 15, 18), so stepping the barbarian's Danger Sense needs no row change for it; its `assert_gains` exception is `Some(&[])`. The new rows at 10 and 16 are not step levels of Danger Sense or Extreme Endurance and must stay free of them.
