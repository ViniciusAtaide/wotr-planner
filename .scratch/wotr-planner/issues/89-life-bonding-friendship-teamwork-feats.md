# Life-Bonding Friendship gives its two teamwork feats

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Found by the QA builds ([ticket 88](88-qa-full-builds.md)). The Azata superpower Life-Bonding Friendship is a plain option in the data (`azata/superpowers/life-bonding-friendship` in `libs/game-data/mythic/paths/azata.ron`): it grants nothing, so a Build that takes it gets no teamwork feat pick.

**Repro:** open the Jack-in-the-Green Build from ticket 88 (`#b=AeWOv0tCURTH77n3vqfW60bRllRrkUNj1BCEFQQVZBGJooOLOjxQBx3cdNHJQXRyEAU3UVBc3N1ERREUfyCI-Efo9YHK2908cL7nez58ORyf7C_3gqls_8XWeTK5D7N3s-nR-Z74m3ZifPVX_77VGZje8OqFyeJN4IEfyaF5jAv3pwHWtLpyn5df5sZDJN3ymD6cXTH03qkVtL82OaMPz-VKfWAxPl9ca3AtPzdWi__O7ug4Wq4mh6VY3m5GAIgASIxJ2pOzlYWtiOp1N4x7QAJifPLCq14LoZhDUEOEVUGeUUG-HgAGgjZCiKJUAZRS5eC2ReUBHWC8BA`), Mythic track. MR 6 holds "Superpowers: Life-Bonding Friendship" and a Mythic feat slot; MR 6 and MR 7 have no other slot. The same holds for the Enforcer Build (LBF at MR 6). Both write-ups take a teamwork feat at each of those ranks: "Life-Bonding Friendship (Shake It Off)" at 6 and "(Allied Spellcaster)" at 7; the Enforcer "(Shield Wall)" at 6 and "(Shake It Off)" at 7.

**The guide** (`azata#life-bonding-friendship`): "You choose a teamwork feat this rank and the next rank." Its "Playing an Azata" adds that LBF is very powerful on a class that receives free teamwork feats.

**Expected:** taking Life-Bonding Friendship gives a teamwork-feat slot (any feat of category Teamwork, prerequisites as for a normal slot unless the guide says otherwise) at the rank it's taken and another at the next rank. Taken at MR 10, the second slot has no rank to land on; say what happens (none, as the game can't give it).

## Build

- A data change in `azata.ron` (the option's rank rows, the shape Combat Trick's `levels` with a `feat_slots` row uses, if a superpower option can carry rank rows; otherwise the smallest rules change that lets it), with a `libs/game-data/README.md` note and a real-data test in `libs/seeds/tests/game_data/`.
- The checks in the repo's `CLAUDE.md` (and `--schema` if the migration changes).

**Done when:** a Build with LBF at MR 6 shows a teamwork-feat slot at MR 6 and MR 7 that offers Shake It Off, Allied Spellcaster and Shield Wall (Shield Wall needs Shield Proficiency) and not a non-teamwork feat; a real-data test pins it; an LBF taken at MR 10 behaves as the ticket decides.

## Progress log

- 2026-11-06: started (agent; the user away, small choices settled here). Read the repo's `CLAUDE.md`, the map's Notes, this ticket, ticket 88, and tickets 55 and 62's Answers. Scratch in `t89/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/20f0bb4f-7437-4fe9-8ef8-ebb8716f6fbe/scratchpad/t89/`).
- 2026-11-06: a data-only fix isn't possible. A superpower option can carry rows (`levels`, Combat Trick's shape), but an option's level comes only from class levels, the other options held or the granting option, and the walk advances options at each level, before the level's ascensions; nothing counts Mythic ranks, so "this rank and the next" can't be said. Smallest rules change: a fourth `LevelFrom`, `MythicRanks`, whose rows' slots are the rank's path slots (`PathFeat`/`PathSelection`, numbered after the path row's own), so no new slot id and no link-layout change. `level_from` is a database column with a CHECK, so the migration changes too (plus a path-only constraint).
- 2026-11-06: rules test-first on the small world: the Trickster's tricks get `level_from: MythicRanks` and an invented Friendship trick (rows 1 and 2: a Teamwork slot); 5 tests in `mythic_rank_rows.rs`, red (all 5) with the variant stubbed to level 0, green after the change; the other 403 stayed green with the world change. Real-data tests (`life_bonding_friendship.rs`, 6) red on HEAD's data (`t89/red-real-data.txt`), green after the `azata.ron` change. Dev database rebuilt per `CLAUDE.md`. RustRover's `create_new_file` staged the two new test files in git; unstaged them (`git restore --staged`), nothing else staged.
- 2026-11-06: proof (`scripts/proof.sh both`, against HEAD) and `scripts/check.sh --schema` passed (below). The proof's 64 Azata Builds that differ without Life-Bonding Friendship differ only in their sheet's hash; a scratch program (`t89/sheetcheck/`, the working tree's rules over HEAD's and the working tree's data) showed that's the superpowers' new option levels in the state and nothing else. Answer written; Status resolved.

## Answer

Done 2026-11-06, uncommitted. Taking Life-Bonding Friendship now gives a Teamwork feat slot at the rank it's taken and another at the next rank. Both are the rank's own path feat slots, shown under the rank on the Mythic track. A data change alone couldn't say "the next rank", so this needed a small `rules` change and a schema change too. Nothing changed in the link layout, the slot ids or the frontend.

### What the guide says, and the MR 10 decision

The guide (`azata#life-bonding-friendship`) says "You choose a teamwork feat this rank and the next rank." Nothing there waives prerequisites, so the slots check them as any Teamwork category slot does. Shield Wall needs Shield Proficiency.

**Taken at MR 10** (settled here, as the ticket proposed), it gives one slot, at MR 10. The Azata has no MR 11, so the second slot has no rank to land on, and the game can't give it. Picked at MR 4 or 8, the second slot comes at MR 5 or 9. MR 8 and 9 share level 17.

### Data

In `libs/game-data/mythic/paths/azata.ron`:

- `azata/superpowers` has `level_from: MythicRanks`.
- Life-Bonding Friendship has two rows, `(level: 1, feat_slots: [Category(Teamwork)])` and `(level: 2, feat_slots: [Category(Teamwork)])`. This is Combat Trick's `levels` shape.
- No other superpower has rows.

The note is in `libs/game-data/README.md`: the ticket 89 paragraph, plus the `level_from` list under ticket 33. `GLOSSARY.md`'s **Option level** says a path's selection may count Mythic ranks.

### Rules

- **New level source:** `LevelFrom::MythicRanks`. An option held through such a selection takes as its option level the Mythic ranks reached since its pick, with the pick's own rank counting as 1 (`holding_level`).
- **Where the slots go:** at each rank, after the path row's slots, `fill_rank_rows` advances those holdings. It turns the row slots they give into that rank's `PathFeat` slots (or `PathSelection` for a row selection), numbered after the path row's own.
  - It leaves every other option's row slots pending for the level, as before.
  - It loops until no new rank-row slot appears.
  - The slots are filled with the rank as their source (`from_path`).
- **What follows from existing rules:**
  - A Path switch treats the slots as the path's grants of those ranks. Gold Dragon and Legend drop them; the Devil keeps them through MR 7.
  - The superpower can't be removed while one of its slots holds a pick. Once the slots are cleared it can be, and removing it takes both slots with it.
- **Docs:** `SlotId::PathFeat` and `PathSelection` are updated.
- **Codec:** no change, since both slot kinds already carry any index.
- **Not exercised:** the `PathSelection` branch has no data yet.

### Schema

`feature_selection.level_from` also accepts `'mythic_ranks'`. A new constraint, `feature_selection_mythic_ranks_on_a_path`, allows it only on a path's selection. `seeds::columns` maps the new value. The dev database was rebuilt per `CLAUDE.md`:

1. Deleted it.
2. `sqlx database create`, then `migrate run`.
3. Reseeded (unchanged counts).
4. Ran `cargo sqlx prepare` (`.sqlx` unchanged).

`azata/superpowers` is the only `mythic_ranks` row.

### Tests

**Rules, on the small world** (`mythic_rank_rows.rs`, 5 tests): written first, and red with the new variant stubbed to level 0. The world's Trickster tricks get `level_from: MythicRanks` and an invented Friendship trick, and the other 403 tests stay green. The tests check that:

- picked at MR 3, the trick gives a "Teamwork feat" slot at MR 3 and at MR 4 (index 1, after the bonus-feat slot), and never a level's option slot;
- the slot offers only Outflank, the world's one teamwork feat, held from the rank's level; Dodge is refused, and a neutral character is refused Outflank;
- picked at MR 9, its second slot comes at MR 10 and not before;
- removal behaves as above;
- a Gold Dragon switch drops either pick.

**Real data** (`libs/seeds/tests/game_data/life_bonding_friendship.rs`, 6 tests): red on HEAD's data (`t89/red-real-data.txt`), green now. They check that:

- the data has the shape above, and `azata/superpowers` is the only Mythic-ranks selection;
- a sorcerer 16 with Life-Bonding Friendship at MR 6 gets a Teamwork slot at MR 6 and at MR 7, while Zippy Magic gives none;
- both slots offer exactly the 13 teamwork feats:
  - Shake It Off and Allied Spellcaster are open;
  - Shield Wall is `Unmet` for the sorcerer and open for a fighter;
  - both write-ups' picks (Jack: Shake It Off, then Allied Spellcaster; Enforcer: Shield Wall, then Shake It Off) are possible and held from levels 15 and 16;
  - Dodge, or Shield Wall without the proficiency, is refused;
- picked at MR 4 or 8, the second slot comes at MR 5 or 9;
- picked at MR 10, the Build has one Teamwork slot in all;
- removal behaves as above.

**Database:** `database.rs` has one more case: `mythic_ranks` on a class's selection is refused.

### Proof

The proof ran as `scripts/proof.sh both`, against HEAD (`t89/proof/`, `t89/breakdown.txt`, `t89/sheetcheck.txt`). It covered 2,146 Builds on each side, none of them only on one side, and 112 differed.

- **48 Builds that pick Life-Bonding Friendship:** each is impossible on HEAD, where its teamwork picks have no slot, and possible now. The only new slots are Teamwork `PathFeat`s:

  | Rank | New slots |
  |---|---|
  | 4 | 13 |
  | 5 | 12 (one Build stops before MR 5) |
  | 6 | 22 |
  | 7 | 22 |
  | 8 | 3 |
  | 9 | 3 |
  | 10 | 10 |

  No slot disappears.
- **64 Azata Builds that don't pick it:** they differ only in `sheet`, the sheet hash. The superpowers they hold now carry an option level in the state (`CharacterState::option_levels`, which only `GrantingOption` selections read). Comparing every Build's sheets, with HEAD's and the working tree's data on the working tree's rules (`t89/sheetcheck/`):
  - 2,034 are equal as they stand;
  - 2,098 are equal once those option levels are left out;
  - the remaining 48 are exactly the Builds that pick Life-Bonding Friendship.
- **Ticket 48's shortcut** stayed exact on both sides: 1,392,450 feat options on A and 1,392,717 on B, the 267 more being the new slots.

### Checks

- `scripts/check.sh --schema`: ALL-OK (fmt, clippy, wasm-clippy, test, rules-wasm, seed-check, sqlx-migrate, sqlx-prepare; `t89/check.txt`).
- `cargo test --workspace -- --list`: 729 tests, including the 11 new ones (`t89/test-list.txt`).
- `--full` (the cargo-leptos builds) wasn't run, as is usual per ticket.
- What the browser downloads changes only by the data's new field and the rules wasm.
- Nothing is staged or committed.
