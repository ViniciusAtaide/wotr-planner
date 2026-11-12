# Bloodline Ascendance grants the 20th-level row a sharing selection skips

Type: bugfix
Status: open
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

## Done when

The chosen rule is built with a real-data test of the repro, the README's Bloodline Ascendance line says it, and `scripts/check.sh` passes.
