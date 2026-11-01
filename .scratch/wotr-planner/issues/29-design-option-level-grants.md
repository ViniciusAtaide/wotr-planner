# Design options that grant over class levels

Type: grilling
Status: open
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
