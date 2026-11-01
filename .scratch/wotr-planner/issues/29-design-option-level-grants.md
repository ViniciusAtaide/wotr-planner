# Design options that grant over class levels

Type: grilling
Status: resolved
Blocked by:
Map: [WotR Build Planner](../map.md)

## Question

Bloodlines, wizard schools, cavalier orders, shaman spirits, Magic Deceiver ways and the ranger combat style are choices whose option grants features, feat slots and selections at later class levels. It's the biggest gap left in the data (39+ report items in `game-data/EXTRACTION_REPORT.md`): a sorcerer or wizard is incomplete without it. Decide the shape with the user (HITL, `grilling` + `domain-modeling`), then write the build ticket.

Starting point: [ticket 18](18-handfix-base-classes.md)'s proposal (`SelectionOption.levels: Vec<OptionLevelRow>`, an `option_level` table, a fourth grant owner) and [ticket 21](21-schema-questions-real-data.md)'s follow-up A. Questions to settle:

- the `OptionLevelRow` shape, and whether a sparse archetype row ([ticket 23](23-class-progressions-not-copies.md)) can replace an option's rows
- how archetypes that change a bloodline (Crossblooded, Nine-Tailed Heir, Seeker, Overwhelming Mage, Geomancer, Primalist, Mixed-Blood Rager) get their own options
- whose class level drives an option's rows (Dragon Disciple levels continue sorcerer bloodline rows; Eldritch Scion, Dragonheir Scion)
- nested choices (dragon or element type), the bloodline feat lists, Dragon Disciple's "no bloodline" and spontaneous-caster entry conditions
- the orphan `ranger/combat-style-feat` selection
- which other selections have levelled options (oracle mysteries, domains, blessings, witch patrons)

## Facts from the guide (2026-11-01)

A read-only survey of `guide-cache/` (agent report, summarised):
- No option table (bloodline, school, order, way) has BAB, save, CL or spells-per-day columns. Tables repeat the class table, and the option's column is the difference. Shaman spirits and ranger combat styles are prose only.
- Bloodlines run on other classes' levels too: Dragon Disciple continues a sorcerer's bloodline (bloodline level = sorcerer + DD), or a non-sorcerer picks Draconic on entry (DD level). Eldritch Scion uses magus levels; Winter Witch stacks with shaman/witch levels; Hellknight's Pentamic Faith domain uses Hellknight level.
- Archetypes reshape options five ways: fix the option (Empyreal, Sage, Sylvan, Wildland, Cruromancer, Elemental Specialist, Spell Master), restrict the list (Hag-Riven, Reformed Fiend, Elemental Witch, Hagbound), drop option levels (Seeker 3/9/15; Geomancer), turn a level into a choice (Primalist: bloodline power or two rage powers at 4/8/12/16/20), or add a second option (Crossblooded; mythic Second Bloodline/Spirit/Mystery).
- More families grant over levels: domains (37, shared by cleric, druid, inquisitor, Divine Hunter, Hellknight), oracle mysteries with mystery-dependent revelations, warpriest blessings (minor at 1, major at 10), kineticist elements. Witch patrons grant only spells.
- Some choices are daily, not build-time: Mixed-Blood Rager, Unsworn/wandering spirits, hunter Animal Focus, Abjurer resistance.
- Options are mostly chosen at the class's level 0/1, but can come later with earlier rows applying at once (DD entry, mythic Second X, kineticist Expanded Element at 7/15).

## Decisions

**Round 1 (user, 2026-11-01, "all recommended"):**
1. **Scope:** every family whose options grant features or slots: bloodlines (sorcerer, bloodrager), wizard schools, cavalier orders, shaman spirits, Magic Deceiver ways, the ranger combat style, then domains, oracle mysteries with revelations, warpriest blessings and kineticist elements. Two build tickets: the first six families, then the rest on the same model. Witch patrons stay a plain pick (spells only).
2. **Spells and scaling:** option spells stay out (spells are out of scope). A number that scales without a new grant stays in the feature's summary, except ability-score bonuses, which use ticket 26's feature ability bonus at each breakpoint (they count toward prerequisites).
3. **Option levels:** each option has its own rows by *option level*. The grant of the selection says which classes' levels count toward it (sorcerer; sorcerer + Dragon Disciple). An option chosen late gets every row up to its current option level at once.
4. **Option-dependent feat lists:** the slot stays on the class's row and takes its list from the option held in a named selection. Each option names its feat list; a growing list (ranger styles at 2/6/10) names a list per stage. The ranger combat style becomes the choice of list.
5. **Fixed-option archetypes:** grant the selection limited to one option (no pick, not in the URL) where the archetype's table matches the option's rows; an archetype that changes the rows (Sylvan) keeps its own rows.
6. **Restricted lists:** the archetype grants the same selection limited to a subset of its options.

**Round 2 (user, 2026-11-01, "all recommended"):**
1. **Skipped option levels:** an archetype's grant of the selection lists the option levels it skips (Seeker 3/9/15, Geomancer); its replacements are its own rows.
2. **Primalist:** exact. At 4/8/12/16/20 a choice: the bloodline power, or two rage-power picks that skip that level's bloodline row.
3. **Two options at once:** a selection can be held twice with different options. Crossblooded's second pick adds only its feat list (the bloodline feat slots offer both lists). A mythic Second Bloodline/Spirit/Mystery is a full second pick on the same class levels, getting every row up to its current level at once.
4. **Daily choices** (Mixed-Blood Rager, Unsworn and wandering spirits, Animal Focus, Abjurer resistance) aren't part of a Build: a note on the sheet. Mixed-Blood Rager gets no bloodline pick.
5. **Dragon Disciple entry:** new prerequisite leaves "holds no option from selection X" and spontaneous arcane spell-level access, with a `Class` fact for spontaneous vs prepared casting. A non-sorcerer picks a Draconic bloodline on entry (driven by DD levels); a sorcerer's bloodline continues on sorcerer + DD levels.
6. **Nested choices:** flat options where the game lists separate bloodlines (10 draconic colours, 4 elemental types; "Draconic bloodline" = any of the 10); a nested selection granted by an option row only for choices inside an option (Arcane's school power at 15, Elemental Specialist's Focused Element).
7. **Option-dependent pools:** a class's selection slot can draw on the selection named by a held option (revelations from the mystery), the same mechanism as round 1's feat lists.
8. **Thassilonian:** shares the 7 matching school options, limited, with the guide's names in a note.
9. **Bloodline Ascendance and similar:** a feature can grant "option level N's row of the option held in selection X".

## Answer

Designed with the user over two rounds (2026-11-01); the Decisions above are the design. Two assumptions the user confirmed with it: oracle curses stay a plain pick (their later effects are mostly spells), and the Animal and Scalykind domains' companion is out of scope (their class skill stays). Built in [ticket 30](30-build-option-levels.md) (the model, bloodlines, schools, orders, spirits, ways, ranger style, Dragon Disciple) and [ticket 31](31-build-option-levels-more-families.md) (domains, mysteries and revelations, blessings, elements).
