# Athletics III raises the base attack bonus

Type: design
Status: open
Blocked by: none
Model: Sonnet to build; Opus to review.
Map: [WotR Build Planner](../map.md)

## Question

Found by the third martial run (2026-11-14): two agents built the same "maximum weapon damage per round" character, one from the saved guide only and one through the MCP only, and both settled on a Human Two-Handed Fighter 20. The guide-only Build, entered into the planner, holds the Trickster's Athletics III (MR 7) with 20 Athletics ranks. The sheet still gives BAB 20 at level 20, and the attack line +29/+24/+19/+14.

The guide (`trickster#athletics-iii`): "Your BAB is at least your Athletics rank. If your BAB is equal to your character level, it is at least your Athletics rank + 5." The planner's summary says the same. Nothing in the engine applies it, so:

- **The numbers:** a full-BAB character with maxed Athletics should read BAB 25 at level 20 (21 from MR 7's level on). The guide-only agent counted +5 to hit, a fifth iterative attack and a seventh Power Attack step; the MCP agent could only guess at them and marked the Trickster branch "contested".
- **Prerequisites:** for a character whose BAB lags its level (a wizard or a rogue with Athletics ranks), the trick raises BAB to the ranks. A BAB prerequisite it opens is refused today.

Questions:

1. **Where it counts.** In BAB on the sheet, the attack line, and every BAB prerequisite from the level the rank is taken at? *Recommend:* yes, all three, as an always-on effect of a held pick. The walk already holds what a Mythic rank gives "only from its level".
2. **The fifth attack.** Does a BAB of 21 or more give a fifth iterative attack (at −20) and a seventh Power Attack step (1 + ⌊BAB/4⌋)? The guide states no iterative rule past 20. The guide-only agent inferred one from the Legend section's note that attacks keep coming every +5 BAB. *Recommend:* count the extra iterative and the Power Attack step, citing that note in the row, and say so in `leaves_out` until the user confirms it from the game.
3. **Its order in the walk.** Athletics ranks are spent at each level, before or after the level's feats. A BAB prerequisite at the level the trick is taken depends on the ranks the level adds. *Recommend:* ranks first, the order a level's choices are made in the game.

## Answer

**Recommendations taken** (user, 2026-11-14, in a grilling session):

1. **Where it counts:** everywhere a BAB counts: the sheet's BAB, the attack line, Power Attack's steps and every BAB prerequisite (feats and a prestige class's entry), from the level MR 7 is taken at.
2. **Extra attacks:** a raised BAB gives the attacks any BAB gives. The engine's `iteratives` and `power_attack` already follow the guide's Legend rule ("an additional main hand attack each additional +5 BAB", `other-mythic-paths#playing-a-legend`), so a BAB of 25 gives five attacks and seven Power Attack steps. The row cites that note.
3. **"Athletics rank"** means the ranks spent, not the skill's total.
4. **Same-level order:** the trick counts from the next level's picks. At MR 7's own level, the class and general feat slots are checked before the rank's picks, as for every Mythic pick (ticket 32).

## Done when

The guide-only Build from this run (link in the map's History entry for 2026-11-14) reads BAB 25 at level 20 and 21 at level 16, and its attack line has the attacks question 2 decides. A non-full-BAB Build with Athletics III opens a BAB-gated feat it couldn't before. Real-data tests cover both. The snapshot check is regenerated if a corpus Build holds the trick, with each changed line following from this. `scripts/check.sh` passes.
