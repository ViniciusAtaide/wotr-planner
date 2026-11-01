# Drop what a switch or a removed level takes away

Type: task
Status: open
Blocked by: 40
Map: [WotR Build Planner](../map.md)

## Question

Build [ticket 40](40-stale-links-generic-error.md)'s two decisions (from the user's words, 2026-11-01: "I don't care to store what feats you chose before selecting Legend"; the Build holds only what the sheet shows):

1. **Remove level drops the path and the Path switch** once their rank (MR 3, MR 8) isn't reached any more, as it already drops those ranks' picks.
2. **A Path switch drops the picks it takes away at earlier, reached ranks:** Legend's MR 3–7 general and path picks, Gold Dragon's and Swarm's old-path picks of MR 3–7, Devil's gaze picks, and anything else ticket 26's `kept_through_rank` / `path_kept_through_rank` / dropped grants cover. They leave the `Build`; the Mythic track still shows those ranks as dropped or partly dropped, with the lost slots empty. Undoing the switch doesn't bring them back.

## Build

AFK, test-first (`tdd`), in `rules`; no schema, game-data or link-format change expected. Only agent in the repo.

- Both go through `Engine::apply` (ticket 40), next to its dropping of unreached ranks' picks.
- With these picks gone from the Build, simplify whatever only existed to carry them: for example whether `dropped_slots` can now be read straight from the rank rules rather than from held picks, and whether `Engine::is_possible` should now refuse a Build that holds a pick a switch takes away (a link can't carry one any more). Keep what the sheet needs to draw a dropped rank.
- Removability: the switch's lock (`BreaksLater`) should now only fire when a later choice needs a pick the switch drops, as ticket 40 did for MR 8–10.
- **Docs:** ticket 12's Mythic track line if it mentions dropped picks being kept, `GLOSSARY.md` (Mythic track, Path switch), the map's line for ticket 26 if it says the picks are kept, and a map "Decisions so far" line for ticket 41.
- **Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus tests: removing level 9 drops the path (and level 17 the switch); Legend's switch drops MR 3–7's general and path picks and the track still marks those ranks dropped; Gold Dragon/Swarm drop the old path's MR 3–7 picks and keep the general ones; Devil drops the gaze picks; a link holding a pick a switch takes away is a stale link.

## Progress log

## Answer
