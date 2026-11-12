# Mythic picks can come after the levels: Mythic Shield Focus at the Enforcer's rank 10

Type: task
Status: open
Blocked by: 160, 166, 167, 168
Model: Fable to build (it changes the order the walk checks picks in); review on Opus.
Map: [WotR Build Planner](../map.md)

## Question

The gold test ([160](160-gold-builds.md)) lists the Enforcer's Mythic Shield Focus at Mythic rank 10 as a known gap. The planner places rank 10 at character level 19 (ticket [04](04-progression-model.md)'s rank table), and checks a level's picks before the ranks that come at it. The Enforcer takes Shield Focus at level 20, so "Shield Focus (Mythic)" at rank 10 is locked: "Needs Shield Focus".

The user, 2026-11-11: "you can select mythic feats after leveling all base levels so it's perfectly fine to level mythic shield focus mythic 10 at level 20."

That reverses the map decision for this case. Two ways to build it, for the user to choose before this starts:

- **A. Rank 10 moves to character level 20.** Only the rank table changes. It fixes this build, and no other rank.
- **B. Any rank may be taken later than its table level.** A rank's picks are checked at the level the user places it, anywhere from its table level to the last level. This matches "after leveling all base levels" for every rank, and it changes the Build, its link and the level screen.

*Recommend:* A if the game always gives rank 10 at the very end; B if the user wants ranks placed freely, as a larger ticket.

**Decided** (user, 2026-11-11): B. Any rank may be taken later than its table level.

## Build

AFK. `libs/rules`, the web app and the MCP. It runs after the gear tickets (166, 167, 168), since it edits the Build, the link, the level screen and the MCP as they do.

- Read ticket 04's Answer and the map's Notes on Mythic rank timing first: they record why each rank has its level.
- **The model:** each rank has a character level it is taken at. It defaults to the rank table's level, which stays the earliest allowed, and the user may move it later, up to the Build's last level. Ranks keep their order: a rank is never placed before the rank below it. A rank's picks are checked at the level it is placed, after that level's own picks, as today.
- **The Build and the link:** a rank's placement is stored only when it differs from the table, in a trailing optional part of the link (ticket 91's pattern, and 166's for gear). Every existing link opens unchanged and writes the link it always did.
- **The web app:** on the level screen, a rank shows at the level it is placed, with a control to move it later or back. The summary shows each rank's level.
- **The MCP:** `sheet`'s `character_level` per rank is the placed level; an address moves a rank (`mythic.R.level`), offered by `options`. The server's instructions say a pick that "comes after this slot" can also be fixed by taking the rank later.
- If the design needs a decision this ticket doesn't settle (a rank's features that the level's own picks depend on, a path switch at a moved rank), stop and put it to the user rather than choosing.
- Prove what changes with the snapshot check: the ranks' `character_level` lines and whatever opens because of them, and nothing else.
- With nothing moved, the snapshots don't change at all.
- Remove the Mythic Shield Focus entry from the gold test's known gaps: place the Enforcer's rank 10 at level 20 and fill its Mythic feat slot. The test then proves it.
- Old links must still open: a Build whose rank picks were legal stays legal.

## Done when

- The Enforcer's Mythic Shield Focus at rank 10 agrees in the gold test, with its known-gap entry deleted.
- The snapshot diff is reviewed and holds only the intended change.
- `scripts/check.sh` passes.
