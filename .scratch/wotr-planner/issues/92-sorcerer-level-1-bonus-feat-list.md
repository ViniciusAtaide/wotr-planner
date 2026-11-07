# The sorcerer's level-1 bonus feat: which list?

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Found by the QA builds ([ticket 88](88-qa-full-builds.md)). Since [ticket 44](44-sorcerer-feat-slayer-deities.md), every sorcerer's level-1 "Bonus Feat" is a slot of the held bloodline's bonus-feat list (`OptionList(selection: "sorcerer/bloodline")`). [Ticket 43](43-extraction-report-gaps.md)'s question 1 chose that reading because the guide never says what the level-1 feat is, and said "if you can check in the game, go by it". The Jack-in-the-Green write-up (a guide-style sample build of a Sylvan Sorcerer) takes **Spell Penetration** as its level-1 "Bonus Feat". Spell Penetration isn't on the Fey list, so the app can't build it.

**Repro:** the Jack-in-the-Green Build at level 1 (`#b=AQFHALj_C3B5t9x-n6bdRmDXQFRsC6Yx6ecNG2DXQCrWmNBTLAktDgABAQADAQEMDg4MCBEBAAMBAQEBAQAAAAEAAgEBAAAAAAIAAAA`), level 1, the "Bloodline feat" slot. It offers Dodge, Improved Initiative, Lightning Reflexes, Metamagic (Quicken Spell), Point-Blank Shot and Skill Focus; locked are Agile Maneuvers ("Needs Bull Rush or Dirty Trick or Sunder Armor or Trip") and Precise Shot ("Needs Point-Blank Shot"). Spell Penetration isn't listed. (In ticket 88 the QA build took Spell Penetration in the level-1 general feat slot instead, which drops the write-up's Mounted Combat, and put Dodge in the bloodline slot.)

**The guide:** every sorcerer table but the Overwhelming Mage's has "Bonus Feat" at level 1 in the column that holds "Bloodline Feat" at 7, 13 and 19 (`sorcerer#sylvan-sorcerer`, `sorcerer#fey`, …); `character-creation#bloodlines` names only 7, 13 and 19 as bloodline-feat levels. The Fey bonus-feat list (`character-creation#fey-bloodline`) is Agile Maneuvers, Dodge, Improved Initiative, Lightning Reflexes, Quicken Spell, Point-Blank Shot, Precise Shot and Skill Focus. The write-up's level 1 reads "Bonus Feat - Spell Penetration; Mounted Combat": two feats, one of them off the bloodline list.

**The options:**
1. Keep the bloodline list (today); the write-up is wrong.
2. Any feat (a General slot), as the Geomancer has: the level-1 feat isn't tied to the bloodline, only 7/13/19 are. This matches the write-up and `character-creation#bloodlines`.
3. A Magic feat (Spell Penetration is one; Mounted Combat is the write-up's general feat).

**Recommendation:** 2, if you can confirm in the game that a sorcerer's level-1 bonus feat isn't limited; it fits both the write-up and the character-creation sentence. Otherwise keep 1 and close this ticket.

## Build

Once the user decides: a data change in `libs/game-data/classes/sorcerer.ron` (each row-1 `OptionList` slot that becomes General or Magic), a `libs/game-data/README.md` note, and real-data tests in `libs/seeds/tests/game_data/` replacing ticket 44's level-1 tests; the behaviour proof shows only the row-1 slots move.

**Done when:** the decision is recorded here, and, if it changes the slot, a Sylvan Sorcerer 1 can take Spell Penetration in the level-1 bonus slot and Mounted Combat in its general slot.
