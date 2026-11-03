# Archetype bonus-feat choices become feat slots

Type: task
Status: resolved
Blocked by: 60
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 54](54-mercies-and-enforcer-exploits.md)'s decision 2 (user, 2026-11-02): some archetypes choose a bonus feat from a fixed list, but the data holds a feature with no slot. Make each a bonus feat slot from its list (`OneOf` feat lists, the shape of the Divine Guardian's bonus feats), with prerequisites as the guide says (ignored or checked):

- the Titan Fighter's Massive Attacks: a feat from the ranger's two-weapon list, at 2 and every 4 levels;
- the Urban Hunter's Captor: a bonus feat at 6 and every 3 levels;
- the Hunter's Precise Companion: Precise Shot or Outflank;
- the Stormwalker's Archery: the ranger's archery style feats.

## Build

AFK, test-first (`tdd`); the only agent in the repo; schema change and DB reset allowed per `CLAUDE.md` if the types need one; `V1_LINK` regenerated with a comment if the link format changes. Same method as tickets 45, 54 and 55: an extractor module asserting the guide's sentences, a one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, and a before/after proof in which only Builds that have one of the listed archetypes differ. Ticket 48's picker shortcut must stay exact. Engine code reaches game data through ticket 56's type.

Data from the saved guide pages in `guide-cache/` only. If the guide doesn't settle something, write a question in this ticket with a recommendation and go on.

**Done when:** every check from [ticket 53](53-rename-packages.md)'s Done when passes, plus real-data tests: each listed archetype has its bonus feat slots at its table's levels, offering exactly its list; a feat already held is not offered again unless it repeats; prerequisites are checked or ignored as the guide says; the sheet labels the new slots by name (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

- 2026-11-03: started. Baseline: HEAD 6cddcd5 (the user's uncommitted doc-comment line in `apps/backend/src/main.rs` saved as `t61/main-rs-user-edit.diff`, identical to ticket 60's, and left alone). Scratch in `t61/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t61/`): `old-src/` is `git archive HEAD`, `extract-old/` HEAD's extraction (23 report items; byte-identical to ticket 60's fresh extraction). Read `fighter#massive-attacks`, `fighter#titan-fighter`, `hunter#precise-companion`, `hunter#captor`, `ranger#combat-style-feat` and its five styles, `ranger#archery`, `ranger#stormwalker`, `ranger#nomad`, `paladin#divine-guardian-bonus-feat`, and every hunter, fighter and ranger table's "Massive Attacks", "Precise Companion", "Captor", "Archery" and "Archery Feat" cells. Found: only Massive Attacks lacks a slot in `libs/game-data/`. Precise Companion (`hunter/bonus-feats-2`), Captor (`hunter/urban-hunter/bonus-feats-6`, `-12`) and the Stormwalker's Archery (`ranger/bonus-feats-2b`, `-6b`, `-10b`) are ticket 18's hand fixes there, each slot sitting beside a feature of the same name. In an extraction all four are the feature alone, which is what ticket 54 saw.
- 2026-11-03: real-data tests first (`game_data.rs`, 7 new; ticket 27's feat-list count updated). Red on HEAD's data (`t61/real-data-red.txt`): 195 pass, 6 fail, each at the missing Titan Fighter lists, slots or labels, or at a feature still there. The hunters' and the Stormwalker's offer test passes on HEAD, as a guard on the existing slots.
- 2026-11-03: no `rules`, schema, `Build` or codec change was needed (`OneOf` lists ignoring prerequisites already exist). `git diff HEAD -- libs/rules libs/seeds/src libs/seeds/migrations libs/ui apps/frontend` is empty.
- 2026-11-03: data and extractor. New `extract::bonus_feat_choices` (`apply`, `style_feats`), run after `borrowed_selections::apply`. It handles both inputs: it adds the lists and slots where they are missing (an extraction), and asserts them where they exist (`libs/game-data/`). `t61/rewrite/` wrote `libs/game-data/` byte-identical to its dry run: 9 changes in 3 files (`t61/game-data-log.json`), 0 check problems. A second run refuses ("on no row"). Real-data tests green (201, `t61/real-data-green.txt`). A fresh extraction (`t61/extract-new/`) is byte-identical, report included (23 items), to HEAD's extraction rewritten by `t61/rewrite/` (`t61/dry/extract-old-rewritten/`: 12 changes, the lists and slots added too). Every changed line in `libs/game-data/` is among the extraction's changed lines (`t61/lines-game-data.txt`, `t61/lines-extract.txt`). The extraction's extra lines are the hunter's, Urban Hunter's and Stormwalker's lists, byte for byte the hand-fixed lines `libs/game-data/` already held. `extract/tests/extract/bonus_feat_choices.rs`: 1 test.
- 2026-11-03: proof, UI, docs, DB, checks. Proof: `t61/proof/` (ticket 60's program plus ticket 61's Builds; `t61/patch_proof.py`), `t61/compare.py`, `t61/compare.txt`. UI: `cargo leptos watch` on 3900, headless Chromium (`t61/links/`, `t61/ui-check.js`, `t61/ui-check.txt`, `t61/ui-*.png`), server stopped. Docs: `libs/game-data/README.md`, the map. Reseeded (no schema change). Checks below (`t61/verify.txt`, `t61/test-workspace.txt`).

## Answer

Done 2026-11-03, uncommitted. Ticket 54's decision 2 for bonus-feat choices is built. Each of the four is a `OneOf` slot of its fixed lists where its table has the cell, the shape of the Divine Guardian's bonus feats, with no feature standing for it. A data change only: no `rules`, schema, `Build` or link change, so `V1_LINK` stays.

### Data

`extract::bonus_feat_choices` asserts each guide sentence, each table's cell levels, the lists against the guide, and that nothing needs the features, before changing anything.

| Choice | Guide | Slots at | Lists (all ignore prerequisites) | Before |
|---|---|---|---|---|
| Titan Fighter, Massive Attacks | "selects a feat from the Ranger's Two-Weapon Combat Style feats" | 2, 6, 10, 14, 18 (in place of the fighter's Bonus Combat Feat) | new `fighter/titan-fighter/bonus-feats-2` (Double Slice, Shield Bash, Two-Weapon Fighting), `-6` (+ Improved Two-Weapon Fighting), `-10` (+ Greater Two-Weapon Fighting); "Massive Attacks", "Massive Attacks (from level 6)", "… (from level 10)" | feature only |
| Hunter, Precise Companion | "chooses either Precise Shot or Outflank … does not need to meet the prerequisites" | 2 (hunter and every archetype but the Forester) | `hunter/bonus-feats-2` | slot and feature |
| Urban Hunter, Captor | four feats at 6, six more from 12; "Neither she nor her companion need to meet the prerequisites" | 6, 9, 12, 15, 18 | `hunter/urban-hunter/bonus-feats-6`, `-12` | slot and feature |
| Stormwalker, Archery | "May select from the Ranger's list of Archery combat style feats" | 2, 6, 10, 14, 18 | `ranger/bonus-feats-2b`, `-6b`, `-10b` (the Nomad's lists too) | slot and feature |

The ranger styles' feats are read from `ranger#two-weapon-combat-feats` and `#archery-feats` (`style_feats`). Each is asserted to be exactly the data's style list less one set of extras per level, the same set for both styles. The four features (`fighter/titan-fighter/massive-attacks`, `hunter/precise-companion`, `hunter/urban-hunter/captor`, `ranger/stormwalker/archery`) go from their owners and from every row; nothing needed them, and the sheet never showed them.

`t61/rewrite/` (in `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t61`) ran `bonus_feat_choices::apply` over `libs/game-data/`: 9 changes in `classes/fighter.ron`, `hunter.ron` and `ranger.ron`, byte-identical to its dry run, 0 check problems. The extractor was never run over `libs/game-data/`.

### Rules, schema, codec, UI

No code change was needed. The sheet labels each slot with its list's name ("Massive Attacks", "Precise Companion", "Captor", "Archery Feats", each "(from level N)" past its first list). Ticket 48's shortcut needs no change and stays exact (Proof below).

### Tests

Real-data tests (`game_data.rs`, 7 new; ticket 27's feat-list count is 78, up from 75; red on HEAD's data first):

- **The features are gone:** none of the four is defined or on a row.
- **Slots at the tables' levels:** the rows in force hold each choice's slots at its levels with the right list, and nowhere else. The Titan Fighter's combat feat slots are at 1, 4, 8, 12, 16 and 20; no other fighter has Massive Attacks; the Forester has no Precise Companion.
- **The lists are the guide's:** the feats, names, guide link and `ignores_prerequisites` of each list. The Titan Fighter's and the Stormwalker's lists are the ranger's two-weapon and archery lists less the extras.
- **Titan Fighter:** each slot offers exactly its list, every feat open (Two-Weapon Fighting without Dexterity 15, Greater Two-Weapon Fighting without Improved), and a filled Titan Fighter 18 is possible. Greater Two-Weapon Fighting at 6 is impossible.
- **Held feats:** Two-Weapon Fighting taken at 2 is "Already taken" at 6. Precise Shot taken through Precise Companion is "Already taken" in the hunter's teamwork slot at 3. Improved Critical repeats per weapon, so a Stormwalker takes it for the long bow at 10 and the longsword at 14, but the long bow twice is "Already taken".
- **Hunter, Urban Hunter and Stormwalker:** each slot offers exactly its list, every option open.
- **Labels:** the sheet's labels above, and "Combat feat" at the Titan Fighter's 4.

Extractor: `bonus_feat_choices.rs`, 1 test of `style_feats` (levels, names growing, a note after the list passed over).

### Proof

Ticket 61 changes no rules, so one program (`t61/proof-run`, the working tree's rules) runs on HEAD's data (A) and ticket 61's (C). It is ticket 60's proof plus ticket 61's Builds, 1,680 Builds in A and 1,684 in C counting their full versions and switches:

- a Titan Fighter, a hunter and each of its 7 archetypes, and a Stormwalker at 20, plus a fighter, a ranger and a Nomad as controls, each empty and filled;
- a ranger 2 / Titan Fighter 6, a hunter 3 / Titan Fighter 2 and a Stormwalker 6 / Urban Hunter 6;
- six with fixed picks;
- five that no sheet holds.

`t61/compare.py` (`t61/compare.txt`): 0 problems.

- **A checks the program:** stripped of the added fields, A equals ticket 60's proof output on all 1,576 Builds.
- **Builds with no hunter, Titan Fighter or Stormwalker:** all byte-identical, class pickers included. So are the fighter, ranger and Nomad controls.
- **The 107 that differ** all take a hunter, a Titan Fighter or a Stormwalker (76 bases). Their sheets differ by the features gone. Where the picks are the same and the Build is possible alike, every slot both have answers the same. All 26 new slots are Massive Attacks slots (10 of `-2`, 7 of `-6`, 9 of `-10`); no hunter or Stormwalker Build gains or loses a slot.
- **Fixed picks:**
  - The Titan Fighter's three (its five Massive Attacks; Greater Two-Weapon Fighting at 10 without Improved; a ranger 2's two-weapon style and Massive Attacks with Two-Weapon Fighting at fighter 4) are impossible in A and possible in C.
  - The hunter's Outflank, the Urban Hunter's Captor picks and the Stormwalker's archery picks are possible in both.
  - The five meant to be impossible are impossible in both: Greater Two-Weapon Fighting at 6, Two-Weapon Fighting twice, Power Attack in Massive Attacks, Precise Shot twice, Improved Critical (long bow) twice. In "Two-Weapon Fighting twice" three earlier pickers keep 0 open of the same options but give other reasons: the walk now fails at the second pick, not at a pick with no slot.
- **Ticket 48's shortcut stays exact:** on ticket 55's to 61's Builds (and their full versions), each of 934,376 feat options a feat slot accepts is open exactly when `allows` keeps the Build with it picked. On HEAD's data the count is 924,895, also exact.
- Run time: 189 s on A, 193 s on C (8 threads).

### UI

`cargo leptos watch` on 3900, headless Chromium at 412×920 (`t61/ui-check.js`, `t61/ui-check.txt`):

- A Titan Fighter 10's link: "MASSIVE ATTACKS Two-Weapon Fighting" at 2, "MASSIVE ATTACKS (FROM LEVEL 6) Improved Two-Weapon Fighting" at 6, "MASSIVE ATTACKS (FROM LEVEL 10) Greater Two-Weapon Fighting" at 10, and "COMBAT FEAT +" at 4 and 8.
- A hunter 2: "PRECISE COMPANION Outflank".
- An Urban Hunter 12: "CAPTOR Trip" at 6, "CAPTOR +" at 9, "CAPTOR (FROM LEVEL 12) Greater Trip" at 12.
- A Stormwalker 10: "ARCHERY FEATS Precise Shot" at 2, "ARCHERY FEATS (FROM LEVEL 6) Manyshot" at 6, "ARCHERY FEATS (FROM LEVEL 10) +" at 10.
- A Titan Fighter 6's empty "MASSIVE ATTACKS (FROM LEVEL 6) +" opens "Massive Attacks (from level 6) · Level 6" with exactly Double Slice, Improved Two-Weapon Fighting, Shield Bash and Two-Weapon Fighting, all open (no "Show locked").
- A Stormwalker 6's picker shows its six archery feats, all open.
- No stale-link error and no console errors. Server stopped.

### Docs

- `libs/game-data/README.md`: ticket 61's paragraph.
- `GLOSSARY.md`: unchanged.
- The map: the Decisions-so-far line.

### Checks

(`t61/verify.txt`, `t61/test-workspace.txt`)

- `cargo fmt --all --check`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean. `cargo clippy -p frontend --target wasm32-unknown-unknown -- -D warnings`: clean.
- `cargo test --workspace`: pass (extract 101, rules 338 + unit 4, seeds files 25 + game_data 201 + postgres 28, server 4).
- `cargo test --workspace -- --list` (`t61/test-list.txt`, 701) differs from ticket 60's (693) only by the 8 new tests.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `cargo run -p seeds --bin seed -- --check-only`: pass, unchanged counts: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths, 38 backgrounds.
- No schema change, so no DB reset and no `cargo sqlx prepare --check`. `cargo run -p seeds --bin seed` reseeded `wotr-planner-postgres` on 5434 only. Rows: `feature` 1909 (−4), `feat_list` 78 (+3), `feat_list_member` 641 (+12), `feat_slot` 292 (+5).
- `cargo leptos build` and `--release`: skipped, as asked. `cargo leptos watch` serves the sheet (UI above).
- The v1 link vector is unchanged (no `Build` or format change).
- Proof: passes. Browser: passes.
- `apps/backend/src/main.rs`: the user's edit is byte-for-byte as it was (its diff equals `t61/main-rs-user-edit.diff`).

### Questions for the user

1. **Massive Attacks ignores prerequisites.** The Titan Fighter's section doesn't say. It is built as the ranger's list it names ("does not need to meet the prerequisites"), so Greater Two-Weapon Fighting is open at 10 without Improved, as the guide's ranger note allows. The Stormwalker's archery list is read the same way. **Recommendation:** keep it. The alternative is lists that check prerequisites, which would close Two-Weapon Fighting to a Titan Fighter with Dexterity below 15.
2. **Massive Attacks offers the style's feats only.** Built as "Two-Weapon Combat Style feats": Double Slice, Shield Bash and Two-Weapon Fighting, plus Improved from 6 and Greater from 10, counted by fighter level. It leaves out the five feats the ranger may take "Besides the style feats" (Dodge, Great Fortitude, Iron Will, Lightning Reflexes, Toughness) and Skill Focus from 10. The Stormwalker's and the Nomad's archery lists are read the same way. **Recommendation:** keep it. The other reading would give the Titan Fighter the ranger's full lists (`ranger/combat-style-feat/two-weapon-combat-feats-*`).
3. **Labels past the first list read "(from level N)"**, such as "Massive Attacks (from level 10)" at 14 and 18. That is the list's name, as for Captor and the Stormwalker before. **Recommendation:** keep it; the label says which list the slot offers.

## Decisions on the questions

**User, 2026-11-02:** take every recommendation without asking (overnight run). All three kept as built: Massive Attacks ignores prerequisites, offers the style feats only, and its later lists read "(from level N)".
