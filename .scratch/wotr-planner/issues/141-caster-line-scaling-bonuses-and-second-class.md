# The caster line: bonuses that grow, School Mastery and a second casting class

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

`CASTER_LINE_LEAVES_OUT` (`libs/rules/src/engine/caster.rs`) leaves out three things that are unconditional, or are a fixed choice of the Build:

- **Bonuses that grow with class level:** Combat Casting Adept, Horn Call, Cold Specialization, Overwhelming Penetration, Haunting Refrain, Blessed Script.
- **School Mastery's** caster level for one school.
- **A second casting class:** the line follows only the class with the highest caster level, so a cleric/wizard or a Mystic Theurge-style Build gets one class's line.

Options:
- **A.** A new `ItemBonus` amount that scales with a named class's level, as ticket [120](120-initiative-and-skill-totals.md) added ten-ranks and ability-modifier amounts. School Mastery becomes a source limited to the school it names. A `class` argument on the line (MCP `caster`, and the web app if it shows a caster line) picks the casting class, defaulting to the highest caster level.
- **B.** Only the scaling bonuses, and leave School Mastery and the second class in `leaves_out`.

*Recommend:* A.

Questions for the build: each bonus's amount by level as the guide states it, and how School Mastery names its school in the data (a parameter, or a fixed school per archetype).

## Done when

A Build holding each source shows it as a `CasterPart` with the guide's amount at its level. A two-class caster can ask for either class's line. Real-data tests cover each case. `CASTER_LINE_LEAVES_OUT` no longer names what is now counted. The snapshot check's message shows that only caster lines changed, and the snapshots are regenerated (`CLAUDE.md`, "Behaviour checks"). `scripts/check.sh --schema` passes.

## Answer

**Recommendation taken** (user, 2026-11-10): A.
