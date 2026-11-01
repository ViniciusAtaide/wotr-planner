# Bonus feats ignore prerequisites

Type: task
Status: resolved
Blocked by:
Map: [WotR Build Planner](../map.md)

## Question

The guide's character-building page says a class's bonus feats typically ignore the feat's prerequisites. For example, a Ranger or Fire Kineticist can take Dazzling Display without Weapon Focus, and a Monk gets Crane Style as a bonus feat. Follow-on feats still check theirs: Crane Wing still needs Dodge. Today the engine checks prerequisites on every slot.

**User decisions (2026-10-31):**

- **D1:** slots that draw from a class's own fixed bonus-feat list ignore the picked feat's prerequisites. Examples are Monk bonus feats, Ranger combat style and Kineticist. Slots open to "any combat feat" (Fighter), category slots and general feat slots still check prerequisites. A feat taken this way still counts as held for later prerequisites. The extractor sets the flag per slot from the class page's wording ([Extract the guide into data files](10-extract-guide.md)), and anything unclear goes to its report of unparsed lines.
- **D2:** gear is out of scope. Prerequisites check the natural ability scores only.

**Build** (AFK, test-first `tdd`):

- `rules::FeatSlot` carries whether the slot ignores prerequisites.
  - Pick the simplest shape and justify it, for example a flag on the `OneOf` slot, or a separate variant.
  - Only list slots can ignore prerequisites.
- Engine:
  - `options()` and validation skip the picked feat's prerequisite check in such a slot. Already-held and the slot's list still apply.
  - The feat counts as held afterwards.
  - Would-break never reports a bonus-slot pick as broken by a prerequisite change.
- Schema: edit the one migration, add the column with no derived values, and reset the dev database per `CLAUDE.md`. Update the seed, the loader, the round-trip test, `GameData::check()`, and the fixture: a monk-style class with a fixed bonus list including a feat whose prerequisites aren't met.
- Tests:
  - a Crane Style-style bonus pick without its prerequisite is allowed;
  - the follow-on feat in a general slot still requires its prerequisites, and sees the bonus-picked feat as held;
  - a Fighter "any combat feat" slot still checks prerequisites.

## Answer

Built test-first and verified 2026-10-31 (uncommitted).

### Shape

**A flag on the fixed-list slot:** `FeatSlot::OneOf { feats: BTreeSet<FeatKey>, ignores_prerequisites: bool }`, with
`FeatSlot::ignores_prerequisites()` as the one question the engine asks. Why this over a separate variant:

- A Monk list and an Armiger list are the same kind of slot (one feat from a fixed list). The only difference is
  whether prerequisites apply, which is one yes/no property, so it's a field, not a second variant that
  repeats `OneOf`'s list and its handling in `slot_accepts`, the seed and the loader.
- Only list slots can ignore prerequisites, by construction: no other variant has the field.
- It maps one-to-one onto the schema: one `BOOLEAN` column, no new `kind` value.
- In RON the field defaults to `false` (`#[serde(default)]`), so a list that checks prerequisites is written
  `OneOf(feats: [...])` and a bonus list adds `ignores_prerequisites: true`. Leaving it out is the safe choice.

The glossary calls such a slot a **Bonus-list slot**.

### Engine

`check_pick` (shared by `options()`, `evaluate`/`validate` and therefore `would_break`) checks, in order: known key,
accepted by the slot (the list), not already held, then prerequisites, unless the slot ignores them. The pick is
held as before, so follow-on feats see it. Would-break needed no code of its own: a bonus-list pick can no longer
get an `Unmet` problem, so no edit can newly break it.

`GameData::check` gains `DataIssue::BonusListOutsideAClass`: a race or a mythic rank that grants a slot ignoring
prerequisites is refused, because the list is the class's own (D1). Class and archetype level rows (prestige
classes included) may grant one.

### Schema

`feat_slot` gains `ignores_prerequisites BOOLEAN NOT NULL` and `CHECK (kind = 'one_of' OR NOT
ignores_prerequisites)`, edited into `20261031000000_game_data.sql`. It's a stored fact from the class page's wording,
not derived. Still 25 tables. The seed writes `slot.ignores_prerequisites()`. The loader reads it and treats `TRUE` on
any other kind as a corrupt row. The dev database was dropped, migrated, reseeded from the fixture, and `cargo sqlx
prepare` rerun (two query files replaced).

The extractor doesn't exist yet. [Extract the guide into data files](10-extract-guide.md) already says to set the
flag from the class page's wording and report unclear cases.

### Tests

- `rules/tests/engine/bonus_feats.rs` (new, `mod bonus_feats;`). The test world gains a monk (bonus-list slot at
  1, 2, 6, …: Crane Style or Dodge) and the feats Crane Style (needs Dodge) and Crane Wing (needs Crane Style and
  Dodge):
  - `a_bonus_list_pick_ignores_the_feats_prerequisites`: a Dex 12 monk can take Crane Style and Dodge in the bonus
    slot, not Crane Style in a general slot, and validates clean.
  - `a_bonus_list_slot_still_takes_only_its_list_and_no_feat_already_held`
  - `a_follow_on_feat_checks_its_prerequisites_and_sees_the_bonus_pick_as_held`: Crane Wing in a general slot
    reports only Dodge missing, then qualifies once Dodge comes from the bonus slot.
  - `a_fighter_combat_feat_slot_still_checks_prerequisites`
  - `a_fixed_list_that_is_not_a_bonus_list_still_checks_prerequisites` (the Armiger's list)
  - `would_break_never_reports_a_bonus_list_pick`: removing Dodge breaks Crane Wing only, not Crane Style in the
    bonus slot.
  - The tests that passed on their first run were checked against two temporary mutants (the flag ignored; every
    feat slot ignoring prerequisites): each test fails against at least one of them.
- `check::only_class_levels_grant_bonus_list_slots`.
- `data/tests/files.rs::reads_whether_a_fixed_list_slot_ignores_prerequisites`. The fixture gains `classes/monk.ron`
  (bonus-list slots at 1 and 2) and the feats Dodge, Crane Style and Crane Wing. The round trip
  `files_to_postgres_to_game_data_equals_files_to_game_data` went red on the dropped flag before the column existed.
  The schema test rejects `ignores_prerequisites = TRUE` on a `combat_feat` slot.
- World-listing literals updated for the monk: the class picker, catalog order and counts, the fighter's Combat-feat
  list (the crane feats are Unarmed Combat). The unknown-key test now uses `class:removed-class` instead of
  `class:monk`.

Also updated: `GLOSSARY.md` (Feat slot wording, new **Bonus-list slot**), `game-data/README.md` (feat slot syntax).

### Verified

```sh
cargo test --workspace                                 # rules 1 + 97, data files 11, data postgres 14: all pass
cargo clippy -p rules -p data -p server --all-targets  # clean
cargo fmt --all --check                                # clean
cargo build -p rules --target wasm32-unknown-unknown   # OK
SQLX_OFFLINE=true cargo leptos build                   # OK
cd data && cargo sqlx prepare --check                  # OK
cargo run -p data --bin seed -- data/tests/fixtures/game-data  # seeded: 15 feats, 4 classes, ...
```
