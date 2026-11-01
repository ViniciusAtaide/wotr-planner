# Backgrounds

Type: task
Status: resolved
Blocked by: 36
Map: [WotR Build Planner](../map.md)

## Question

The user decided (2026-11-01) that a background is a level-1 character-creation choice that makes its skills class skills.

The user then gave the full list (2026-11-01): 38 backgrounds in 8 groups plus None, each with its class skills, some with weapon or armour proficiencies, three with a feat, and other bonuses. It's in [research/backgrounds-from-user.md](../research/backgrounds-from-user.md), and it is the source: the saved guide pages don't list backgrounds.

**Decisions** (user, 2026-11-01, "all recommended"):
1. **Weapon proficiencies:** the Aldori Swordsman grants Exotic Weapon Proficiency (Dueling Sword) as a granted feat for that value. The other weapon proficiencies (no feat in the data checks a single simple or martial weapon) and the bucklers are notes on the background.
2. **Light armour:** Bounty Hunter, Gladiator, Guard and Mercenary grant Armor Proficiency (Light Armor).
3. **Feats:** Martial Disciple grants Improved Unarmed Strike, the Arcane Scholar Scribe Scrolls, the Divine Scholar Brew Potions. Granted feats count for prerequisites like any other.
4. **Everything else** (save, AC, damage and skill bonuses, Acolyte's and Healer's ability swaps, the Nature Scholar's orison, the Nomad's companion bonus) is a note on the background, shown in the summary. Nothing else is modelled.

**Build** (AFK, test-first `tdd`; only agent touching the migration and DB): a `backgrounds.ron` from the user's list, the Build start's background choice (regenerate the frozen v1 link vector; no v2), class skills from it, and the schema, seed and loader. **Done when** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a Shackles Corsair holds Athletics and Mobility as class skills; an Aldori Swordsman holds Exotic Weapon Proficiency (Dueling Sword); a Guard can take Armor Proficiency (Medium Armor) at level 1; a Martial Disciple's Improved Unarmed Strike meets Stunning Fist's Improved Unarmed Strike prerequisite; None grants nothing.

## Progress log

- 2026-11-01: started. Baseline: HEAD 17df6d2, clean tree; scratch in `t34/` (`/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t34/`), `old-src/` is `git archive HEAD`. Milestones: (1) `rules`, test world first; (2) schema, seed, loader, fixture; (3) data: a hand table module `extract::backgrounds` (the user's list) written to `game-data/backgrounds.ron` by a one-off `t34/write/`, real-data tests first; (4) extractor wiring and a fresh extraction; (5) proof, docs, DB reset, Verification. Shape decided:
  - **Background** (`game-data/backgrounds.ron`, one file, 39 entries): `key`, `name`, `group: Option<BackgroundGroup>` (Craftsman, Noble, Oblate, Regional, Scholar, Street Urchin, Wanderer, Warrior; None has no group), `note` (everything not modelled, in the user's words), `class_skills`, `feats` and `feats_for` (as on a class level row). No guide link: no saved page lists backgrounds. None is an entry (the game's choice "None", granting nothing); a Build start with no background is a background not yet chosen, as with race.
  - **Build start:** `BuildStart.background: Option<BackgroundKey>` (the v1 link vector is regenerated). The walk holds the background's feats before level 1 (no slot, no pick, prerequisites skipped, as a class row's granted feats) and its skills as class skills. An unknown key is `Choice::Background` / `UnknownKey`.
  - **Class skills:** the engine had none. `CharacterState.class_skills`: the class skills of each class taken (an archetype's list where it replaces its class's) plus the background's. Options' "Class Skill: …" features stay features (not counted here).
- 2026-11-01: milestone 1 (`rules`) done. Types: `BackgroundKey`, `Background`, `BackgroundGroup` (with `ALL`, `name`), `GameData.backgrounds` (canonical by key; `counts` gains "Backgrounds"), `BuildStart.background`, `Choice::Background` (in `Edit::Start`'s choices), `CharacterState.class_skills`, `DataAt::Background`. Engine: the Build start's background (`Walker::background`, after the ability scores): an unknown key is `UnknownKey` at `Choice::Background`; its skills join the class skills, its feats and feats for a value are held with no source (as a class row's). A class's class skills (or its archetype's replacing list) join on first taking it. Check: a background's key once per kind, its `feats_for` values of the feat's kind. Tests first: the world gains five backgrounds (Shackles Corsair, Martial Disciple, Guard, Aldori Swordsman, None); new `backgrounds.rs` (9) and 2 check tests; `canonical.rs` updated (counts, reversal). Red run (`t34/rules-red.txt`): 11 failures (8 new background tests, 2 check tests, the v1 vector); None's test passed at once (nothing to grant). v1 link vector regenerated with a Shackles Corsair background (`version_1_body_layout_is_frozen` gains the background's `None` byte). `cargo test -p rules` 274 + 1, clippy clean. Next: schema, seed, loader, fixture.
- 2026-11-01: milestone 2 (`data`) done. Migration: new table `background` (key, name, `background_group` CHECKed to the eight groups, NULL for None; `note`); `class_skill` gains a second owner `background_key` (`class_key` nullable; CHECK exactly one of class and background, an archetype only with a class; unique per owner and skill); `granted_feat` gains a third owner `background_key` (exactly one owner; value columns allowed on a class level or a background, as before not on an option level). 41 tables. `files.rs`: `backgrounds.ron` is a required single file. Seed (`insert_background`; `SlotOwner::Background`; `insert_granted_feats_for` takes an owner), loader (`load_backgrounds`; class skill and granted feat rows by owner), `columns` (`BackgroundGroup`). Fixture: `backgrounds.ron` with Shackles Corsair (skills, note), Martial Disciple (a feat), Aldori Swordsman (a feat for a value) and None (no group, nothing). Tests first: files (1 new, 1 updated for the count), postgres (every group round trips with skills, a feat and a feat for a value; a CHECK/FK test of 8 statements). Red run with a loader that read no backgrounds and only class-owned skills (`t34/data-red-postgres.txt`): 13 postgres failures (the round trips, and every test seeding the fixture: its background's granted feat named no owner the loader knew); the files tests passed at once (reading the RON is serde's, done with the layout change). 2 tests fixed for the fixture's new rows (a `granted_feat` copy now picks a class level's row; the FK statement updates one skill). `game_data.rs` (the real data) fails until `game-data/backgrounds.ron` exists. Dev DB reset with `t34/reset-db.sh --no-seed` (only `wotr-planner-postgres`, 5434). `cargo test -p data --test files --test postgres` 25 + 24 pass. Next: the data.
- 2026-11-01: milestone 3 (data) done. One implementation: new `extract::backgrounds` (the user's table as it stands, 39 rows; the decisions map light armour to Armor Proficiency (Light Armor), the Aldori Swordsman's dueling sword to Exotic Weapon Proficiency (Dueling Sword) for that value, the feat column to granted feats, and every other proficiency and bonus to the note, "Proficient with … . <bonus>."; skills, feats and the weapon found by name, panicking on one it doesn't find); `ron_out` writes `backgrounds.ron` (group and note left out when unset). The one-off `t34/write/` (Rust) ran it over `game-data/` and wrote only `backgrounds.ron` (dry run in `t34/dry/game-data/`, byte-identical to the real run; 0 check problems). Real-data tests first (`game_data.rs`, 7 new; red on an empty `backgrounds.ron`: 7 failures, `t34/real-data-red.txt`). An independent check (`t34/compare_list.py`, parsing the user's markdown table and the RON) finds 39 rows on each side and no difference in names, groups, class skills, feats, feats for a value or the note's contents. `cargo test --workspace` passes (rules 274 + 1, extract 84, data 25 + 109 + 24). Next: extractor wiring, a fresh extraction.
- 2026-11-01: milestone 4 (extractor) done: `extract` sets `data.backgrounds` from `extract::backgrounds` after `second_follow_ups::apply` (it settles no report item). A fresh extraction (`t34/extract-new/`) differs from ticket 36's (`t36/extract-new/`, itself identical to ticket 35's) only by the new `backgrounds.ron`, which is **byte-identical** to `game-data/backgrounds.ron`; the report is unchanged (33 items); counts gain 39 backgrounds; 0 check problems. Tests: new `extract/tests/extract/backgrounds.rs` (2: the carried list is `game-data/`'s; the files write backgrounds, None as key and name only, and read back), written after the code (both passed at once). `cargo test -p extract` 86 pass. Next: proof, docs, DB reset, Verification.
- 2026-11-01: milestone 5 done. Proof (`t34/proof-{old,new}` on one shared `t34/proof-common.rs`, ticket 35's plus a human wizard 3, `bg base`, on both sides and the same Build with each of the 39 backgrounds on the new side; `t34/compare.py`) passes: the new data is the old plus `backgrounds`; all 5553 old walk lines are identical in the new walk (class skills aside); each background's Build differs from `bg base` only by its granted feats (held), their values, its class skills and the options they open (Medium Armor and Armor Focus for the four light-armour backgrounds, Stunning Fist for the Martial Disciple) or close (the granted feat itself, now held); 5220 lines of class and prestige Builds hold exactly their classes' class skills. Docs: `game-data/README.md` (the file and its shape), `GLOSSARY.md` (new Background and Class skill; Build start and Granted feat updated). Final DB reset and reseed (`t34/reset-db.sh`), `cargo sqlx prepare` (8 query files new, 5 gone). Verification below.

## Answer

Done 2026-11-01, uncommitted. Every decision in the Question is built. A background is a Build start choice (`BuildStart.background`) from `game-data/backgrounds.ron`, the user's 39 entries (38 in eight groups, plus None). Its skills become class skills, so a Shackles Corsair wizard holds Athletics and Mobility. Its feats are granted feats, held from the start and counted for prerequisites like any other. The Aldori Swordsman holds Exotic Weapon Proficiency (Dueling Sword). Bounty Hunter, Gladiator, Guard and Mercenary hold Armor Proficiency (Light Armor), so a Guard wizard can take Medium Armor at level 1. The Martial Disciple holds Improved Unarmed Strike (Stunning Fist opens), the Arcane scholar Scribe Scrolls and the Divine scholar Brew Potions. Everything else is the background's note, for example "Proficient with scimitar. +2 CMD against trip." None grants nothing. The `Build` type changed, so the v1 link vector is regenerated (no v2). Every Build without a background walks exactly as before.

### Rust (`rules`)

- **Background.** `Background { key, name, group: Option<BackgroundGroup>, note, class_skills, feats, feats_for }` (no guide link: the guide doesn't list backgrounds), `BackgroundGroup` (Craftsman, Noble, Oblate, Regional, Scholar, Street Urchin, Wanderer, Warrior; `ALL`, `name`), `BackgroundKey`, `GameData.backgrounds` (canonical by key; `counts` gains "Backgrounds").
- **Build start.** `BuildStart.background: Option<BackgroundKey>`. `None` means not chosen yet; the background None is a key like any other. The walk holds the background's feats (a parameterised one for every value) and feats for a value before level 1, with no source, as a class row's granted feats. An unknown key is `UnknownKey` at the new `Choice::Background`, which is also among `Edit::Start`'s choices, so changing the background reports what it would break.
- **Class skills.** The engine had none. `CharacterState.class_skills` holds the class skills of each class taken (an archetype's list where it replaces its class's), joining when the class is first taken, plus the background's. Nothing else reads them yet.
- **Check.** A background key is used once (`DataAt::Background`). Its `feats_for` values must be of the feat's kind (`ValueOfWrongKind`).
- **Tests first** (new `backgrounds.rs`, 9, plus 2 check tests). The world gains Shackles Corsair, Martial Disciple (its Improved Unarmed Strike's invented Dex 13 skipped), Guard (light armour for Armor Focus (Light)), Aldori Swordsman and None. Red run: 11 failures (`t34/rules-red.txt`); None's test passed at once.

### Schema (the one migration)

- New table `background` (key, name, `background_group` CHECKed to the eight groups, NULL for None, `note`).
- `class_skill` gains a second owner, `background_key`. `class_key` becomes nullable, with a CHECK of exactly one owner and an archetype only with a class.
- `granted_feat` gains a third owner, `background_key`. Value columns are allowed on a class level or a background, and still not on an option level.
- 41 tables. `backgrounds.ron` is a required file. Seed (`insert_background`, `SlotOwner::Background`), loader (`load_backgrounds`), `columns` (`BackgroundGroup`).
- The fixture gains `backgrounds.ron`: skills only, a feat, a feat for a value, and None.
- Tests: files (1 new, 1 updated), postgres (every group round trips; a CHECK/FK test of 8 statements). Red run with a loader that read no backgrounds: 13 postgres failures (`t34/data-red-postgres.txt`). 2 older tests were adjusted for the fixture's new rows.

### Data

One implementation, `extract::backgrounds`, holds the user's table as it stands and applies the decisions:

- light armour is the feat;
- the dueling sword is Exotic Weapon Proficiency for that weapon;
- the feat column becomes granted feats;
- other proficiencies and bonuses go in the note, as sentences in the user's words.

It finds skills, feats and the weapon by name, and panics on one it doesn't find. Keys are the names' slugs (`warrior-of-the-linnorm-kings`, `arcane`, `none`). The one-off `t34/write/` (Rust, in the scratchpad `/private/tmp/claude-501/-Users-thaynamarcela-code/764fca33-bbfa-4ae3-a071-779e1a791cb4/scratchpad/t34/`) wrote only `game-data/backgrounds.ron`. It is byte-identical to its dry run in `t34/dry/game-data/` and has 0 check problems.

An independent script, `t34/compare_list.py`, parses the user's markdown table and the RON. It finds 39 rows on each side and no difference in names, groups, class skills, feats, feats for a value or the note's contents.

Real-data tests (`game_data.rs`, 7 new, red on an empty `backgrounds.ron` first: 7 failures, `t34/real-data-red.txt`):

- the 39 backgrounds in their groups;
- exactly the decided grants;
- the Shackles Corsair's class skills;
- the Aldori Swordsman's proficiency;
- the Guard's Medium Armor at 1;
- the Martial Disciple's Stunning Fist;
- None grants nothing.

### Proof

`t34/proof-old/` (against `git archive HEAD`, 17df6d2) and `t34/proof-new/` build one shared source, `t34/proof-common.rs`. It is ticket 35's, plus a human wizard 3 (`bg base`) and, on the new side, that Build with each of the 39 backgrounds. `t34/compare.py` checks:

- **Data:** the new data is the old plus `backgrounds`; every other part is identical.
- **Walk:** all 5553 old lines (every base class and archetype through 20 levels, each prestige class after wizard, cleric or witch 10, ticket 35's targeted Builds, `bg base`) are identical in the new walk, class skills aside.
- **Each background against `bg base`:** only its granted feats (held), their values and its class skills differ. Its options change only where it opens Medium Armor and Armor Focus (the four light-armour backgrounds) or Stunning Fist (Martial Disciple), and the granted feat itself is no longer offered (already held). 34 backgrounds open nothing.
- **Class skills:** 5220 lines of class and prestige Builds hold exactly their classes' class skills (the archetype's list where it replaces the class's).

### Extractor

- `extract` sets `data.backgrounds` from `extract::backgrounds` after `second_follow_ups::apply`; it settles no report item.
- A fresh extraction (`t34/extract-new/`) differs from ticket 36's only by the new `backgrounds.ron`, which is byte-identical to `game-data/backgrounds.ron`. The report is unchanged (33 items).
- `ron_out` writes a background on one line, leaving out an unset group, note or grant (None is key and name only).
- Tests: `extract/tests/extract/backgrounds.rs` (2), written after the code.

### Docs

- `game-data/README.md`: the file and its shape.
- `GLOSSARY.md`: new **Background** and **Class skill**; Build start and Granted feat updated.
- `game-data/EXTRACTION_REPORT.md`: unchanged.

### Verification

- `cargo test --workspace`: pass (rules 274 + unit 1, extract 86, data files 25 + game_data 109 + postgres 24).
- `cargo clippy --workspace --all-targets`: clean.
- `cargo fmt --all --check`: pass.
- `cargo build -p rules --target wasm32-unknown-unknown`: pass.
- `SQLX_OFFLINE=true cargo leptos build`: pass.
- `cd data && cargo sqlx prepare --check`: pass (8 query files new, 5 gone).
- Dev DB reset per `CLAUDE.md`, only `wotr-planner-postgres` on 5434 (`t34/reset-db.sh`): drop/create `wotr_planner`, `sqlx migrate run --source data/migrations`, `cargo run -p data --bin seed`, `cargo sqlx prepare`.
- `--check-only` and the real seed both give 12 races, 11 skills, 73 weapons, 261 feats, 26 classes, 196 archetypes, 13 prestige classes, 63 mythic abilities, 56 mythic feats, 10 mythic paths and now 39 backgrounds. Rows: 41 tables; `background` 39 (1 without a group); `class_skill` 372 class rows (unchanged) + 55 background rows; `granted_feat` 8 background rows (1 with a weapon).
- The v1 link vector is regenerated (a Shackles Corsair background).
- Proof: passes (above).

### Open questions (for the user)

1. **Options' class skills.** Some options grant a "Class Skill: …" feature (an oracle's mystery, a kineticist's element). These stay features, so `class_skills` doesn't include them. Should they count?
2. **A blank Build's background.** A new Build has no background chosen, which differs from the background None. Should a blank Build start on None instead, so the choice is never empty?
3. **What class skills do.** Nothing reads them yet: skill points and the rank cap don't depend on them, and the game's +3 on a trained class skill is a number this planner doesn't show. Is showing them on the sheet enough?

## Decisions on the open questions

**User, 2026-11-01 ("all recommended"):**
1. Options' "Class Skill: …" features become real class skills, now that the engine holds class skills. [Ticket 37](37-option-class-skills.md).
2. A blank Build has no background chosen; the sheet asks for it as it does for race. None stays a data entry.
3. Class skills are shown on the sheet; nothing else uses them for now.
