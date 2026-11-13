# Bloodline Ascendance grants the 20th-level row a sharing selection skips

Type: bugfix
Status: resolved
Blocked by: none
Model: Sonnet to build once the question is answered; it is one engine condition or one prerequisite.
Map: [WotR Build Planner](../map.md)

## Question

Found by ticket [179](179-dragonheir-scion-gains-draconic-resistances.md)'s sweep. Bloodline Ascendance (`sorcerer/bloodline-ascendance`, `option_row: (selection: "sorcerer/bloodline", level: 20)`) grants the 20th-level row of every option held that `sorcerer/bloodline` offers. `Walker::grant_option_row` (`libs/rules/src/engine/walk/options.rs`) passes over a row the holding already had, but not one its selection skips (`skipped_levels`). The Dragonheir Scion's bloodline is a sorcerer Draconic option held through `fighter/dragonheir-scion/bloodline`, which skips every row, the 20th's too, since its own table gives Might of Wyrms there instead.

**Repro:** sorcerer 1 (Abyssal) then Dragonheir Scion 19 (Draconic Red), Bloodline Ascendance at MR 1: the Build is possible, and it holds the Draconic bloodline's `power-of-wyrms` from level 6, beside the Abyssal's 20th-level power.

**The guide** (`mythic-paths`): "Bloodline Ascendance Prerequisites: Bloodline You gain the 20th level ability of your bloodline. This ability can be taken by Sorcerers, but not, for example, by Bloodragers". The data's prerequisite is `ClassLevel(class: "sorcerer", level: 1)`. It says nothing of a sorcerer who also holds a bloodline through another class.

No stepped feature is on any bloodline's 20th-level row (179's test asserts it), so this is not a step leak; it needs a decision on which bloodlines the ability reaches:

- **A.** Only rows the holding's selection does not skip: `grant_option_row` also passes over a skipped row. The Dragonheir Scion's bloodline gets nothing; an Eldritch Scion's (which skips nothing) still gets its 20th-level power, as each held bloodline does now (`interactions.ron`'s note on Bloodline Ascendance).
- **B.** Only bloodlines held through the sorcerer's own selection (the sorcerer's, Second Bloodline's): `holding.selection == row.selection`. The Eldritch Scion's and the Dragonheir Scion's get nothing.
- **C.** Leave it, and say so in `interactions.ron`.

Recommended: A, the smallest, and it reads as `skipped_levels` already does elsewhere ("rows an option held through this selection doesn't get").

**Decided** (user, 2026-11-13, "do as the recommended"): A.

## Done when

The chosen rule is built with a real-data test of the repro, the README's Bloodline Ascendance line says it, and `scripts/check.sh` passes.

## Answer

Done 2026-11-13 (branch `ticket-180`). Option A, one engine condition in `Walker::grant_option_row` (`libs/rules/src/engine/walk/options.rs`): a holding is passed over also when `holding.selection != row.selection` and the holding's selection lists the row's level in `skipped_levels`. No data, schema or migration change.

**The exemption.** The condition first read as only "the holding's selection skips the row", and it broke two real-data tests: the Kinetic Knight's Defensive Wild Talent has `option_row: (selection: "kineticist/kinetic-knight/element", level: 2)` and that selection is `skipped_levels: [2]` (its own row 2 is the one the feature unlocks). A feature granting a row of the very selection a holding comes through is the feature's point, so that case is not stopped by the skip (it is the lists-only branch's rule already: held "through that very selection").

**Every `option_row` in the data, and every caller.** `grant_option_row` has one caller, `Walker::hold` (`libs/rules/src/engine/walk/hold.rs`), for a feature with an `option_row`. Three features have one:

| Feature | Row | Before | After |
| --- | --- | --- | --- |
| Bloodline Ascendance (`sorcerer/bloodline-ascendance`) | `sorcerer/bloodline`, 20 | each held option the selection offers, whichever selection holds it, unless it had the row | the same, except one held through a selection that skips 20 |
| Expanded Defense (`kineticist/wild-talents/expanded-defense/defense`) | `kineticist/expanded-element`, 2 | the lists-only holdings of that selection | unchanged: that selection skips nothing, and a lists-only holding takes the first branch (the exemption would cover it anyway) |
| Kinetic Knight's Defensive Wild Talent | `kineticist/kinetic-knight/element`, 2 | the holding through that selection, which skips 2 | unchanged, by the exemption above |

Bloodline Ascendance's holdings, by the selection that holds the bloodline (every selection that shares a `sorcerer/bloodline` option, and its `skipped_levels`): the sorcerer's own and Second Bloodline's (none), Crossblooded's second (lists-only, passed over as before), Seeker `[3, 9, 13, 15, 17]`, Sage `[1]`, Sylvan `[1]`, Eldritch Scion (none), the Geomancer, Overwhelming Mage and Nine-Tailed Heir (none, as the sorcerer's), and two that skip 20:

- **Dragonheir Scion** `[1, 3, 9, 15, 20]`: before, the Draconic bloodline's `power-of-wyrms`; after, nothing. This is the ticket's repro.
- **Empyreal Sorcerer** `[1, 20]` (`sorcerer/empyreal-sorcerer/bloodline`, shared Celestial): before, the Celestial's Ascension from MR 1; after, nothing. **This is a second case the ticket did not name.** The guide (`sorcerer`, the Empyreal Sorcerer): "Empyreal Sorcerer... has the following level progression", 20 is "Apotheosis", and no table row lists the Celestial's Ascension; the data's own 20th row is `sorcerer/empyreal-sorcerer/apotheosis`. So it is the Dragonheir Scion's case exactly (the archetype's table replaces the bloodline's 20th row), and A reads it the same. The guide says nothing of what Bloodline Ascendance does for it, so the change rests on that reading; `interactions.ron`'s note says so. The user may want to confirm it.

The druid's Blight Druid domains, the oracle's Divine Herbalist and Enlightened Philosopher mysteries also skip rows, but offer no `sorcerer/bloodline` option, so Ascendance never reaches them.

**Tests.**

- `libs/seeds/tests/game_data/bloodline_ascendance.rs` (real data; `mod bloodline_ascendance;` in `main.rs`): the repro (sorcerer 1 Abyssal / Dragonheir Scion 19 Draconic Red, Ascendance at MR 1: possible, holds `demonic-might` from level 6 and never `power-of-wyrms`; it failed before the fix at level 6, holding `power-of-wyrms`); a plain sorcerer (`demonic-might` at 6, once); a Second Bloodline (MR 1) with Ascendance through Extra Mythic Ability (MR 2): both `demonic-might` and `power-of-wyrms` at the rank's level and neither before; an Eldritch Scion (sorcerer 1 / magus 19, Arcane): `arcane-apotheosis` and the Abyssal's at 6; a Seeker, whose skips do not include 20: it keeps `demonic-might`; an Empyreal Sorcerer: no Celestial Ascension, and its Apotheosis (it failed before the fix).
- `libs/rules/tests/engine/option_levels.rs`, `bloodline_ascendance_passes_over_a_row_the_holdings_selection_skips`: in the test world the Seeker holds the Abyssal's `demonic-might` from Ascendance; with the world changed so its selection also skips 20, it does not (failed before the fix). A test-world archetype was tried first and left out, since the world's archetype lists are asserted by four other tests.

**Behaviour checks.** The snapshot check passes with the snapshot files untouched, and the gold test passes with its two known gaps (`cargo test -p seeds --test game_data`, 372 passed). `scripts/proof.sh` was not run.
