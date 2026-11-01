# Store class progressions, not copies

Type: task
Status: resolved
Blocked by: 21
Map: [WotR Build Planner](../map.md)

## Question

**User decisions (2026-11-01, [ticket 21](21-schema-questions-real-data.md) S1 and S2):** the database stops storing numbers a rule derives, and archetypes stop copying their base class.

**BAB and saves:**
- `Class` gains `bab: BabProgression { Full, ThreeQuarters, Half }` and `fort`/`reflex`/`will: SaveProgression { Good, Poor }`. `ClassLevelRow` loses `bab`, `fort`, `reflex` and `will`.
- The engine computes them: base good `2 + ⌊L/2⌋`, base poor `⌊L/3⌋`, prestige good `⌊(L+1)/2⌋`, prestige poor `⌊(L+1)/3⌋`; BAB full `L`, ¾ `⌊3L/4⌋`, ½ `⌊L/2⌋`. Check these against the current stored rows before trusting them.
- The 9 cells where the guide's table differs from the formula (ticket 21, section 2: hunter, paladin, shifter, warpriest and five prestige classes at Will L4) are kept exactly as the guide prints them: a `save_exception (class_key, level, save, value)` table and a matching Rust field. `GameData::check` flags an exception equal to the formula.

**Sparse archetype rows:**
- `Archetype.levels: Vec<ArchetypeLevelRow>`, with `ArchetypeLevelRow { level, spell_levels: Option<SpellLevels>, features, feat_slots, selections }`. A present row replaces the base row's grants in full, and its `spell_levels` replace the base's when set. A missing level uses the base row.
- One `effective_row(class, archetype, level)` in the engine; nothing else merges rows.
- `GameData::check` flags an archetype row identical to the base row, and a spell override equal to the base.
- Ticket 21 section 1 has the shapes and the counts (2298 archetype rows should go).

**Build** (AFK, test-first `tdd`; you're the only agent touching the migration and the DB, per `CLAUDE.md`):
- **Schema:** edit the one migration (progression domains, four columns on `class`, drop them from `class_level`, spell levels nullable on archetype rows with a CHECK, `save_exception`). Reset and reseed per `CLAUDE.md`.
- **Data:** rewrite `game-data/**/*.ron` and the fixture with a one-off script (scratchpad), never the extractor. **Prove the rewrite is lossless:** for every class, archetype and level 1–20, the effective row and the computed BAB/saves (with exceptions) after the change equal the stored row before it. Record the script and the result here.
- **The extractor** emits the new shape, so a future run matches.
- Update `game-data/README.md`, `GLOSSARY.md` if a term changes, and the `data/tests/files.rs` allow-list test (it goes away if nothing copies any more).
- **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes (workspace tests, clippy, `cargo fmt --all --check`, wasm build of `rules`, `SQLX_OFFLINE=true cargo leptos build`, `cargo sqlx prepare --check`, `--check-only` and the real seed with the same counts), plus the lossless proof.

## Progress log

- 2026-11-01: baseline in the session scratchpad `t23/`: `old-src/` is `git archive HEAD`; `extract-old/` is the HEAD extractor's output (82 report items, 0 check problems); `old-rows.jsonl` (4570 rows) and `old-fixture-rows.jsonl` (25) dump every stored class/archetype/prestige row with the HEAD code (`proof-old/`). The rewrite script `rewrite.py` dry-runs clean: real data 39 classes, 2298 of 3920 archetype rows dropped, 1622 kept (72 with a spell override), 9 save exceptions (exactly ticket 21's), BAB has no exceptions. Next: rules types and engine, test-first.
- 2026-11-01: `rules` done: `Class` has `bab`/`fort`/`reflex`/`will` progressions and `save_exceptions`, `ClassLevelRow` lost its numbers, `Archetype.levels` is `Vec<ArchetypeLevelRow>` (sparse, `spell_levels: Option<SpellLevels>`), and `engine::effective_row` is the one merge (public, with `Class::base_attack_bonus`/`save_bonus`). `GameData::check` flags rows outside the class or twice, rows identical to the base, spell overrides equal to the base, and save exceptions outside the class, twice or equal to the formula (new tests in `check.rs`, `derived.rs`, new `archetype_rows.rs`; the check tests were red first, the engine tests were written right after the engine change). Test world moved to the new shape. `cargo test -p rules` 111 pass, clippy clean, wasm builds. Next: `data` crate (files, schema, seed, load) and the data rewrite.
- 2026-11-01: `data` done (migration: `bab_progression`/`save_progression` domains, four columns on `class`, `save_exception`, `class_level` without numbers and with nullable spell levels/advances plus CHECKs; seed, load, columns; tests: allow-list test removed, new reads/round-trip/schema tests). Dev DB reset (only `wotr-planner-postgres`, 5434), migrated, real seed, `cargo sqlx prepare`. `rewrite.py` ran over `game-data/` and the fixture (same counts as the dry run). **Lossless proof passes**: `proof-new/` (new code: `effective_row` + `base_attack_bonus`/`save_bonus`) dumps 4570 real rows and 25 fixture rows byte-identical to `old-rows.jsonl`/`old-fixture-rows.jsonl`. Next: extractor, docs, full verification.
- 2026-11-01: extractor done: new `extract::progressions` (`progressions()` picks each column's progression, save misses become exceptions, BAB misses are reported; `sparse_archetype_rows()` runs after `drop_dangling`), `build.rs` keeps numbers beside rows, `ron_out.rs` writes `bab`/`fort`/`reflex`/`will`/`save_exceptions` after `skill_points` and archetype rows with `spell_levels`. 5 new tests in `extract/tests/extract/progressions.rs`. A fresh extraction (`t23/extract-new/`) equals `rewrite.py` applied to the HEAD extractor's output (`t23/extract-old-rewritten/`), all files and the report. Next: docs, full verification.
- 2026-11-01: docs (`game-data/README.md`, `GLOSSARY.md` gains Progression, Save exception and Archetype row), final DB reset and reseed, `cargo sqlx prepare`, every Verification check passes, lossless proof rerun on the final files. Done.

## Answer

Done 2026-11-01, uncommitted. Classes store their BAB and save progressions plus the guide's 9 deviating save cells; level rows store no numbers; archetypes store only the levels where they differ from their base class, and `rules::effective_row` is the one place rows merge.

### Rust (`rules`)

- `Class` gains `bab: BabProgression { Full, ThreeQuarters, Half }`, `fort`/`reflex`/`will: SaveProgression { Good, Poor }` and `save_exceptions: Vec<SaveException { level, save: Save, value }>` (`Save { Fortitude, Reflex, Will }`). `ClassLevelRow` loses `bab`, `fort`, `reflex`, `will`.
- Formulas live on the enums: `BabProgression::bab(L)` is `L`, `⌊3L/4⌋`, `⌊L/2⌋`; `SaveProgression::save(L, prestige)` is base good `2 + ⌊L/2⌋`, base poor `⌊L/3⌋`, prestige good `⌊(L+1)/2⌋`, prestige poor `⌊(L+1)/3⌋`. Checked against every stored cell before trusting them: BAB fits all 39 classes, saves fit all but the 9 cells below. `Class::base_attack_bonus(L)` and `Class::save_bonus(save, L, prestige)` apply the exceptions.
- `Archetype.levels: Vec<ArchetypeLevelRow { level, spell_levels: Option<SpellLevels>, features, feat_slots, selections }>` with `SpellLevels { arcane, divine }`.
- `engine::effective_row(class, archetype, level) -> Option<EffectiveRow>` (public): the archetype's row at that level replaces the base row's grants in full and its spell levels when set; a missing level is the base row; the advances flags always come from the class row. The engine's level grants, slots, spell-level access, "casts arcane/divine" and the advances check all go through it. An archetype's level cap is its class's table length. Behaviour is otherwise unchanged (a class level past the table still adds no BAB or saves).
- `GameData::check` adds `ArchetypeRowOutsideClass`, `DuplicateArchetypeRow`, `ArchetypeRowSameAsBase` (grants equal as multisets and no spell override, or the base's), `SpellOverrideSameAsBase`, spell overrides above 9, and `SaveExceptionOutsideClass`, `DuplicateSaveException`, `SaveExceptionSameAsFormula`. `LevelRowsNotContiguous` now applies to class tables only.
- New tests: `archetype_rows.rs` (5), `derived.rs` (prestige save formula; an exception replaces one cell), `check.rs` (4). The test world uses progressions; its prestige classes now get the prestige save formula, and the Armiger stores rows at 1 and the even levels only.

### Schema (the one migration)

- Domains `bab_progression` (`full`, `three_quarters`, `half`) and `save_progression` (`good`, `poor`); `class.bab`, `fort`, `reflex`, `will` NOT NULL.
- `save_exception (class_key → class, level 1–40, save IN ('fortitude','reflex','will'), value u8, PRIMARY KEY (class_key, level, save))`. 26 tables.
- `class_level` drops `bab`/`fort`/`reflex`/`will`. Spell levels are nullable, set together, and required on class rows; `advances_arcane`/`advances_divine` are set on class rows only (NULL on archetype rows). A postgres test checks each CHECK and domain.
- Seed and loader: archetype rows insert NULL spell levels when unset; the loader splits class rows from archetype rows and rejects a row with the other kind's columns; save exceptions round-trip (new test with two exceptions added to a fixture copy).

### Data rewrite

Script: `t23/rewrite.py` in the session scratchpad (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t23/`). Line-based over `classes/*.ron` and `prestige/*.ron` (every level row is one line): classify each base table's columns (asserting BAB fits exactly), write `bab:`/`fort:`/`reflex:`/`will:` (and `save_exceptions:`) after `skill_points:`, strip the numbers from rows, and for each archetype row assert its numbers equal the base row's, then drop it if its spell levels and grants (multisets; `OneOf` feats as a set) equal the base row's, else keep it with `spell_levels: (...)` only when they differ. Everything else stays byte for byte; the extractor was never run over `game-data/`.

| | Real data | Fixture |
| --- | --- | --- |
| classes rewritten | 39 | 6 |
| archetype rows before → after | 3920 → 1622 (2298 dropped) | 9 → 6 (3 dropped) |
| rows keeping a spell override | 72 | 3 |
| save exceptions | 9 | 0 |

- The 9 exceptions are exactly ticket 21's: hunter Will 7 (+3), paladin Will 20 (+20), shifter Reflex 2 (+2), warpriest Will 20 (+11), and Will 4 (+3) for arcane-trickster, loremaster, mystic-theurge, student-of-war and winter-witch.
- BAB: 15 full, 16 ¾, 8 ½ (base 9/13/4, prestige 6/3/4).
- Fixture: the two-row prestige classes fit ¾ and ½ alike; the script takes ½ (what the game gives both). Its `cleric/herald` was all copies, so it now has no `levels` (the empty-archetype case is covered).
- Database rows after the reseed: `class_level` 4570 → 2272, `granted_feature` 4026 → 2996, `granted_selection` 687 → 334, `feat_slot` 618 → 365, `feat_slot_option` 3714 → 2325: exactly ticket 21's predicted savings.

### Lossless proof

Two scratch programs (`t23/proof-old/` built against `git archive HEAD`, `t23/proof-new/` against the working tree) dump one JSON line per class, archetype and level of the class's table: level, BAB, Fort, Reflex, Will, arcane and divine spell levels, both advances flags, and the sorted features, feat slots and selections. The old one prints the stored rows; the new one prints `effective_row` plus `base_attack_bonus`/`save_bonus` (with exceptions). **Result: identical**, 4570 real lines (520 base, 3920 archetype, 130 prestige) and 25 fixture lines, byte for byte (`cmp`), run after the rewrite and again on the final files.

### Extractor

- New `extract::progressions`: `progressions(rows, prestige)` picks each column's closest progression; save cells it misses become exceptions, BAB cells it misses are reported (none in the guide). `sparse_archetype_rows(class)` turns full archetype tables into the rows that differ, and runs after `references::drop_dangling`, so a difference that was only a dangling key doesn't keep a row. 5 tests in `extract/tests/extract/progressions.rs`.
- `build.rs` keeps each row's numbers beside it, sets the class's progressions from them, and reports an archetype table whose numbers differ from its base class's (none do). `ron_out.rs` writes the new header fields and archetype rows with `spell_levels`.
- A fresh extraction into the scratchpad (`t23/extract-new/`) equals `rewrite.py` applied to the HEAD extractor's output (`t23/extract-old-rewritten/`): every `.ron` file and `EXTRACTION_REPORT.md` (82 items, 0 check problems) are identical.

### Docs

- `game-data/README.md`: progressions, `save_exceptions`, sparse archetype rows and what the check refuses; the allow-list paragraph is gone.
- `GLOSSARY.md`: new Classes section with Progression, Save exception and Archetype row.
- `data/tests/files.rs`: the allow-list test and its helper are gone (nothing copies numbers any more); new tests read progressions, sparse rows and save exceptions.

### Verification

- `cargo test --workspace`: pass (rules 111 + unit 1, extract 43, data files 13 + postgres 15).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass (8 query files added, 5 removed).
- Dev DB reset per `CLAUDE.md`, only `wotr-planner-postgres` on 5434: drop/create `wotr_planner`, `sqlx migrate run --source data/migrations`, `cargo run -p data --bin seed`, `cargo sqlx prepare`.
- `--check-only` and the real seed both give 12 races, 11 skills, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats and 10 mythic paths (unchanged).
- Lossless proof: identical (above).

### Left as it was

- The extractor's existing report text "no BAB or saves for this level; stored as 0" is wrong (the row is skipped, not stored as 0). It never fires on the guide; changing it would change the report, so it's untouched.
