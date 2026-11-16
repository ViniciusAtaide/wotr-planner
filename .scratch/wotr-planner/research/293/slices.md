# Ticket 293, stage S1: the slices (research, 2026-11-16)

Read-only research at `main` fc584cda (after 304; 305 is building in lane-tests). Game: the pinned dump
`blueprints_raw_2.7.0x_0.binz` (236,660 blueprints). No game text below. Evidence beside this file:
- `run.txt`: a fresh game-check run (lane-tests' debug build, `--data` main's `libs/game-data`), exit 0: 4,556
  differences in 212 groups, all known.
- `addclassskill.txt`: `game-check grep AddClassSkill` (116 blueprints).
- `classskill-features.txt`: per 262 entry, the features its archetype grants that carry `AddClassSkill`.

Slice names (293a…293m) are placeholders; the main session numbers them. "Fixture" = `tools/game-check/tests/fixture/`.
Every slice also runs `cargo fmt --all --check`.

## 0. Decisions taken here (the design settles them; say if you disagree)

1. **The guard skips a held user fact, it doesn't refuse the run.** Design §3.2 (skip, print under "to the user", the
   finding stays) over §1.3's list of refusals. Exit 2 is for a real refusal: a `planner/` file, an entry with no
   blueprint, a comment inside a written span, a read-back that differs, a failed data check, a registry line naming
   no entry.
2. **No-op test = the effective value under the target**, not the field's text: an archetype's saves, BAB, skill
   points and class skills are compared as `effective(planner) == effective(target)` against the class *as it will be
   written* (classes are planned first, archetypes after). A field is never written when the check would see no
   difference, so a person's order and redundant-but-equal fields (`druid/feyspeaker`'s `skill_points: 4`) stay.
3. **An archetype override is written only where it differs from its class's target** (`Some(p)` when the game's
   archetype names its own table and `p` ≠ the class's; `None` otherwise): the data check refuses an override equal to
   the class's (`DataIssue::SaveProgressionSameAsClass`, `libs/rules/src/check.rs:1772`; README line 69).
4. **What the writer can't express is left and listed** (class skills naming `CheckIntimidate`; a save table no
   formula gives): the expressible rest is written, the finding stays.
5. **Read-back is type-agnostic**: each written file is parsed as `ron::Value` before and after; after must equal
   before with only the written field set (or removed). The typed check is the closing validate step
   (`seeds::check_game_data_files` in `main.rs`; a closure in the library so the fixture tree needn't pass the full
   data check).
6. **The list style follows the field**: a value that spans lines is rewritten one element a line at the field's
   indent plus four, `]` at the field's indent; a one-line value stays one line; an inserted field is one line, except
   `components` in a file whose fields are indented (one element a line, as 296/297 wrote).

## 1. The slices, in order

Parallel groups (CLAUDE.md: two builders at most):
- **Lane W (writer, `tools/game-check/`):** a → b → c → d → e → g → h, one after the other (they share
  `src/write/*`, `src/main.rs`, the fixture).
- **Lane R (rules, then data):** f (beside a–e) → i → j → k (k waits on e).
- **Then** l (after g, h, i, j, k), then m (main session).

---

### 293a. The span writer

**Goal:** replace, insert or remove one top-level field of an entry's RON text, every other byte kept.

**Edits:**
- `tools/game-check/Cargo.toml`: `[dependencies]` gains `ron.workspace = true` and `serde.workspace = true` (ron is a
  dev-dependency today).
- `tools/game-check/src/lib.rs`: `pub mod write;`.
- `src/write/mod.rs` (new): module doc (design §1.2) and `pub mod span;` only.
- `src/write/span.rs` (new), lifted from 296's rewrite crate
  (`/private/tmp/claude-501/-Users-thaynamarcela--claude-spares-spare-nijyzu/f65a1f87-6cde-4710-9c11-116f63b89469/scratchpad/game/296/rewrite/src/main.rs`
  lines 390–489: `components_text`, `ron_text`, `field_span`, `trim_end`, `is_ident`, `skip_string`; another
  session's scratchpad, so copy it now):
  - `pub fn field_span(text, name) -> Option<Range<usize>>` (depth 1, outside strings and `//` comments);
  - `pub fn ron_text<T: Serialize>(value: &T) -> String` (one line, `IMPLICIT_SOME`, 296's `PrettyConfig`);
  - `pub enum Value { One(String), List(Vec<String>) }`;
  - `pub fn write_field(text, name, value: Option<&Value>, declared: &[&str]) -> Result<String, SpanError>`:
    `Some` replaces the field (style per decision 6) or, when missing, inserts it after the last present field that
    precedes it in `declared`; `None` removes it (its whole line in a pretty file; `name: value, ` or `, name: value`
    in a one-line one);
  - `pub enum SpanError { CommentInside { field }, NoAnchor { field } }`: a span holding `//` is refused.
  - `pub const CLASS_FIELDS`, `ARCHETYPE_FIELDS`, `GEAR_FIELDS: &[&str]`: the declared orders
    (`libs/rules/src/game_data.rs` `Class` 753, `Archetype` 841; `libs/rules/src/gear.rs` `GearItem` 23), with
    `bab` placed between `casting_ability` and `fort` in `ARCHETYPE_FIELDS` (293f adds it there).
- `tests/fixture/span.rs` (new) and `mod span;` in `tests/fixture/main.rs`.

**Tests** (`tests/fixture/span.rs`, on inline texts shaped like `classes/paladin.ron`, `archetypes/druid/feyspeaker.ron`,
`features/oracle/seeker/seekers-trapfinding.ron`, `gear/dirge-for-sarkoris.ron`):
- `replacing_a_field_keeps_every_other_byte`: only the span changes; a comment above and after survives.
- `a_value_that_spans_lines_is_written_one_element_a_line`: dirge's column-0 `components` stays multi-line.
- `a_one_line_file_stays_one_line`.
- `a_missing_field_goes_after_the_last_field_before_it`: `bab` into a Feyspeaker-shaped text lands after
  `class_skills`, before `levels`.
- `removing_a_field_takes_its_line` (pretty and one-line).
- `a_comment_inside_the_field_is_refused`.
- `a_name_in_a_string_or_a_nested_struct_is_not_the_field`: `summary: "skill_points: 3"`, a row's `features:`.

**Proof:** none of §4.2 (no data). `cargo test -p game-check`; the 7 names in `-- --list`.
**Blockers:** 305 merged (the ticket's own blocker). **Shares:** `Cargo.toml`, `lib.rs`, `src/write/mod.rs`,
fixture `main.rs` with b–h. **Estimate:** 15 min.

---

### 293b. Applying changes, the guard's refusals, the write fixture's tree

**Goal:** apply a list of field changes to a data folder, all or nothing, with read-back and restore.

**Edits:**
- `src/write/mod.rs`:
  - `pub struct Change { at: DataAt, file: PathBuf /* relative to the data dir */, field: &'static str, kind: &'static str /* the check's finding kind, for the registry (293e) */, value: Option<span::Value>, old: String, new: String /* normal forms, for the log */ }`;
  - `pub fn entry_file(dir, data, at) -> PathBuf`: `classes/` or `prestige/` (by `data.prestige_classes`),
    `archetypes/`, `gear/`, then `<key>.ron`;
  - `pub fn apply(dir, changes, validate: impl Fn(&Path) -> Result<(), String>) -> Result<Applied, WriteError>`:
    group by file; refuse a file under `planner/` or missing; build every new text first (any `SpanError` refuses
    all, nothing written); write; read back (decision 5); run `validate`; on any failure restore every touched file's
    original bytes.
  - `pub enum WriteError { Planner(PathBuf), Missing(PathBuf), Span { file, field, error }, ReadBack { file, field }, Invalid(String), Io { .. } }` with `Display`.
- The write fixture's tree, `tests/fixture/data/` (new; the entries of `planner.ron` that S1 writes, as files, with
  their planted comments): `progression.ron` (`(mythic_ascensions: [])`), `skills.ron` (`[]`),
  `classes/warden.ron` (pretty, its rows with `features` only), `prestige/sentinel.ron`,
  `archetypes/warden/stoic.ron`, `archetypes/warden/stray.ron`, `gear/{belt-of-test-strength,cloak-of-test,headband-of-test,tome-of-test,ring-of-nothing,circlet-of-test}.ron`
  (one pretty, one one-line, as the real data mixes them), `planner/gear/test-trinket.ron` (`(why: "…", entry: (…))`).
- `tests/fixture/support.rs`: `pub fn data_copy() -> tempfile::TempDir` (copies `tests/fixture/data/`).
- `tests/fixture/write.rs` (new) and `mod write;` in fixture `main.rs`.

**Tests** (`tests/fixture/write.rs`):
- `the_write_fixture_reads_as_game_data`: `seeds::read_game_data_files` on the tree.
- `applying_a_change_writes_only_its_field`: a hand `Change` on `cloak-of-test`; the file differs only in
  `components`.
- `a_planner_file_is_refused_and_nothing_is_written`: two changes, one on `planner/gear/test-trinket.ron`; exit
  `Planner`, the other file untouched.
- `a_failed_validation_restores_every_file`: `validate` returns `Err`; every byte back.

**Proof:** none of §4.2. `cargo test -p game-check`. **Blockers:** a. **Shares:** `src/write/mod.rs`, fixture
`main.rs`, `support.rs` with c–h. **Estimate:** 20 min (the tree is half of it).

---

### 293c. The items target and the library run

**Goal:** the game's always-on bonuses of every gear item as `GearStat` components, written where they differ.

**Edits:**
- `src/areas/items.rs`: lift the game side of `compare` (lines 44–70: components, then enchantments) into
  `pub(crate) fn game_bonuses(ctx, out, json) -> Vec<Bonus>`; make `stacked`, `target`, `bonus_type` and `Bonus`
  `pub(crate)`. `compare`'s output is unchanged (the fixture's `items_*` tests prove it).
- `src/write/items.rs` (new): `pub(crate) fn changes(ctx, dir) -> (Vec<Change>, Vec<Unexpressed>)`: per non-usable
  gear item with a blueprint, in key order: game = `stacked(game_bonuses)`; planner = `item.stats()` through
  `target`/`bonus_type`; equal as sorted lists → nothing; else `components` = the game's bonuses through
  `fn gear_stat(&Bonus) -> Option<Component>` (the inverse of `target` and `descriptor`: ability names,
  `AllSaves`, `Save*`, `Initiative`, `Skill*` through the inverse of `skill_stat`, `SpellDC[ school]`, `CasterLevel`,
  `SpellPenetration`, `Concentration`; `Untyped` → `Descriptor::Untyped`, `Armour`/`NaturalArmour`, else the
  variant of that name), sorted (`Component: Ord`). A bonus with no inverse leaves the item unwritten and listed.
  `kind: "bonuses"`.
- `src/areas/mod.rs`: `pub fn game_skill(stat) -> Option<SkillKey>` (inverse of `skill_stat`, used here and by 293h).
- `src/write/mod.rs`: `pub enum WriteArea { ClassTables, Items }`; `pub fn run(dump, dir, areas, validate) -> Result<Outcome, WriteError>`:
  read with `seeds::read_game_data_files`, `Planner::new`, plan the chosen areas, `apply`. `Outcome { changes,
  unexpressed }`.

**Tests** (`tests/fixture/write.rs`):
- `writing_items_leaves_only_the_slot_finding`: after `run(Items)`, the check over the re-read tree reports for
  `items` only `slot differs` on `headband-of-test`; `cloak-of-test` is +3, `headband-of-test` has Perception +1
  Insight and Initiative +1 Morale, `ring-of-nothing` has `components: []`; `circlet-of-test` and `tome-of-test`
  untouched (bytes).
- `a_second_items_run_writes_nothing`.

**Proof:** none on real data yet. `cargo test -p game-check` (the `items_*` fixture tests unchanged).
**Blockers:** b. **Shares:** as b, plus `areas/items.rs`, `areas/mod.rs`. **Estimate:** 20 min.

---

### 293d. The command, `--dry-run` and the change log

**Goal:** `game-check --write <areas> [--dry-run] [--log <file>] [--data <dir>] [--binz <file>]`, deterministic.

**Edits:**
- `src/main.rs`: doc header and `USAGE`; parse `--write class-tables,items` (an unknown area: usage, exit 2),
  `--dry-run`, `--log`; validate = `seeds::check_game_data_files`; after the write, `check` on the new data;
  stdout: per area the fields written and files touched, the unexpressed list, (293e) "to the user"; exit 0 written
  or nothing to write, 2 refused.
- `src/write/log.rs` (new): `pub fn tsv(changes, before: &Report, after: &Report) -> String`: one row per change,
  `entry\tfield\told\tnew\tfindings removed\tfact` (findings removed = that entry's `area / kind` present before and
  gone after; `fact` filled by 293e, `-` until then), rows in entry then field order.
- `src/write/mod.rs`: `dry_run: bool` on `run` (plans and guards, writes nothing).

**Tests:**
- `two_runs_on_two_copies_give_the_same_files_and_logs` (two `data_copy()`s, byte compare of every file and the log).
- `a_dry_run_writes_nothing`.

**Proof (§4.2 point 2 on the fixture).** `cargo test -p game-check`; by hand: `game-check --write items --dry-run`
on main's data prints the 4 items of 293k (no headband yet), writes nothing, exit 0.
**Blockers:** c. **Shares:** `main.rs`, `src/write/*`. **Estimate:** 15 min.

---

### 293e. The user-fact registry and guard

**Goal:** a change to a user fact is written only under an `applied` line; a held one goes to the user.

**Edits:**
- `src/write/facts.rs` (new): `pub struct Facts` in `known.txt`'s grammar with another right side:
  `<area> / <kind> / <entry> [/ <value>] -> applied <ticket>` or `-> held`; `Facts::embedded()`
  (`include_str!("../../user-facts.txt")`), `parse`; `fn decide(area, kind, entry) -> Decision { Free, Applied(ticket), Held }`,
  most specific line wins, `*` any kind; `fn stale(&data) -> Vec<line>`: lines whose entry the data doesn't have
  (fails the run, exit 2).
- `src/write/mod.rs`: `run` takes `&Facts`; a `Held` change is dropped from the apply list and returned in
  `Outcome.to_the_user`; `Applied(t)` sets the log's `fact` column to `applied t`.
- `src/main.rs`: prints "to the user" (entry, field, old, new).
- `tools/game-check/user-facts.txt` (new), header comment citing design §3, then:
  - `class tables / class skills differ (the user's facts) -> held` and
    `class tables / skill points differ (the user's facts) -> held` (`research/skills-from-user.md`, map Notes);
  - `applied 262` for the 13 entries of 262's table (`class tables / class skills differ (the user's facts) / archetype alchemist/metamorph -> applied 262`, … `class stalwart-defender`) and
    `class tables / skill points differ (the user's facts) / archetype fighter/defender-of-the-hearth -> applied 262`;
  - `class tables / skill points differ (the user's facts) / class arcane-trickster -> applied 261`;
  - `items / * / gear item <key> -> held` for the 22 gear files with no `guide:` (the list: `grep -L "guide:"
    libs/game-data/gear/*.ron`; README line 40).
  (The headband +8's line comes with 293k, since a line naming no entry fails the run.)

**Tests:**
- `a_held_fact_is_not_written_and_goes_to_the_user` (`items / * / gear item cloak-of-test -> held`).
- `an_applied_fact_is_written_and_logged_with_its_ticket`.
- `a_fact_naming_no_entry_fails_the_run_and_writes_nothing`.
- `the_embedded_registry_names_only_entries_the_data_has` (on the real data, `seeds::game_data_dir()`).

**Proof:** `game-check --write items --dry-run` on main's data: unchanged from 293d (no generic item differs).
**Blockers:** d. **Shares:** `main.rs`, `src/write/mod.rs`. **Estimate:** 15 min.

---

### 293f. `Archetype::bab` (rules; beside a–e)

**Goal:** an archetype can replace its class's BAB progression, and every BAB reader uses it.

**Edits:**
- `libs/rules/src/game_data.rs`: `Archetype` gains `#[serde(default)] pub bab: Option<BabProgression>` between
  `casting_ability` and `fort` (doc: "`None`: the base class's. `Some`: replaces it (Feyspeaker: half)");
  `Class::bab`'s doc loses "Archetypes never change it"; `Class::base_attack_bonus(&self, level, archetype: Option<&Archetype>)`
  takes the archetype's when set.
- Callers: `libs/rules/src/engine/walk/level.rs:624` (move `let archetype = catalog.archetype_of(…)` above the BAB
  line and pass it); `apps/mcp/src/describe.rs:326` (pass `archetype`); `tools/game-check/src/areas/class_tables.rs:49`
  (`None`) and `:125` (`Some(archetype)`, and compare the BAB for every archetype, the game's table being the
  archetype's when it names one, else the class's, as the saves already do at lines 129–138).
- `libs/rules/src/check.rs`: `DataIssue::BabProgressionSameAsClass` beside `SaveProgressionSameAsClass` (185, its
  `Display` at 423, the check at 1772).
- Struct literals: `libs/rules/src/test_world/world/nested.rs:202` (`bab: None`), `libs/rules/tests/engine/check.rs:239`,
  `libs/rules/tests/engine/alignment.rs:55`.
- `libs/game-data/README.md` line 67: an archetype names `bab: Some(Half)` too.

**Tests:**
- `libs/rules/tests/engine/derived.rs`: `an_archetypes_bab_replaces_its_classes` (a Full class at 4: 4; with a
  `Some(Half)` archetype: 2).
- `libs/rules/tests/engine/check.rs`: the Armiger test at 222 also gets `BabProgressionSameAsClass` for a
  `bab: Some(<fighter's>)`, or a new `an_archetype_bab_equal_to_its_classes_is_refused`.

**Proof:** no data change; the game check prints the same 4,556 (Feyspeaker's "BAB differs" stays until 293l).
Checks (rules and apps touched): `cargo test -p rules -p mcp -p game-check -p seeds -p corpus`,
`cargo build -p rules --target wasm32-unknown-unknown`. **Blockers:** 305 merged. **Shares:**
`areas/class_tables.rs` with g/h (so f merges before g starts). **Estimate:** 20 min (most of it the checks).

---

### 293g. Class tables: BAB, saves and the save exceptions

**Goal:** each class's `bab`/`fort`/`reflex`/`will` the formula the game's table gives, its exceptions emptied; each
archetype's overrides against its class's target.

**Edits:**
- `src/areas/class_tables.rs`: lift `Row::progression` (line 175) into `pub(crate) fn bonuses(dump, json, field) -> Option<Vec<i64>>`; `game_skills` `pub(crate)`.
- `src/write/tables.rs` (new):
  - `pub fn bab_progression(table: &[i64], levels: u8) -> Option<BabProgression>` and
    `pub fn save_progression(table, levels, prestige) -> Option<SaveProgression>`: the variant whose formula
    (`BabProgression::bab`, `SaveProgression::save`) gives `table[1..=levels]` exactly;
  - classes in key order: target `bab`, `fort`, `reflex`, `will`; `save_exceptions` keeps only the exceptions of a
    save no formula gives (so emptied, then removed, for the 10 classes here); no-op when every level's BAB and save
    equals the target's; a table no formula gives is unexpressed;
  - archetypes in key order, against the class's target (decisions 2 and 3): `bab`, `fort`, `reflex`, `will`.
  - kinds: `BAB differs`, `Fortitude differs`, `Reflex differs`, `Will differs`.
- `src/write/mod.rs`: `WriteArea::ClassTables` plans `tables::changes`.
- Fixture: `blueprints.json` gains a `TestBabLow` stat progression and `StrayArchetype`'s `m_BaseAttackBonus` naming
  it; `tests/fixture/areas.rs` `class_tables_find_…` asserts the new `BAB differs` on `archetype warden/stray`.

**Tests:**
- `writing_class_tables_brings_bab_and_saves_to_the_game`: warden's `reflex: Good`; stoic's `will` removed (the game
  keeps the class's); stray gets `bab: Half`; the check then reports no BAB/save finding.
- `a_save_table_no_formula_gives_is_none` (`tables::save_progression` on an odd table).
- `a_class_table_run_is_idempotent`.

**Proof:** dry run on main's data lists 47 save changes' worth of fields on 10 classes + 3 archetypes and 1 BAB
(Feyspeaker): `paladin`, `hunter`, `shifter`, `warpriest` and the 5 prestige classes lose `save_exceptions`;
`stalwart-defender` becomes `fort: Good, reflex: Poor, will: Good`; `cleric/priest-of-balance` `fort: Some(Poor)`,
`kineticist/dark-elementalist` `fort: Some(Poor), reflex: Some(Poor), will: Some(Good)`, `monk/traditional-monk`
`will: Some(Good)`, `druid/feyspeaker` `bab: Some(Half)`. No archetype of the hunter, paladin, shifter or warpriest
is written (their findings go with their class's exception). `cargo test -p game-check`.
**Blockers:** e, f. **Shares:** `areas/class_tables.rs` (with f), `src/write/*`, `blueprints.json`, `areas.rs`.
**Estimate:** 20 min.

---

### 293h. Class tables: skill points and class skills (the user's facts)

**Goal:** skill points and class skills from the game, through the registry.

**Edits:**
- `src/write/tables.rs`: class `skill_points` = `SkillPoints`; `class_skills` = `ClassSkills` through
  `areas::game_skill` (a stat with none, `CheckIntimidate`, is unexpressed: the rest is written). Archetype
  `skill_points` = class target + `AddSkillPoints`, `Some` when it differs from the class's, else `None`;
  `class_skills` = `Some(game list)` when `ReplaceClassSkills` and it differs from the class's, else `None`
  (`sorcerer/empyreal-sorcerer` and `sylvan-sorcerer` have `ReplaceClassSkills: false`: their field is removed).
  Kinds `skill points differ (the user's facts)`, `class skills differ (the user's facts)`, so 293e's lines apply.
- Fixture: `WardenClass`'s `ClassSkills` gains `CheckIntimidate`; `areas.rs`'s class-skills assertion takes the new
  game value.

**Tests:**
- `the_users_class_tables_are_held_without_an_applied_line` (default registry text: held, listed, untouched).
- `applied_class_tables_are_written`: warden 3 points, Athletics and Stealth (Intimidate left, the finding stays);
  stoic 4 = 3 + 1, its `skill_points: Some(4)`.

**Proof:** dry run on main's data with the embedded registry: 2 skill-point and 13 class-skill fields, all
`applied`; none held. `cargo test -p game-check`. **Blockers:** g. **Shares:** as g. **Estimate:** 15 min.

---

### 293i. The features' `ClassSkill` 262 needs (data, hand; lane R after f)

**Goal:** a class skill the game gives through an archetype's own feature is held through that feature, so the
game's list (293l) loses nothing on the sheet.

**Edits** (each from the blueprint's `AddClassSkill`; `classskill-features.txt`):
- Existing entries gain components: `features/witch/witch-of-the-veil/secrecy.ron` `ClassSkill("stealth")`,
  `ClassSkill("trickery")` (`WitchOfTheVeilSecrecyFeature` 8c189560d7f94dd688f0dc29f6963610);
  `features/oracle/seeker/seekers-trapfinding.ron` `ClassSkill("trickery")` (`SeekerTrapfinding`
  cb3abfaa682b67c48953dfa200aab325); `features/sorcerer/seeker-sorcerer/seekers-trapfinding.ron`
  `ClassSkill("trickery")` (`SorcererSeekerTrapfinding` b1c4f14bc87534e4da7ecd8e0b0658d2: 261's item).
- Four new feature files (no entry today; each visible, `Ranks` 1, granted at level 1) and a grant in the archetype's
  row 1 `features`:
  - `MetamorphClassSkills` 1d42191a101f2fb42b6e87b7bfc99a5e (Stealth) → `archetypes/alchemist/metamorph.ron`;
  - `SoheiLoreNatureFeature` d59fe46a1aa1ace4e987d843c8e309ff (Lore (Nature)) → `archetypes/monk/sohei.ron`;
  - `BloodlineCelestialClassSkill` b0520c35f31ba544189d3293ed14dd67 (Lore (Religion)), level 1 of
    `EmpyrealBloodlineProgression` 8a95d80a3162d274896d50c2f18bb6b1, which the data flattens into
    `archetypes/sorcerer/empyreal-sorcerer.ron` row 1 (as `heavenly-fire`);
  - `BloodlineFeyClassSkill` 243f474ce797acb4086c2dbc58660c4a (Lore (Nature)), level 1 of
    `SylvanBloodlineProgression` a46d4bd93601427409d034a997673ece → `archetypes/sorcerer/sylvan-sorcerer.ron` row 1.
  Names and one-line summaries in our own words (user's Q1 answer); no name starting "Class Skill" or equal to
  "Skills", and no key `monk/sohei/skills` (`class_skills.rs` lines 334 and 484 test those).
- The Wind Whisperer needs none: its mystery's option row already gives Mobility and Stealth
  (`options/oracle/mystery/wind.ron` line 8; `OracleWindMysteryFeature` 7c1fdd831af747b47bb2cce0051f309b at its 1).
- 261's other feature skill: `options/cleric/domain/animal.ron`'s `class_skills: ["lore-nature"]` moves from row 4 to
  a new row 1 (`AnimalDomainBaseFeature` d577aba79b5727a4ab74627c4c6ba23c at level 1 of `AnimalDomainProgression`
  23d2f87aa54c89f418e68e790dba11e0).
- `libs/game-data/notes/293-class-skill-features.md`.

**Tests** (`libs/seeds/tests/game_data/class_skills.rs`):
- un-ignore `a_seeker_sorcerer_holds_trickery_and_its_bloodlines_class_skill` (397),
  `the_animal_domains_class_skill_is_on_its_first_row` (312) and
  `the_animal_domains_lore_nature_comes_with_the_domain_at_its_first_level` (563; if its Separatist half fails on
  the second domain's level, rename its reason to 294 instead);
- `options_class_skills_are_on_their_first_row` (220): drop the Animal filter;
- new `an_archetypes_own_feature_holds_the_class_skill_its_list_lacks`: for the 7 archetypes above, an engine whose
  archetype list lacks the skill (as `engine_without_class_skill`) still holds it at 1, each line citing its
  blueprint;
- `no_feature_stands_for_a_fixed_class_skill` (329): its comment at 341 says the skill now comes through the feature.

**Proof:** game check: `progression rows / in game only` loses 4 (`monk/sohei` under 279, `alchemist/metamorph`
under 280, `sorcerer/empyreal-sorcerer` and `sylvan-sorcerer` under 281; their entry lines stay, other findings
remain: 19, 8, 6 and 7), 4,556 → 4,552; mapping compares 4,089. Checks: `cargo test -p seeds -p corpus`,
`cargo run -p seeds --bin check-data`, the game check. **Blockers:** 305 merged (independent of the writer).
**Shares:** `class_skills.rs` (with nothing else in S1). **Estimate:** 20 min.

---

### 293j. The class tables' real-data tests, ignored (lane R)

**Goal:** the game's values as failing tests before the run, so 293l only removes `#[ignore]`.

**Edits:** `libs/seeds/tests/game_data/class_tables.rs` (new) and `mod class_tables;` in
`libs/seeds/tests/game_data/main.rs`; each test `#[ignore = "waits on ticket 293: …"]`, table-driven, each row
cited (blueprint names and GUIDs in §2 below):
- `the_save_tables_are_the_games`: (entry, save, level, value): paladin Will 20 → 12; warpriest Will 20 → 12;
  hunter Will 7 → 2; shifter Reflex 2 → 3; arcane-trickster, loremaster, mystic-theurge, student-of-war,
  winter-witch Will 4 → 2; stalwart-defender Fort 1 → 1, Reflex 1 → 0, Will 1 → 1; priest-of-balance Fort 1 → 0;
  dark-elementalist Fort 1 → 0, Reflex 1 → 0, Will 1 → 2; traditional-monk Will 1 → 2.
- `the_feyspeakers_bab_is_low`: druid 3 → 2, Feyspeaker 3 → 1.
- `skill_points_are_the_games`: arcane-trickster 3, defender-of-the-hearth 3.
- `class_skills_are_the_games`: the 13 lists of 262 as the game has them, Stalwart without Intimidate (comment: G4).
`libs/seeds/tests/game_data/archetype_saves.rs` is left for 293l.

**Proof:** the 4 names in `cargo test -p seeds --test game_data -- --list`; run once with `--ignored`: each fails
for its reason. **Blockers:** none. **Shares:** `game_data/main.rs` (one line). **Estimate:** 12 min.

---

### 293k. The items run (+ the +8 headband)

**Goal:** the items' bonuses as the game has them, and the user's Headband of Mental Perfection +8.

**Edits:**
- `libs/game-data/gear/headband-of-mental-perfection-8.ron` (new): `key`, `blueprint: "f94d69b19fe07ca4a8ade7993a1a7510"`,
  `name: "Headband of Mental Perfection +8"`, `slot: Headwear`, no `guide` (the guide lists no plain +8 headband;
  the 22 generic items have none either), `components: []`.
- `tools/game-check/user-facts.txt`: `items / * / gear item headband-of-mental-perfection-8 -> applied 288` (288 line 15).
- Run `game-check --write items --log <scratchpad>/293k-items.tsv`: writes the headband's Int/Wis/Cha +8
  Enhancement (`HeadbandOfPerfection8`'s enchantments `Intelligence8` bc66f5712a147874aaac6b397b83f874, `Wisdom8`
  02a37956195faf645bb956f2e6c1ef57, `Charisma8` 7669e7a9af237314dbb2a22e78e1a0b4), `dirge-for-sarkoris` Strength
  +2 Enhancement, `half-plate-of-vigor` Constitution +2 Enhancement, `belt-of-the-venerated-champion` Fortitude +2
  untyped beside Con +6, and per Q2 `trusted-friend`.
- Hand: the Venerated Champion's `other_effects` line keeps only its conditional part (AC against evil); per Q2 A,
  Trusted Friend's two bonuses become an `other_effects` line (its toggle).
- `tools/game-check/known.txt`: delete the 288 lines of `belt-of-the-venerated-champion`, `dirge-for-sarkoris`,
  `half-plate-of-vigor` (and `trusted-friend`'s stays for its slot).
- `libs/seeds/tests/game_data/gear.rs`: un-ignore `the_game_has_a_headband_of_mental_perfection_8` (171);
  `exactly_the_users_twenty_two_generic_items_have_no_guide_reference` → twenty-three, the +8 added with its
  citation; `DATA.gear.len()` 92 → 93 (224); `the_venerated_champions_fortitude_bonus_is_always_on` (367) asserts the
  `GearStat` (Save Fortitude +2, untyped), un-ignored; the Half-Plate test (349) splits: its Con +2 enhancement part
  un-ignored, its "+2 half-plate" line still waiting on 288; Trusted Friend's tests per Q2.
- `libs/game-data/notes/293-items.md`.

**Proof:** game check items 8 → 3 (the 3 `slot differs`, 288), or 5 under Q2 B; items compares 83; the run's second
pass writes 0 files; §4.2 point 5 for these 5 fields. Checks: `cargo test -p seeds -p corpus`, `check-data`, the game
check. **Blockers:** e (and i's merge if the same lane). **Shares:** `known.txt` and `user-facts.txt` with l
(sequential). **Estimate:** 15 min.

---

### 293l. The class-tables run

**Goal:** the class tables as the game has them; the game check's class tables leave only G4.

**Edits:**
- Run `game-check --write class-tables --log <scratchpad>/293l-class-tables.tsv`: 62 findings on 10 classes
  and 17 archetypes (27 files: 10 class/prestige files, 3 save archetypes, Feyspeaker, Defender of
  the Hearth, the 12 class-skill archetypes).
- `tools/game-check/known.txt`: delete section 261 (46 lines) and 262 (14 lines); add
  `class tables / class skills differ (the user's facts) / class stalwart-defender -> <G4>` (Q4).
- Tests: un-ignore 293j's four and `archetype_saves.rs` `four_archetypes_replace_a_save_progression` (104).
- `.scratch/wotr-planner/research/skills-from-user.md`: the 15 rows changed, citing 261/262 (262's Change asks it).
- `libs/game-data/notes/293-class-tables.md`.

**Proof:** game check class tables 63 → 1; total 4,552 → 4,485 (with k under Q2 A); the second
`--write class-tables,items` writes 0 files; §4.2 point 5 on all 62 fields. Checks: `cargo test -p seeds -p corpus`,
`check-data`, the game check. **Blockers:** g, h, i, j, k merged. **Shares:** `known.txt` (after k).
**Estimate:** 15 min.

---

### 293m. S1's proof (main session; heavy runs alone)

- **Census (§4.2 point 3):** `FORM_PROOF_OUT` record on the commit before 293i merges (the first S1 data change),
  `compare` after 293l, and both recorded dirs diffed by a std-only Rust classifier in the scratchpad (sheet lines
  `Level N: Class (Archetype)` and gear lines → the Build's entries; each differing line of `sheets.txt` and
  `numbers.txt` attributed to a change-log row or a 293i feature). Expected: 0 differences in `builds.tsv` and
  `slots.txt`, every link opens with the same picks (§4.2 point 4), 0 unattributed.
- **Idempotence and determinism (point 2):** `--write class-tables,items --dry-run` on the tip: 0; two `cp -R`
  copies of the base data, one run each, `diff -r` empty, logs equal.
- **Hand check (point 5):** all ~67 fields against `game-check show`.
- **Long self-checks (point 6), gold builds (point 7)** (Jack-in-the-Green is a Sylvan Sorcerer: its gains may show
  the new Fey class-skill feature, which the gold test allows: "the engine may show more").
- Classifier crate: ~15 min to write; the runs as long as they take.

## 2. The facts each slice needs

**Today's findings** (`run.txt`, main fc584cda; 4,556 in 212 groups, exit 0):

| Area / kind | n | Known under | Entries |
| --- | --- | --- | --- |
| class tables / Will differs | 33 | 261 | hunter + 7 archetypes (3 → 2 at 7), paladin + 8 (20 → 12 at 20), warpriest + 7 (11 → 12), 5 prestige (3 → 2 at 4), dark-elementalist, traditional-monk, stalwart-defender |
| class tables / Reflex differs | 11 | 261 | shifter + 8 archetypes (2 → 3 at 2), dark-elementalist, stalwart-defender |
| class tables / Fortitude differs | 3 | 261 | priest-of-balance, dark-elementalist, stalwart-defender |
| class tables / BAB differs | 1 | 261 | druid/feyspeaker |
| class tables / skill points differ (user) | 2 | 261, 262 | arcane-trickster 4 → 3; defender-of-the-hearth 2 → 3 |
| class tables / class skills differ (user) | 13 | 262 | the 12 archetypes of 262's table and stalwart-defender |
| items (4 kinds) | 8 | 288 | slot 3 (stay), trusted-friend 2, dirge/half-plate type 2, venerated 1 |
| progression rows / in game only (the 4 class-skill features) | 4 | 279, 280, 281 | sohei, metamorph, empyreal, sylvan |

Writable: 62 of 63 class tables (Stalwart's `CheckIntimidate`: G4), 5 of 8 items (3 slots: 288's model part).

**The game's tables** (each via the class's `m_BaseAttackBonus`/`m_*Save`, read with `game-check show`):
`PaladinClass` bfa11238e7ae3544bbeb4d0b92e897ec and `WarpriestClass` 30b5e47d47a0e37438cc5a80c96cfb99 Will
`SavesHigh`; `HunterClass` 34ecd1b5e1b90b9498795791b0855239 Will `SavesLow`; `ShifterClass`
a406d6ebea5c46bba3160246be03e96f Reflex `SavesHigh`; `ArcaneTricksterClass` 9c935a076d4fe4d4999fd48d853e3cf3,
`LoremasterClass` 4a7c05adfbaf05446a6bf664d28fb103, `MysticTheurgeClass` 0920ea7e4fd7a404282e3d8b0ac41838,
`StudentOfWarClass` 46b1fe9759ea01f4d883b23d8f0aecbb, `WinterWitchClass` eb24ca44debf6714aabe1af1fd905a07 Will
`SavesPrestigeHigh`; `StalwartDefenderClass` d5917881586ff1d4d96d5b7cebda9464 `SavesPrestigeHigh`/`Low`/`High`;
`DarkElementalistArchetype` f12f18ae8842425489d29f302e69134c `SavesLow`/`Low`/`High` (its `BABMedium` is the
kineticist's: no `bab` written); `TraditionalMonk` 8f3242d209994884bbb6c0999660f4df Will `SavesHigh`;
`PriestOfBalanceArchetype` a4560e3fb5d247d68fb1a2738fcc0855 Fort `SavesLow`; `FeyspeakerArchetype`
da69747aa3dd0044ebff5f3d701cdde3 `BABLow`; `DefenderOfTheHearthArchetype` 6c96dc1f5573487399336cd351ef8f2c
`AddSkillPoints` 1 on the fighter's 2. Code: `BlueprintStatProgression.Bonuses` by level, as the check reads it
(`areas/class_tables.rs`).

**The user's facts S1 applies, and where each is recorded:**

| Fact | Recorded | Slice |
| --- | --- | --- |
| The Arcane Trickster's 3 skill points | 261 line 14; 242 Answer line 42 | e (registry), l |
| 262's 13 class-skill rows and the Defender of the Hearth's 3 points ("Apply the game's") | 262 Status line 4 and its table | e, i, l |
| Belt and Headband of Perfection +8 (the belt exists) | 288 line 15 | k |
| The 22 plain stat belts and headbands stay the user's (held) | `libs/game-data/README.md` line 40 | e |

## 3. Open questions

**Q1. The four class-skill features with no entry (Metamorph, Sohei, Empyreal, Sylvan).** Evidence: the game drops
the skill from the archetype's list (Metamorph, Sohei: `ReplaceClassSkills`; Empyreal, Sylvan:
`ReplaceClassSkills: false`, the sorcerer's list) and gives it back through a level-1 feature with `AddClassSkill`
(`classskill-features.txt`); ticket 38 (2026-11-01, guide-based, no user decision) folded those into the lists, which
`no_feature_stands_for_a_fixed_class_skill` pins. On the sheet the two shapes are equal.
- A (recommended, the design's): the game's shape: 293i's four visible features with `ClassSkill`, granted at row 1;
  the lists become the game's. 4 row findings go too.
- B: a normal form in the check (an archetype's level-1 `AddClassSkill` features count in its list); 7 of the 13
  rows then agree, nothing is created; the 4 row findings stay.

**Q2. Trusted Friend's two bonuses.** `TokenOfHonor` 905ec00744cd4627b0e00f187e215d4b has no always-on bonus; its
Cha +2 and Persuasion +5 circumstance come from a toggle's buff (`TokenOfHonorBuff` ab092ad237384b46b2fb192e6305f673).
The run would leave it a necklace that gives nothing; `trusted_friend_is_a_necklace` asserts the summary.
- A (recommended, 234): write it; the bonuses move into an `other_effects` line (the toggle); slot stays 288's.
- B: hold it (`items / * / gear item trusted-friend -> held`) until 288 models usables; 2 findings stay.

**Q3. The check doesn't compare through the writer's targets in S1.** Design §1.1 has each area compare via
`Target`. S1 instead proves agreement by running the unchanged check after each write (fixture and real data),
which keeps every finding text and `known.txt` matching.
- A (recommended): defer the refactor to S4, where the prerequisites area is rewritten onto components anyway.
- B: refactor `class_tables.rs` and `items.rs` onto targets now (about two more slices).

**Q4. The Stalwart Defender's Intimidate needs G4's ticket for its `known.txt` line.** The design's numbering
(G1–G9 = 310–318) is stale: 310 and 311 are taken.
- A (recommended): file G4 before 293l and name it.
- B: keep the line under 293 with a comment until the queue's close.
