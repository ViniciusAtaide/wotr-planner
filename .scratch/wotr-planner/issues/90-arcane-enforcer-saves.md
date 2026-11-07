# Arcane Enforcer saves: good Reflex and Will, poor Fortitude

Type: task
Status: resolved
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

## Progress log

- 2026-11-07: started (agent; the user away, small choices settled here). Read the repo's `CLAUDE.md`, the map's Notes and this ticket. Scratch in `t90/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-7437-4fe9-8ef8-ebb8716f6fbe/scratchpad/t90/`).
- 2026-11-07: the save sweep (`t90/sweep.py`: every "saving throws for" sentence in `guide-cache/` with the nearest section id). Each base class page has its own sentence (twice, under `#faqwrap`): alchemist, arcanist, barbarian, bard, bloodrager, cavalier, cleric, druid, fighter, hunter, inquisitor, kineticist, magus, monk, oracle, paladin, ranger, rogue, shaman, shifter, skald, slayer, sorcerer, warpriest, witch, wizard. The only archetype with one is `slayer#arcane-enforcer`. The rest are not save progressions: `character-creation#companion-levelling` (companions, out of the planner), and bonuses "to saving throws for N rounds" (`character-creation#nobility-domain`, `oracle#battlecry`, `paladin#shining-light`, `paladin#bestow-grace-abjuration`, `rogue#emboldening-strike`, `shifter#breath-weapon-buse-dragon`, `warpriest#nobility`, `warpriest#protection`, `wizard#diviners-fortune`). So the QA agent's finding holds: the Arcane Enforcer is the only archetype to change.
- 2026-11-07: settled here: an archetype's replaced save also drops the class's `save_exceptions` for that save (they are cells of the class's own table; no data has both today), and the check refuses a replacement equal to the class's progression (as it refuses an archetype row equal to the base row). Three flat optional fields `fort`/`reflex`/`will` on `Archetype`, mirroring `Class`'s, like `skill_points` and `spontaneous`.
- 2026-11-07: rules test-first on the small world: 3 tests (2 in `derived.rs`, 1 in `check.rs`), red with the fields added and nothing reading them (the armiger read the fighter's saves; the check found nothing), green after. Real-data tests (`archetype_saves.rs`, 4): 3 red on the current data (the Enforcer and the QA link read Fort +2 / Ref +2 / Will +0), green after the `slayer.ron` change. Dev database rebuilt per `CLAUDE.md`. RustRover's `create_new_file` staged the new test file; unstaged it (`git restore --staged`).
- 2026-11-07: proof (`scripts/proof.sh both` against HEAD; HEAD's crates can't read the new fields, `deny_unknown_fields`) and a save-only sheet check (`t90/sheetcheck/`); `scripts/check.sh --schema` ALL-OK. Answer written; Status resolved.

## Answer

Done 2026-11-07, uncommitted. An archetype can now replace any of its class's three save progressions, and the Arcane Enforcer has poor Fortitude and good Reflex and Will. The repro Build reads Fort +0 / Ref +2 / Will +2 at level 1; Arcane Enforcer 13 reads Fort +4 / Ref +8 / Will +8; a plain slayer is unchanged (Fort +2 / Ref +2 / Will +0 at 1, +8 / +8 / +4 at 13).

### Model (`rules`)

- `Archetype` has `fort`, `reflex`, `will: Option<SaveProgression>` (serde default `None`: the class's). `Archetype::save_progression(save)` reads one.
- `Class::save_bonus(save, class_level, prestige, archetype)`: the archetype's progression's formula when it replaces that save (the class's `save_exceptions` for that save no longer apply: they are cells of the class's table), else the class's exception cell, else the class's formula. The level walk's `recompute_tables` passes each class entry's archetype (`Catalog::archetype_of`).
- `GameData::check`: `DataIssue::SaveProgressionSameAsClass(Save)` refuses an archetype's progression equal to its class's (leave it out instead).
- No derived value is stored: the data and the database hold the chosen progressions; `rules` computes the numbers.

### Data

`libs/game-data/classes/slayer.ron`: `slayer/arcane-enforcer` has `fort: Some(Poor)` and `will: Some(Good)`; Reflex is left out (the slayer's good). `libs/game-data/README.md`: the class/archetype save paragraph says how an archetype names its progressions, and a ticket 90 paragraph records the guide's sentence and the sweep. No other archetype changes (sweep in the progress log).

### Schema

`archetype` has three nullable columns `fort`, `reflex`, `will` (`CHECK IN ('good', 'poor')`, NULL: the class's). The seed writes them and the load reads them back (`seeds::columns`' existing save-progression mapping). The dev database was rebuilt per `CLAUDE.md` (deleted, `sqlx database create` + `migrate run`, reseeded with unchanged counts, `cargo sqlx prepare`: the archetype insert's and select's two `.sqlx` files replaced).

### Tests

- **Rules, small world** (3): the armiger given poor Fortitude and good Will has F0 R0 W2 at 1 and F1 R1 W3 at 3, a plain fighter F3 R1 W1 at 3; a replaced Will ignores the fighter's Will cell while the fighter's Fortitude cell still applies; the check refuses the armiger's Fortitude Good and Will Poor (the fighter's own) and accepts Reflex Good.
- **Real data** (`libs/seeds/tests/game_data/archetype_saves.rs`, 4): Arcane Enforcer 1 and 13; a plain slayer 1 and 13; the ticket's repro link opens at slayer 1 with (0, 2, 2); the Arcane Enforcer is the only archetype with a save progression of its own.
- `cargo test --workspace -- --list`: 736 tests, the 7 new ones among them (`t90/test-list.txt`).

### Proof

`scripts/proof.sh both`, against HEAD (`t90/proof/`, log `t90/proof.log`): 2,146 Builds on each side, none only on one side, 31 differ, each in `sheet` only (the sheet hash; the proof records no save field of its own). The 31 are exactly the proof's 31 Arcane Enforcer Builds. Ticket 48's shortcut stayed exact (1,392,717 feat options on 955 Builds, both sides).

The save-only sheet check (`t90/sheetcheck/`, `t90/sheetcheck.txt`: the working tree's rules over HEAD's data and the working tree's data, every proof Build): 2,115 sheets equal; 31 differ, and with every level's saves left out all 31 are equal, so nothing but saves moved; all 31 take the slayer as an Arcane Enforcer; 369 levels' saves change, each by exactly the Enforcer's difference at its slayer level (Fortitude poor − good, Reflex 0, Will good − poor), at slayer levels 1–20. No other difference.

### Checks

- `scripts/check.sh --schema`: ALL-OK (fmt, clippy, wasm-clippy, test, rules-wasm, seed-check, sqlx-migrate, sqlx-prepare).
- `cargo fmt --all --check` passes. `--full` not run (per ticket, as usual).
- Nothing staged or committed.
