# Second round of follow-ups

Type: task
Status: open
Blocked by: 33
Map: [WotR Build Planner](../map.md)

## Question

Build the user's decisions (2026-11-01, "all recommended") from the open questions of tickets [32](32-feat-and-rule-follow-ups.md) and [33](33-option-follow-ups.md). Each ticket's `## Decisions on the open questions` is the spec.

- **Half-elf subraces** (32-1): Dual Heritage and Kindred Raised don't get Adaptability.
- **Impossible Domain** (33-2): a Living Deity's extra domain is levelled by arcanist levels from the pick; a Hellknight with Pentamic Faith's domain qualifies, levelled by Hellknight levels.
- **Signifer's order** (33-3): offered only to a character holding no order.
- **Separatist** (33-6): no alignment domain that conflicts with its alignment.
- **Ranger/slayer** (33-7): one shared style; the slayer's feats come from its talent window's stage.
- **Bloodrager groups** (33-8): its draconic and elemental bloodlines are grouped.

Data from the saved guide pages only (offline). If the guide doesn't settle something, stop and write the question here.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB). **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a Dual Heritage half-elf has no Adaptability pick; a Living Deity with Impossible Domain holds the third domain's level-1 power before arcanist 20; a Hellknight 3 with Pentamic Faith's domain can take Impossible Domain; a Hellknight who becomes a Signifer gets no second order; a lawful good Separatist can't take Chaos or Evil; a ranger 10 / slayer 2 takes the slayer's 2-or-4 feat from the first stage; a bloodrager with a dragon bloodline can't take a second dragon through Second Bloodline.
