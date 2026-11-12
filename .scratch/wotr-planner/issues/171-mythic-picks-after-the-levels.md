# Mythic picks can come after the levels: Mythic Shield Focus at the Enforcer's rank 10

Type: task
Status: open
Blocked by: 160, 166; the user's choice below
Model: Fable to build (it changes the order the walk checks picks in); review on Opus.
Map: [WotR Build Planner](../map.md)

## Question

The gold test ([160](160-gold-builds.md)) lists the Enforcer's Mythic Shield Focus at Mythic rank 10 as a known gap. The planner places rank 10 at character level 19 (ticket [04](04-progression-model.md)'s rank table), and checks a level's picks before the ranks that come at it. The Enforcer takes Shield Focus at level 20, so "Shield Focus (Mythic)" at rank 10 is locked: "Needs Shield Focus".

The user, 2026-11-11: "you can select mythic feats after leveling all base levels so it's perfectly fine to level mythic shield focus mythic 10 at level 20."

That reverses the map decision for this case. Two ways to build it, for the user to choose before this starts:

- **A. Rank 10 moves to character level 20.** Only the rank table changes. It fixes this build, and no other rank.
- **B. Any rank may be taken later than its table level.** A rank's picks are checked at the level the user places it, anywhere from its table level to the last level. This matches "after leveling all base levels" for every rank, and it changes the Build, its link and the level screen.

*Recommend:* A if the game always gives rank 10 at the very end; B if the user wants ranks placed freely, as a larger ticket.

## Build

AFK once the user has chosen. `libs/rules`, and the web app and MCP where a rank's level shows.

- Read ticket 04's Answer and the map's Notes on Mythic rank timing first: they record why each rank has its level.
- Prove what changes with the snapshot check: the ranks' `character_level` lines and whatever opens because of them, and nothing else.
- Remove the Mythic Shield Focus entry from the gold test's known gaps, and fill the Enforcer's rank 10 Mythic feat slot. The test then proves it.
- Old links must still open: a Build whose rank picks were legal stays legal.

## Done when

- The Enforcer's Mythic Shield Focus at rank 10 agrees in the gold test, with its known-gap entry deleted.
- The snapshot diff is reviewed and holds only the intended change.
- `scripts/check.sh` passes.
