# Bloodline Ascendance grants an archetype's own 20th-level power, where it replaces the bloodline's

Type: task
Status: open
Blocked by: 177
Model: Sonnet to build (a small data or engine addition with a clear repro); review on another model.
Map: [WotR Build Planner](../map.md)

## Question

Ticket [180](180-bloodline-ascendance-ignores-skipped-rows.md) made Bloodline Ascendance pass over a bloodline row that the holding's selection skips. Its sweep found a second case besides the Dragonheir Scion: the **Empyreal Sorcerer**. It has the Celestial bloodline, and its selection skips the bloodline's 20th-level row (`skipped_levels: [1, 20]`) because its own class table gives Apotheosis at level 20 in place of the bloodline's Ascension (`libs/game-data/classes/sorcerer.ron`: `sorcerer/empyreal-sorcerer/apotheosis`, on the archetype's level 20 row).

- Before 180, Bloodline Ascendance gave it the Celestial's Ascension early, a power the archetype trades away, and it then also got Apotheosis at 20.
- Since 180, the ability gives it nothing. The pick is still open to it, and does nothing.

The guide says only: "You gain the 20th level ability of your bloodline."

**Decided** (user, 2026-11-13): B. The ability grants the archetype's own replacement, Apotheosis, early. That is what "the 20th level ability of your bloodline" means for an archetype that replaces it.

## Build

AFK. Game data, and the smallest engine addition the data needs.

- Apotheosis is a feature on the archetype's class row at level 20, not a bloodline option's row, so `option_row` cannot reach it today. Find the smallest way to say "what this holding's selection gives at level 20 in place of the row it skips": for example the archetype's level 20 row named as the replacement on the selection, read by `Walker::grant_option_row` where it now passes over a skipped row. Prefer data the README can describe in one line over a special case in code. Say what you chose and what you rejected.
- The feature must be held once: gained early through the ability, it is not gained again at sorcerer level 20, and removing the ability gives it back at 20. Check how a row the holding "already had" is tracked (`holding.applied`) and that the count never reaches 2, since ticket 177's stepped features made repeated grants meaningful.
- **Every other skipper.** Ticket 180's Answer lists every selection that shares sorcerer bloodline options with its `skipped_levels`. For each that skips 20 or replaces its 20th-level power some other way, say from the guide's table for that archetype what it has at 20 and what the ability should give it:
  - the Dragonheir Scion (skips every row; its table gives Might of Wyrms at fighter 20): it is a fighter archetype, and the guide says the ability "can be taken by Sorcerers, but not, for example, by Bloodragers". A sorcerer 1 / Dragonheir Scion 19 holds two bloodlines: the ability reaches the sorcerer's own, and the question is whether it should also grant Might of Wyrms. Do not decide it: build nothing for it, record the question in the Answer.
  - any other the sweep finds.
- **Tests,** in `libs/seeds/tests/game_data/bloodline_ascendance.rs`: an Empyreal Sorcerer with the ability holds Apotheosis from the rank's level, once, and never the Celestial's Ascension; without the ability it gains Apotheosis at 20; withdrawing the ability returns it to 20. The existing six tests keep passing.
- Update the README's Bloodline Ascendance line and `interactions.ron`'s note.
- The snapshot check passes with the snapshot files untouched; `scripts/check.sh` passes.

## Done when

- The Empyreal Sorcerer's repro is pinned by a real-data test, in both directions.
- The Answer has the table of skippers and the open question on the Dragonheir Scion.
- `scripts/check.sh` passes.
