# Store class progressions, not copies

Type: task
Status: open
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
