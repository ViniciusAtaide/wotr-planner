# Apply the schema and layering review

Type: task
Status: resolved
Blocked by: 09, 11
Map: [WotR Build Planner](../map.md)

## Question

A read-only code review (2026-10-31) of everything since commit `811e9a5` checked:
- database normalization
- derived values living in Rust
- whether every entity is mapped
- controller/service/entity layering

The user approved applying all of it. This ticket is that fix pass. AFK: driven by the agent, test-first (`tdd`).

### User decisions

1. **Archetype level rows:** keep full rows (ticket 05), and add a files test that flags archetype bab/saves/spell numbers differing from the base class at the same level, unless allow-listed.
2. **Combat feats** become a Rust rule, `FeatCategory::is_combat()`: every category except class, racial, skill, teamwork and magic. Drop the `combat_feat_category` table and `Progression.combat_feat_categories`.
3. **Saves are not a prerequisite.** Remove "saves" from the map's checked-prerequisites list; add no save leaf.

### Fixes

**Bug: archetype and spellcasting choices**
- Move `archetype` and `advances` out of `LevelUp` into `Build.class_choices: BTreeMap<ClassKey, ClassChoice { archetype, advances }>`; `LevelUp` keeps only the class.
- Delete `normalize_class_choices`, and rework `Edit::Class` accordingly. Add tests for the two reproduced failures:
  - changing level 1 from Fighter with archetype A to Rogue must not lose A for later Fighter levels;
  - setting an archetype on a non-first Fighter level applies to Fighter.
- Validate `advances`: each entry names a held caster class, and Mystic Theurge advances one arcane and one divine class.
- Regenerate the frozen v1 codec vector, since no link has shipped yet.

**Derived values to Rust**
- Delete `SkillRankCap`. Make `path_choice_rank` (3) and `path_switch_rank` (8) Rust constants next to `BASE_LEVEL_CAP`. Drop the `progression` table; `progression.ron` keeps only the mythic ascension table.
- Derive `PathSwitch.rank_after_switch` as max(kept ranks) + 1, and drop the column and field.
- Reduce `LevelCaps` to a single `level_cap: Option<u8>` (the character level cap). Per-class caps come from the class tables' row counts.
- Class row 0: replace it with a numberless `on_entry` grant (features, feat slots, selections) on `Class`, or fold it into row 1. Choose one and document why.

**Data integrity**
- Add `rules::GameData::check() -> Vec<DataProblem>`, pure, and run it in `data::seed` before inserting and in the tests. It covers:
  - contiguous level rows
  - kept ranks only on a path with a switch
  - archetypes only on base classes
  - `advances_*` only on prestige classes
  - prerequisite leaves without children
  - duplicate keys
  - rank and level ranges
- `load.rs`: make grouped rows error when left over, instead of being silently dropped.
- Schema:
  - `path_switch(path_key PK, …)` as a 1:0..1 table owning `path_switch_kept_rank`
  - class/archetype as a hierarchy, `class_key NOT NULL, archetype_key NULL` with a composite FK to `archetype(key, class_key)`, replacing the class/archetype exclusive arc where it applies
  - Postgres `DOMAIN`s for ability, feat_category and u8 instead of copy-pasted CHECKs
  - a surrogate `id` on `granted_feature` and `granted_selection`
  - enforce that a selection option's key starts with its selection key

**Ticket 12 (Sheet) service API**
- `class_options` (class plus archetype, with locked reasons from class entry), `path_options` (MR 3) and `switch_options` (MR 8), all with `Unmet` reasons.
- `Edit::RemoveLevel` (removes the last level) and `Edit::SetName`.
- Make `options()` cheap: compute `would_break` lazily for one chosen option, or reuse the walk up to the slot. Pick one and justify it.
- Server: load `GameData` once into an `Arc<GameData>` in context, refreshed only by a reseed or a restart. `get_game_data` returns it and no longer queries per call. `data`'s public API returns its own error type and stops re-exporting `PgPool` and leaking `sqlx::Error`.
- A shared `GameData::counts()` replaces the duplicated nine-count summary in `app` and `bin/seed.rs`.

**Coverage.** Extend the round-trip fixtures to cover:
- an `Archetype` prerequisite leaf
- `divine_spell_level` and `advances_divine`
- `on_entry` grants
- an archetype with `class_skills: Some([])`
- an archetype with `skill_points: None`
- a switch path without caps, and `further_ascensions: true`

Add a test that round-trips every `Prerequisite` variant.

**Smells.** Fix only where cheap:
- remove the dead `CharacterLevel`
- rename `Pick::Option` / `Held::Option` to `SelectionOption`
- replace the `(bool, usize)` class index with a named type
- deduplicate the stray-slot block and the row-sort code
- route the prerequisite, feat_slot and ability_modifier kind strings through `text_column!`

**Docs**
- `data/src/columns.rs:3` names the old migration file.
- `README.md` says sqlx-cli is "for new migrations".
- Tickets 09 and 11 and `map.md` still describe `position` columns and the old migrations; add a dated note that this ticket superseded them.
- `GLOSSARY.md`: add Sheet, Slot, Pick, Mythic track, Selection option, Problem, Would-break. Fix Build, which is not "a saved plan" but lives in the URL. Use glossary names in code where the review flagged "level" and "requirement" (*Avoid* terms) and it's cheap.

## Answer

Applied test-first and verified 2026-10-31 (uncommitted). Every fix is done except the `on_entry` coverage item, which
the row 0 decision made moot.

### Fixes

**Bug: archetype and spellcasting choices** (all done)

- `Build.class_choices: BTreeMap<ClassKey, ClassChoice { archetype: Option<ArchetypeKey>, advances: BTreeSet<ClassKey> }>`.
  `LevelUp` is `{ class, ability_increase, skill_ranks }`. `normalize_class_choices` is deleted.
- `Edit::Class { level, class, choice: Option<ClassChoice> }`. `Some` replaces the class's choice and `None` keeps it,
  so changing a level's class never wipes another class's archetype. Choices for classes no level takes any more are
  dropped, so links carry no dead entries. Both reproduced failures are tests (`edits.rs`), plus "a change without a
  choice keeps the new class's archetype" and "unused choices are dropped".
- `advances` is validated on the prestige class's first level. Each entry must be already taken and cast the spell
  type the prestige class advances. There's at most one arcane and one divine, so Mystic Theurge takes one of each
  and Arcane Trickster one arcane. A class whose rows advance nothing takes none. Unknown keys get `UnknownKey`, one
  per key; anything else is one `NotAllowed` at `Choice::Advances { class }`. Missing advances still mean "no
  advancement", not a problem, because an unfinished Build isn't broken.
- `Choice::Archetype` and `Choice::Advances` are keyed by class (`{ class: ClassKey }`), not by level.
- The frozen v1 link (`rules/tests/engine/codec.rs`) and the frozen body layout (`codec.rs`) were regenerated. Both
  now include a class choice.

**Derived values to Rust** (all done)

- `SkillRankCap` is deleted (the cap is the Character level, in code). `PATH_CHOICE_RANK = 3` and
  `PATH_SWITCH_RANK = 8` sit next to `BASE_LEVEL_CAP` in `engine.rs`. `Progression` is just `{ mythic_ascensions }`,
  `progression.ron` holds only that table, and the `progression` table is gone.
- `FeatCategory::is_combat()` covers every category except class, racial, skill, teamwork and magic.
  `Progression.combat_feat_categories`, the `combat_feat_category` table and `Engine::is_combat_feat` are gone.
- `PathSwitch::rank_after_switch()` is max(kept ranks) + 1 (1 if none). The field and the column are gone.
- `MythicPath.level_cap: Option<u8>` replaces `LevelCaps`. A class's level cap is its table's row count, which gives
  20 and 10 in the data.
- **Row 0: folded into row 1** (see Choices).

**Data integrity** (all done)

- `rules::GameData::check() -> Vec<DataProblem { at: String, issue: DataIssue }>` is pure, sorted and `Display`s
  as a sentence. It checks:
  - level rows run 1..=n with none missing (n ≤ 40);
  - archetypes only on base classes;
  - `advances_*` only on prestige classes;
  - prerequisite groups have children, and `AtLeast(n)` has 1 ≤ n ≤ children;
  - duplicate keys across every catalog;
  - a selection option's key starts with its selection's key and `/`;
  - ranks and levels in range: ascension ranks 1–10 with no duplicates, ascension levels 1–40, rank rows 1–10,
    kept ranks below the switch rank, `level_cap` 20–40, spell levels ≤ 9.

  "Kept ranks only on a path with a switch" can't be broken in the types (`kept_ranks` lives inside `PathSwitch`).
  In the schema, `path_switch_kept_rank` references `path_switch`. "Prerequisite leaves without children" can't be
  broken in Rust either, so `check` covers empty groups and the loader rejects a stored condition that has
  children.
- `data::replace_game_data` (and so `seed`) runs `check` first and returns `data::Error::Invalid(problems)` without
  writing anything.
- `load.rs`: `Grouped::finish(table)` errors when rows are left over: `"N feat_slot_option row(s) belong to nothing
  the game data has"`. It covers every grouped table.
- Schema, all in `20261031000000_game_data.sql`:
  - `path_switch (path_key PK, further_ascensions)` owns `path_switch_kept_rank`;
  - `class_skill`, `class_level`, `feature` and `feature_selection` have `class_key` (NOT NULL for the first two)
    and a nullable `archetype_key`, with `FOREIGN KEY (archetype_key, class_key) REFERENCES archetype (key,
    class_key)`. `feature` and `feature_selection` keep `path_key`, with `num_nonnulls(class_key, path_key) = 1`;
  - domains `u8`, `ability` and `feat_category` replace the copy-pasted CHECKs. sqlx 0.9's `query!` accepts them
    with no casts;
  - `granted_feature.id` and `granted_selection.id`;
  - `CHECK (starts_with(key, selection_key || '/'))` on `selection_option`.

**Ticket 12 (Sheet) service API** (all done; see the API section)

- **Server:** `server/src/main.rs` loads `GameData` once into an `Arc<GameData>` after connecting and provides it
  as the only context. `get_game_data` clones it out, with no query per call. A reseed is picked up on the next
  restart: the seed is a separate process, and the warning and the seed output both say so. `app` no longer depends
  on `data` at all.
- **`data` API:** `Database` (`Database::connect(url)`, `From<sqlx::PgPool>` for `#[sqlx::test]`) and `data::Error
  { Files, Invalid(Vec<DataProblem>), Database(DatabaseError) }`. `DatabaseError` is opaque and keeps sqlx's message
  and `source()`. `PgPool` isn't re-exported, no signature names `sqlx::Error`, and `MIGRATOR` is private (the
  tests use `#[sqlx::test]`'s default migrations path).
- **Counts:** `GameData::counts() -> [(&'static str, usize); 9]` serves both the home page and the seed binary.

**Coverage** (done, except one item)

- The fixture gained `classes/cleric.ron` (divine spell levels) and its archetype `herald` (`class_skills: []`, no
  `skill_points`), plus `prestige/mystic-theurge.ron` (`advances_arcane` + `advances_divine`). It also gained
  `mythic/paths/gold-dragon.ron` (a switch, no `level_cap`, `further_ascensions: true`) and `feat:armiger-drill`
  (an `Archetype` prerequisite).
- **Skipped: `on_entry` grants.** Row 0 was folded into row 1, so there's no `on_entry` to cover.
- `every_prerequisite_variant_round_trips` (`data/tests/postgres.rs`) seeds a feat whose prerequisite holds all 15
  leaf kinds plus `Any` and `AtLeast` under an `All`, and reads it back unchanged. An exhaustive `match` in the test
  stops compiling when a variant is added.

**Smells** (all done)

- `CharacterLevel` (the dead newtype) is removed.
- `Pick::Option` is now `Pick::SelectionOption`, and `Held::Option` is `Held::SelectionOption`. Postcard encodes
  variants by index, so links are unaffected.
- `ClassAt::{Base, Prestige}(usize)` replaces `(bool, usize)`.
- Stray-slot reporting is one `Walker::fill_all`, and row grants are sorted by one `sort_grants`.
- `NodeKind`, `FeatSlotKind` and `ModifierKind` go through `text_column!`. The loader matches enums, not strings.

**Docs** (all done)

- `data/src/columns.rs` names the current migration. `README.md` gives sqlx-cli's real uses, describes the one
  migration and the restart after a reseed, and updates the crate table.
- Tickets 09 and 11 open their Answers with a dated "superseded in part" note. `map.md`'s lines for 09 and 11 got
  dated notes, and saves were dropped from the checked prerequisites.
- `GLOSSARY.md`:
  - Build is now "lives in the URL", not "a saved plan".
  - Level-up no longer includes the archetype.
  - New entries: Class choice, Selection option, Slot, Pick, Sheet, Mythic track, Problem and Would-break.
- Code: `Choice::Level` is now `Choice::CharacterLevel`, and "requirement" became "prerequisite" in doc comments and
  test names.
- `game-data/README.md` describes the new file shapes, the rule facts in code, the row rules and the archetype
  allow-list.

### Choices

- **Row 0: folded into row 1, with no `on_entry`.** Nothing in the guide or the fixtures has a row 0. It was a
  speculative special case: `granting_rows`, level 0 in the schema, and a loophole in "contiguous rows". Folding it
  keeps one rule (rows 1..=n, row count = cap) and one shape, and archetypes already repeat full rows. What taking a
  class grants is written in row 1.
- **`options()` strategy: would-break is lazy, for the option the player taps.** `options()` is one walk that stops
  after the slot's level (`Probe::Slot`), and `SlotOption` is `{ pick, status }`. Precomputing would-break cost a
  full Build walk per available option, which means hundreds of walks for a general feat slot in the real data,
  run in wasm on a phone. Reusing the walk up to the slot would still walk the rest of the Build once per option.
  Since would-break only matters for the option being chosen, the picker calls `would_break(&build,
  &Edit::Pick { .. })` on tap and shows "Would break: …" before applying. The cost is two walks per tap. Ticket 06's
  idea of pre-disabling every breaking option in the list becomes "blocked on tap".
- **`ClassChoice.advances` is a `BTreeSet`:** it's a set of at most two classes.
- **Path options carry `status` that is always `Ok` for now:** the data gives paths no prerequisites, and I added
  none I couldn't confirm. The field keeps the three pickers alike for ticket 12.
- **`check` sorts its problems** (`at`, then issue), so output and tests are deterministic.

### API for ticket 12 (new or changed)

| Item | What it is |
| --- | --- |
| `Build.class_choices`, `ClassChoice { archetype, advances }` | per-class archetype and advancement |
| `LevelUp { class, ability_increase, skill_ranks }` | no archetype or advances |
| `Edit::Class { level, class, choice: Option<ClassChoice> }` | `None` keeps the class's choice |
| `Edit::RemoveLevel` | pops the last level and its level picks, and drops unused class choices. Mythic picks stay (ignored while unreached, back with the level) |
| `Edit::SetName(Option<String>)` | |
| `engine.options(&build, &slot) -> Option<Vec<SlotOption { pick, status }>>` | one walk, no would-break |
| `engine.would_break(&build, &edit) -> Vec<Problem>` | call it for the tapped option, a class change or Clear |
| `engine.class_options(&build, level) -> Option<Vec<ClassOption { class, archetype, status }>>` | base classes (each followed by its archetypes), then prestige classes. A class taken before `level` is offered only with its archetype. `status` is the class's and archetype's entry `Unmet` (combined under `All` when both fail) against the state before `level`, or `ClassLevelCap`. `None` unless 1 ≤ level ≤ len + 1 |
| `engine.path_options(&build)`, `engine.switch_options(&build) -> Vec<PathOption { path, status }>` | paths without / with a switch rule, by key |
| `Choice::Archetype { class }`, `Choice::Advances { class }`, `Choice::CharacterLevel { level }` | renamed and re-keyed |
| `Pick::SelectionOption` | was `Pick::Option` |
| `PATH_CHOICE_RANK`, `PATH_SWITCH_RANK`, `FeatCategory::is_combat()`, `PathSwitch::rank_after_switch()`, `MythicPath.level_cap` | rule facts |
| `GameData::check()`, `DataProblem`, `DataIssue`, `GameData::counts()` | |
| `get_game_data()` | returns the server's one `Arc<GameData>` |

Removed: `SlotOption.would_break`, `Engine::is_combat_feat`, `SkillRankCap`, `LevelCaps`, `CharacterLevel`, the
`Progression` fields other than `mythic_ascensions`, `PathSwitch.rank_after_switch` (now a method), row 0.

### Tables

Before (26): `game_data_seed`, `progression`, `mythic_ascension`, `combat_feat_category`, `prerequisite_node`,
`skill`, `race`, `subrace`, `ability_modifier`, `feat`, `class`, `archetype`, `class_skill`, `mythic_ability`,
`mythic_feat`, `mythic_path`, `path_switch_kept_rank`, `feature`, `feature_selection`, `selection_option`,
`class_level`, `mythic_rank`, `granted_feature`, `granted_selection`, `feat_slot`, `feat_slot_option`.

After (25): the same, without `progression` and `combat_feat_category`, plus `path_switch`.

| Table | Before | After |
| --- | --- | --- |
| `progression` | singleton: path choice/switch ranks, skill rank cap | dropped (Rust constants) |
| `combat_feat_category` | category list | dropped (`FeatCategory::is_combat`) |
| `mythic_path` | `switch_rank_after`, `switch_further_ascensions`, three `cap_*` columns, all-or-none CHECKs | `level_cap` (20–40, nullable) |
| `path_switch` | — | `path_key` PK → `mythic_path`, `further_ascensions` |
| `path_switch_kept_rank` | FK → `mythic_path` | FK → `path_switch` |
| `archetype` | `skill_points` smallint CHECK | `u8`; `UNIQUE (key, class_key)` for the composite FKs |
| `class_skill`, `class_level` | exclusive arc `class_key` XOR `archetype_key` | `class_key NOT NULL`, `archetype_key` NULL, FK `(archetype_key, class_key)` → `archetype (key, class_key)` |
| `feature`, `feature_selection` | arc over class/archetype/path | `class_key`/`archetype_key` hierarchy (composite FK) or `path_key` |
| `class_level` | `level` 0–40, spell levels 0–10, numbers smallint CHECK | `level` 1–40, spell levels 0–9, `u8` |
| `granted_feature`, `granted_selection` | no key | `id` identity PK |
| `selection_option` | `key LIKE 'selection:%'` | `starts_with(key, selection_key \|\| '/')` |
| `skill`, `ability_modifier`, `prerequisite_node`, `feat`, `feat_slot`, `class` | copy-pasted enum/range CHECKs | domains `ability`, `feat_category`, `u8` |

### Verified

```sh
cargo test --workspace                                 # rules 1 + 90, data files 10, data postgres 11: all pass
cargo clippy -p rules -p data -p server --all-targets  # clean
cargo fmt --all --check                                # clean
cargo build -p rules --target wasm32-unknown-unknown   # OK
SQLX_OFFLINE=true cargo leptos build                   # OK
cd data && cargo sqlx prepare --check                  # OK (52 queries)
cargo run -p data --bin seed -- data/tests/fixtures/game-data
#   seeded … (hash 86f93c21…): 3 races, 4 skills, 12 feats, 3 classes, 3 archetypes, 2 prestige classes, 2 mythic abilities, 2 mythic feats, 3 mythic paths
cargo leptos serve                                     # the page lists the counts; hydrates with no console messages
```

The dev database was dropped, recreated, migrated and reseeded per `CLAUDE.md`, and `data/.sqlx` was regenerated.
New rules test files: `advances.rs`, `check.rs`, `edits.rs`, `facts.rs` and `pickers.rs`, all declared in
`rules/tests/engine/main.rs` and listed by `cargo test -p rules -- --list`.
