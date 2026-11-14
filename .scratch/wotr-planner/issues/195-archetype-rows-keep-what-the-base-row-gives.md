# Nothing checks that an archetype row keeps what the base row gives

Type: design
Status: resolved
Blocked by: none
Model: the user decides; then Sonnet to build, Opus to review the lists.
Map: [WotR Build Planner](../map.md)

## Question

Found by the code review of ticket [188](188-base-classes-stepped-features.md) (2026-11-13). An archetype row replaces the base row's features, feats, feat slots and selections in full (`merged_row`, `libs/rules/src/engine.rs`), so a row added for one reason can drop something else the base row gives at that level. Tickets 170, 177, 182, 188 and 193 each added such rows: 188 alone about 40.

Nothing permanent catches a dropped carry-over:

- `assert_gains` (`libs/seeds/tests/game_data/scaling_features.rs`) pins the stepped feature on every archetype, and only that. A handful of 188's new rows have an assertion for what they carry over (the Thug's talents and Sneak Attack, Supreme Charge at 20, Channel Positive Energy at 4, the Feral Champion's 11 and 15); `slot_counts` compares how many slots a level has, not their kinds.
- The snapshot corpus has no Build with a class archetype, which is why those tickets' notes say "no snapshot changed".
- 182's and 188's old-against-new dumps of all 222 classes and archetypes at level 20 were one-off checks, removed before the commit. The proof covers archetypes, once per queue.

Nothing is wrong today: 188's Opus review compared every row of the seven class files it touched, at every level and field, old against new. The gap is that a later edit that drops, say, the Flesheater's rage power at 12 or the Stonelord's Divine Bond at 5 passes `scripts/check.sh`.

Options:

- **A.** One generic real-data test: for every archetype row, what the base row gives at that level and the row lacks must be on a per-archetype list of what the archetype loses there. The lists are written once from the data as it is, then checked against the guide's archetype tables. Catches a dropped carry-over in every class, at the cost of maintaining the lists.
- **B.** One Build per archetype in the snapshot corpus (about 200 more snapshot files). Catches the same and more, with no lists, but every stepping ticket then moves many snapshot files.
- **C.** Keep the old-against-new dump as a permanent tool (a `tools/` binary or an ignored test that writes every class and archetype at 20 to a file), run by the ticket's agent on both sides. No new check in `scripts/check.sh`; it depends on the agent running it.

**Recommendation: A.** It fails in `scripts/check.sh` at the edit that causes it and names the row; B's snapshot churn would bury the signal the snapshot check exists for.

## Done when

The user has chosen. For A: the test and its lists, each list checked against the guide's table for that archetype, and `scripts/check.sh` passing.

## Answer

User, 2026-11-14 ("do every recommendation", to the retrospective's list, which held this ticket's): option A, the generic real-data test with a per-archetype list of what each archetype loses.

Built 2026-11-14, corrected after the Opus review.

1. **What was built.** `libs/seeds/tests/game_data/archetype_losses.rs` and `archetype_losses.ron` (the lists, read with `include_str!` and serde, as `gold.rs` reads its files). For every archetype row, the base row's shortfall in features, feat slots and selections (multisets), granted feats and feats-for-a-value (sets), and spell levels set below the base row's must match the archetype's list exactly. A list entry is `(item, at, count, replaced_by, why)`: no `at` means lost at every level where the base row gives it (for `SpellLevels`, where it sets spell levels), so a base step added later at a level where the archetype has no row fails the entry; `at` means exactly those levels, with a `why` saying what the guide's table shows there; `count` is exact (default 1).
2. **Substitutions.** An entry may say `replaced_by: [..]`, naming what the archetype's rows give in the item's place; the test checks a row grants each one at or after the first level the item is lost at (at a lost level for a substitution, later for a delay: the Standard Bearer's companion is lost at 1 and its own copy comes at 5). 302 entries in 103 archetypes carry it: the archetype's own copy of a feature or selection under the same name, or the same name with the archetype's in brackets, filled mechanically, and the monk archetypes' own bonus-feat lists. An editor can then tell a substitution from a loss.
3. **The lists.** 196 archetypes, 961 entries, 76 of them with `at`; the 3 archetypes that lose casting (the Tandem Executioner, the Divine Guardian, the Warrior of the Holy Light) have a bare `SpellLevels` entry, with the `why` of each. 2 archetypes (the Crossblooded and the Nine-Tailed Heir sorcerers) lose nothing and are pinned with `loses: []`. The 961 entries stand for 1610 dropped (level, item) pairs. Each `at` entry's `why` paraphrases what the guide's progression table shows at those levels (the names in its cells, the level-0 row read with level 1). An archetype-level `why` is optional and kept for three only: the Titan Fighter (the user's ticket 131 decision, against its table), the Underground Chemist (its own Discovery selection carries the picks) and the Flesheater (the list follows its table, which shows more than its overview). Earlier archetype-level texts restating the list were dropped because they were untrue where the archetype replaces an item with its own copy; the notes for the Spell Dancer, Bladebound and Cruromancer are in their slot entries' `why`.
4. **Decisions** (the main session, 2026-11-14): `why` on every entry with `at` and optional per archetype; class skills, save overrides and `skipped_levels` stay with `class_skills.rs`, `archetype_saves.rs` and `scaling_features.rs` (the test's doc comment says so); counts exact; no note file (no data changed); the hand assertions in `scaling_features.rs` stay.
5. **Tests** (`archetype_losses.rs`): `an_archetype_row_keeps_what_the_base_row_gives_unless_the_list_says_it_is_lost`, `a_listed_loss_the_rows_do_not_make_fails`, `every_archetype_has_one_list_and_every_list_names_an_archetype`, `the_check_names_the_class_the_archetype_the_level_and_the_item`, `a_count_of_two_is_exact_and_the_suggestion_replaces_the_existing_entry`, `lowered_spell_levels_are_a_loss_and_a_bare_entry_covers_every_casting_level`, `replaced_by_names_what_the_rows_grant_in_the_items_place`. A failure names the class, the archetype, the level and the item, offers both ways out and prints the entry to paste (for an item that already has an entry: "replace the existing entry", its reasons kept).
6. **Snapshots:** silent; no data or rules changed. **The queue's proof** should show nothing new from this ticket.
