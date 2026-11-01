# Drop the type prefix from keys

Type: task
Status: resolved
Blocked by: 17, 18, 19, 20
Map: [WotR Build Planner](../map.md)

## Question

**User decision (2026-11-01):** the `feat:`, `skill:`, `class:`, … prefix on every text key is a derived value, because the table, or the Rust key type, already says what kind of thing it is. Drop it everywhere.

For example, `feat:power-attack` becomes `power-attack`, and `archetype:rogue/eldritch-scoundrel` becomes `rogue/eldritch-scoundrel`.

**Out of scope:** owner prefixes stay (`rogue/eldritch-scoundrel`, `rogue/rogue-talents/fast-stealth`). They keep names unique where the guide reuses them, for example Paladin's and Inquisitor's "Divine Hunter". Revisit them with [Answer the schema questions from the real data](21-schema-questions-real-data.md).

**Build** (AFK, test-first `tdd`; run it only when no other agent is working, per `CLAUDE.md`):
- **Schema:** edit the one migration, replacing every `CHECK (key LIKE '<type>:%')` with a slug-format check if useful. Reset and reseed the dev database per `CLAUDE.md`.
- **Rust:**
  - the `rules` key newtypes and any parsing that relies on prefixes
  - `GameData::check()`, including the selection-option-extends-selection rule, now without the type prefix
  - the URL codec. No link has shipped, so regenerate the frozen v1 vector; don't add a v2.
- **Data:** rewrite every key and reference in `game-data/**/*.ron`, the test fixture, `game-data/README.md` and `GLOSSARY.md`. Use a one-off script and diff the counts; the files are hand-edited now, so don't re-run the extractor over them.
- **The extractor:** stop emitting the prefix, so a future run matches.
- **Done when:**
  - `cargo test --workspace`, clippy, `cargo fmt --check`, the wasm build of `rules` and the offline build all pass
  - `--check-only` passes and the real data seeds
  - `grep -rE '"(feat|skill|class|archetype|race|subrace|feature|selection|path|mythic-ability|mythic-feat):' game-data data/tests/fixtures rules` finds nothing

## Progress log

- 2026-11-01: (pre-step, not this ticket) merged tickets 17–20 into `game-data/EXTRACTION_REPORT.md`: 82 → 79 items (Feats 2, Base classes 56, Prestige 12, Mythic 9, Races and skills 0).
- 2026-11-01: rewrite script (scratchpad `drop_prefixes.py`) ran: game-data 12968, fixture 121, rules/tests 590, data/tests 56 replacements, each matching an independent grep count; a second script confirmed all 85 files differ from HEAD only by the dropped prefixes. Real-data collision scan: only `shaman/spirit` (a feature and a selection).
- 2026-11-01: rules done test-first: `DataProblem.at` is now a typed `DataAt` (kind + key), uniqueness is per kind, the frozen v1 link is regenerated, and a codec test shows a feat, mythic feat and option with one key stay apart. Extractor: emits bare keys, `references.rs` checks references per kind, the prerequisite owner is a `DataAt`, report items about data are filed by kind (`Report::add_about`). A fresh extraction into the scratchpad equals stage A (commit 4b7c81b) minus prefixes, file for file, report included. Next: schema, docs, DB reset.
- 2026-11-01: schema (`text_key` domain), docs, dev DB reset and reseed, `cargo sqlx prepare`; every done-when check passes. Done.

## Answer

Done 2026-11-01, uncommitted. Every key and reference is now bare: `feat:power-attack` → `power-attack`, `archetype:rogue/eldritch-scoundrel` → `rogue/eldritch-scoundrel`. Owner prefixes stay.

### Collisions

- **Real data:** one. `shaman/spirit` is both a feature (the Spirit class feature) and a feature selection (the spirit choice from ticket 18). They live in different tables and Rust types, so nothing changes. Test fixture: none. Rules test world: none.
- **Places that mixed kinds, now kept apart:**
  - `GameData::check` held every key in one `HashSet<String>`, so a feat and a feature with one key would have been a `DuplicateKey`. `DataProblem.at` is now `DataAt`, a typed enum (kind + key, or `Progression`), and uniqueness is per kind. Base and prestige classes stay one kind (one `class` table). Problems read `feat cleave: the key is used twice`.
  - Extractor `references.rs` checked references against one set of all keys, so a feature of the same name would have satisfied a feat reference. It's now a set of `DataAt`.
  - Extractor prerequisite owners were strings (`Pending.owner`, `resolve_prerequisites`), so a feat and a mythic ability with one key would have shared prerequisites. Owners are `DataAt` now.
  - The extractor report filed data-check items by the key's prefix. Items about data now go through `Report::add_about(&DataAt)` and are filed by kind.
- **Already separate:** the engine (`CharacterState` keeps one typed set per kind), the URL codec (a `Pick` variant per kind), the loader's `Owner` enum, and every table.
- The selection-option rule (an option's key starts with its selection's key plus `/`) needed no change. A new test also shows `rogue/rogue-talents-x` isn't inside `rogue/rogue-talents`.
- The UI has no code that reads or strips prefixes.
- `Issue::UnknownKey(String)` stays a string: its `Choice` already says which kind the key is.
- **Schema:** the 11 `CHECK (key LIKE '<type>:%')` are gone. All 12 key columns, `selection_option.key` included, use a new domain, `text_key`: lowercase slugs, `/`-separated (`^[a-z0-9]+(-[a-z0-9]+)*(/[a-z0-9]+(-[a-z0-9]+)*)*$`). All 3093 distinct keys matched it before the change. A postgres test checks that `skill:stealth`, `Stealth`, `stealth/`, `two  words` and `''` are rejected and `stealth-2/x` is accepted.

### Codec

- There's no v2. The frozen v1 vector (`V1_LINK` in `rules/tests/engine/codec.rs`) is regenerated from the bare-key `v1_build()`, and its comment records why.
- The v1 body-layout unit test in `rules/src/codec.rs` uses `x`/`y` now.
- The proptest key strategy no longer adds prefixes.
- New test `picks_of_different_kinds_with_the_same_key_stay_apart_in_a_link`: a feat, a mythic feat and a selection option all keyed `power-attack` round-trip, each as its own `Pick`.

### Rewrite script

The script is `drop_prefixes.py` in the session scratchpad. It's a regex over `"<type>:` (plus `'<type>:` in SQL inside Rust tests), and it asserted each group's count against a grep made before the run:

| Group | Replacements | Files |
| --- | --- | --- |
| `game-data/**/*.ron` | 12968 | 54 |
| `data/tests/fixtures/game-data/**/*.ron` | 121 | 14 |
| `rules/tests/**/*.rs` | 590 | 15 |
| `data/tests/**/*.rs` | 56 | 2 |

A second script checked all 85 files against `HEAD`: dropping the prefixes from the old text gives exactly the new text. The extractor was never run over `game-data/`.

Edited by hand:
- `rules/src` docs and the codec unit test
- `feat_row` in the test world, which stripped `feat:`
- the codec proptest strategy
- `EXTRACTION_REPORT.md` (3 mentions)
- `game-data/README.md`, `GLOSSARY.md` and the migration comments

### Extractor

- It builds bare keys. Output file names no longer strip `class:`/`path:`, and the prose rules' fixed keys are bare.
- New tests: `references::a_reference_must_exist_as_its_own_kind` and `report::an_item_about_a_data_entity_is_filed_by_its_kind`.
- I ran a fresh extraction from `guide-cache/` into the scratchpad. It matches stage A (commit 4b7c81b) with the prefixes dropped: all 55 `.ron` files and `EXTRACTION_REPORT.md` are identical (82 items, 0 check problems).

### Verification

- `cargo test --workspace`: pass (rules 100, extract 38, data files 13 + postgres 14, rules unit 1).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass. 12 query files changed; insert parameters are now the `text_key` domain.
- Dev DB reset per `CLAUDE.md`, only `wotr-planner-postgres` on 5434: drop/create `wotr_planner`, `sqlx migrate run --source data/migrations`, `cargo run -p data --bin seed`, `cargo sqlx prepare`.
- `--check-only` and the real seed both give 12 races, 11 skills, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats and 10 mythic paths.
- The acceptance grep finds nothing, also over `extract` and `data/src`.
- A read-only sample shows:
  - `feat`: `ability-focus-bombs`, `accomplished-sneak-attacker`, …
  - `ability_modifier` owners: `aasimar`, `aasimar/angelkin`, …
  - `selection_option`: `aeon/enforcing-gaze/…`
  - No key in `feat`, `feature`, `selection_option` or `class` contains `:`, and the domain rules it out everywhere.
