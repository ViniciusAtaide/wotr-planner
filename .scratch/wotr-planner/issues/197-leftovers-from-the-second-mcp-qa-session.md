# Leftovers from the second MCP QA session

Type: bugfix
Status: open
Blocked by: none
Model: Sonnet to build; Opus to review the wording.
Map: [WotR Build Planner](../map.md)

## Problems

Found by the second MCP QA session (2026-11-14, on `main` at `8a457a0`), after tickets 188 to 193. What that queue's own changes caused was fixed the same day (ticket [192](192-refusals-that-dont-say-why.md)'s "QA session fixes"); these are older, or were left as settled. Each is as the session reported it; reproduce before building.

1. **A refused skill step names the wrong skill.** A sorcerer 6 / Dragon Disciple 10 / sorcerer 4 with 4 skill points at level 20, `level.20.skill` `{"knowledge-arcana":1,"persuasion":1,"use-magic-device":1,"perception":2}`: "At Use Magic Device ranks at level 20, 0 → 1: there is no such step (a level adds ranks only up to its skill points, and no skill's total goes above the character level)." The map spends 5 of 4 points; the text blames whichever skill is applied last. Say the level has 4 points and the map spends 5.
2. **A valid code reported as a stale link while the server restarts.** `options` on a valid code returned once: "`stale_link`: The Build code doesn't open: it is damaged or hand-edited, or the game data changed since it was made." The same call succeeded on retry. Find what a request sees while the server is starting (the data not yet read?) and answer with an error that says so.
3. **A refusal that lists every option of a selection.** Extra Rage Power for a character with no rage power: "Needs Animal Fury or Beast Totem or … or Wrecking Blows", all 38, about 1,000 characters; Extra Rogue Talent the same. The shortened form `search` uses ("one of 11 alternatives (…)") would do, or a name for the group ("a rage power").
4. **A name with a comma inside a comma list.** "Needs Barbarian level 6 (have 2), Beast Totem, Lesser" reads as three needs.
5. **A value quoted as escaped JSON.** "`\"scholar\"` isn't an option for `start.background`. Closest: Nexian Scholar (`\"nexian-scholar\"`)".
6. **A closest match that isn't close.** The unknown feat `mobility` in `validate`: "Closest: Ability Focus - Bombs (`ability-focus-bombs`), Combat Mobility (`combat-mobility`)".
7. **The parameterised-feat hint on a plain feat key.** A `not_an_option` for a feat slot still ends "with `values: true` for a parameterised feat's values" when the value given is a plain key.
8. **Two forms for the same need.** The unmet reason says "Needs Str 13 (have 12)" and the breaks-later reason "needs Strength above 12".
9. **`describe` lists a feature gained twice as two entries.** The Trickster's rank 3 lists Sneak Attack twice; `sheet` and `compare` give one entry with `times: 2` since ticket 192's review fixes.
10. **A locked move of a rank doesn't name the pick when the rank is the one moved.** On ticket 192's item 8 Build, `options` on `mythic.4.level`: "Stat Focus (AC) needs MR 4 earlier than level 13"; on `mythic.3.level` the text names "MR 4's Perception II". Settled so in 192; the session read it as a gap.
11. **A rank's pick whose prerequisite is a feat of a later level doesn't say where the feat is.** `slot.mythic.4` `toughness-mythic` with Toughness at level 15 and MR 4 at 11: "Needs Toughness. MR 4 taken at level 15 opens it, which moves MR 5 to level 15: `choose` `mythic.4.level` 15 first" (ticket 171's hint). Ticket 192's item 7 says where the pick is for the reverse case.
12. **The Dragon Disciple's "no Bloodline yet"** still doesn't say what the alternative is (a character with no bloodline takes one through the class), nor what the character holds. Settled as a dedupe in 192; the session read the ticket as asking for more.
13. **`search` matches on "of" and "the".** "Defender of the Hearth" returns the archetype, then 87 summary matches.
14. **`improve` on the slot that holds the Build's Weapon Focus** returns `options: []` with `closed: 895` rather than saying why nothing is ranked.

Not problems, checked: "Knowlege (World) III" is the guide's spelling; Favorite Metamagic's parameter is a feat key, which the tool docs' list of parameter kinds leaves out (add it there).

## Done when

Each is fixed or struck with the reason; tests pin the new texts; the snapshot check's lines move only for these; `scripts/check.sh` passes.
