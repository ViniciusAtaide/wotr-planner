# The walk records the Item bonuses in force

Type: task
Status: resolved
Blocked by: 148, 150
Map: [WotR Build Planner](../map.md)

## Question

From the architecture review of 2026-11-11 (candidate 3, *Strong*). The user asked for a ticket for each strong candidate.

What a character holds is worked out twice:

- **In the walk:** the walk tracks holdings as it fills each slot (`Held`, `walk.rs:80–91`; `holdings`, `class_skill_holds`, `counted_features` on the `Walker`).
- **Again for the bonuses:** `Engine::held_bonuses` (`engine/caster.rs:282`) works it out from a `CharacterState` and a `BuildStart`, through `holdings()` (`caster.rs:305–392`). Race, subrace and background come from the start, and feats and features from the state.

Because of that:

- **`Engine::totals(start, state)`** (`totals.rs:97`) and **`Engine::caster_line(start, state, spell)`** (`caster.rs:100`) take a pair that callers must keep in step. The callers are `apps/mcp/src/totals.rs:60`, `caster.rs:188`, `compare.rs:128` and `rank.rs:96`: each passes `&build.start` beside a state taken from the sheet.
- **`totals.rs` imports `caster.rs`'s private helpers:** `use super::caster::{amount, modifier}` (`totals.rs:17`).
- **Expanded Arsenal is a special case:** `BonusAmount::SpellFocusOfAnotherSchool` gives 0 in the generic `amount` (`caster.rs:457`) and is summed apart (`caster.rs:420–443`).
- **The totals and caster tests build `CharacterState` by hand** (`libs/rules/tests/engine/totals.rs`). So nothing tests that the walk fills the state these sums read: `class_skills`, `feat_values`, `skill_ranks_at_least_mythic_rank`.
- **A new kind of bonus touches many files:** a new `BonusTarget` changes `bonus.rs`, `check.rs`, the summing code in `caster.rs` or `totals.rs`, and its consumer.

Tickets [139](139-attack-line-weapon-features-and-mythic-feats.md), [141](141-caster-line-scaling-bonuses-and-second-class.md), [143](143-switched-on-abilities.md) and [144](144-bonus-types-and-stacking.md) all add to these sums. 144's stacking rule ("the highest per type and target") needs every bonus in force in one list.

**Shape.** As the walk holds an item, it records the bonuses in force on the `CharacterState`:

- each bonus's item, its Parameter value, the amount resolved at that point (the Mythic rank, an ability modifier, ranks), and the part's display name;
- Expanded Arsenal's copy, resolved there too.

Totals and the Caster line then become sums over that list, and `start` leaves their interfaces: `totals(state)`, `caster_line(state, spell)`.

- An amount that depends on the final ability scores (a modifier) may stay unresolved and be summed at read time. Say which way, and why.
- The field's name is a suggestion. Pick one that reads well next to the glossary's **Item bonus**, and add it to `GLOSSARY.md` if it's a term callers see.

Out of scope: bonus types and stacking (144, which this prepares); the Attack line, which counts no item bonuses today (`attack_line(state, weapon)` has no `start` already). If 139 has landed by then, its weapon bonuses use the same list.

## Build

AFK. A refactor: the numbers don't change.
- Blocked by [148](148-naked-numbers.md), which removes `assumed_totals`, `assumed_caster_line` and `Assumed` from these files, and by [150](150-the-walk-judges-every-choice.md), which also reshapes the walk. Run it after both.
- Run it before 139, 141 and 143 where the queue allows, so their sums go into the one list.
- No schema, data or link-format change.
- The tests that built `CharacterState` by hand build a Build and walk it instead, through `Engine` (`evaluate`, then `totals` or `caster_line` on a level's state). They keep their assertions.

## Done when

- `Engine::totals` and `Engine::caster_line` take no `BuildStart`.
- `caster.rs`'s `holdings()` is gone, and `totals.rs` imports nothing private from `caster.rs`.
- No test of totals or the caster line builds a `CharacterState` by hand.
- `cargo test --workspace -- --list` lists the same tests, plus any new ones.
- `scripts/proof.sh rules` gives an empty comparison. If the proof doesn't record totals or caster lines, add them to its record first (one commit of its own) and say so.
- `scripts/check.sh` passes.

## Progress log

- 2026-11-11, built by an agent in its own worktree (branch `worktree-agent-a214ec48dab52e680`, fast-forwarded to `main`'s `c98154f`; no other agent on the repo). Baseline: `cargo test --workspace -- --list` saved (1,162 lines with the crate headers), and `c98154f` exported to the scratchpad (`t151/before`) with ticket 150's bench harness for the timing.
- The proof recorded neither totals nor caster lines, so they went into its record first, in a commit of their own (`668f2ba`): `totals`, every sheet level's `Engine::totals`, and `caster_lines`, every level's `Engine::caster_line` for 27 spells (each school, each subschool, each energy type as an evocation, one with metamagic), each the hash of their Debug texts.
- The refactor, in one commit: the walk's new `walk/bonuses.rs`, `CharacterState::bonuses_in_force` (`rules::BonusInForce`), the two lines over it, their callers, and the tests that built a `CharacterState` by hand now walking Builds. The proof harness compiles against both sides of the change (`tools/proof/build.rs` sets `cfg(lines_take_start)` before ticket 151) and hashes the sheet without the new field.
- The first version walked every bonus of the data at each level's end and doubled `Engine::evaluate` (71 → 145 µs on the bench Builds). Now the walk notes a bonus's row as it holds its item (the start's race, subrace and background at the start) and looks only at those, and a level that finds the same bonuses as the last shares its list (`Arc<[BonusInForce]>`): 70 → 75 µs. An FNV hasher for the catalog's new index and keeping the new code out of line were tried and gained nothing.
- Checks, proof, agreement and timing: see the Answer.
- 2026-11-11, the code review's fixes, by an agent in the same worktree (no other agent on the repo), test-first where they change behaviour; see the Answer's "Review fixes". The two lines take a level of the sheet (`&LevelSheet`), so a state without its bonuses in force can't reach them; the walk builds a changed level's list once, from a sorted list of noted rows and a reused key buffer, reusing the last level's entries; the catalog indexes the bonus rows by one `BonusItemKind` derived from `BonusItem`; the saves test is a fighter 5 / rogue 1; the proof's build script reads the two functions' parameters. Timed against `c98154f` before merging `main`.

## Answer

**Done.** The walk records the Item bonuses in force on each state of the sheet, and the caster line and the totals add up that list: `Engine::totals(level)` and `Engine::caster_line(level, spell)`, for a level of the sheet (the review's fix 1). The numbers don't change (the proof, below), and no schema, data or link format changed.

### The record

- **`CharacterState::bonuses_in_force: Arc<[BonusInForce]>`**, in the data's order. A `rules::BonusInForce` is `{ item: BonusItem, value: Option<Parameter>, target: BonusTarget, amount: i16, name: String }`: one per bonus whose item the character holds and whose `requires` it meets, once for each value its item is held for. Expanded Arsenal's entry carries its copy as its amount, and its name says which school it copies; one that copies nothing isn't in force.
- **The name**: "bonus in force", next to the glossary's **Item bonus** (the row of `bonuses.ron`) and read as "the Item bonuses in force". `GLOSSARY.md` gains **Bonus in force**, since `sheet` callers of `rules` see the field; **Item bonus** now says the lines sum the bonuses in force.
- **Where**: `engine/walk/bonuses.rs`. At each level's end of a walk that keeps the sheet (`Engine::evaluate`), the walk finds the bonuses in force among those it noted as it held their items (`Walker::note_bonuses`, from `hold`; the Build start's race, subrace and background at the start), through the catalog's new index of bonus rows by item kind and key (`Catalog::bonus_rows`). While each entry it finds matches the last level's (its row, value, amount and copied school), the last level's list stands and is shared; from the first that doesn't, it builds a new list once, reusing the last level's entry (its name) for each bonus it had. A walk without the sheet (a picker's, a recheck's) leaves the list empty, and so does the state just before a slot (`state_before`); the field's docs say so.
- **`requires` moved into the walk**: it's checked at each level's end against the state there, so the last prerequisite check outside the walk is gone (ticket 150's review).
- **`caster.rs`'s `holdings()`, `held_bonuses()`, `amount()` and `modifier()` are gone.** `totals.rs` imports nothing from `caster.rs`: both read the ability modifier through `AbilityScores::modifier`. The caster line treats Expanded Arsenal like any other spell DC bonus of `SpellScope::Value`.

### Read time or walk time

**Every amount is worked out in the walk**, the ability modifiers included. The list is recorded at the level's end, when the state's ability scores, Mythic rank and ranks are the ones the state holds, so a modifier worked out there is the one read time would give: there is no later "final" score for that state. Worked out once, each entry is a number, so the lines only pick and add, and ticket 144's "the highest per type and target" compares numbers in one list. Tickets 139 and 141 fit the same shape: a weapon or grip limit is a target and a value read when the line is asked for, and a class-level amount is worked out at the level's end like the Mythic rank.

One consequence, recorded as a data rule: a ten-ranks amount is worked out for one skill, so `GameData::check` now refuses `TenRanks` on `AllSkills` (`DataIssue::TenRanksWithoutSkill`, "needs one skill as its target"). The data has none, so nothing changes; the old code worked it out per skill.

### Interfaces

- `Engine::totals(&LevelSheet) -> Totals` and `Engine::caster_line(&LevelSheet, SpellTraits) -> Option<CasterLine>`, for a level of the sheet.
- Callers: `apps/mcp/src/{totals,caster,compare,rank,sheet}.rs` pass the sheet's level; `sheet` gives no `totals` for a Build with no level yet; `rank::Number::of` lost its `Build` argument, which it read only for the start. `libs/ui` calls neither. The seeds real-data tests and the proof harness are updated too.
- New public: `rules::BonusInForce`, `CharacterState::bonuses_in_force`.

### Tests

- **Walked instead of hand-built**: every test of `libs/rules/tests/engine/totals.rs` and `caster_line.rs` that built a `CharacterState` now builds a Build, checks it holds up (`support::walked`, `last_level`: `Engine::failures` empty) and reads a level of `evaluate`'s sheet. They keep their assertions, with three adjustments the walk forces: the saves test is a fighter 5 / rogue 1 (base saves 4/3/1, totals 11/10/7 rather than 13/9/9, each save's its own, so swapping two would fail: no class mix gives 6/2/3 at the level 6 a mythic ability needs); Perfect Mind comes from a real Gold Dragon switch, whose ability bonuses give Strength 24 and Dexterity 18 (athletics 22, mobility's Dexterity +4); the requirement tests read two levels (initiative 2 at fighter 7, 6 at fighter 8; Spell Penetration (Mythic) at a sorcerer 16, 19 and 25). Expanded Arsenal's test lets Spell Focus repeat per school, and Heighten Spell's adds the feat to the test world.
- **New** (3): `bonuses_in_force::each_state_of_the_sheet_records_the_bonuses_held_there_with_their_item_value_and_target`, `bonuses_in_force::a_bonus_is_in_force_once_its_requirement_holds_with_its_amount_at_each_level`, `totals::the_data_check_refuses_a_ten_ranks_bonus_on_every_skill`. `cargo test --workspace -- --list`: the same tests, plus these three.
- **New in the review fixes** (2): `bonuses_in_force::a_level_with_the_last_levels_bonuses_shares_its_list_and_one_that_changes_keeps_the_rest` (rules) and `totals::a_build_without_a_level_has_no_totals` (MCP server).

### Review fixes

1. **The lines take a level of the sheet.** `Engine::totals(&LevelSheet)` and `Engine::caster_line(&LevelSheet, spell)`: only the sheet's states record the bonuses in force, so a state from `state_before` or one built by hand gave a DC without Spell Focus and no error; now it doesn't type-check. Their docs say why, and the field's docs point to it. Every caller held the level already (`sheet`, `compare`, `rank`/`improve` through `Number::of`, the seeds tests, the proof). One caller didn't: `sheet` gave totals for a Build with no level from a default state (a hand-built one); now `totals` is absent there (`SheetResult::totals: Option`, skipped when `None`; test `a_build_without_a_level_has_no_totals`, red first).
2. **One list, built once.** `Found<V>` and `Walker::bonuses_found` are gone: a level's end compares what it finds with the state's list itself, with a small `(row, copied school)` key kept beside each entry (`Walker::bonus_keys`), and builds the new list only on a change.
3. **Cheaper:** the noted rows are a sorted `Vec<usize>` (no `vec![false; bonuses]` per walk, no scan of every bool at each level's end); the key buffer is reused between levels; on a change, each entry the last level had is cloned with its new amount rather than named again with `format!`; Expanded Arsenal's Spell Focus totals are worked out only when an Expanded Arsenal entry is met.
4. **`BonusGiver` and `BonusRows`' eight maps are gone**: `BonusItem::kind_and_key` gives a `BonusItemKind` (the variant without its key) and the key's text, and the catalog indexes rows by kind (a `Vec` slot each) then key text (`HashMap<String, _>`, looked up by `&str`). The walk still builds no `BonusItem` (no key clone) as it holds an item, which was the speed reason. A new giver is now a `BonusItem` variant, a `BonusItemKind` variant and an arm of `kind_and_key` (all in `bonus.rs`), plus where the walk holds it.
5. **The saves test** is a fighter 5 / rogue 1: base saves 4/3/1 and totals 11/10/7, so swapping any two saves fails.
6. **The proof's build script** reads the parameters of `pub fn totals(` and `pub fn caster_line(` wherever `libs/rules/src` defines them (whitespace ignored): `&BuildStart` and `&CharacterState` → `cfg(lines_take_start)` (before ticket 151), `&CharacterState` alone → `cfg(lines_take_state)` (this ticket's first commits), `&LevelSheet` → neither. The two must agree; anything else, or no such function, fails the build with what it found. Checked against every commit of `main` that changed the two signatures: it reads back to ticket 120 (`ba47fc9`), which brought the totals. Found on the way: `proof.sh` reuses `$CARGO_TARGET_DIR/A` across refs, and `git archive` dates a ref's files at its commit, so a run against an older ref after a newer one can build side A from the newer ref's stale units; delete `A/` when switching refs.

Left as they are: the `TenRanks`-on-`AllSkills` refusal in `check.rs` (all 10 `TenRanks` rows of `bonuses.ron` name one skill), and the UI's `evaluate` recording bonuses it doesn't read (recording only when asked would bring back states without their bonuses, which fix 1 rules out).

**Timing after the fixes** (ticket 150's harness, native release, 41 runs a median, `c98154f` and this tree alternated twice, load about 4): an edit is within the noise between runs: fighter20 0.384/0.401 vs 0.390/0.404 ms, fighter20_nopath 0.260/0.260 vs 0.266/0.267, multi_prestige20 0.876/0.892 vs 0.901/0.881, legend40 0.992/1.000 vs 1.012/1.016; the pickers within 1–2%. `evaluate` alone (the bench Builds in a loop, the best of six alternations): 69.3 µs at `c98154f`, 73.7–75.4 µs now, as before the fixes. Switching the noting and recording off puts it back at 69–70 µs, about 2 µs each; the bonus code's own samples are about 1.5% of a profile, the rest in what it calls (`check`, the holdings' map lookups). A faster (Fx-style) hasher for the catalog's index gained nothing, as FNV didn't before.

### Done when

- `Engine::totals` and `Engine::caster_line` take no `BuildStart`: yes.
- `holdings()` gone from `caster.rs`, nothing private imported by `totals.rs`: yes.
- No test of totals or the caster line builds a `CharacterState` by hand: yes (`rules`' engine tests; the seeds and MCP tests already walked).
- The test list: the same 1,162 lines plus the three new tests (1,165).
- **The proof**: the totals and caster lines went into the record first (`668f2ba`, its own commit). `CARGO_TARGET_DIR=$TMPDIR/wotr-proof-151 scripts/proof.sh rules <out>` against it, on the finished tree: 2,163 Builds a side, 0 differing, `compare.txt` empty. The new fields aren't constant (594 distinct `caster_lines` hashes and 1,373 `totals` hashes on side B).
- **The agreement test** (ticket 150): the suite tests pass, and the test world's full sweep in release found 0 disagreements (2,387 Builds, 2,878,430 options, 23 s).
- **Timing** (ticket 150's harness, native release, 41 runs a median, the baseline `c98154f` and this tree alternated twice). The level-1 feat picker (`opt-L1`) is unchanged (0.754/0.753 vs 0.764/0.758 ms on fighter20; the rest within 1%). An edit costs 0–4% more: fighter20 0.384/0.386 vs 0.395 ms (one outlier run at 0.479), fighter20_nopath 0.260/0.260 vs 0.266/0.266, multi_prestige20 0.888/0.870 vs 0.887/0.888, legend40 1.000/0.999 vs 1.024/1.046. That's the sheet walk's recording, about 5 µs of `evaluate`'s 70: at the edge of the noise between runs, measurably above it on legend40.
- `scripts/check.sh`: ALL-OK (fmt, clippy, wasm-clippy, test, rules-wasm, seed-check).
