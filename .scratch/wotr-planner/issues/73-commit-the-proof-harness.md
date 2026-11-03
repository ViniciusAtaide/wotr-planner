# Commit the behaviour proof as a harness

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Each ticket since 47 proved "no behaviour change except the intended one" with a proof program kept in the lead's scratchpad (latest: `scratchpad/t67/proof/main.rs` with `t47.rs`, run by `t67/proof-run`; compared by `t67/compare.py`). Every ticket copied the last ticket's program, added its own fields and Builds, and built it with its own build folder. Across this session's agents that came to 256 crate copies, 329 scratch builds and 55 GB of scratchpad, and the program now carries a field for each ticket (`aligned63`, `reach64`, `iw66`, `dual67`, slot hashes leaving out one feat…).

Commit it as a tool in the repo instead (user, 2026-11-03, after the session retro):

1. **A command, not a snapshot test.** The full proof checks about 2,000 Builds and takes about 4 minutes on 8 threads, too slow for `cargo test`.
2. **Only the general fingerprint.** Drop the per-ticket fields. A ticket that needs a special breakdown filters the comparison's output itself.

## Build

The user is awake: settle small design choices yourself and record them here; bring a real open question back in the report.

**Shape:**
- **A workspace crate, `libs/proof`** (a binary, depending on `rules` and `seeds` as `proof-run` does), with:
  - `record <game-data dir> [--links <file>] [--threads N] [--only <name substring>]`: writes one JSON line per Build, in a stable order.
  - `compare <a.jsonl> <b.jsonl>`: lists the Builds only in one side and, per Build that differs, the fields that differ and, for slots, the slot ids. It exits non-zero when anything differs.
  - `export <game-data dir>`: writes the harness's own Builds as share links (`name<TAB>link`), so another revision's harness can record the same Builds through `--links`.
- **The fingerprint per Build:** what ticket 67's proof records that isn't tied to one ticket: whether it's possible, the sheet hash, `removable`, each slot's options with statuses (hashed, with the count open) and its lock-reason texts, the class picker at each level and one past the end, the path and switch pickers and their reasons, and the `allows` battery.
  - `record` also keeps the consistency checks that panic: `is_removable` against `removable`, `removable_classes`, `allows_many` against `allows`, and ticket 48's picker shortcut against full walks.
- **The Builds:** everything the ticket 67 proof builds:
  - ticket 47's Builds;
  - the benchmark's links, committed as a data file (now `scratchpad/t56/bench/links.json`);
  - the seeded full fills and the path switches;
  - the Builds of tickets 49 and 54–67.
  - Group the builders by subject (feats, selections, mythic, alignment…), not by ticket number.
  - The generator keeps its exact behaviour, quirks included (for example how it treats Witch Wandering Hex), so the output matches; document each quirk where it lives.
- **`scripts/proof.sh`** runs both sides and the comparison:
  - **Game-data change:** one build of the working tree's harness, run on HEAD's `libs/game-data` (taken through `git worktree` or `git archive` into the scratch dir) and on the working tree's.
  - **Rules change:** the harness built at a given ref in a git worktree, and the working tree's, on the same data. The working tree's `export` supplies the links, so both sides record the same Builds.
  - One build folder shared through `CARGO_TARGET_DIR`, outside the repo: no new `target/` per run.
  - Outputs go to a directory passed as an argument, never into the repo.
- **A short usage note** where an agent will find it: the crate's doc comment, plus one line in the repo's `CLAUDE.md` pointing to `scripts/proof.sh`.
- **A fast test** in `cargo test`: a record-and-compare round trip on a handful of Builds, so the crate can't rot unnoticed.

**Done when:**
- **Identity with ticket 67's proof:** `record` on the current `libs/game-data` gives, for every Build in `scratchpad/t67/proof-C.jsonl`, the same general fields as that file. Tickets 68–72 changed neither `rules`' behaviour nor the game data, so the two must match. Write down which of C's fields count as general, and why each dropped one is ticket-specific.
- `compare` of that record against itself reports nothing and exits 0. `compare` against a record made with a deliberately changed game-data copy reports exactly the changed Builds and slots.
- `scripts/proof.sh` works in both modes. The rules mode is tried against `HEAD` with no change: an empty comparison.
- The `cargo test --workspace -- --list` diff is only the new test or tests.
- The lighter per-ticket checks pass: fmt, clippy native and wasm, `cargo test --workspace`, the `rules` wasm build, seed `--check-only`.
- No change to `rules`, `libs/game-data/`, the schema or the link format.

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- 2026-11-03: saved `cargo test --workspace -- --list` before any edit (739 tests). Ported ticket 67's `proof/main.rs` and `t47.rs` into `libs/proof` (lib + bin): `fingerprint` (the general fields and the consistency checks), `generate` (the fillers and the full Builds), `links` (the benchmark's links, `export`, `--links`), `compare`, and `builds/` grouped by subject (`classes`, `feats`, `selections`, `talents`, `mythic`). The `new` and `t55` features are gone (always on). Committed the benchmark's links as `libs/proof/bench-links.tsv`. Added `scripts/proof.sh`, the crate docs, a "Behaviour proof" line in `CLAUDE.md`, and the round-trip test `tests/round_trip.rs`.
- Identity: `proof record libs/game-data` (release, 8 threads, 2 min 48 s) gives 1,999 Builds, the same names as ticket 67's `proof-C.jsonl`, and every general field equal on all 1,999 (`scratchpad/t73/identity.py`, 0 problems). Ticket 48's shortcut was checked on the same 803 Builds and 1,341,108 feat options as C.
- `compare` of the record against itself: no lines, exit 0. Against a record of a game-data copy with Barkskin moved from monk 4 to monk 6: exit 1, one Build only in A and 78 differing; the lines equal an independent diff of the two records, every named Build holds a monk, every monk Build with a ki-power slot below 6 is named (the Sohei's, which start at 6, are not), and in the Builds with the same picks every slot named is a ki-power slot, but in the one that holds Barkskin at 4 and turns impossible (`scratchpad/t73/changed.py`).
- `export` gives back all 1,999 Builds through links (none skipped).
- `scripts/proof.sh data <out>` (HEAD's `libs/game-data` against the working tree's) and `scripts/proof.sh rules <out> HEAD` (the harness built against HEAD's crates in a `git archive` copy, and against the working tree's, both recording the working tree's `export`ed links): each an empty comparison, exit 0, in 5 min 56 s and 5 min 35 s with the builds. All four side records are byte-identical to the standalone record.
- Checks: fmt, clippy native and wasm, `cargo test --workspace` (the list diff is only `record_export_and_compare_round_trip`, 740 tests), the `rules` wasm build, seed `--check-only`. No change to `rules`, `libs/game-data/`, the schema, `.sqlx` or the link format; the user's edit in `apps/backend/src/main.rs` is untouched.

## Answer

`libs/proof` is a workspace binary (`proof record | compare | export`) with its usage in the crate docs; `scripts/proof.sh data|rules <out-dir> [<ref>] [-- <record options>]` builds it once in `$CARGO_TARGET_DIR` (outside the repo), records both sides and compares them. A full record: 1,999 Builds in about 2 min 50 s on 8 threads.

**General fields kept** (C's field → the record's):
- per Build: `build`, `possible`, `sheet`, `removable`, `classes` (each level and one past the end), `class_reasons`, `paths`, `path_reasons`, `switches`, `switch_reasons`, `allows`; and `who`, `levels`, `picks`, which describe the Build (its classes with archetypes, its counts), so a ticket can filter a comparison by class;
- per slot, now a named object rather than a 14-item array: `options` (C's 0, the options with statuses, hashed), `open` (1), `total` (2), `kind` (4), `reasons` (8, the lock-reason texts of options whose issue holds no `Not`, hashed) and `negated` (9, the others, listed in full).

**Dropped, each tied to one ticket:**
- `extra_feats` (49), `mythic_extras` (55), `monk_levels` (58), `kineticist_levels` (59), `listed` (60), `listed61` (61), `takes62` (62), `aligned63` (63), `reach64` (64), `iw66`, `rage66`, `erp_reasons66` (66), `dual67` (67): each says whether a Build holds what its ticket changed, so that ticket could split its Builds; `who` and the Build names do that now.
- `classes_without` (60: without the bard, skald, shifter, alchemist, inquisitor and magus), `classes_without61` (hunter, fighter, ranger), `classes_without63` (the archetypes with an alignment rule): the class picker hashed without one ticket's entries, so its intended change stayed out; `classes` holds the whole picker.
- slot items 3 (without Extra Slayer Talent and the 2nd-or-4th combat style talent: 60), 5 (without Witch Wandering Hex: 62), 6 and 7 (without and the open flags of Extra Discovery and Extra Arcana: 64), 10–13 (without the Instinctual Warrior's powers and Extra Rage Power, their reasons and open flags: 66): the same, per slot; `options`, `open` and `reasons` hold the whole.
- `texts` and `needs_not`: ticket 65's counts of lock-reason texts and of those reading "Needs not"; the texts themselves are in `reasons`, `negated` and the pickers' `*_reasons`.
- `allowed`: the count of `1`s in `allows`, which is kept.

**Generator quirks kept** (each documented where it lives, in `generate.rs` and `builds/`):
1. `fill_full` fills in three rounds, each over the slots empty when it starts, in sheet order.
2. Witch Wandering Hex counts among a slot's choices when its only issue is an unmet prerequisite, and the slot stays empty when it's drawn then.
3. The path is drawn only for a Build of 9 levels or more.
4. Ability increases are drawn where the sheet has one and the level holds none, kept only if `allows` keeps them; skills get at most 40 draws per level, each kept only when it changes the Build and `allows` keeps it.
5. The full Builds come from the harness's own possible Builds only (not the benchmark's), seeded by `fnv(name) | 1`; a switched one is kept only when the switch holds and the Build stays possible.
6. `with_selections_filled` fills the levels' selection slots only, never a rank's, with the first open option, until none has one.
7. Ticket 47's fill (`fill_first_choices`) runs two passes, each reading the sheet once before it fills, over the levels' slots of its listed selections and three one-feat lists only.
8. Many of ticket 47's Builds (the targeted ones, the backgrounds' wizard) name no class choice for their classes.
9. The groups that read the game data's lists (every class and archetype, prestige classes, backgrounds, the sorcerer's bloodlines and the options of four selections) follow the data, so a data change can add or drop Builds.
10. Ticket 62's fixed picks choose a hex or curse by what the filled Build already holds; ticket 55's Second Bloodline takes Abyssal or Arcane, whichever the level-1 bloodline isn't.

**Decisions** (small, settled here):
1. **Build names keep their ticket tags** (`t60 bard 20 / empty`); only the builder code is grouped by subject. The full Builds are seeded by their names' hashes, so renaming would change their draws, and records made before (ticket 67's C) would no longer compare by name.
2. **`negated` stays listed in full** beside the hashed `reasons` (ticket 65's split), so the identity with C holds and a negated reason's change is readable in the record.
3. **`record --links <file>` records exactly that file's Builds**; without it, the own Builds, the benchmark's and the full ones. Links are decoded with `decode_build`, not `open_link`, so a Build no sheet holds round-trips too. `export` checks each link decodes back to the same Build and leaves out (and names on stderr) any that doesn't; today none.
4. **The benchmark's links** are `libs/proof/bench-links.tsv` in `export`'s `name<TAB>link` form, named `bench <name>` when recorded.
5. **`compare`'s output** is tab-separated lines (`only in A|B`, `differs`, `slot`), the counts go to stderr, and it exits 0 (agree), 1 (differ) or 2 (bad input).
6. **Rules mode builds the working tree's harness against the ref's crates**: `git archive` of the ref into `<out-dir>/src-A`, the working tree's `libs/proof` copied in and added to its members. Both sides then fingerprint alike, and it works for a ref from before the harness (today's `HEAD`). If a rules change alters the API the harness calls, the copy in `src-A` needs the matching edit. Both sides record the working tree's game data.
7. **`git archive`, not `git worktree`**, for both modes: an interrupted run leaves nothing in `.git` to clean up.
8. The script builds with `SQLX_OFFLINE=true`, so it never needs the database, and refuses an out dir or `CARGO_TARGET_DIR` inside the repo.
9. **Ticket 48's shortcut checks stay on the Builds ticket 67 checked them on** (the `t55`–`t67` names and their full ones, `fingerprint::SHORTCUT_CHECKED`); the other consistency checks run on every Build, as before.
10. The fast test (`tests/round_trip.rs`, about 0.5 s) records four small Builds of the real game data, checks a record agrees with itself and with one made through `export`/`--links`, and that a changed and a missing Build give exactly the expected lines.

Open: none.
