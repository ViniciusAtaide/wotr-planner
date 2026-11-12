# Intelligence's skill points: take or drop the held branch

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Ticket [131](131-gaps-from-the-second-martial-run.md)'s item 2: the guide's character creation page says Intelligence "grants bonus skills equal to one point per two character levels for each point of Intelligence bonus". So Int 12 gives a point at odd levels, Int 14 one at every level, and Int 18 two. `main` adds the whole Int modifier at every level, twice the guide's figure.

The guide's rule is built on branch `held/131-int-skill-points` (commit `beec821`, with the bench link change) and reverted on `main`. It turns every Int 12+ link that spent all its skill points into a stale link. The user chose on 2026-11-09 to keep it held back for a later decision. This ticket holds that decision.

Options:
- **A.** Keep the whole modifier and delete the branch. Record in the map that the planner differs from the guide here, and why.
- **B.** Take the branch as it is. Every Int 12+ link that spent all its points opens with ticket [40](40-stale-links-generic-error.md)'s generic error.
- **C.** Take the guide's rule, and open an overspent link instead of refusing it: drop the ranks the new rule can't pay for, from the last level backwards, as ticket 40 drops unreached picks, and say so once. Old links open with fewer skill ranks.

*Recommend:* the user's call. The guide wins where it and the game disagree (map Notes), which points to B or C. C keeps old links opening.

## Done when

The user has chosen. For A, the branch is deleted and the map records the difference. For B, the hold-back commit is reverted on `main`. For C, B is built plus the drop and a test that an old Int 18 link opens with its last ranks dropped. In each case the snapshot check's message names what changed (`CLAUDE.md`, "Behaviour checks"), and `scripts/check.sh` passes.

## Answer

**Decided** (user, 2026-11-10): C. Take the guide's half-Int rule from `held/131-int-skill-points`, and open an overspent link with the ranks the new rule can't pay for dropped, from the last level backwards, instead of the stale-link error.
