# The queue's scope, cut by value (user, 2026-11-17)

The importer's remaining work was ranked by what a player of the planner notices: does a change alter which picks a Build is offered or refused, and how widely, or does it mainly reshape the data and close game-check findings? The user chose ranks 1 to 7 for this queue ("lets do 1 up to 7"). The rest is deferred to a later queue, with its tickets kept.

## In this queue

1. **S4 (308): feat prerequisites.** Prerequisites decide which picks are legal, and every Build picks feats. Prerequisites that name a carrier S3 hasn't made stay unwritten, listed under 307.
2. **306s:** the general and group feat lists (fighter, wizard, combat, teamwork, Extra Mythic Ability).
3. **306q and 306r:** the feats holding several blueprints (Advanced Weapon Training 2–4, Extra Channel, Magical Tail, Abundant Smite, Bleeding Critical's lists), built as one batch.
4. **306n:** the sorcerer's, Seekers' and skald's bonus feats (user fact 281).
5. **306aa:** `needs_slot_of` goes; the Extra feats open the slots the game gives them.
6. **306m:** the cavalier's bonus feats.
7. **S4: the rest,** the mythic and class-feature prerequisites that don't need S3's carriers.

Then the queue's close: S2's proof (306ad) over the merged range, `scripts/queue-checks.sh`, the self-checks' long runs, the game check, a code review, a QA session and the map's records.

## Deferred to a later queue

| Work | Why it waits |
| --- | --- |
| S5 (309): progression rows | Mostly which features show at which level; 557 of its findings are how per-step features show on the sheet. |
| S3 (307c–307o): the carriers | Tooling that enables part of S4 and S5. 307a, 307b and 307m are already merged. |
| 306t–306z: kineticist elements, wild talents, mysteries, curses, bloodlines, blessings | They reshape per-class copies into the game's blueprints. Old links keep opening through aliases, so offers barely change. |
| 306o and 306p: prestige and archetype feat lists | Today's data refuses nothing there. |
| 306ab: the check compares feat selections by GUID | Tooling. Its note on the writer's skipped selections stays on the ticket. |
| The S4 prerequisites that name a carrier | They wait for S3. |

Each deferred ticket keeps its research (`research/306/`, `research/307/`) and its `known.txt` lines, so the game check still exits 0.
