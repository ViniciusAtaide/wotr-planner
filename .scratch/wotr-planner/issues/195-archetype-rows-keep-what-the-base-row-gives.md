# Nothing checks that an archetype row keeps what the base row gives

Type: design
Status: open
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
