# Class-skill cleanup

Type: task
Status: resolved
Blocked by: 37
Map: [WotR Build Planner](../map.md)

## Question

Build [ticket 37](37-option-class-skills.md)'s decisions 2, 3 and 5 (recommended, per the user's standing rule for skills, 2026-11-01):

- **Overwhelming Mage and Seeker:** drop their "Class Skill" features (and the definitions if nothing else grants them). Each holds its chosen bloodline's 1st-level row, so it already holds that bloodline's class skill.
- **Sohei:** give the archetype its own class-skill list, the monk's plus Lore (Nature), as the Empyreal and Sylvan Sorcerers have, and drop its "Skills" feature. Check how archetype lists are modelled (ticket 37 dropped the Empyreal's and Sylvan's features because their lists already held the skill) and follow that shape.
- **Animal domain:** its Lore (Nature) class skill moves from option level 1 to option level 4, the level the guide's sentence sits under ("At 4th level, you gain an Animal Companion"). Make sure option level 4 has a row (add one holding only the class skill if needed), and that a druid's, inquisitor's or other holder's domain levels give it at the right class level.

Decisions 1 and 4 (the Arcane/Sage choice and the unmodelled sources) stay as they are: no change.

Data from the saved guide pages only (offline). If the guide doesn't settle something, take the option that keeps the data closest to the guide, record it in the Answer, and carry on: skills questions don't go to the user.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB). Put the rules in an extractor module (or extend `extract::option_class_skills`) and apply them to `game-data/` with a one-off scratchpad program, as tickets 30–37 did; a fresh extraction must be byte-identical to the rewrite. **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests:

- an Overwhelming Mage and a Seeker hold their bloodline's class skill, and no "Class Skill" feature;
- a Sohei holds Lore (Nature) as a class skill and has no "Skills" feature; a plain monk lacks it;
- a cleric with the Animal domain lacks Lore (Nature) at cleric 3 and holds it at cleric 4; a druid with a Nature Bond Animal domain likewise by druid level;
- the five open features of ticket 37 are down to two (Arcane, Sage).

## Progress log

- 2026-11-01: started. Baseline: HEAD 5d6a72c, clean tree; scratch in `t38/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t38/`), `old-src/` is `git archive HEAD`. Survey of the data and the saved pages:
  - **Overwhelming Mage and Seeker:** each table has "Class Skill" at level 0 and nothing else about it; each "chooses a standard bloodline" and its row 1 grants the bloodline selection (the Overwhelming Mage `sorcerer/bloodline`, the Seeker its own, which shares the 21 bloodlines and skips 3, 9, 13, 15 and 17, not 1). Every bloodline offered holds a class skill on its row 1 but the Arcane, whose row 1 has its own "Class Skill" (a choice, left as it is).
  - **Sohei:** "A Sohei gains Lore (Nature) as a class skill." (monk#skills). It keeps the monk's list, which lacks it. An archetype's own list is `Archetype.class_skills` (a full list replacing its class's: the Empyreal's, the Sylvan's, the Sensei's, the Tortured Crusader's from `follow_ups`); the schema already holds it (`archetype.overrides_class_skills` and `class_skill` rows with `archetype_key`). So no schema change.
  - **Animal domain:** "At 4th level, you gain an Animal Companion. Your class level - 3 is used to determine your companion's effective level. Lore (Nature) becomes a Class Skill." Its row 4 exists (the companion); its row 1 holds only the class skill, so it goes once the skill moves. The druid, inquisitor, hunter and arcanist lists already hold Lore (Nature), so only clerics (and a Separatist's second domain, two levels behind) see the move in their class skills; the Blight Druid skips row 4.
  - Plan: real-data tests first; `extract::option_class_skills` gains `cleanup` (ticket 38's rules, run after ticket 37's entries by `apply`, and alone by a one-off `t38/rewrite/` on `game-data/`); fresh extraction; proof; docs; DB reset; Verification.
- 2026-11-01: real-data tests first (`game_data.rs`, 3 new, 2 of ticket 37's updated): an Overwhelming Mage and a Seeker with 8 bloodlines hold the bloodline's class skill and no `…/class-skill` feature (and the definitions are gone); a lawful neutral Sohei holds the monk's skills plus Lore (Nature) and no "Skills", a plain monk lacks it; the Animal domain's Lore (Nature) from cleric 4, a Separatist's second domain from cleric 6, and a druid (Nature Bond) and an inquisitor from class level 4 on an engine whose class list lacks Lore (Nature) (both lists hold it, which the test also checks); every option row's class skills against the guide's with the Animal domain's at 4; the class-skill features left are the Arcane's and the Sage's, and the Sohei holds Lore (Nature) through its list. The Blight Druid test's comment and level follow the move (it holds Lore (Nature) from the druid's list). Red run on HEAD's data: 5 failures, each for the reason meant (`t38/real-data-red.txt`).
- 2026-11-01: data. `extract::option_class_skills::cleanup` asserts each guide phrase and the data's shape (the archetype's row 1 grants the bloodline selection, which doesn't skip row 1; each bloodline offered holds a class skill on row 1, or the Arcane its choice; the Sohei had no own list; the Animal domain's row 1 held exactly Lore (Nature) and row 4 none), then drops the three features (definitions and rows), sets the Sohei's list and moves the Animal skill, dropping the emptied row 1. `apply` runs it after the entries, so `check_left` now expects 2 features. `t38/rewrite/` (from `t37/rewrite/`, calling `cleanup`) wrote `game-data/` byte-identical to its dry run (`t38/dry/game-data/`): 4 changes in 3 files (cleric, monk, sorcerer; `t38/game-data-log.json`), 0 check problems. Real-data tests green (119).
- 2026-11-01: extractor. A fresh extraction (`t38/extract-new/`) differs from HEAD's (`t38/extract-old/`, built from `old-src/`, itself identical to `t37/extract-new/`) only in those 3 files, and is byte-identical to HEAD's extraction rewritten by `t38/rewrite/` (`t38/dry/extract-old-rewritten/`, the same 4-line log). Each file's change is the same as `game-data/`'s from HEAD: cleric and sorcerer line for line; monk the same three edits (its Sohei row 1 differs between the two in hand-fixed feat slots, and loses the same "monk/sohei/skills"). Report unchanged (33 items); 0 check problems. `cargo test -p extract` 88 pass.
- 2026-11-01: proof, docs, DB, Verification. Proof (`t38/proof-{old,new}` on one shared `t38/proof-common.rs`: ticket 37's plus `cleanup_builds`; `t38/compare.py`, 76457 checks) passes. Docs: `game-data/README.md` (the domains' row 4, the ticket's paragraph). DB reset and reseed (`t38/reset-db.sh`), `cargo sqlx prepare` (no query file changed). Verification below.

## Answer

Done 2026-11-01, uncommitted. Ticket 37's decisions 2, 3 and 5 are built; 1 and 4 are unchanged. Two class-skill features are left (the Arcane bloodline's and the Sage Sorcerer's "Class Skill", a choice of one of three skills), down from five. No schema, `rules` or `Build` change, so the v1 link vector stays.

### Data

`extract::option_class_skills` gains `cleanup`, which `apply` runs after ticket 37's entries. It asserts each guide statement and the data's shape before changing anything:

- **Overwhelming Mage and Seeker:** their "Class Skill" (the tables' level-0 cell, "chooses a standard bloodline") goes, definition and row 1. Each archetype's row 1 grants its bloodline selection (`sorcerer/bloodline`; `sorcerer/seeker-sorcerer/bloodline`, which skips 3, 9, 13, 15 and 17, not 1), and every bloodline offered holds a class skill on its row 1. An Overwhelming Mage with Fey holds Lore (Nature); with the Arcane, the Arcane's own "Class Skill" choice, as before.
- **Sohei:** its own list (`Archetype.class_skills`, the Empyreal's and Sylvan's shape), the monk's 7 skills plus Lore (Nature); its "Skills" goes. A plain monk lacks Lore (Nature).
- **Animal domain:** Lore (Nature) moves from its row 1 to its row 4, with the companion. Row 1 held nothing else, so it goes (528 option rows, was 529). A cleric holds Lore (Nature) from cleric 4, a Separatist's second domain from cleric 6. Every other holder levels the domain by its own class (druid through Nature Bond, inquisitor, Divine Hunter, Living Deity, Pentamic Faith, Impossible Domain), and the druid's, inquisitor's, hunter's and arcanist's lists hold Lore (Nature) anyway.

`t38/rewrite/` (in the scratchpad `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t38/`) ran `cleanup` over `game-data/`: 4 changes in 3 files (cleric, monk, sorcerer), byte-identical to its dry run, 0 check problems. The extractor was never run over `game-data/`.

Real-data tests (`game_data.rs`, 3 new and 2 updated, red on HEAD's data first: 5 failures, `t38/real-data-red.txt`):

- an Overwhelming Mage and a Seeker with 8 bloodlines: the bloodline's class skill, no "Class Skill" feature held or defined;
- a Sohei holds Lore (Nature) and no "Skills"; a plain monk doesn't;
- the Animal domain: a cleric lacks Lore (Nature) at 3 and holds it at 4; a Separatist's second domain at 5 and 6; a druid (Nature Bond) and an inquisitor by class level 3 and 4, on the real data less their list's Lore (Nature);
- every option row's class skills against the guide's (38 rows, the Animal domain's at 4, the rest at 1);
- the class-skill features left are the Arcane's and the Sage's; the Sohei, Empyreal and Sylvan hold their skill through their lists.

### Extractor

A fresh extraction (`t38/extract-new/`) differs from HEAD's only in the three class files and is byte-identical to HEAD's extraction rewritten by `t38/rewrite/` (the same 4 changes). Each file's change is the same as `game-data/`'s; the monk's Sohei row 1 differs between them only in hand-fixed feat slots. Report unchanged (33 items).

### Proof

`t38/proof-old/` (HEAD 5d6a72c) and `t38/proof-new/` build one shared source, `t38/proof-common.rs`: ticket 37's, plus an Overwhelming Mage and a Seeker with each of the 21 bloodlines, a lawful neutral monk and Sohei to 20, and the Animal domain through a cleric (either slot), a Separatist, a druid's Nature Bond, a Blight Druid and an inquisitor. `t38/compare.py` (76457 checks pass):

- **Data:** the old data, with the three features dropped, the Sohei's list set and the Animal domain's skill moved from row 1 (then dropped) to row 4, is exactly the new data.
- **Walk:** all 6175 lines are identical except for the gone features (212 lines) and Lore (Nature) in the class skills. 40 lines gain it, the Sohei's to 20 (2 Builds). 60 lose it, each holding the Animal domain below option level 4 with no class list, background or other option row giving it (20 Builds: clerics at 1-3, Separatists at 3-5, including the class walks that fill a cleric's second domain with the Animal). 12 lines below 4 keep it through another source (druid, Blight Druid and inquisitor lists, the Knowledge domain).

### Docs

- `game-data/README.md`: the domains' class skills (the Animal's on row 4) and the ticket's paragraph.
- `GLOSSARY.md`: unchanged (**Class skill** already covers archetype lists and option rows).

### Decided under the skills rule (the guide doesn't settle it)

1. **The Animal domain's empty row 1 goes** rather than staying as an empty row. A row holding nothing gives nothing, and the guide has nothing for the domain at 1 that the data models.
2. **The Blight Druid** skips the domains' row 4 ("Does not receive an Animal Companion"), so its Animal domain now gives no Lore (Nature). The skill follows the companion's sentence, so it goes with that row. It has no effect: the druid's list holds Lore (Nature).

### Verification

- `cargo test --workspace`: pass (rules 281 + unit 1, extract 88, data files 25 + game_data 119 + postgres 25).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass (no query file changed).
- Dev DB reset per `CLAUDE.md`, only `wotr-planner-postgres` on 5434 (`t38/reset-db.sh`): drop/create `wotr_planner`, `sqlx migrate run --source data/migrations`, `cargo run -p data --bin seed`, `cargo sqlx prepare`.
- `--check-only` and the real seed both give unchanged counts: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths and 39 backgrounds.
- Rows: 41 tables. `class_skill` has 380 class rows (167 of them archetype lists; 8 new, the Sohei's), 55 background rows and 47 option-level rows on 38 option levels (the Animal's now at 4). `feature` has 1922 rows (3 fewer); `option_level` 528 (1 fewer).
- The v1 link vector is unchanged (no `Build` change).
- Proof: passes (above).

### Open questions

None.
