# Class rules from the hand-fix follow-ups

Type: task
Status: open
Blocked by: 27
Map: [WotR Build Planner](../map.md)

## Question

**User decisions (2026-11-01, [ticket 21](21-schema-questions-real-data.md) R5 and R6)**, plus ticket 21's implementation follow-ups 8 and 11:

- **Prestige levels counting as another class:** a `counts_as (class, as_class, for)` fact, where `for` is feat prerequisites or a feature's progression. Covers Diverse Training (Eldritch Knight levels as fighter levels for feats) and Frost Power (Winter Witch stacks with witch/shaman for hexes). Dragon Disciple's bloodline continuation waits on [ticket 29](29-design-option-level-grants.md).
- **Advances one class, arcane or divine** (Signifer, Loremaster, Winter Witch): a per-class `advances: Either | Each` that replaces the always-true flags on these three.
- **Story conditions** (Vow of Dahak, Uncorrupted): out of scope. The option is allowed, and the sheet shows a note. Choose the smallest shape (for example a note on the prerequisite that never blocks).
- **Crusader:** keeps skipping all prerequisites (no change; note it in the data).
- **Assassin:** "neutral or evil" means true neutral or any evil alignment; neutral good is excluded. Fix the data and add a test.
- **Druid Nature Bond:** a `druid/nature-bond` selection (animal companion or domain) at L1, with Domain Zealot requiring the domain option. The domain's own powers wait on ticket 29.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB). **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus a test per bullet.
