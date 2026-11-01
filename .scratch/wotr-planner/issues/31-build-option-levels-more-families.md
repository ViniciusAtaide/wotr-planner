# Option levels for domains, mysteries, blessings and elements

Type: task
Status: open
Blocked by: 30
Map: [WotR Build Planner](../map.md)

## Question

Fill the option-level model from [ticket 30](30-build-option-levels.md) for the second group of families in [ticket 29](29-design-option-level-grants.md):
- **Domains** (37, shared by cleric, druid via Nature Bond, inquisitor, Divine Hunter and Hellknight's Pentamic Faith): powers at option levels 1, 6 or 8, and sometimes 12/20; War's bonus combat feat at 8; class skills. A cleric holds two. Animal and Scalykind keep their class skill; their companion is out of scope.
- **Oracle mysteries:** class skill, Final Revelation at 20, and the revelation picks at 1/3/7/11/15/19 drawn from the held mystery's list. Revelations that grant later (Weapon Mastery's feats at 8/12) are option rows too. Curses stay a plain pick.
- **Warpriest blessings:** two held; minor power at 1, major at 10, replacing today's second pick at 10.
- **Kineticist elements:** class skills, the element's wild-talent feat list, the defensive talent at 2, and Expanded Element at 7/15 as a second pick.
- The domain part of Druid Nature Bond ([ticket 28](28-class-rules-follow-ups.md)) and Domain Zealot's domain.

Data from the saved guide pages only (offline). Model changes only if a family doesn't fit; if so, stop and write the question here.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB). **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a cleric 8 holds both domains' second powers, an oracle's revelation slot offers only its mystery's revelations, a warpriest 10 holds both major blessings.
