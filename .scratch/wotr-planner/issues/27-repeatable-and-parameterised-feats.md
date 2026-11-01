# Repeatable and parameterised feats

Type: task
Status: open
Blocked by: 26
Map: [WotR Build Planner](../map.md)

## Question

**User decision (2026-11-01, [ticket 21](21-schema-questions-real-data.md) R4):** mark the feats (and selection options) that can be taken more than once, and record which skill or weapon each copy is for, because prerequisites name it (Weapon Specialization needs Weapon Focus in the same weapon; some prestige entries need Weapon Focus (X)).

- **Repeatable:** a fact on `Feat` and `SelectionOption` (once / a fixed number / per parameter). Examples from the hand-fix tickets: Skill Focus (Master of All ×3), Magical Tail, the slayer's ranger combat feats "three times", Increased Damage Reduction "twice".
- **Parameter:** a feat names what it's taken for (a skill, a weapon type, a school, an energy type, …), and a pick of it carries the value. Prerequisites can require a specific value, or "the same value as this pick".
- **Values:** skills come from `skills.ron`. For weapons and anything else, use only what the saved guide pages list (`guide-cache/`); never fetch. If the guide doesn't settle the list or the rule (for example weapon type vs weapon group), **stop and write the question in this ticket** for the user instead of inventing it.
- **URL codec:** a pick carries its parameter. No link has shipped, so regenerate the frozen v1 vector; no v2.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB). **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus tests that Weapon Focus can be taken twice for two weapons, not twice for one, and that Weapon Specialization checks the matching weapon.
