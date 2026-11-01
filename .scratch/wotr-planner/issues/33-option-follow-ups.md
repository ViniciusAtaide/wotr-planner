# Option follow-ups

Type: task
Status: open
Blocked by: 32
Map: [WotR Build Planner](../map.md)

## Question

Build the user's decisions (2026-11-01, "all recommended") from the open questions of tickets [30](30-build-option-levels.md) and [31](31-build-option-levels-more-families.md). Each ticket's `## Decisions on the open questions` is the spec.

- **Option groups** (30-1, 31-6): options carry a group (Draconic, Elemental, …) and a prerequisite "no other option of this group". A Crossblooded or Second Bloodline sorcerer can't hold two dragons; Expanded Element at 15 can't repeat the 7th-level element.
- **Crossblooded** (30-2): the second bloodline also grants its Arcana.
- **Daily choices** (30-10): the Unsworn Shaman's spirits and the hunter's Animal Focus.
- **Elemental Movement** (30-11): one per element, none for Earth.
- **Separatist** (31-1): a selection's second pick can lag by a set number of class levels; its second domain lags by two.
- **Divine Hunter** (31-2): every domain power, by hunter level.
- **Impossible Domain** (31-5, a user fact): grants one domain from the full list, levelled by the class that gives the character's other domains (cleric, inquisitor or hunter).
- **Revelations** (31-7): Cinder Dance needs no Lame curse; the Dual-Cursed Oracle is offered Misfortune and Fortune; Maneuver Mastery and Second Mystery count arcanist levels for a Living Deity.
- **Champion of the Faith** (31-8): its alignment blessing checks the character's alignment.
- **Signifer** (31-10): its order is a selection sharing the Hellknight's orders.

Data from the saved guide pages only (offline). If the guide doesn't settle something, stop and write the question here.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB). **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a Crossblooded sorcerer can't take Red after Blue; a Separatist cleric 1 holds only its first domain's level-1 power, and a Separatist cleric 3 holds both domains' level-1 powers; a Divine Hunter 8 holds its domain's level-8 power; a Living Deity with Impossible Domain holds a third domain.
