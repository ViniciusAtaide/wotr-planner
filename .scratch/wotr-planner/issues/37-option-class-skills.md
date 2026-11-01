# Class skills from options

Type: task
Status: resolved
Blocked by: 34
Map: [WotR Build Planner](../map.md)

## Question

Build [ticket 34](34-backgrounds.md)'s decision 1 (user, 2026-11-01): the "Class Skill: …" features that options grant (oracle mysteries, kineticist elements, domains, bloodlines and any others in the data) become real class skills of the option, so a character holding the option has them as class skills. Tickets 30 (open question 8) and 31 (open question 9) left them as features only because nothing used class skills then.

A feature that offers a *choice* of class skill (the Arcane bloodline's) is a choice, not a fixed skill: if one is found, stop and write the question here rather than guessing.

Data from the saved guide pages only (offline). If the guide doesn't settle something, stop and write the question here.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB). **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: an oracle with a mystery holds that mystery's class skills, and a kineticist holds its element's; a character without the option doesn't; the "Class Skill: …" features are gone from the data.

## Progress log

- 2026-11-01: started. Baseline: HEAD 71598d9 (only `map.md` modified, not mine); scratch in `t37/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t37/`), `old-src/` is `git archive HEAD`. Survey of the data and the saved pages:
  - **Fixed skills on option rows** (all at option level 1): "Class Skill: X" features on 20 sorcerer bloodlines' rows (all but Arcane), 7 mysteries, 3 domains (Animal, Knowledge, Trickery) and 4 elements; and the cavalier orders' "Skills" features (Lion: Knowledge (World); Shroud, Star, Sword: Lore (Religion); the guide: "The Cavalier adds … to her list of class skills"). Each guide sentence names the skill.
  - **Archetype "Class Skill: X" features** (Empyreal: Lore (Religion); Sylvan: Lore (Nature)): their archetypes already replace the class skills with a list holding that skill, so the feature only repeats it. Same for the Sohei's "Skills" (Lore (Nature), in its list).
  - **Four features named just "Class Skill"**, which the guide doesn't settle: the Arcane bloodline's and the Sage Sorcerer's ("Class Skill: one of Knowledge (World), Lore (Nature), Lore (Religion)": a choice), the Overwhelming Mage's and the Seeker's (their tables say only "Class Skill"; each also holds the chosen bloodline's 1st-level row). Per the ticket these stay features and become questions.
  - Shape decided: an option row's class skills (`OptionLevelRow.class_skills`), held with the row, so everything that governs rows governs them (an archetype skipping the row, Divine Herbalist: "he does not gain additional class skills from his choice of mystery"; Crossblooded's second bloodline, which gets only its lists and arcana; a Primalist's replaced row; a Path switch). Schema: `class_skill` gains a third owner, `option_level_id`. Milestones: (1) `rules`, test world first; (2) schema, seed, loader, fixture; (3) data: new `extract::option_class_skills` (asserted against the guide), applied to `game-data/` by a one-off `t37/rewrite/`, real-data tests first; (4) extractor wiring, fresh extraction; (5) proof, docs, DB reset, Verification.
- 2026-11-01: milestone 1 (`rules`) done. `OptionLevelRow.class_skills` (serde default). Engine: a new `Held::ClassSkill`, held like a row's features and feats (so skipping, lists-only holdings, a replaced row and a Path switch govern it); the walker counts each skill's sources (`class_skill_holds`), so a skill taken back from a row stays while a class or the background gives it. Classes' and the background's class skills go through the same hold. Tests first: the world's Abyssal row 1 gains Athletics and the Draconic rows Persuasion (invented; the real one is Perception); new `option_class_skills.rs` (7). Red run (`t37/rules-red.txt`): 6 failures; "a character without the option lacks its class skills" passed at once. `cargo test -p rules` 281 + 1, clippy clean. No `Build` change, so the v1 link vector stays. Next: schema, seed, loader, fixture.
- 2026-11-01: milestone 2 (`data`) done. Migration: `class_skill` gains a third owner, `option_level_id` (CHECK exactly one of class, background, option level; unique per owner and skill), and moves below `option_level`, which it now references. Still 41 tables. Seed (an option row's skills as `class_skill` rows), loader (`load_class_skills` first, rows by owner including `Owner::OptionLevel`, taken by `load_selections`). Fixture: the Abyssal row 1 gains Athletics, and the Red Draconic bloodline a row at 9 holding only Mobility and Persuasion. Tests first: files (1 updated), postgres (a CHECK/FK/unique test of 5 statements). Red run with the old seed and loader (`t37/data-red.txt`): 8 postgres failures (the round trips and the new schema test); the files test passed at once (serde's, done with the field). Dev DB reset with `t34/reset-db.sh --no-seed`. `cargo test -p data --test files --test postgres` 25 + 25 pass. Next: the data.
- 2026-11-01: milestone 3 (data) done. Survey correction: the Sohei has no list of its own (it keeps the monk's, which lacks Lore (Nature)), so its "Skills" feature isn't a repeat; it stays, with the four "Class Skill" features, as an open question. One implementation: new `extract::option_class_skills::apply(guide, data)`, a hand table of 28 guide statements (37 features) (the sorcerer bloodline tables' level-0 cells, the mysteries', domains', elements' and orders' sentences, the Empyreal and Sylvan Sorcerer lines), each asserted against the saved pages; it moves each feature off every option row granting it onto the row's `class_skills`, drops the two archetype features after checking their lists hold the skill, removes the definitions, and asserts what's left (the four "Class Skill" and the Sohei's "Skills"). `ron_out` writes a row's class skills last, left out when empty. The one-off `t37/rewrite/` (Rust, from `t35/rewrite/`) ran it over `game-data/` and wrote only the 5 files whose data changed (cavalier, cleric, kineticist, oracle, sorcerer; 37 changes, `t37/game-data-log.json`; 0 check problems), byte-identical to its dry run in `t37/dry/game-data/`. Real-data tests first (`game_data.rs`, 7 new; red on HEAD's data: 7 failures, `t37/real-data-red.txt`): an oracle with each mystery (and a Divine Herbalist with none), a kineticist with each element, a cavalier with each order, a cleric's two domains, a sorcerer's bloodlines, every option row's class skills against the guide's list, and no feature left standing for a fixed skill. 4 older tests read class skills instead of the gone features (the Blight Druid's Animal domain, the Battle mystery and Divine Herbalist, the Earth element, the Living Deity's second mystery). The extract test of the fixture's file text gains the row's class skills.
- 2026-11-01: milestone 4 (extractor) done: `extract` runs `option_class_skills::apply` after the backgrounds. A fresh extraction (`t37/extract-new/`) differs from ticket 34's (`t34/extract-new/`) only in those 5 class files and is byte-identical to ticket 34's extraction rewritten by `t37/rewrite/` (`t37/dry/extract-old-rewritten/`, the same 37 changes); each file's change from ticket 34's extraction is the same diff, line for line, as `game-data/`'s from HEAD. Report unchanged (33 items); 0 check problems. Tests: new `extract/tests/extract/option_class_skills.rs` (2, the files' text), written after the code (both passed at once). `cargo test -p extract` 88 pass. Next: proof, docs, DB reset, Verification.
- 2026-11-01: milestone 5 done. Proof (`t37/proof-{old,new}` on one shared `t37/proof-common.rs`, ticket 34's plus option Builds on both sides: every bloodline, mystery, element and order at its class's levels 1-3, three domain pairs, an inquisitor's Trickery domain, a Divine Herbalist with Flame and with Battle, Crossblooded Abyssal + Celestial, Sage, Empyreal, Sylvan, an Overwhelming Mage and a Seeker, a sorcerer's Second Bloodline (Undead) at MR 1; `t37/compare.py`, 62164 checks) passes. Docs: `game-data/README.md` (a row's `class_skills`; the ticket's paragraph), `GLOSSARY.md` (Class skill). Final DB reset and reseed (`t37/reset-db.sh`), `cargo sqlx prepare` (2 query files new, 1 gone). Verification below.

## Answer

Done 2026-11-01, uncommitted. The user's decision is built for every option whose class-skill feature names its skills. An option row's skills are now class skills while the row is held (`OptionLevelRow.class_skills`), so an oracle with the Flame mystery holds Athletics and Mobility, and a kineticist with Earth holds Athletics and Knowledge (World). A character without the option doesn't. The "Class Skill: …" features are gone from the data: those of 20 sorcerer bloodlines, 7 mysteries, 3 domains and 4 elements, and the cavalier orders' "Skills" (the guide: "The Cavalier adds … to her list of class skills"), all on option level 1. The Empyreal and Sylvan Sorcerers' went too, since their own lists already hold the skill. Five class-skill features stay, because the guide doesn't settle their skill: they are open questions 1-3 below. The `Build` type didn't change, so the v1 link vector stays. Every Build walks as before except for the class skills it now holds.

### Rust (`rules`)

- **Option rows.** `OptionLevelRow.class_skills: BTreeSet<SkillKey>` (serde default).
- **Engine.** A row's class skills are held with the row (`Held::ClassSkill`), as its features and feats are. So whatever governs the row governs them too:
  - an option taken late gets them from the pick (Second Bloodline);
  - an archetype skipping the row gets none (a Divine Herbalist: "he does not gain additional class skills from his choice of mystery");
  - a lists-only holding gets none (Crossblooded's second bloodline);
  - a replaced row (Primalist) or a Path switch takes them back.
- **Counting.** The walk counts each skill's sources, classes' and the background's included. A skill taken back stays while another source gives it.
- `CharacterState.class_skills` is now the classes', the background's and the option rows held.
- **Tests first** (new `option_class_skills.rs`, 7). The world's Abyssal row 1 gains Athletics, and its Draconic rows gain an invented Persuasion. Red run: 6 failures (`t37/rules-red.txt`). "A character without the option lacks its class skills" passed at once.

### Schema (the one migration)

- `class_skill` gains a third owner, `option_level_id`. A CHECK requires exactly one of class, background and option level, and each skill is unique per owner. The table moves below `option_level`, which it now references. Still 41 tables.
- Seed: an option row's skills become `class_skill` rows. Loader: `load_class_skills` runs first, and `load_selections` takes the option levels' rows.
- Fixture: the Abyssal row 1 gains Athletics, and the Red Draconic bloodline gains a row at 9 holding only Mobility and Persuasion.
- Tests: files (1 updated), postgres (a CHECK/FK/unique test of 5 statements). Red run with the old seed and loader: 8 postgres failures (`t37/data-red.txt`).

### Data

One implementation, `extract::option_class_skills`, holds a hand table of 28 guide statements covering 37 features. Each statement is asserted against the saved pages: the bloodline tables' level-0 cells ("Class Skill: Athletics"), the mysteries' "adds … to the list of class skills", the domains' and elements' lines, the orders' sentences, and the Empyreal and Sylvan Sorcerers' "Class Skill: …".

- **Option features:** the module takes each one off every option row that grants it and onto the row's `class_skills`.
- **Archetype features:** it drops the two archetype features after checking that the archetype's own list holds the skill.
- **Definitions:** it removes them, and asserts that only the five open ones are left.
- **Writing:** `ron_out` writes a row's class skills last, and leaves them out when empty.

The one-off `t37/rewrite/` (Rust, in the scratchpad `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t37/`) ran it over `game-data/`. It wrote only the five files whose data changed (cavalier, cleric, kineticist, oracle, sorcerer; 37 changes, `t37/game-data-log.json`). The result is byte-identical to its dry run in `t37/dry/game-data/` and has 0 check problems. The extractor was never run over `game-data/`.

Real-data tests (`game_data.rs`, 7 new, red on HEAD's data first: 7 failures, `t37/real-data-red.txt`):

- an oracle with each of the 9 mysteries (and a Divine Herbalist with Flame, who gets none);
- a kineticist with each element;
- a cavalier with each order;
- a cleric with the Knowledge and Trickery domains;
- a sorcerer with each kind of bloodline;
- every option row's class skills against the guide's list (38 rows, all at option level 1);
- no feature left standing for a fixed skill, while Empyreal and Sylvan still hold theirs.

4 older tests now read class skills instead of the gone features.

### Proof

`t37/proof-old/` (against `git archive HEAD`, 71598d9) and `t37/proof-new/` build one shared source, `t37/proof-common.rs`. It is ticket 34's, plus option Builds on both sides:

- every bloodline, mystery, element and order through its class;
- three domain pairs and an inquisitor's Trickery domain;
- a Divine Herbalist with Flame and with Battle;
- Crossblooded, Sage, Empyreal, Sylvan, Overwhelming Mage and Seeker Sorcerers;
- a sorcerer's Second Bloodline at MR 1.

`t37/compare.py` checks (62164 checks pass):

- **Data:** the old data, with each gone feature moved onto its option rows' class skills (its skill read from its name, or from the guide's sentence for an order) and off the two archetypes, is exactly the new data.
- **Walk:** all 5921 lines are identical in both walks except in two fields. Features lose the gone ones. Class skills gain exactly the skills of the gone option features the old line held: 678 lines in 72 Builds.
  - Each option Build gains its option's skills (Abyssal: Athletics; Flame: Athletics and Mobility; Knowledge + Animal: Lore (Nature), the rest a cleric's already).
  - These gain nothing: Divine Herbalist, Arcane, Sage, Empyreal, Sylvan, inquisitor, Scalykind + Law, Ancestor, Life and the Order of the Cockatrice.

### Extractor

- `extract` runs `option_class_skills::apply` after the backgrounds. It settles no report item.
- A fresh extraction (`t37/extract-new/`) differs from ticket 34's (`t34/extract-new/`) only in the five class files. It is byte-identical to ticket 34's extraction rewritten by `t37/rewrite/` (the same 37 changes), and each file's change is the same diff, line for line, as `game-data/`'s. The report is unchanged (33 items).
- Tests: `extract/tests/extract/option_class_skills.rs` (2, the files' text), written after the code. The fixture file-text test gains the row's class skills.

### Docs

- `game-data/README.md`: a row's `class_skills`, and the ticket's paragraph.
- `GLOSSARY.md`: **Class skill** now counts option rows.
- `game-data/EXTRACTION_REPORT.md`: unchanged.

### Verification

- `cargo test --workspace`: pass (rules 281 + unit 1, extract 88, data files 25 + game_data 116 + postgres 25).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass (2 query files new, 1 gone).
- Dev DB reset per `CLAUDE.md`, only `wotr-planner-postgres` on 5434 (`t37/reset-db.sh`): drop/create `wotr_planner`, `sqlx migrate run --source data/migrations`, `cargo run -p data --bin seed`, `cargo sqlx prepare`.
- `--check-only` and the real seed both give unchanged counts: 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths and 39 backgrounds.
- Rows: 41 tables. `class_skill` has 372 class rows and 55 background rows (both unchanged) and 47 option-level rows on 38 option levels. `feature` has 1925 rows (37 fewer).
- The v1 link vector is unchanged (no `Build` change).
- Proof: passes (above).

### Open questions (for the user)

1. **The Arcane bloodline's and the Sage Sorcerer's class skill.** The guide says "Class Skill: one of Knowledge (World), Lore (Nature), Lore (Religion)", which is a choice. Both stay features named "Class Skill". Recommendation: a pick. The bloodline's row 1 would grant a three-option selection levelled by the bloodline (`level_from: GrantingOption`), each option's row 1 holding its skill. The Sage's own row would grant a selection sharing those options, levelled by sorcerer. Neither the types nor the `Build` would change. The alternative is to leave them as features shown on the sheet.
2. **The Overwhelming Mage's and the Seeker's "Class Skill".** Their tables say only "Class Skill". Each takes a standard bloodline and holds its 1st-level row, so each already holds that bloodline's class skill (an Overwhelming Mage with Fey: Lore (Nature)). Recommendation: drop both features as that bloodline's.
3. **The Sohei's "Skills"** ("A Sohei gains Lore (Nature) as a class skill"). The Sohei keeps the monk's list, which lacks Lore (Nature), so the feature isn't a repeat as the Empyreal's was. Recommendation: give the Sohei its own list (the monk's plus Lore (Nature)) and drop the feature.
4. **Class skills the guide gives outside these features**, which are still not modelled:
   - the rogue talents Mystery Seeker (Knowledge (Arcana)) and Survivalist (Lore (Nature)), plain options with no rows;
   - the slayer talent Trapfinding (Trickery);
   - the Seeker Sorcerer's Trickery (Seeker's Trapfinding; it keeps the sorcerer's list);
   - the Shadow Shaman's Stealth (it keeps the shaman's list);
   - the Student of War's Additional Class Skill at odd levels (a choice each time);
   - the Gold Dragon's MR 9 "All skills are class skills".

   Should any of these count? The first four are fixed skills: an option's own `class_skills` for the talents, and an own list for each archetype.
5. **The Animal domain's level.** Its guide sentence puts "Lore (Nature) becomes a Class Skill" after "At 4th level, you gain an Animal Companion". Ticket 31 read the skill as option level 1, and this ticket kept that row. Should it be level 4?

## Decisions on the open questions

**User, 2026-11-01:** "for skills always go for the recommended option without asking me", so each question takes the recommendation in the report to the user (which, for 1, differs from the agent's above).

1. **Arcane bloodline and Sage Sorcerer:** leave both as features named "Class Skill", shown on the sheet. Class skills are a nice-to-have, and an extra pick isn't worth it.
2. **Overwhelming Mage and Seeker:** drop both "Class Skill" features; each already holds its bloodline's skill. Built by [ticket 38](38-class-skill-cleanup.md).
3. **Sohei:** its own class-skill list, the monk's plus Lore (Nature), and drop the "Skills" feature. Built by ticket 38.
4. **Class skills outside these features:** leave them unmodelled for now.
5. **Animal domain:** its Lore (Nature) moves to option level 4, following the guide's sentence order. Built by ticket 38.
