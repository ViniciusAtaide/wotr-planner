# Extract the guide into data files

Type: task
Status: resolved
Blocked by: 08, 09, 16
Map: [WotR Build Planner](../map.md)

## Question

Build the extractor crate and run it once over `guide-cache/` to produce the real `game-data/` files (layout in [Game data schema and seeding](05-game-data-schema.md)). Parsing facts are in [research/guide-anatomy.md](../research/guide-anatomy.md).

- Read the `div#faqwrap` content and rebuild nesting from heading levels. Keys come from the `<a id>` anchors.
- Parse progression tables, handling these quirks:
  - "+6 / +1" BAB values
  - row 0
  - the `Feat` column, which marks single-class odd levels and is to be ignored
  - varied headers
  - footnote digits glued to cell text
  - bonus slots shown only in the Special columns
- Build the prerequisite-line grammar test-first, using short hand-written examples:
  - `or`, `+`, "One of" / "Two of"
  - missing colons, BAB abbreviations
  - "Ability to cast Nth-level arcane spells"
  - the misspelt "Prerequsites" / "Prerequsities" labels
- Write a report of every line the parser couldn't handle, so it can be fixed by hand.
- The extractor refuses to overwrite existing data files.
- Encode the rules that the guide only states in prose: totem exclusivity, advanced rogue talents at 10, Major/Grand hexes, Trickster tiers, and path switch rules.
- Facts the user gave go straight into `progression.ron`:
  - the mythic rank table
  - combat feats are every category except class, racial, skill, teamwork and magic (miscellaneous counts)
  - skill ranks are capped at character level

Done when `cargo run -p data --bin seed` loads the full data cleanly and the report of unparsed lines is empty or has been fixed by hand. AFK: driven by the agent.

## Comments

**2026-10-31:** bonus-feat slots that ignore prerequisites are a per-slot flag ([Bonus feats ignore prerequisites](16-bonus-feats-ignore-prerequisites.md)). Set it from the class page's wording: a class's own fixed bonus-feat list ignores prerequisites, while "any combat feat" and general slots don't. Report unclear cases. Also record whether each class's BAB and saves follow a standard progression, to answer the schema question on the map.

**2026-11-01:** the user narrowed this ticket to stage A of a staged extraction: the extractor writes `game-data/` and a report grouped by category (Feats, Base classes, Prestige classes, Mythic, Races and skills), only what blocks `GameData::check()` or the seed is hand-fixed, a database-free check command is added for the later stages, and the real data is seeded once. The broad hand-fix pass, the schema-question analysis and the follow-up tickets for prose rules move to the per-category stages.

## Progress log

- **2026-11-01:** partial crate assessed. The `extract` crate is complete enough to run: `cargo test -p extract` passes 33 tests, and a dry run into a scratch folder writes every file (12 races, 11 skills, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 paths) with 0 `GameData::check` problems and 43 distinct report items. Next: fix the printed report count (it counts duplicates), write `game-data/`, seed.
- **2026-11-01:** scope narrowed to stage A (see Comments). Found that the sorcerer and wizard base tables grant nothing (their per-bloodline and per-school tables sit under h4 headings the extractor doesn't read); left for the Base classes stage and to be listed in the report. Next: report grouped by category, `seed --check-only`, write `game-data/`.
- **2026-11-01:** extractor runs end to end, and `game-data/` written (`cargo run -p extract -- --force`, next to the README): 82 distinct report items, grouped by category; a second run refuses to overwrite. `seed --check-only` added and passes on it. Next: seed the dev database.
- **2026-11-01:** seed passes on the first try (real data into `wotr-planner-postgres`, same counts read back); nothing blocked `GameData::check()` or the foreign keys, so no hand-fix was needed. Next: workspace checks, `cargo leptos serve`, recording.
- **2026-11-01:** workspace checks pass and `cargo leptos serve` showed the real counts (server stopped). Stage A done; recorded below.

## Answer

Stage A only (scope narrowed by the user, see Comments). Verified 2026-11-01, uncommitted.

### What exists

- **`extract` crate** (`cargo run -p extract -- [--guide DIR] [--out DIR] [--force]`): reads the 38 pages in `guide-cache/` and writes `game-data/**/*.ron` in the documented layout plus `game-data/EXTRACTION_REPORT.md`. It refuses a folder that already holds `.ron` files even with `--force` (`--force` only allows other files such as the README), and checks that the written files read back as exactly what it built.
- **Report** grouped by category, one section per later stage: `## Feats`, `## Base classes`, `## Prestige classes`, `## Mythic`, `## Races and skills`, each split by kind (prerequisite lines, entry requirements, table cells, bonus feat slots, prose-only rules). Items repeated by archetypes are listed and counted once. Category comes from the item's page (`character-creation#bloodlines` counts as Base classes).
- **Database-free check:** `cargo run -p data --bin seed -- --check-only [DIR]` reads the files and runs `GameData::check()` without connecting to Postgres (new `data::check_game_data_files`). Keys that don't exist are still caught only by a real seed's foreign keys.

### Counts (seeded and shown by `cargo leptos serve`)

| Category | Extracted | Guide |
| --- | --- | --- |
| Races | 12 (with subraces) | — |
| Skills | 11 | 11 |
| Feats | 261 | 256 on `feats`, plus 5 Trickster bonus feats |
| Base classes | 26 | 26 |
| Archetypes | 196 | about 196 |
| Prestige classes | 13 | 13 |
| Mythic abilities | 63 | 63 |
| Mythic feats | 56 | 56 |
| Mythic paths | 10 | 7 path pages + Legend, Devil, Swarm That Walks |

`progression.ron` holds the mythic rank table (MR 1–10 at levels 6/7/9/11/14/15/16/17/17/19).

### Report: 82 items

| Section | Items | Mostly |
| --- | --- | --- |
| Feats | 4 | prerequisite lines naming nothing in the data |
| Base classes | 64 | 39 progression tables under h4 headings that aren't read (sorcerer bloodlines 9, wizard schools 9, bloodrager bloodlines 10, cavalier orders 5, arcanist Magic Deceiver 6), so the **base sorcerer and wizard grant nothing** beyond BAB, saves and spells; 11 prerequisite lines (mostly shaman spirit names); 9 bonus slots with no list found (stored as general); 3 prose rules; 1 entry requirement |
| Prestige classes | 7 | 4 entry requirements, 3 "advances arcane or divine" |
| Mythic | 7 | 2 prerequisite lines, the Gold Dragon feat slots, 4 prose rules (Legend has no rank table, switch details, Devil switch limit) |
| Races and skills | 0 | — |

### Fixed

Nothing needed hand-fixing for stage A: the first write had 0 `GameData::check()` problems and the first seed passed (the extractor already drops references to missing keys and reports them). Code changes this session:

- `extract`: the report grouping and distinct count (`report.rs`, tests in `tests/extract/report.rs`); reporting the unread nested progression tables; the printed item count now counts distinct items; one clippy fix.
- `data`: `check_game_data_files` (tests `checking_the_files_needs_no_database_and_passes_the_fixture`, `checking_the_files_lists_the_data_problems`) and `seed --check-only`.
- `game-data/README.md`: describes the extracted data, the report and `--check-only`.

Bonus-list slots: every fixed list a class grants is `ignores_prerequisites: true` unless the class page says the prerequisites must be met; the bloodline feat lists (sorcerer, bloodrager, dragon disciple) check them; feats a class grants outright are one-feat lists that ignore them. The category stages review this.

### Verified

```sh
cargo test --workspace                                # rules 1 + 97, data files 13, data postgres 14, extract 36
cargo clippy --workspace --all-targets                # clean
cargo fmt --all --check                               # clean
cargo build -p rules --target wasm32-unknown-unknown  # OK
SQLX_OFFLINE=true cargo leptos build                  # OK
cargo run -p data --bin seed -- --check-only          # OK, real counts
cargo run -p data --bin seed                          # seeded the real data
cargo leptos serve                                    # home page shows the counts above; stopped afterwards
```

Left for the category stages: the broad hand-fix pass over the report, the schema questions on the map, and follow-up tickets for the prose-only rules (listed in the report under their category).
