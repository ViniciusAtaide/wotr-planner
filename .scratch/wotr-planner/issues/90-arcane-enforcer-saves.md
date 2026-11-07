# Arcane Enforcer saves: good Reflex and Will, poor Fortitude

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Found by the QA builds ([ticket 88](88-qa-full-builds.md)). An Arcane Enforcer gets the base slayer's saves (good Fortitude and Reflex, poor Will). The guide gives it good Reflex and Will and poor Fortitude. Archetypes can't change a save progression today: `rules::Archetype` has no save fields, and `Class::fort`/`reflex`/`will` are the only ones (`libs/rules/src/game_data.rs`).

**Repro:** the Enforcer Build from ticket 88 at level 1 (`#b=AQFQAK__DFwlwpHwUQ-GdkLAnXq5_aYx6Q-Gdo0MN_9IBaYx6WEsyN0dGAABAAEDAQEQEA4KCAwBAAMBAQEBAQAAAAIAAgEAAAABAAIAAAABAQEAAQA`): the summary reads "Level 1 · Slayer 1", "BAB +1 · Fort +2 · Ref +2 · Will +0". At Slayer 13 it reads "Fort +8 · Ref +8 · Will +4".

**The guide** (`slayer#arcane-enforcer`): "High saving throws for Reflex and Will, low saving throws for Fortitude". It's the only archetype on the saved pages whose saves differ from its class's (a sweep of every "saving throws for" sentence in `guide-cache/`).

**Expected:** Arcane Enforcer 1 has Fort +0, Ref +2, Will +2; Arcane Enforcer 13 has Fort +4, Ref +8, Will +8 (the guide's good and poor formulas, as the base classes use).

## Build

- Let an archetype replace one or more of its class's save progressions (optional fields, `None` = the class's), carried through the files, the migration, the seed and `GameData::check`. Set the Arcane Enforcer's. Test-first (`tdd`) in `rules`; a real-data test in `libs/seeds/tests/game_data/`; a `libs/game-data/README.md` note.
- `scripts/check.sh --schema`, and the behaviour proof (`scripts/proof.sh`) showing only Arcane Enforcer saves move.

**Done when:** the repro Build shows Fort +0 / Ref +2 / Will +2 at level 1 and Fort +4 / Ref +8 / Will +8 at Slayer 13; a plain slayer is unchanged; the tests and checks pass.
