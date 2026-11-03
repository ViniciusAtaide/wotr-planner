# Extra Discovery and Extra Arcana reach the Vivisectionist and the Hexcrafter

Type: task
Status: resolved
Blocked by: 63
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 60](60-borrowed-class-selections.md)'s decisions 2 and 3 (user, 2026-11-02): Extra Discovery gives a Vivisectionist one more slot of its `alchemist/vivisectionist/medical-discovery`, and Extra Arcana gives a Hexcrafter one more slot of its `magus/hexcrafter/hex-arcana`, by ticket 49's rule (`extra_pick_of` names the selection; the slot is of the one the character had a slot of first). Check each feat's prerequisite reaches them too.

## Build

AFK, test-first (`tdd`); the only agent in the repo. Same data method as tickets 54–60: extractor module, one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, before/after proof with the latest proof program (only Builds that take Extra Discovery or Extra Arcana differ). Ticket 48's picker shortcut must stay exact. Data from the saved guide pages in `guide-cache/` only; a question the guide doesn't settle is written in this ticket with a recommendation, and the recommendation is taken (user, 2026-11-02).

**Done when:** the lighter per-ticket checks pass (fmt, clippy native and wasm, `cargo test --workspace` with the `-- --list` diff, the `rules` wasm build, seed `--check-only`, `sqlx prepare --check` if the schema changed); plus real-data tests: a Vivisectionist's Extra Discovery gives a Medical Discovery slot at the level it's taken; a Hexcrafter's Extra Arcana gives a Hex Arcana slot; a plain alchemist's and magus's extra picks are unchanged; the sheet labels the slots (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- 2026-11-03: started. Baseline: HEAD ec3e072 (the user's uncommitted doc-comment line in `apps/backend/src/main.rs` saved as `t64/main-rs-user-edit.diff`, identical to ticket 63's, and left alone). Scratch in `t64/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t64/`): `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (23 report items; byte-identical to ticket 63's fresh extraction). Read the feats page's Extra Discovery ("Prerequisites: Dicovery", "You gain one additional discovery") and Extra Arcana ("Prerequisites: Magus Arcana", "You gain one additional magus arcana"), `alchemist#vivisectionist` and `#medical-discovery`, `magus#hexcrafter` ("Loses Spell Recall and one Magus Arcana") and `#hex-arcana`, and the alchemist's and magus's tables. Found: neither archetype has a slot of the class's own selection, so ticket 49's rule needs only the archetype's selection in `extra_pick_of`; the prerequisites (`Any` of the alchemist's discoveries or the Underground Chemist's feature; `Any` of the magus arcana) reach a Vivisectionist or Hexcrafter only if it happened to pick an alchemist discovery or a magus arcana in its own slot (question 1).
- 2026-11-03: real-data tests first (`game_data.rs`, 4 new, 2 extended, ticket 60's "no slot" test replaced): red on HEAD's data (`t64/real-data-red.txt`: 210 pass, 5 fail, each at the missing slot, extra pick or prerequisite; the plain alchemist/magus test passes, pinning them).
- 2026-11-03: no `rules`, schema, `Build` or codec change was needed. Data and extractor: new `extract::borrowed_extra_picks` (`apply`, `slot_levels`, `from_its_first_slot`), run after `archetype_alignments::apply`. `t64/rewrite/` wrote `libs/game-data/` byte-identical to its dry run (`t64/dry/game-data/`, also HEAD's `libs/game-data/` rewritten): 4 changes in 1 file (`feats.ron`, 2 lines; `t64/game-data-log.json`), 0 check problems; a second run refuses ("extra-discovery's extra pick"). Real-data tests green (215, `t64/real-data-green.txt`). A fresh extraction (`t64/extract-new/`) is byte-identical, report included (23 items), to HEAD's extraction rewritten (`t64/dry/extract-old-rewritten/`: the same 4 changes), and its two changed lines are `libs/game-data/`'s (`t64/lines-extract.txt`, `t64/lines-game-data.txt`). Extractor tests: `borrowed_extra_picks.rs`, 2.
- 2026-11-03: proof (`t64/proof/`, `t64/patch_proof.py`, `t64/compare.py`, `t64/compare.txt`), UI (`cargo leptos watch` on 3900, headless Chromium: `t64/links/`, `t64/links.txt`, `t64/ui-check.js`, `t64/ui-check.txt`, `t64/ui-*.png`; server stopped), docs (`libs/game-data/README.md`, `GLOSSARY.md`, the map), reseed (no schema change), checks (`t64/test-workspace.txt`, `t64/test-list.txt`, `t64/test-list.diff`).

## Answer

Done 2026-11-03, uncommitted. Ticket 60's decisions 2 and 3 are built: Extra Discovery gives a Vivisectionist one more Medical Discovery slot, and Extra Arcana a Hexcrafter one more Hex Arcana slot, at the level the feat is taken, by ticket 49's rule. A data change only: no `rules`, schema, `Build` or link change, so `V1_LINK` stays.

### Data

`extract::borrowed_extra_picks` asserts each guide phrase it rests on, the tables and the data's rows in force before changing anything: the Vivisectionist's "Medical Discovery" is at exactly the alchemist's "Discovery" levels (2 to 18), the Hexcrafter's "Hex Arcana" at the magus's "Magus Arcana" levels but the first (6 to 18; "Loses … one Magus Arcana"), and neither archetype has a slot of the class's own selection.

| Feat | `extra_pick_of` gains | Its `Any` gains |
|---|---|---|
| Extra Discovery | `alchemist/vivisectionist/medical-discovery` | `All([Archetype("alchemist/vivisectionist"), ClassLevel(alchemist, 2)])` |
| Extra Arcana | `magus/hexcrafter/hex-arcana` | `All([Archetype("magus/hexcrafter"), ClassLevel(magus, 6)])` |

`t64/rewrite/` (in `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t64/`) changed these 2 lines of `feats.ron`, 0 check problems. The extractor was never run over `libs/game-data/`.

### Rules, schema, codec, UI

No code change. Ticket 49's rule gives the slot of the selection the character had a slot of first: neither archetype ever has a slot of `alchemist/discovery` or `magus/magus-arcana`, so its slot is Medical Discovery's or Hex Arcana's. The slot offers what the archetype's own slots offer (the Vivisectionist's 25, the Hexcrafter's 15 arcana and 26 hexes, major hexes from magus 12), and the sheet labels it by the selection's name. Ticket 48's shortcut needs nothing new: both feats have an extra pick, so they were already always walked. Lock reasons show only the branches the character's classes could give (`labels`), so a character without the archetype reads Extra Discovery's and Extra Arcana's reasons as before.

### Tests

Real-data tests (`game_data.rs`; red on HEAD's data first):

- **New:** a Vivisectionist 5's Extra Discovery at 3 and 5 gives a "Medical Discovery" slot at each (none at 4), offering what its slot at 2 offers; filled, it's possible.
- **New:** a Hexcrafter 7's Extra Arcana gives a "Hex Arcana" slot at 7, offering what its slot at 6 offers (41), Agony locked "Needs Magus level 12 (have 7)"; filled, it's possible.
- **New:** the prerequisites reach them whatever they picked: a Vivisectionist 3 holding only Fast Stealth may take Extra Discovery, and a Hexcrafter 7 holding only hexes Extra Arcana. Before their first slot neither can (a Vivisectionist 1, a Hexcrafter 5), nor can a rogue 2 / Vivisectionist 1.
- **New:** a plain alchemist 3 and magus 3 are unchanged. The feat is locked with nothing held and open with a discovery or arcana held. It gives a "Discovery" or "Magus Arcana" slot offering what their own slot offers. This passes on HEAD's data too, pinning it.
- **Extended:** ticket 49's table of extra picks; ticket 49's each-feat-gives-its-class's-slot table, with a Vivisectionist and a Hexcrafter.
- **Replaced:** ticket 60's `extra_discovery_and_extra_arcana_give_a_vivisectionist_and_a_hexcrafter_no_slot`.

Extractor (`borrowed_extra_picks.rs`, 2): `slot_levels` (rows in force, an archetype's row replacing the base's) and `from_its_first_slot`.

### Proof

Ticket 64 changes no rules, so one program (`t64/proof-run`, the working tree's rules) runs on HEAD's data (A) and ticket 64's (C). It is ticket 63's proof plus:

- each slot's options hashed without the two feats, and their open flags;
- each Build's Vivisectionist and Hexcrafter (`reach64`);
- 55 ticket-64 Builds:
  - the real-data cases before the feat, with it and filled;
  - a Vivisectionist and a Hexcrafter 20, empty and filled;
  - the talent-only and hexes-only Builds;
  - three Builds no sheet holds.

That is 1,860 Builds in A and 1,864 in C. `t64/compare.py` (`t64/compare.txt`): 0 problems.

- **A checks the program:** with the added fields removed, A equals ticket 63's proof output on all 1,805 Builds both have.
- **Builds without a Vivisectionist or a Hexcrafter (1,775):** all byte-identical, plain alchemists, magi and Underground Chemists with their Extra feats included. The lock reasons don't change either, since they show only the branches the character's classes could give.
- **The 85 with one:**
  - 59 have the same sheet. In those, only the two feats' entries in feat slots differ: open earlier, or a reason naming the new branch.
  - 26 differ in their sheet. These are ticket 64's Builds with the feat, which gain 1 to 3 new slots each, and full Builds whose generator draws differently once the feat opens. The only new FeatSelection slots are Medical Discovery (+12 over C) and Hex Arcana (+26).
  - Possibility changes only for the talent-only and hexes-only Builds (impossible → possible). Their full versions exist only in C (4 Builds).
  - The three meant to be impossible are impossible in both: Extra Discovery on a Vivisectionist 1, Extra Arcana on a Hexcrafter 5, Extra Discovery on a rogue 2 / Vivisectionist 1.
- **Ticket 48's shortcut stays exact:** on ticket 55's to 64's Builds and their full versions, every one of 1,197,959 feat options in C (1,196,123 in A) is open exactly when `allows` keeps the Build with it picked.
- **Run time:** 3 min 13 s (A), 3 min 59 s (C), 8 threads.

### UI

`cargo leptos watch` on 3900, headless Chromium at 412×920 (`t64/ui-check.txt`):

- **A Vivisectionist 5's link:**
  - The sheet shows "MEDICAL DISCOVERY Fast Stealth" at 2, "FEAT Extra Discovery · MEDICAL DISCOVERY Cognatogen" at 3, and "FEAT Extra Discovery · MEDICAL DISCOVERY +" at 5.
  - The empty slot opens "Medical Discovery · Level 5" with its discoveries and talents.
  - "Show 11 locked" gives "Cognatogen Already taken", "Fast Stealth Already taken", "Greater Mutagen Needs Alchemist level 12 (have 5)" and "Crippling Strike Needs Advanced Talents (Vivisectionist)".
- **A Hexcrafter 7's link:**
  - The sheet shows "HEX Slumber" at 4, "HEX ARCANA Cackle" at 6, and "FEAT Extra Arcana · HEX ARCANA +" at 7.
  - The empty slot opens "Hex Arcana · Level 7" with the 15 arcana and the open hexes.
  - "Show 15 locked" gives "Agony Needs Magus level 12 (have 7)", "Death Curse Needs Magus level 18 (have 7)" and "Cackle Already taken".
- No stale-link error and no console errors. Server stopped.

### Docs

- `libs/game-data/README.md`: ticket 64's paragraph; ticket 60's last sentence now says "until ticket 64".
- `GLOSSARY.md`: **Extra pick** names an archetype's own version of the selection (Medical Discovery, Hex Arcana).
- The map: the Decisions-so-far line.

### Checks

- `cargo fmt --all --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean. `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings`: clean.
- `cargo test --workspace`: pass (extract 105, rules 345 + unit 4, seeds files 25 + game_data 215 + postgres 28, server 4).
- `cargo test --workspace -- --list` (726) vs ticket 63's (721) (`t64/test-list.diff`): 6 new tests (4 real-data, 2 extractor) and ticket 60's replaced test renamed.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `cargo run -p seeds --bin seed -- --check-only`: pass, unchanged counts (12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds).
- No schema change, so no DB reset and no `cargo sqlx prepare --check`. `cargo run -p seeds --bin seed` reseeded `wotr-planner-postgres` on 5434 only: `feat_extra_pick` 15 rows (+2).
- `cargo leptos build` and `--release`: skipped, as asked.
- The v1 link vector is unchanged (no `Build` or format change).
- `apps/backend/src/main.rs`: the user's edit is byte-for-byte as it was (its diff equals `t64/main-rs-user-edit.diff`).

### Questions decided by recommendation

1. **Do the prerequisites reach the archetypes?**
   - **What was there:** in the data, "Prerequisites: Dicovery" is "holds an alchemist discovery" (or the Underground Chemist's Discovery feature), and "Prerequisites: Magus Arcana" is "holds a magus arcana". A Vivisectionist whose Medical Discovery holds only rogue talents, or a Hexcrafter whose Hex Arcana holds only hexes, could not take the feat, though both have the class feature in their version.
   - **Recommendation, built:** each `Any` gains the archetype from its first slot (Vivisectionist from alchemist 2, Hexcrafter from magus 6), whatever it picked there. This matches how ticket 60 let a Sanctified Slayer 8 meet Extra Slayer Talent and how the Underground Chemist's feature meets Extra Discovery.
   - **Unchanged:** the plain alchemist's and magus's encoding (an option held), so one with an empty slot is still locked.
2. **Which slot when a character has both selections?** Already settled by ticket 49's decision 2: the one the character had a slot of first. Neither archetype has the class's own slot, so a single-class character always gets its own. Only a multiclass Underground Chemist and Vivisectionist could have both.

None needs the user.
