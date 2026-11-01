# Backgrounds

Type: task
Status: open
Blocked by: 33
Map: [WotR Build Planner](../map.md)

## Question

The user decided (2026-11-01) that a background is a level-1 character-creation choice that makes its skills class skills. Healer counts Int instead of Wis for its skills, and Acolyte Wis instead of Cha. Backgrounds don't change which feats are legal.

The facts are in [research/skills-from-user.md](../research/skills-from-user.md), but they name some backgrounds only by group: "any Craftsman", "any Wanderer", "any other Oblate", "any Noble" and "any Street Urchin" background. The saved guide pages don't list backgrounds. **Ask the user for each group's backgrounds** (HITL) before building; never fetch them from the web.

**Build** (AFK once the list is in, test-first `tdd`; only agent touching the migration and DB): a `backgrounds.ron`, the Build start's background choice (regenerate the frozen v1 link vector; no v2), class skills from it, and the schema, seed and loader. **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, and a Shackles Corsair holds Athletics and Mobility as class skills.
