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

Built 2026-11-14.

1. **What was built.** `libs/seeds/tests/game_data/archetype_losses.rs` and `archetype_losses.ron` (the lists, read with `include_str!` and serde, as `gold.rs` reads its files). For every archetype row, the base row's shortfall in features, feat slots and selections (multisets), granted feats and feats-for-a-value (sets), and spell levels set below the base row's must match the archetype's list exactly. A list entry is `(item, at, count, why)`: no `at` means lost at every level where the base row gives it (so a base step added later at a level where the archetype has no row fails the entry); `at` means exactly those levels; `count` is exact (default 1).
2. **The lists.** 196 archetypes, 961 entries, 79 of them with `at` (76 partial levels and 3 spell-level entries); 2 archetypes (the Crossblooded and the Nine-Tailed Heir sorcerers) lose nothing and are pinned with `loses: []`. The 961 entries stand for 1610 dropped (level, item) pairs. Each archetype has one `why`, each entry with `at` has its own `why`, in our words: a paraphrase of what the guide's table shows at those levels (names of the table's cells at the lost levels). Most archetype `why`s are written from the list itself (the items it gives up) and bear out against the table, which review.txt sets beside every dropped item; those of the Flesheater, Spell Dancer, Bladebound, Sensei, Tactical Leader, Shadowcaster, Scroll Savant, Cruromancer, Spell Master, Titan Fighter and Underground Chemist are written by hand from the guide's sentences. The Titan Fighter's Weapon Training (the user's ticket 131 decision, against its table) and the Underground Chemist's rogue talents (its own Discovery selection carries the picks) are listed with those reasons.
3. **Decisions** (the main session, 2026-11-14): one `why` per archetype plus one per `at` entry; class skills, save overrides and `skipped_levels` stay with `class_skills.rs`, `archetype_saves.rs` and `scaling_features.rs` (the test's doc comment says so); counts exact; no note file (no data changed); the hand assertions in `scaling_features.rs` stay.
4. **Tests** (`archetype_losses.rs`): `an_archetype_row_keeps_what_the_base_row_gives_unless_the_list_says_it_is_lost`, `a_listed_loss_the_rows_do_not_make_fails`, `every_archetype_has_one_list_and_every_list_names_an_archetype`, `the_check_names_the_class_the_archetype_the_level_and_the_item` (the check on made-up rows). A failure names the class, the archetype, the level and the item, offers both ways out (carry it on the row, or list it as lost with the guide's reason) and prints the entries to paste.
5. **Snapshots:** silent; no data or rules changed. **The queue's proof** should show nothing new from this ticket.
