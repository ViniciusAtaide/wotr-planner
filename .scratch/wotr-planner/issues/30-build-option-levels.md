# Build option levels: bloodlines, schools, orders, spirits, ways

Type: task
Status: open
Blocked by: 28, 29
Map: [WotR Build Planner](../map.md)

## Question

Build the model designed in [Design options that grant over class levels](29-design-option-level-grants.md) (its Decisions, rounds 1 and 2, are the spec) and fill it for the first families:
- sorcerer bloodlines and bloodrager bloodlines (flat draconic colours and elemental types)
- wizard schools (including Thassilonian sharing them)
- cavalier orders
- shaman spirits
- Magic Deceiver ways
- the ranger combat style (choosing the slot's list; the orphan `ranger/combat-style-feat` gets this job)
- Dragon Disciple (entry checks; continuing a sorcerer's bloodline; a non-sorcerer picking a Draconic one), Eldritch Scion, Dragonheir Scion (own rows)

**Model** (all from ticket 29):
- option rows by option level; the selection's grant names the classes whose levels count, the option levels it skips, and the options it's limited to (one = granted with no pick)
- slots whose feat list, or selections whose options, come from a held option
- a selection held twice (Crossblooded's second pick adds only its feat list; mythic Second Bloodline/Spirit as full picks)
- Primalist's per-level choice that skips the bloodline row
- nested selections granted by option rows
- a feature granting option level N's row (Bloodline Ascendance)
- prerequisite leaves "holds no option from selection X" and spontaneous arcane spell-level access, with a `Class` spontaneous/prepared fact
- ability-score breakpoints use ticket 26's feature ability bonus; spells stay out; daily choices get a note

**Data:** from the saved `guide-cache/` pages only (offline), in the extractor where the tables are regular (it already finds the 39 nested tables), by hand where they're prose (spirits, ranger styles). Close the matching `EXTRACTION_REPORT.md` items.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB, per `CLAUDE.md`). If the URL encoding changes, regenerate the frozen v1 vector (no link has shipped). Update `GLOSSARY.md` (Option level, and others the build earns). **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus tests in the rules test world for each model bullet and on the real data: a sorcerer 20 holds their bloodline's powers at 1/3/9/15/20 and its feat list at 7/13/19; a Seeker skips 3/9/15; a sorcerer 5 / Dragon Disciple 5 has bloodline level 10; a wizard 1 holds their school's level-1 power.
