# The Dual-Cursed Oracle's second curse becomes a pick

Type: task
Status: resolved
Blocked by: 66
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 62](62-wandering-hex-and-beneficial-curse.md)'s decision (user, 2026-11-02, by recommendation): the Dual-Cursed Oracle's "Second Curse" is a feature in the data, but the guide says the oracle chooses it. Make it a slot of `oracle/oracles-curse` at the level its table gives (1), in place of the feature, so a curse taken there isn't offered again by Beneficial Curse's slot or anywhere else.

## Build

AFK, test-first (`tdd`); the only agent in the repo. Same data method as tickets 54–62: extractor module, one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, before/after proof with the latest proof program (only Builds with a Dual-Cursed Oracle differ). Ticket 48's picker shortcut must stay exact. Data from the saved guide pages in `guide-cache/` only; a question the guide doesn't settle is written in this ticket with a recommendation, and the recommendation is taken (user, 2026-11-02).

**Done when:** the lighter per-ticket checks pass (fmt, clippy native and wasm, `cargo test --workspace` with the `-- --list` diff, the `rules` wasm build, seed `--check-only`, `sqlx prepare --check` if the schema changed); plus real-data tests: a Dual-Cursed Oracle 1 has two curse slots (its own and the second), the second not offering the first's curse; Beneficial Curse doesn't offer either; the sheet labels the slot (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- 2026-11-03: started. Baseline: HEAD db90eed (the user's uncommitted doc-comment line in `apps/backend/src/main.rs` saved as `t67/main-rs-user-edit.diff`, identical to ticket 66's, and left alone). Scratch in `t67/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t67/`): `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (23 report items; byte-identical to ticket 66's fresh extraction). Read `oracle#dual-cursed-oracle` (its summary "Must accept two curses"; its table: "Oracle's Curse" and "Second Curse" both at 1) and `oracle#second-curse` ("The Oracle must choose a second curse. One of these curses doesn't change as the oracle gains levels."). In the data, the archetype's level-1 row held one `oracle/oracles-curse` slot and the feature `oracle/dual-cursed-oracle/second-curse`; nothing else names the feature; every curse is taken once and none has rows. A row may hold one selection twice already (a cleric's `cleric/domain`).
- 2026-11-03: real-data tests first (`game_data.rs`, 3 new), red on HEAD's data (`t67/real-data-red.txt`: 221 pass, 3 fail, each at the missing slot).
- 2026-11-03: no `rules`, schema, `Build` or codec change was needed. Data and extractor: new `extract::second_curse` (`apply`), run after `instinctual_rage_powers::apply`. `t67/rewrite/` wrote `libs/game-data/` byte-identical to its dry run (`t67/dry/game-data/`): 1 change in 1 file (`classes/oracle.ron`, 2 lines; `t67/game-data-log.json`), 0 check problems; a second run refuses (`t67/second-run.txt`). Real-data tests green (224, `t67/real-data-green.txt`). A fresh extraction (`t67/extract-new/`) is byte-identical, report included (23 items), to HEAD's extraction rewritten (`t67/dry/extract-old-rewritten/`, the same change), and its changed lines are `libs/game-data/`'s (`t67/lines-extract.txt`, `t67/lines-game-data.txt`).
- 2026-11-03: proof (`t67/proof/`, `t67/patch_proof.py`, `t67/proof-run/`, `t67/compare.py`, `t67/compare.txt`, `t67/picker/`, `t67/picker.txt`), UI (`cargo leptos watch` on 3900, headless Chromium: `t67/links/`, `t67/links.txt`, `t67/ui-check.js`, `t67/ui-check.txt`, `t67/ui-*.png`; server stopped), docs (`libs/game-data/README.md`, the map), reseed (no schema change; `t67/seed.txt`), checks (`t67/test-workspace.txt`, `t67/test-list.txt`, `t67/test-list.diff`).

## Answer

Done 2026-11-03, uncommitted. Ticket 62's decision is built: the Dual-Cursed Oracle's "Second Curse" is a second slot of `oracle/oracles-curse` at its level 1, in place of the feature. A data change only: no `rules`, schema, `Build` or link change, so `V1_LINK` stays.

### Data

`extract::second_curse` asserts the guide's two sentences, that "Second Curse" stands in the Dual-Cursed Oracle's table only, at 1, and "Oracle's Curse" at 1 in both the oracle's and the archetype's, that every curse is taken once, that nothing names the feature, and that the rows in force hold one curse slot (at 1) before and two after. Then it takes the feature off the archetype's list and its level-1 row, which now holds `oracle/oracles-curse` twice (`selections: ["oracle/dual-cursed-oracle/revelation", "oracle/mystery", "oracle/oracles-curse", "oracle/oracles-curse"]`), as a cleric's holds `cleric/domain` twice. `t67/rewrite/` changed 2 lines of `classes/oracle.ron`, 0 check problems. The extractor was never run over `libs/game-data/`.

### Rules, schema, codec, UI

No code change. The level-1 slots of a Dual-Cursed Oracle are, in key order, its revelation (0), the mystery (1) and two curse slots (2 and 3): the first curse keeps the index it had, so existing links keep their picks, and the new slot is `Selection { level, index: 3 }` at the archetype's first level. Each offers the nine curses; a curse taken in the first is "Already taken" in the second, and one taken in the second locks it in the first as breaking that later pick (as a cleric's two domain slots answer). Beneficial Curse's slot (ticket 62) offers neither held curse. The sheet labels both slots "Oracle's Curse". Ticket 48's shortcut needs nothing new and stays exact (Proof).

### Tests

Real-data tests (`game_data.rs`, 3 new, red on HEAD's data first):

- **Data:** the Dual-Cursed Oracle's rows in force hold two curse slots at 1 and none after; the plain oracle has one, and no other archetype more than one; the feature is gone from its list and rows.
- **A Dual-Cursed Oracle 1:** two slots of `oracle/oracles-curse` (indices 2 and 3), both labelled "Oracle's Curse"; a plain oracle 1 has one; the feature isn't held. With Plagued in either slot, the other offers the nine curses, Plagued locked ("Already taken" in the second, breaking the later pick in the first), the rest open. Plagued and Lame are possible and both held at 1; Plagued twice is refused ("Already taken").
- **Beneficial Curse:** a Dual-Cursed Oracle 6 with both curse slots filled takes it at MR 1; its slot offers the nine curses with exactly her two "Already taken" (each refused there), and a third curse is possible.

No extractor test: `second_curse` has no reader of its own (it reuses `levels_in_tables`, ticket 57's test), and the fresh extraction exercises `apply` on the guide.

### Proof

Ticket 67 changes no rules, so one program (`t67/proof-run`, the working tree's rules) runs on HEAD's data (A) and ticket 67's (C). It is ticket 66's proof plus each Build's Dual-Cursed Oracle (`dual67`) and 26 ticket-67 Builds: a Dual-Cursed Oracle 1, 5, 6 and 20 and a fighter 1 / Dual-Cursed Oracle 5, empty and filled; two curses, the second only, one curse twice; Beneficial Curse on a Dual-Cursed Oracle 6 holding two curses with its slot empty, a third curse, or either held one; a plain oracle 1 with a pick in a fourth level-1 slot (each fixed one also filled). 1,991 Builds in A, 1,999 in C. `t67/compare.py` (`t67/compare.txt`): 0 problems.

- **A checks the program:** without `dual67`, A equals ticket 66's proof C (the same rules on the same data) on all 1,953 Builds both have.
- **Builds without a Dual-Cursed Oracle (1,952):** 1,950 byte-identical. The other two are the plain oracle with a pick in a fourth level-1 slot (and its filled version): impossible in both; only their class pickers differ, and only in the Dual-Cursed Oracle's entry at level 1 (`t67/picker.txt`: BreaksLater on A, open on C, since that slot now exists for it).
- **Builds with one (39 in both, 8 only in C):** the 13 with the same picks in both gain exactly one slot, the curse slot at index 3 of the archetype's first level (level 1, or 2 for the fighter 1), and lose none. The 6 of them possible in both (ticket 47's Dual-Cursed Oracle and the empty ones) differ only by that slot: every slot both have answers the same. Of the 12 filled ones, the 5 possible in both hold one more pick on C (the new slot's), and the 7 others are the fixed ones above; the 14 generated full ones draw differently once the new slot is drawn, and are all possible.
- **Fixed picks:** two curses, the second only, and Beneficial Curse with a third curse or its slot empty are impossible on A (a pick in a slot it lacks) and possible on C; one curse twice, Beneficial Curse with either held curse, and the plain oracle's fourth slot are impossible in both. Every full Build with a Dual-Cursed Oracle in C is possible.
- **Ticket 48's shortcut stays exact:** on ticket 55's to 67's Builds and their full versions, every one of 1,341,108 feat options in C (1,338,480 in A) is open exactly when `allows` keeps the Build with it picked.
- **Run time:** 4 min 18 s (A, sharing the machine with the build for the browser check), 3 min 50 s (C), 8 threads.

### UI

`cargo leptos watch` on 3900, headless Chromium at 412×920 (`t67/ui-check.txt`):

- A Dual-Cursed Oracle 1's link: "MYSTERY Battle · ORACLE'S CURSE Plagued · ORACLE'S CURSE Lame · REVELATION (DUAL-CURSED ORACLE) +" (`t67/ui-both.png`).
- With Plagued only: "ORACLE'S CURSE Plagued · ORACLE'S CURSE +"; that picker opens as "Oracle's Curse · Level 1" with 8 open curses and "Show 1 locked": "Plagued Already taken".
- A Dual-Cursed Oracle 6 with both curses and Beneficial Curse: MR 1 reads "MYTHIC ABILITY Beneficial Curse · ORACLE'S CURSE +"; its picker ("Oracle's Curse", "Mythic rank 1") lists 7 open curses and "Show 2 locked": "Lame Already taken", "Plagued Already taken".
- No stale-link error and no console errors. Server stopped.

### Docs

- `libs/game-data/README.md`: ticket 67's paragraph.
- `GLOSSARY.md`: unchanged (Feature selection already allows a selection held twice).
- The map: the Decisions-so-far line.

### Checks

- `cargo fmt --all --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean. `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings`: clean.
- `cargo test --workspace`: pass (extract 106, rules 347 + unit 4, seeds files 25 + game_data 224 + postgres 28, server 4).
- `cargo test --workspace -- --list` (738) vs ticket 66's (735) (`t67/test-list.diff`): exactly the 3 new real-data tests.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `cargo run -p seeds --bin seed -- --check-only`: pass, unchanged counts (12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds).
- No schema change, so no DB reset and no `cargo sqlx prepare --check`. `cargo run -p seeds --bin seed` reseeded `wotr-planner-postgres` on 5434 only.
- `cargo leptos build` and `--release`: skipped, as asked.
- The v1 link vector is unchanged (no `Build` or format change).
- `apps/backend/src/main.rs`: the user's edit is byte-for-byte as it was (its diff equals `t67/main-rs-user-edit.diff`).

### Questions decided by recommendation

1. **The second slot's label.**
   - **Recommendation, built:** "Oracle's Curse", the selection's name, as every selection slot is labelled (ticket 54's and 57's slots of another list read by that list's name). The table's "Second Curse" names the cell, not a list; a label of its own would be a `rules` change for one slot, and the two slots offer the same list.
2. **Where the slot sits.**
   - **Recommendation, built:** on the archetype's level-1 row beside the first, so in key order the first curse keeps index 2 and the second is index 3. Existing links to a Dual-Cursed Oracle keep their curse pick where it was and gain an empty slot; no link change.
3. **"One of these curses doesn't change as the oracle gains levels."**
   - **Recommendation, built:** nothing to model. No curse has rows (its later effects are prose), and the archetype's "Curse 2", "Curse 3" and "Curse 4" features stay as the guide's table has them. Which curse stays put is a play-time detail with no effect on picks.
4. **A curse in the second slot shown in the first.**
   - **What happens:** the first slot's picker locks it as breaking the later pick (the engine's answer for an earlier slot), not "Already taken", as a cleric's first domain slot does with the second's domain.
   - **Recommendation, built:** keep it; it's the engine's existing rule for two slots of one selection.

None needs the user.
