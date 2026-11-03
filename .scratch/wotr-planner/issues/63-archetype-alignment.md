# An archetype's alignment requirement replaces its class's

Type: task
Status: open
Blocked by: 62
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 58](58-monk-ki-powers.md)'s question 1, answered by the user (2026-11-02): "drunken master can be built as long as you check the alignment per archetype and not on the base class". Today a Drunken Master can't be built at all: the monk's prerequisite needs a Lawful alignment (`classes/monk.ron`, `Any([Alignment(LawfulGood), …])`), the archetype's needs a non-Lawful one, and both are checked when the class is taken.

The rule (a user fact): when an archetype states an alignment requirement, it replaces the class's alignment requirement; the class's other prerequisites still hold. An archetype without one keeps the class's.

More examples from the user (2026-11-02): "the mantis archetype and the vampire slayer must be evil".

- **Bloodseeker** (the slayer's vampire archetype: "Alignment must be evil" in `slayer#bloodseeker`) already has `Any([LawfulEvil, NeutralEvil, ChaoticEvil])` in `classes/slayer.ron`. The slayer has no alignment rule of its own, so it's buildable today; keep it enforced (a test).
- **Mantis Zealot** (`warpriest/mantis-zealot`) has no alignment rule in the data: the guide says only "May only select Achaekek as a deity", and deities are out of scope. The user's fact adds one. The exact alignments: see the Decisions below.

Check every archetype with an alignment requirement for the same clash with its class (and for one that narrows rather than replaces it: the rule is the same, the archetype's wins). List what you find in the Answer.

## Build

AFK, test-first (`tdd`); the only agent in the repo. Probably a rules change (where a class level's prerequisite is checked with its archetype), maybe a data change if the alignment leaves need separating; schema change and DB reset allowed per `CLAUDE.md` if the types need one. Engine code reaches game data through ticket 56's `Catalog`. Before/after proof with ticket 58's program: only Builds with an archetype that states an alignment may differ. Ticket 48's picker shortcut must stay exact. Reasons on the sheet name the archetype's alignment rule, not the class's, for such an archetype.

**Done when:** the lighter per-ticket checks pass (fmt, clippy native and wasm, `cargo test --workspace`, the `rules` wasm build, seed `--check-only`, `sqlx prepare --check` if the schema changed); plus real-data tests: a Chaotic Neutral monk 1 can take Drunken Master; a Lawful Neutral one can't, with the archetype's alignment as the reason; a plain monk still needs Lawful; a Bloodseeker needs an evil alignment; a Mantis Zealot is open at LE, LN and NE and locked at CE and LG; every archetype found with a clash is buildable at an alignment its own rule allows; the sheet shows the Drunken Master's alignment reason (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer

## Decisions

**User, 2026-11-02:**

1. **The Mantis Zealot allows Lawful Evil, Lawful Neutral or Neutral Evil**: one step from Achaekek (Lawful Evil). A user fact; add it to `warpriest/mantis-zealot` as its alignment rule (`Any([Alignment(LawfulEvil), Alignment(LawfulNeutral), Alignment(NeutralEvil)])`), with a note in `libs/game-data/README.md` that it comes from the user, not the guide.
