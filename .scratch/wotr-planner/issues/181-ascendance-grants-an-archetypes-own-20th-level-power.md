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

**Held once.** The holding's `applied` entry for the row is the one record, as for any bloodline row: whichever of the bloodline's option level reaching 20 and the ability comes first holds the power, and the other finds the entry and does nothing. A Build is walked from scratch, so withdrawing the ability gives it back at 20 with no extra step. A row granted ahead of its level is also recorded in the holding's new `early` map with the ability's source.

**Review fix: a Path switch that takes the ability's rank back (found by the independent review).** Legend drops ranks 3 to 7. `switch_path` released the rank's grants but left the holding's `applied` entry, so the row was never given again when the option level reached it: a Legend Empyreal Sorcerer with the ability at MR 3, 5 or 7 lost Apotheosis for good (held from the rank's level to the switch, then 0 to the end). The same bug was on `main` for an ordinary bloodline row (an Abyssal sorcerer 20 / fighter 5 with the ability at MR 3 lost Demonic Might from the switch on). The fix is the cause: `switch_path` now calls `take_back_early_rows`, which removes the `applied` (and `unlocked`) entry of every row in `early` whose source the switch drops, and advances the holdings, so the option level gives the row, or its replacement, when it reaches it (at once if it already has). **This changes behaviour for existing Builds:** a Legend sorcerer with the ability at MR 3 to 7 regains its 20th-level power at sorcerer level 20. The snapshot corpus has no such Build (the snapshot check passes untouched).

Every path that releases a rank's grants: (1) `switch_path` (`libs/rules/src/engine/walk/ascension.rs`), which drops the grants a switch's `drops` names, the holdings whose own source is dropped, and now the early rows as above; it is the only place that releases a rank grant. (2) `replace_row` (a Primalist's pick) releases and forgets a row's `applied` entry itself, and now forgets its `early` record too. (3) The scaling-feature recount in `level.rs` releases only class-row features it holds with no source, not option rows. A rank "no longer reached", or a withdrawn pick, is not a release: the Build is re-walked from scratch, so nothing is left behind.

**Counts the tests assert** (`libs/seeds/tests/game_data/bloodline_ascendance.rs`, real data, Empyreal Sorcerer 20): without the ability, Apotheosis 0 at levels 1 to 19 and 1 at 20; with the ability at MR 1 (level 6), 0 at 1 to 5 and exactly 1 at every level 6 to 20; with the rank placed at 20 (a Build that reaches sorcerer 20 first cannot place a rank later: levels past 20 need the Legend switch, ranks never come before the ones before them, so a later rank pushes the switch past 20 and the Build is refused), 0 to 19 and 1 at 20; with the ability withdrawn, 0 at 6 and at 19, 1 at 20; the Celestial's Ascension is 0 at every level in every case. The existing six tests pass. Legend cases (review fix; `legend_ascending`, ability at MR 1, 3, 5 and 7, aeon path, Legend switch at level 17): for Empyreal 20 / fighter 5, and fighter 5 / Empyreal 20 (Apotheosis from sorcerer 20 at level 25), Apotheosis counts at every level from the rank's level to the last 1 while the rank is held (MR 1 is kept by Legend: 1 throughout), 0 from the switch until the sorcerer reaches 20 (MR 3, 5, 7), and 1 from that level on; the Abyssal sorcerer 20 / fighter 5 gives the same for Demonic Might. A Dragon Disciple test pins the next question. A rules-level test in the test world (`libs/rules/tests/engine/option_levels.rs`, `a_replacement_for_a_skipped_row_is_held_once_early_or_at_its_level`) asserts the same counts, and `check.rs` has a test for the new data issue.

**Chosen and rejected.**
- Chosen: `replaced_by` on the selection, since the selection is what says which row it skips and the replacement belongs beside that; it reuses the holding's `applied` bookkeeping, so once-only costs no new state.
- Rejected: naming the archetype's class row at 20 on the selection (`replaces_with_class_row`), which leaves the class row granting it at 20 as well and needs a new record that it was already held (the class row has none), and breaks when the ability comes after 20.
- Rejected: a special case in code keyed on `apotheosis` or the Empyreal selection's name.
- Rejected: a new table; a nullable column on `selection_skipped_level` is smaller, since a replacement only exists at a skipped level.
- Known side effect, open question for the user (below): the replacement comes at the holding's option level, which counts Dragon Disciple levels too.

**Open question for the user: do Dragon Disciple levels count toward Apotheosis?** Without the ability, Apotheosis now follows the Empyreal bloodline's option level, and that selection's `option_levels` is `["dragon-disciple", "sorcerer"]`. An Empyreal Sorcerer meets Dragon Disciple's prerequisite through the Mythic Second Bloodline (Draconic) at MR 1, so Empyreal 6 / Dragon Disciple 10 / Empyreal 4 gains Apotheosis at character level 20 with sorcerer level 10; before this ticket it never did (pinned by `dragon_disciple_levels_count_towards_an_empyreal_sorcerers_apotheosis`). Readings: (1) yes, as built: Blood of Dragons "continues your bloodline progression uninterrupted", so Dragon Disciple levels advance the Empyreal bloodline like any sorcerer bloodline's and its 20th-level power arrives at option level 20; (2) no: the Empyreal table puts Apotheosis at sorcerer level 20, so it would need the sorcerer's own levels (the replacement would need its own level count, a new field, or Apotheosis back on the class row with a record for the early grant). Not changed.

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
| Nine-Tailed Heir | none | "20 - - - Bloodline Ability" ("chooses a standard bloodline for the purpose of bonus feats and abilities") | the bloodline's 20th row, as before |
| Overwhelming Mage | none | "20 - - - Bloodline Ability" | the bloodline's 20th row, as before |
| Dragonheir Scion (fighter) | `[1, 3, 9, 15, 20]` | "20 - Bonus Combat Feat Might of Wyrms" | nothing (see below) |

No selection other than the Empyreal Sorcerer's skips 20 or replaces its 20th-level power with a feature of its own on a sorcerer bloodline option, so none else needed a `replaced_by`.

**Decided (user, 2026-11-13): the Dragonheir Scion's Might of Wyrms, option A.** The ability does not grant it, as built: the Scion's Draconic bloodline gets nothing from it, and `interactions.ron` says so. What was known when it was decided: the guide's sentence is "You gain the 20th level ability of your bloodline. This ability can be taken by Sorcerers, but not, for example, by Bloodragers", and a sorcerer 1 / Dragonheir Scion 19 holds two bloodlines, the ability reaching the sorcerer's own. The options put were A (no), B (yes, the Empyreal Sorcerer's reading, one data line `replaced_by: {20: "fighter/dragonheir-scion/might-of-wyrms"}`) and C (yes only for a character with a sorcerer level). The review noted two things the options missed: C cannot be told apart from B, since the ability's prerequisite is already sorcerer 1 (every holder has a sorcerer level), and the guide's Might of Wyrms text is the Draconic bloodline's Power of Wyrms under another name, so the Scion's 20th-level power is the bloodline's own 20th-level power renamed, which weighs towards B. The user chose A with that unknown.

**Behaviour checks.** The snapshot check passes with the snapshot files untouched (no corpus Build is an Empyreal Sorcerer with the ability). `scripts/proof.sh` was not run.
