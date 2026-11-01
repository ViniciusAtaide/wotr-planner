# Class-skill cleanup

Type: task
Status: open
Blocked by: 37
Map: [WotR Build Planner](../map.md)

## Question

Build [ticket 37](37-option-class-skills.md)'s decisions 2, 3 and 5 (recommended, per the user's standing rule for skills, 2026-11-01):

- **Overwhelming Mage and Seeker:** drop their "Class Skill" features (and the definitions if nothing else grants them). Each holds its chosen bloodline's 1st-level row, so it already holds that bloodline's class skill.
- **Sohei:** give the archetype its own class-skill list, the monk's plus Lore (Nature), as the Empyreal and Sylvan Sorcerers have, and drop its "Skills" feature. Check how archetype lists are modelled (ticket 37 dropped the Empyreal's and Sylvan's features because their lists already held the skill) and follow that shape.
- **Animal domain:** its Lore (Nature) class skill moves from option level 1 to option level 4, the level the guide's sentence sits under ("At 4th level, you gain an Animal Companion"). Make sure option level 4 has a row (add one holding only the class skill if needed), and that a druid's, inquisitor's or other holder's domain levels give it at the right class level.

Decisions 1 and 4 (the Arcane/Sage choice and the unmodelled sources) stay as they are: no change.

Data from the saved guide pages only (offline). If the guide doesn't settle something, take the option that keeps the data closest to the guide, record it in the Answer, and carry on: skills questions don't go to the user.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB). Put the rules in an extractor module (or extend `extract::option_class_skills`) and apply them to `game-data/` with a one-off scratchpad program, as tickets 30–37 did; a fresh extraction must be byte-identical to the rewrite. **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests:

- an Overwhelming Mage and a Seeker hold their bloodline's class skill, and no "Class Skill" feature;
- a Sohei holds Lore (Nature) as a class skill and has no "Skills" feature; a plain monk lacks it;
- a cleric with the Animal domain lacks Lore (Nature) at cleric 3 and holds it at cleric 4; a druid with a Nature Bond Animal domain likewise by druid level;
- the five open features of ticket 37 are down to two (Arcane, Sage).

## Progress log

## Answer
