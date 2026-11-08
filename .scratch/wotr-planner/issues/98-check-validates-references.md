# GameData::check validates references

Type: task
Status: resolved
Blocked by: none (runs alongside [97](97-slot-options-presented.md), in its own worktree)
Map: [WotR Build Planner](../map.md)

## Question

The architecture review of 2026-11-07 (candidate 4) found referential integrity split between `GameData::check` and SQLite's foreign keys:

- `libs/rules/src/check.rs` says so at its top and in five places (670, 721, 942, 1089, 1142: "an unknown X is the foreign keys' business"), and `check_game_data_files` (`libs/seeds/src/lib.rs` ~101) says only `seed` catches dangling keys.
- So in-memory data is never checked for them: the hand-built small world in `libs/rules/tests/engine/support/` passes `check()` in only a few tests (`removable.rs:467`, `parameters.rs:404`, `alignment.rs:97`), and a misspelled key there silently offers nothing.
- The engine carries tolerances for missing keys (a missing feat list "accepts none", a missing shared option "offers nothing"): `engine/catalog.rs` ~291, `engine/slots.rs` ~159 and ~175, `engine/pickers.rs` ~61, `engine/switch.rs` ~99 (line numbers before tickets 95–96; search for them).

The user chose to build it (2026-11-08) without a design round, keeping their ticket 75 choice: the database still checks foreign keys. Settle small choices yourself, record them, and bring a real open question back.

## Build

- **`check` validates every reference:** each key a data item names (feats, feat lists, classes, archetypes, selections and options, shared options, features, weapons, skills, deities, domains, mythic paths, backgrounds, races, …) resolves to an item of the right kind, reported as a problem naming the item and the missing key. The SQLite foreign keys stay as they are, as a backstop.
- **The comments go:** the "foreign keys' business" notes in `check.rs` and `check_game_data_files`'s doc say what's true now.
- **The small world is guarded:** one test asserts `world().check() == []` (and any other hand-built fixture the tests share), and missing-key cases become in-memory `check` tests in `libs/rules/tests/engine/check.rs`.
- **The engine's tolerances:** where a missing key can no longer reach the engine (data that passed `check`), the tolerance either goes or becomes an `expect` with the reason; record each one and which you chose. Keep any the stale-link path still needs (a link naming a key the current data lacks is decoded elsewhere; don't change that behaviour).
- **The real data passes:** `scripts/check.sh`'s seed-check runs `check` on `libs/game-data/`; if the new checks find real dangling keys, report them rather than editing the data files, unless the fix is obvious and the guide in `guide-cache/` confirms it.
- Test-first in `rules` (`tdd`).

**Done when:**
- The rules tests above pass, the small world passes `check`, and the real data passes seed-check.
- `scripts/proof.sh rules` against the parent commit shows no behaviour difference.
- `scripts/check.sh` passes, and `--full` once at the end.

## Choices settled

1. **The problem.** `DataIssue::UnknownKey(DataAt)`: the problem's `at` is the entity that names the key, and the `DataAt` inside is the kind and the missing key ("class fighter: names feat dodgee, which the data doesn't have"). `DataAt` gains `Weapon`, the one kind a key can name that it lacked; weapons are keyed in the walk with the rest, so a weapon key used twice is now a `DuplicateKey` too. A path's rank rows and its switch's kept ranks name ranks of the progression (the migration's `mythic_rank.rank` and `path_switch` foreign keys), so `DataIssue::UnknownRank(u8)` reports a rank in range the progression lacks; a rank out of range stays `RankOutOfRange` alone.
2. **How it's checked.** The existing walk notes every key an entity names beside its other rules (`Checker::names`), and once every key has been seen, each named key is looked up in the walk's own set of keys by kind. One problem per entity and missing key, however often the entity names it.
3. **What counts as a reference:** every key the migration has a foreign key for. Feats (feat lists, level, archetype and option rows, feats for one value, an option's feats, chosen feat and rows' chosen feats, backgrounds, prerequisites), mythic feats and abilities (prerequisites), classes (counts as, a selection's option levels, prerequisites), archetypes and races (prerequisites), features (rows, mythic abilities, an option's arcana, counts toward, a switch's dropped and granted features, prerequisites), selections (rows, a slot's `OptionList`, a feature's option row, a feat's extra pick, an option's pool and replaced row, a selection's pool, prerequisites' `NoOptionFrom` and `PickedThrough`), options (shared options, level also from, prerequisites), feat lists (`OneOf` slots, option rows), skills (class skills of classes, archetypes, backgrounds and option rows, values, prerequisites), weapons (values, favoured weapons), deities (prerequisites), paths (a switch's from). A domain (a deity's, an option's `domain`, `DeityOffers`) stays `NotADomain`, which a missing key already fails, so it isn't reported twice.
4. **The small world** already passed: `the_test_world_has_no_problems` is its guard. `option_class_skills.rs`'s shared fixture now asserts it passes `check` too, as `alignment.rs`'s did.
5. **The seed checks first**, so a file with a dangling key now fails `check` (`Error::Invalid`) before any database is built. The foreign keys stay as they were. Their report (`Error::MissingReferences`) is still tested, past the check: `replace_game_data` is the check plus a private `replace_unchecked`, and a unit test in `seed.rs` calls that. The integration tests that seeded a dangling key now expect the check's problems.
6. **The engine's tolerances** (each now unreachable with data that passed `check`):
   - `Catalog::has_option` (catalog.rs, "a shared key the data lacks offers nothing"): **gone**, with its two filters: a selection's shared options in the slot picker (pickers.rs) and a pool's members (`Catalog::new`). A pool's selection lookup there is an **`expect`**.
   - The slot picker's selection lookup (pickers.rs, `.unwrap_or_default()`): **`expect`** (the walk's slots name the data's selections).
   - `slot_accepts` (slots.rs, both `Catalog`'s and the public `Engine::slot_accepts`, "a list the data lacks accepts none"): **`expect`**, documented under `# Panics`. `slot_lists`' lookups of a `OneOf` list, an `OptionList`'s selection and the option rows' lists are the same tolerance and are **`expect`s** too (`slot_list`, `slot_selection`).
   - `Engine::rank_level` (switch.rs, "0 for a rank the data lacks"): **kept**. Its ranks aren't the data's: an edit's slot names whatever rank its Build does, and the engine's own ranks are constants, which `check` doesn't vouch for. Its doc says so.
   - The stale-link path (the codec, and checking a Build's picks against the data) is untouched: a Build may name any key.
7. **Test fallout:** `feat_lists.rs`'s `a_slot_naming_a_list_the_data_lacks_takes_nothing` became `…_is_a_data_mistake` (`should_panic` on the `expect`); `indexed_lookups.rs`'s pool test no longer adds a dangling shared key (`a_pool_offers_the_options_of_its_selection`). Three `check.rs` tests that rename a key compare without the unknown keys the rename leaves behind (`issues_but_unknown_keys`), and `ranks_and_levels_stay_in_range` adds rank 11 rather than renumbering rank 10.
8. **Running the proof:** no scratch clone was needed. Neither side's public interface changed in a way the harness uses, so `scripts/proof.sh rules` compiled the working tree's harness against the parent's crates directly.

## Progress log

- 2026-11-08: started at HEAD aede2ac, clean tree, in its own worktree (ticket 97 ran alongside). Read `CLAUDE.md`, the map's Notes, `GLOSSARY.md`, this ticket and 96.
- 2026-11-08: test-first in `libs/rules/tests/engine/check.rs`. `every_feat_named_is_one_the_data_has` was red at its assertion against the stubbed `UnknownKey` (left `[]`, five problems expected), then green once the walk noted every key. The tests for the other kinds (features, selections and options; classes, races, deities, skills, weapons and mythic picks; feat lists and slot selections; Mythic ranks) and the sentence test came next. Each was confirmed red by switching off `unknown_keys` (5 failed) and green with it back. Four existing check tests changed (choice 7). The small world passed with no change.
- 2026-11-08: engine tolerances (choice 6); two tests that pinned them were replaced. Seeds: three tests that seeded a dangling key now expect the check's problems, and the foreign-key report is tested past the check in `seed.rs` (choice 5). `cargo test -p rules --test engine`: 464 passed; seeds: all pass. The real data passes seed-check, with no dangling keys.
- 2026-11-08: rules proof, parent aede2ac against the working tree, 2,162 Builds: no difference (`A: 2162 Builds, B: 2162; only in A: 0, only in B: 0, differing: 0`; held choices checked identical on both sides).
- 2026-11-08: `scripts/check.sh`: ALL-OK; `--full`: ALL-OK. No query changed, so no `--schema`.

## Answer

Done 2026-11-08. `GameData::check` validates every reference. Each key an entity names must be an entity of its kind (`DataIssue::UnknownKey`, naming the entity, the kind and the key), and each Mythic rank a path names must be in the progression (`DataIssue::UnknownRank`). The SQLite foreign keys stay as a backstop behind the check, and their report is still tested. The "foreign keys' business" notes are gone. The engine no longer tolerates missing data keys, except `rank_level`, whose ranks aren't the data's. The real data had no dangling keys. The proof shows no behaviour difference.
