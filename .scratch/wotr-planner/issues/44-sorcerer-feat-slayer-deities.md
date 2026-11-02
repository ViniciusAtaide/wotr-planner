# Sorcerer level-1 feat, Extra Slayer Talent, deities

Type: task
Status: open
Blocked by: 43
Map: [WotR Build Planner](../map.md)

## Question

Build [ticket 43](43-extraction-report-gaps.md)'s decisions 1, 3 and 4 (user, 2026-11-02):

1. **Sorcerer level-1 bonus feat.** Every sorcerer, base included, gets a level-1 bonus feat slot from the held bloodline's bonus-feat list (`OptionList(selection: "sorcerer/bloodline")`, as at 7/13/19, prerequisites as there). Crossblooded picks from either of its two bloodlines' lists; the Empyreal, Sage and Sylvan from their fixed bloodline's; the Seeker and Nine Tailed Heir from their chosen bloodline's (the Seeker's current Magic-feat slot at level 1 is replaced; its 5/10/15 stay); the Geomancer keeps its General slot. The Overwhelming Mage is unchanged (it gives the feats up). The base sorcerer gains the slot it lacks today.
2. **Extra Slayer Talent** needs slayer level 2 (`ClassLevel(class: "slayer", level: 2)`), the level the slayer's table first gives Slayer Talents. Talented Slayer stays out (not on any saved page).
3. **Deities stay out of scope.** The cleric's deity-alignment item leaves the report; add "Deities: a cleric's deity and its alignment and domain limits" to the map's Out of scope.

## Build

AFK, test-first (`tdd`); the only agent in the repo. Same method as tickets 37, 38 and 43: rules in the extractor (extend `extract::report_gaps`), a one-off rewrite of `game-data/`, real-data tests first (red run saved), a fresh extraction that matches the rewrite, a before/after proof, the report's items removed (it should end with 0 items: the kineticist item moves to ticket 45, so mark it there instead of in the report, or leave it in the report pointing to ticket 45, whichever the report's conventions suit).

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a sorcerer 1 with the Draconic bloodline is offered its bloodline feats and not Power Attack at level 1; a Crossblooded sorcerer is offered both bloodlines' lists; an Empyreal Sorcerer its fixed bloodline's; a Seeker at 1 no longer a Magic feat; a Geomancer still any feat; Extra Slayer Talent locked for a slayer 1 ("Needs Slayer level 2 (have 1)") and open at 2, and locked for a rogue.

## Progress log

## Answer
