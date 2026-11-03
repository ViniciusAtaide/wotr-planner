# Fold the repeats into parameters and prune the dead report filters

Type: task
Status: resolved
Blocked by: 71
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 71](71-ticket-30-families-where-owned.md)'s recommendations (taken, user, 2026-11-02):

1. Ticket 32's repeats and mythic values are still a pass (`follow_ups::apply`), because they rewrite what `parameters::apply` builds. Fold them into `parameters`' own tables, so they're built with the parameters, and update `libs/extract/tests/extract/parameters.rs` to cover the same cases.
2. Some `settled` report-filter entries now match nothing, because those items are no longer reported. Remove each entry that matches no report item, and keep a test that fails if an entry matches nothing (so dead entries can't pile up again).

## Build

AFK; the only agent in the repo. A refactor of `libs/extract` only: no change to `rules`, the schema, `libs/game-data/` or the link format. The user is asleep and said to take every recommendation: decide by recommendation, record it, and go on.

**Done when:**
- `libs/game-data/` is byte-identical (`git diff HEAD -- libs/game-data` empty).
- A fresh extraction into the scratchpad, report included, is byte-identical (`diff -r`) to one made with HEAD's extractor before the change.
- The `cargo test --workspace -- --list` diff is only the new test (and any moved name, explained).
- The lighter per-ticket checks pass (fmt, clippy native and wasm, `cargo test --workspace`, the `rules` wasm build, seed `--check-only`).
- The Answer ends with the updated list of passes still in `build.rs`, each with its reason.

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- **2026-11-03, baseline** (before any edit, HEAD `d526da5`): a fresh extraction with HEAD's extractor into the scratchpad (`t72/before`, identical to `t71/after`), `cargo test --workspace -- --list` saved (738 tests). A throwaway dump of the report items before the filter (removed again) showed which `settled` entries match an item.
- **Step 1, the repeats and mythic values** (`t72/check.sh`: identical): `parameters`' feat shapes say a feat taken for a value repeats once per value; the Trickster's chain is in them; a new `MYTHIC_FEAT_SHAPES` table holds the five mythic feats' values; `PREREQUISITES` gained the six same-value rules. Destiny Beyond Birth is set where the mythic feats are built (`Picks::removes_racial_penalty`). `follow_ups::apply` is gone. `tests/extract/parameters.rs` covers the same cases.
- **Step 2, the `settled` entries** (identical): each module lists its entries (`settles()`, `report::Settled`), `Report::settle` drops what they match and returns the entries matching nothing, and the extraction asserts there are none. Seven dead entries removed.
- **Checks:** fresh extraction, report included, byte-identical to the baseline (`diff -r t72/before t72/after`); `git diff HEAD -- libs/game-data` empty; `cargo fmt --all --check`, clippy native and wasm, `cargo test --workspace` (739 pass), the `rules` wasm build and seed `--check-only` pass. `apps/backend/src/main.rs` untouched. No new files.

## Answer

**Ticket 32's repeats and mythic values are built with the parameters; `follow_ups::apply` is gone.**
- **Repeats (27-1):** `FEAT_SHAPES` rows are now `Shape::Value(kind)` (taken for a value, once per value) or `Shape::Repeats(repeat)` (no value). A parameterised feat that doesn't repeat per value can't be written. Improved Critical, Exotic Weapon Proficiency and the rest of the "not said to repeat" rows are per value in the table itself.
- **Mythic values (27-5):**
  - The Trickster's three-feat critical chain is in `FEAT_SHAPES`, taking a weapon.
  - Improved Critical, Spell Focus and Skill Focus (Mythic), Expanded Arsenal and School Mastery are in the new `MYTHIC_FEAT_SHAPES`, with `parameters::mythic_feat_shape` beside `feat_shape`.
  - The same-value prerequisites are six rows of `PREREQUISITES`: the three mythic feats on their base feat, and each link of the chain on the one before. They're rewritten by the same pass that rewrites the resolved prerequisites.
- **Destiny Beyond Birth (26-4)** isn't a parameter. It's set where the mythic feats are built (`Picks::removes_racial_penalty`, provided by `follow_ups::provide`). `follow_ups::check` asserts the built feat has it.
- **The guide sentences** these rows rest on are still asserted. They're in `follow_ups::provide` (`values`, `destiny_beyond_birth`), where ticket 32's other rules assert theirs (decision 1).
- **Tests:** `tests/extract/parameters.rs` pins the new shapes and rewrites inside its existing tests:
  - Improved Critical and the other parameterised feats per value;
  - the chain's shape and its same-value line;
  - the five mythic shapes, with Destiny Beyond Birth taking none;
  - the three mythic same-value lines.

**The dead `settled` entries are gone, and dead entries now fail the extraction.**
- **How the filter works now:**
  - Each module lists its entries as data (`settles() -> Vec<report::Settled>`: an item `at` a place, with any text, this text, or text containing this). Its `settled(at, text)` predicate is kept, read off that list.
  - `build.rs` concatenates the four lists, and `Report::settle` drops the items they match and returns the entries matching none.
  - The extraction asserts that list is empty, so an entry that matches nothing panics the run, like the other "every rule finds what it names" assertions.
- **Removed (seven):**
  - `option_levels`: `feats#disruptive` (Disruptive's line is stated in place).
  - `report_gaps`: Extra Slayer Talent's "Slayer Talents or Talented Slayer" (stated in place).
  - `report_gaps`: "Bonus Feat" at `sorcerer#crossblooded`, `#empyreal-sorcerer`, `#nine-tailed-heir`, `#sage-sorcerer` and `#sylvan-sorcerer` (ticket 71 reads those cells).
- **Live and kept:**
  - The 39 option tables (9 sorcerer, 10 bloodrager, 9 schools, 5 orders, 6 ways);
  - `character-creation#bloodlines` and Dragon Disciple's two lines;
  - Armor Focus's line;
  - the totems, the Geomancer's "Bonus Feat" and the cleric's deity;
  - the kineticist's bonus-feat wild talent.

**Tests** (739, one more than 738):
- New: `report::settling_drops_the_items_an_entry_matches_and_returns_the_entries_matching_none`.
- Renamed: `report_gaps::the_totems_the_sorcerers_level_1_feat_talented_slayer_and_the_deity_are_settled` is now `the_totems_the_geomancers_level_1_feat_and_the_deity_are_settled`. Talented Slayer and five of the six sorcerer items are no longer settled, and the test now asserts that they aren't.
- The option-levels test asserts Disruptive isn't settled.

**Decisions made by recommendation (the user is asleep):**
1. **The guide-sentence assertions for the folded values stay in `follow_ups::provide`.** `parameters`' tables quote the guide in comments and take no guide. Ticket 32's rules all assert their sentences where the providers are read, so these do too. The two assertions that the values were unset beforehand, and that Expanded Arsenal and School Mastery had no prerequisite, went: the table now sets those values itself, and the byte-identical extraction shows nothing changed.
2. **Destiny Beyond Birth goes through a `Picks` hook**, not a parameters table. It's a fact of the mythic feat, not a value, and the hook builds it with its owner.
3. **The dead-entry guard is an assertion in the extraction plus a unit test of `Report::settle`.** Only the extraction has the real report items: no test reads `guide-cache/`, which is git-ignored. Every ticket re-extracts, so a dead entry fails the next run. The unit test pins that an entry matching nothing is returned.
4. **Each module keeps its `settled(at, text)` predicate, read off its entries**, so the existing settled tests stay as they were apart from the pruned items.

**Passes still in `build.rs`** (the end state), each with its reason:
1. `prose::apply`: joins what several owners' pages define (lists folded into another class's selection, slots and gates the tables can't say, the Trickster's tiers).
2. `place_features`: a feature is defined where a table first names it, and goes to its owner once every table is read.
3. `prose::path_rules`: the paths' prose facts (Legend's switch features, path feature bonuses, the Gold Dragon's list), on placed features. It could go through `FeatureFact` in a later ticket.
4. `resolve_prerequisites`: a line names what any page defines.
5. `references::drop_dangling`: drops and reports a cell or line naming what no page defines.
6. `references::drop_redundant_alternatives`: needs every option's feats.
7. `feat_lists::grant_lone_feats`: a list of one feat is known once dangling feats are gone.
8. `shared_options::apply`: works on resolved prerequisites and full rows.
9. `parameters::apply`: rewrites resolved prerequisites by value. It now also holds ticket 32's repeats and its mythic and Trickster values.
10. `progressions::sparse_archetype_rows`: needs complete grants.
11. `feat_lists::name_lists`: needs sparse rows.
12. `parameters::add_lists`: added after the lists are named.
13. `backgrounds`: not a patch. It builds its own owner from the user's list and names the finished data's skills and parameterised feats' values.
14. `extra_picks`: needs every selection.
15. `borrowed_selections::apply`: widens resolved prerequisites (advanced talents, hexes, Extra feats) and the talents' option levels. [Its option levels could now be stated where the talents are built; the prerequisite widening needs resolution.]
16. `borrowed_extra_picks`: needs the rows in force and ticket 49's extra picks.
17. At the end: `picks::check` (read-only), the `settled` report filter (each entry must settle an item), and `data.check()`.

## Decisions on the questions

**User, 2026-11-02:** take every recommendation without asking (overnight run). The four decisions above stand; the "could later" notes in the pass list (path rules as `FeatureFact`, the borrowed selections' option levels) are not recommendations and open no ticket.

**Lead, 2026-11-03:** verified independently; a fresh extraction is byte-identical to HEAD-before-ticket-68's. End-of-queue `cargo leptos build` and `cargo leptos build --release` pass.
