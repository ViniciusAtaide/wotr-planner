# Keep a switch's earlier picks

Type: task
Status: open
Blocked by: 41
Map: [WotR Build Planner](../map.md)

## Question

Undo half of [ticket 41](41-drop-what-a-switch-takes-away.md) (user, 2026-11-02). The picks a Path switch takes away at earlier, reached ranks (Legend's MR 3–7, the old path's MR 3–7 on Gold Dragon and the Swarm, the Aeon's gaze on Devil) stay in the Build again. They were held from their rank until the switch, so they count for everything before it, and the Mythic track shows them as lost from the switch on, as [ticket 26](26-mythic-path-rules.md) had it. In particular, ticket 26's guide rule holds again: a Trickster who becomes a Legend keeps the feats Perception II unlocked.

Keep the rest of ticket 41 and all of ticket 40:
- the path goes once MR 3 isn't reached and the switch once MR 8 isn't, after any edit (`Engine::apply`), and `is_possible` refuses a path before MR 3 or a switch before MR 8;
- Remove level and Legend's switch drop the picks of ranks no longer reached (MR 8–10);
- a stale link gets the one generic error.

## Build

AFK, test-first (`tdd`), in `rules`; no schema, game-data, link-format or `Build` change expected. Only agent in the repo.

- Restore, for the earlier-rank picks only, the behaviour at 5e7e106 (ticket 40): kept in the Build, held until the switch, then released by the switch with what they granted; `dropped_slots` and `RankStatus` as they were; an empty slot at an earlier rank still has a picker (it's filled before the switch). Bring back what ticket 41 removed only to support dropping them (`GrantSource.slot`/`selection`, the switch's release of holdings from taken-away picks, …) where it's needed again; keep ticket 41's other simplifications (`slot_level`) if they still fit.
- `Engine::apply` no longer drops them; `Walker::fill` no longer fails them; `switch_takes` goes or shrinks to what the track still needs. A link holding such a pick is possible again, not stale.
- The switch lock (`BreaksLater`): back to ticket 40's behaviour for these picks. Legend is no longer locked for the Perception II Trickster.
- Tests: restore ticket 41's three removed tests (`path_rules::a_trickster_who_becomes_a_legend_keeps_the_feats_perception_ii_unlocked`, `path_rules::a_switch_dropping_an_option_drops_its_feats_but_not_the_same_feats_held_otherwise`, `parameters::a_switch_that_drops_an_every_value_grant_keeps_a_value_a_pick_holds`) and bring the rewritten ones back to ticket 40's expectations for earlier-rank picks (`git show 5e7e106:<path>` has them); drop or invert `stale_links::a_link_holding_a_pick_a_switch_takes_away_is_stale` and the Gold Dragon/Swarm "drop the old path's picks" test. Keep ticket 41's tests for the unreached path and switch.
- **Docs:** `GLOSSARY.md` (Path switch, Mythic track), ticket 12's Mythic track and picker lines, the map's notes on tickets 26, 40 and 41, and a "Decisions so far" line for ticket 42.
- **Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus: the Perception II Trickster switches to Legend and keeps its feats; Legend, Gold Dragon, Swarm and Devil keep their earlier-rank picks in the Build, marked lost from the switch on; the unreached path/switch tests of ticket 41 still pass; the proof (ticket 41's Build sets) shows that, against 5e7e106, the only differences left are ticket 41's path/switch drops.

## Progress log

## Answer
