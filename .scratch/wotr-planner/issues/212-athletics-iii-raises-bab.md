# Athletics III raises the base attack bonus

Type: design
Status: resolved
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

### Built (2026-11-14)

- **What was built:** `SelectionOption.bab_from_ranks: Option<BabFromRanks { skill, extra_at_full_bab }>` (data on the option row, as `Feature.bab_floor` is for Perfect Mind; the rule is in Rust). Set only on `trickster/greater-mythic-trick/athletics-iii`: `(skill: "athletics", extra_at_full_bab: 5)`. `Walker::refresh_bab` (`walk/hold.rs`) sets `state.bab` to the larger of the class tables' BAB (`Walker::class_bab`, set in `recompute_tables`) and, for each held option with the field, the ranks spent in the skill (+5 while the class tables' BAB equals the character level). It runs after each level's skill ranks, when such an option is held and when one is released (a Legend switch). Every reader of `state.bab` (sheet, attack line, Power Attack, prerequisites, prestige entry) sees it; no `apps/` or `libs/ui` change. The data check names the skill. The existing option lookup (`catalog.option`) serves, so no new catalog index.
- **Decisions:**
  1. The user (Answer 1 to 4): everywhere a BAB counts; the second attack rule; ranks spent, not the total; the level's class and general slots don't see the trick, later slots do.
  2. The main session: the Answer's "iteratives already follow the Legend rule" was wrong (`attack.rs` capped a full attack at four); the cap is now eight, the guide's Legend maximum (`other-mythic-paths#playing-a-legend`), so BAB 21 gives five attacks. `power_attack` was uncapped already.
  3. The main session: the raise counts for later picks inside MR 7's own rank (immediate on hold); a prestige entry at the level after MR 7 is checked against the character before that level, as every number is.
  4. The main session: data with the rule in Rust. Inference (mine, not the guide's): "BAB equal to your character level" compares the class tables' BAB, so the +5 doesn't feed on itself.
- **Tests:** `libs/seeds/tests/game_data/athletics_iii.rs` (4: the guide-only Build reads BAB 15/21/22/25 at 15/16/17/20, attacks 30/25/20/15/10 at 16 and 34/29/24/19/14 at 20, Power Attack penalty 6 and 7; the ranks spent, not the total; a trickster wizard opens Critical Focus at the next odd level only with Athletics III; the only option with the field); `libs/rules/tests/engine/path_rules.rs` (4: full-BAB class, lagging class, the next level's slot and not the level's own, Legend switch takes it back), `attack_line.rs` (the fifth to eighth attack), `check.rs` (unknown skill). The test world gained an invented `trickster/tricks/athletics-3` and a `critical-focus` feat (BAB 12). The planned `files.rs` test is in `athletics_iii.rs` instead (`files.rs` reads the fixture, which has no Athletics III).
- **Snapshot message:** none; the check stays silent (no corpus Build holds the trick; the snapshots hold no attack line).
- **Review fixes:** the build guide's line (`martial.md`) now says the raise counts from the level after the trick, covers MR 10's pick and the full-BAB condition; a test with MR 7 at level 17 asserts the level's own slot doesn't see it; the "only option" test also covers archetypes and prestige classes; `refresh_bab` checks a catalog list of the options that carry the field (`Catalog::bab_options`) instead of scanning every held option; the test world's `critical-focus` comment says BAB 12 here, 9 in the game.
- **What the queue's proof should show:** BAB and attack-line differences only for Builds holding Athletics III (a Trickster with the trick), and a fifth or later attack for any Build with BAB above 20 (Legend Builds).

## Done when

The guide-only Build from this run (link in the map's History entry for 2026-11-14) reads BAB 25 at level 20 and 21 at level 16, and its attack line has the attacks question 2 decides. A non-full-BAB Build with Athletics III opens a BAB-gated feat it couldn't before. Real-data tests cover both. The snapshot check is regenerated if a corpus Build holds the trick, with each changed line following from this. `scripts/check.sh` passes.
