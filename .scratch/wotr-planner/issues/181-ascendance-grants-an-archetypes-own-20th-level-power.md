# Bloodline Ascendance grants an archetype's own 20th-level power, where it replaces the bloodline's

Type: task
Status: resolved
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

## Answer

Done 2026-11-13 (branch `ticket-181`). Option B, built as data plus one engine addition.

**The mechanism.** A selection that skips an option level can name what it gives there instead: `FeatureSelection::replaced_by: BTreeMap<u8, FeatureKey>`, in the data `replaced_by: {20: "sorcerer/empyreal-sorcerer/apotheosis"}` on `sorcerer/empyreal-sorcerer/bloodline` (README, one line: "the feature the selection gives at a skipped level in place of the row"). The data's schema gets a nullable `replaced_by_feature_key` on `selection_skipped_level` (migration `20261113120000_181-selection-replacement.sql`; the loader and the seed carry it; `GameData::check` names the feature and refuses a level the selection does not skip, `DataIssue::ReplacementAtUnskippedLevel`).

In the walker (`libs/rules/src/engine/walk/options.rs`) a replacement is held as a row of the holding: `advance` grants it when the holding's option level reaches the level, and `grant_option_row` (Bloodline Ascendance) grants it where it used to pass over the skipped row; both go through `grant_replacement`, which does nothing if the holding's `applied` map already has that level, and otherwise holds the feature and records it in `applied`. Apotheosis is removed from the archetype's class row at 20 (the only change to the class data), so the class row cannot grant it a second time.

**Held once.** The `applied` entry is the one record, as for any bloodline row: whichever of the option level (sorcerer level 20) and the ability comes first holds it, the other finds the entry and does nothing. A Build is walked from scratch, so withdrawing the ability gives it back at 20 with no extra step. The ability can come at 20 at the latest (a Build has 20 levels and Apotheosis needs all 20 as sorcerer), and then the class level comes first in the level and the rank after: still once.

**Counts the tests assert** (`libs/seeds/tests/game_data/bloodline_ascendance.rs`, real data, Empyreal Sorcerer 20): without the ability, Apotheosis 0 at levels 1 to 19 and 1 at 20; with the ability at MR 1 (level 6), 0 at 1 to 5 and exactly 1 at every level 6 to 20; with the rank placed at 20, 0 to 19 and 1 at 20; with the ability withdrawn, 0 at 6 and at 19, 1 at 20; the Celestial's Ascension is 0 at every level in every case. The existing six tests pass. A rules-level test in the test world (`libs/rules/tests/engine/option_levels.rs`, `a_replacement_for_a_skipped_row_is_held_once_early_or_at_its_level`) asserts the same counts, and `check.rs` has a test for the new data issue.

**Chosen and rejected.**
- Chosen: `replaced_by` on the selection, since the selection is what says which row it skips and the replacement belongs beside that; it reuses the holding's `applied` bookkeeping, so once-only costs no new state.
- Rejected: naming the archetype's class row at 20 on the selection (`replaces_with_class_row`), which leaves the class row granting it at 20 as well and needs a new record that it was already held (the class row has none), and breaks when the ability comes after 20.
- Rejected: a special case in code keyed on `apotheosis` or the Empyreal selection's name.
- Rejected: a new table; a nullable column on `selection_skipped_level` is smaller, since a replacement only exists at a skipped level.
- Known side effect: the replacement comes at the holding's option level, which counts Dragon Disciple levels too (`option_levels: ["dragon-disciple", "sorcerer"]`), as the Celestial bloodline's own rows do. The guide's table puts Apotheosis at sorcerer level 20; the two differ only for a Build with Dragon Disciple levels and fewer than 20 Empyreal sorcerer levels, which no test reaches and which I did not check the prerequisites for.

**Every selection that shares sorcerer bloodline options, from the guide's tables (`sorcerer.html`, `fighter.html`).**

| Selection | `skipped_levels` | What the table has at 20 | What the ability gives now |
| --- | --- | --- | --- |
| Sorcerer, Second Bloodline, Crossblooded's first | none | the bloodline's own | the bloodline's 20th row |
| Empyreal Sorcerer | `[1, 20]` | level 20 "- - - Apotheosis" ("Most of the class abilities are taken from the Celestial bloodline") | Apotheosis, early, once (this ticket) |
| Sage Sorcerer | `[1]` | "20 - - - Arcane Apotheosis" (the Arcane bloodline's own) | the Arcane bloodline's 20th row, the same feature |
| Sylvan Sorcerer | `[1]` | "20 - - - Soul of the Fey" (the Fey bloodline's own) | the Fey bloodline's 20th row, the same feature |
| Seeker | `[3, 9, 13, 15, 17]` | "20 - - - Bloodline Ability" | the bloodline's 20th row |
| Geomancer | none | "20 - - - Bloodline Ability" | the bloodline's 20th row |
| Eldritch Scion (magus) | none | "20 - - - True Magus"; the guide: "an Eldritch Scion also gains spells and abilities related to his bloodline (for example he might grow Wings at level 15)" | the bloodline's 20th row, as before (nothing in the table says it loses it) |
| Nine-Tailed Heir, Overwhelming Mage | none | not read in the guide (neither skips a level in the data) | the bloodline's 20th row, as before |
| Dragonheir Scion (fighter) | `[1, 3, 9, 15, 20]` | "20 - Bonus Combat Feat Might of Wyrms" | nothing (see below) |

No selection other than the Empyreal Sorcerer's skips 20 or replaces its 20th-level power with a feature of its own on a sorcerer bloodline option, so none else needed a `replaced_by`.

**Open question for the user: the Dragonheir Scion's Might of Wyrms.** Built: nothing. The guide's sentence is "You gain the 20th level ability of your bloodline. This ability can be taken by Sorcerers, but not, for example, by Bloodragers". A sorcerer 1 / Dragonheir Scion 19 holds two bloodlines. The ability reaches the sorcerer's own. Should it also grant Might of Wyrms (`fighter/dragonheir-scion/bloodline` would then say `replaced_by: {20: "fighter/dragonheir-scion/might-of-wyrms"}`, one line of data and the same mechanism, with Might of Wyrms moving off the fighter row at 20 as Apotheosis did)?
- **A.** No (today): the Scion's bloodline gets nothing, since the ability is a Sorcerer's and the Scion's 20th-level power is a fighter class feature, not "of your bloodline". Simplest; `interactions.ron` says so.
- **B.** Yes: the same reading the user chose for the Empyreal Sorcerer ("the 20th level ability of your bloodline" means what the holder's table gives there). The sorcerer's own bloodline already counts as one reaching the ability, so a Scion who is also a sorcerer would get both.
- **C.** Yes only when the character holds a sorcerer level (the ability's prerequisite is sorcerer 1, so the Scion would be reached only through the sorcerer's rank of it).
Recommended: A until the user decides, since the guide's "can be taken by Sorcerers, but not, for example, by Bloodragers" reads as the ability being the sorcerer's own.

**Behaviour checks.** The snapshot check passes with the snapshot files untouched (no corpus Build is an Empyreal Sorcerer with the ability). `scripts/proof.sh` was not run.
