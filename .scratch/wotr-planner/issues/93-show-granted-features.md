# The Sheet shows no granted features

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Found by the QA builds ([ticket 88](88-qa-full-builds.md)). Ticket 88 asked that features granted automatically (Woodland Stride, Studied Target +2, sneak dice) be "checked on the sheet". They can't be: the Sheet shows no granted features anywhere. A level row shows only its slots (class, feats, picks, skills); the summary shows classes, BAB and saves, ability scores, faith, class skills and the daily choices (`libs/ui/src/grid.rs`, the summary view); the Mythic track shows only its slots. So a player can't see:

- class features by level: the Sylvan Sorcerer's Animal Companion (1), Woodland Stride (3), Fleeting Glance (9), Fey Magic (15), Soul of the Fey (20); the slayer's Studied Target and Sneak Attack dice; the rogue's Evasion, Danger Sense, Debilitating Injury; the Sylvan Trickster's Resist Nature's Lure;
- what a pick brings: the Second Bloodline (Draconic Bronze)'s Draconic Arcana, Grow Claws, resistances, Wings and Power of Wyrms;
- a path's rank features: the Azata's Instrument of Freedom, Freedom's Child, Hard to Kill, Dragon Companion, the Songs and Summons.

**Repro:** either final link in ticket 88's Answer; nothing on the Levels tab or the Mythic track names any of the above.

**Expected:** the Destination's "summary up to character level 20" lets a player follow the build in the game, and the engine already walks these features (prerequisites check them: `Feature(feature: …, count: n)`). Not a guide disagreement but a missing view, so it's the user's call whether the Sheet should list them.

**Recommendation:** list each level's newly granted features under its row (one muted line, names only, repeats collapsed with a count such as "Sneak Attack (3d6)" if the engine has the count), and each mythic rank's path features under its rank on the Mythic track. Fold the same lines into [ticket 87](87-build-sheet-export.md)'s build sheet if it isn't built yet.

## Build

Once the user decides: an `Engine` method that returns a level's (and a rank's) newly granted feature names (test-first in `rules`), and the UI lines; an end-to-end smoke check that Woodland Stride shows at Sylvan Sorcerer 3.

**Done when:** the decision is recorded here and, if accepted, the Sheet names the features each level and mythic rank grants, checked on ticket 88's two Builds.
