# Class skills from options

Type: task
Status: open
Blocked by: 34
Map: [WotR Build Planner](../map.md)

## Question

Build [ticket 34](34-backgrounds.md)'s decision 1 (user, 2026-11-01): the "Class Skill: …" features that options grant (oracle mysteries, kineticist elements, domains, bloodlines and any others in the data) become real class skills of the option, so a character holding the option has them as class skills. Tickets 30 (open question 8) and 31 (open question 9) left them as features only because nothing used class skills then.

A feature that offers a *choice* of class skill (the Arcane bloodline's) is a choice, not a fixed skill: if one is found, stop and write the question here rather than guessing.

Data from the saved guide pages only (offline). If the guide doesn't settle something, stop and write the question here.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB). **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: an oracle with a mystery holds that mystery's class skills, and a kineticist holds its element's; a character without the option doesn't; the "Class Skill: …" features are gone from the data.
