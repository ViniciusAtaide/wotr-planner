# The walk judges every choice: a cursor for the pickers

Type: design
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

From the architecture review of 2026-11-11 (candidate 1, the top recommendation, *Strong*). The user asked for a ticket for each strong candidate.

The planner holds only possible Builds ([ticket 39](39-impossible-states-not-validation.md)): a picker offers only what the walk accepts. That promise rests on two implementations agreeing. The walk (`engine/walk.rs`, `walk/*`) judges a Build, and the pickers (`engine/pickers.rs`, `slots.rs`, `removable.rs`) build each option's verdict again from three parts:

1. **A probe for the state.** `self.probe(build, Probe::Slot(slot))` or `Probe::Level(level)` pauses the walk and returns its state (`pickers.rs:33`, `pickers.rs:342`).
2. **A check of their own:**
   - `check_pick(&Catalog, state, slot, pick)` for slot options (`slots.rs:213–342`). It picks one of `CharacterState`'s four check modes (`check`, `check_for`, `check_feat_for`, `check_option`; `state.rs:225–280`) at the call site (`slots.rs:326–341`).
   - `class_status` for classes (`pickers.rs:384–411`).
3. **`Recheck::later(&edit)`** for "breaks a later choice". It clones a paused `Walker`, overwrites `walker.build` and drives `walker.start()` and `walker.level()` itself (`removable.rs:308–356`).

**Where they already disagree in shape.** Class entry is judged twice, and each version has checks the other lacks:

| | `walk/level.rs:24–62` `check_class_entry` | `pickers.rs:384–411` `class_status` |
| --- | --- | --- |
| Class and archetype `entry_prerequisites` | ✓ | ✓ |
| Unknown class or archetype key, archetype of another class (`NotAllowed`) | ✓ | (only offers the class's own) |
| `advances` | ✓ | (`advances_options`, separately) |
| Class level cap | elsewhere in the walk | ✓ |

Whether that disagreement is a bug today or only a hazard is part of the ticket's first step.

**The `Walker` (`walk.rs:156–200`, about 25 fields) runs in five modes:** evaluate (the sheet), `failures`, `is_possible`, the three probes (`Slot`, `Level`, `Later`), and a recheck (`recheck: Option<RecheckWalk>`, `sheet: bool`). The probe hooks are interleaved into `fill` (`walk/hold.rs:40–59`) and `level` (`walk.rs:539–590`). Tests reach the paused state through a `#[cfg(feature = "testing")]` back door, `Engine::state_before` (`walk.rs:71–75`).

**The shape the review suggested** is a cursor on the walk:

- `until(slot | level)`: the walk paused just before a slot (or a level's class), with its state and slot kind.
- `try(edit)` on the paused walk: the candidate's verdict, `Result<(), Issue>`. This covers the slot's own check, in the mode the slot needs, and whether it breaks a later choice.

The pickers would ask the paused walk for each candidate's verdict and keep only presentation (grouping, names, chips, summaries: [ticket 97](97-slot-options-presented.md)'s `slot_entries`). Class entry would have one implementation, inside the walk. `Recheck` would use the cursor rather than `Walker`'s fields.

### The design questions (grill the user; take the recommendation where the user gives none)

1. **Is the cursor the right seam?** Alternatives:
   - **A.** The cursor above: `until` plus `try` on a paused walk. *Recommended*: one module judges, and the pickers keep their speed, because the walk to a slot still runs once per picker.
   - **B.** Keep the probe, but move every check the pickers do (`check_pick`, `class_status`, the check mode) into `walk/` as the walk's own functions, which the pickers call. This is a smaller change, but the pickers still assemble legality from parts.
   - **C.** `try(edit)` by re-walking the edited Build for each candidate (`Engine::allows`). This is simplest, but it costs a walk per candidate, and ticket 48's benchmark will refuse it for a feat slot's several hundred candidates.
2. **What the cursor returns.** The paused state for presentation (the pickers' `slot_lists`, `pools` and steppers read it), or only verdicts, with what the presentation needs given by the cursor too? *Recommended:* the state stays readable (`&CharacterState`), and only verdicts go through `try`.
3. **The walk's modes.** Should the `Walker` split along the modes, for example a judging core with the sheet's rows and the probe as observers? Or should the cursor sit over today's `Walker`, with the split left for later? Answer after step 1 of the Build. Split only what the cursor needs.
4. **Tests.** The test-world tests (`libs/rules/tests/engine/`, 54 files) already go through `Engine`'s public interface. Should the cursor be public, so tests can judge a candidate at a slot directly and `state_before` goes? *Recommended:* yes, if the UI or the MCP has a use for it too; otherwise `pub(crate)`, with `state_before` kept as it is.

Use `grilling` and `domain-modeling`, as the map's Notes say for design tickets. Name the cursor in `GLOSSARY.md` if it becomes a public engine term.

### Findings for the design

From Build step 1 (2026-11-11, branch `worktree-agent-ac3a9c9d58a333ac4`), before the user's answers. Nothing in the engine changed.

**The oracle.** `rules::agreement` (behind the `testing` feature, public engine interface only) judges every picker of a possible Build: an option's verdict is whether the Build with its edit applied **is possible**. `Engine::allows` alone can't be the oracle: it excuses the edit's own choices (`Edit::choices`, "the choices it sets are the pickers' business"), so it accepts a feat whose prerequisites aren't met. The check also flags `BreaksLater` on an edit `allows` accepts. Pickers judged: race, faith, alignment, background, racial ability, point-buy steps, each level's class (and one past the end while the cap allows), advancement, ability increase, skill steps, every slot of the sheet's levels and reached ranks, and the first ascension, path and switch once their rank is reached (as the MCP offers them). A prestige class that asks what it advances is judged as the UI and MCP take it, through `advances_options`, against advancements the check lists itself (every set of at most two of the Build's classes, each judged by a walk): the class is open exactly when one keeps the Build, and `advances_options` offers exactly those that do. An open option whose edit leaves the Build unchanged (a wrong level or slot, which `apply` drops) is a disagreement too, unless it's the pick the Build holds there.

**Disagreements: one family.**

| Corpus | Builds | Options judged | Disagreements |
| --- | --- | --- | --- |
| Test world, grown through the pickers (23 plans × 3 seeds, every level) | 2,387 | 2,878,430 (2,318,667 before the review fixes) | 88, all disagreement 1 (7 pins) |
| Test world, the suite's sample (seed 1, start and levels 1, 2, 6, 9) | about 200 | > 50,000 | 2 (of the 88) |
| Real data, ticket 48's six bench Builds, whole | 6 | — | 0 |
| Real data, the proof's Builds (`builds::all`: its own, the bench's and the full ones) | 2,003 | 24,180,268 (23,782,814 before the review fixes) | 0 |

**Disagreement 1: the class picker locks a prestige class an advancement would keep.** At a level the Build has, `class_options` judges a prestige class that asks what it advances with the edit carrying the Build's own advancement for it (`held_advances`: none, for a class the Build doesn't take). Taking it with no advancement drops the caster level of the class it replaces, which breaks a later pick, so the option is `BreaksLater` and greyed out; but advancing that class keeps the Build, and `advances_options` offers it. The user can't reach the advancement picker from a locked option. The first ones, verbatim:

```
bard #2 level 10: class_options 5 winter-witch (asks advances): picker Err(BreaksLater), walk keeps, allows false
cleric #2 level 5: class_options 4 winter-witch (asks advances): picker Err(BreaksLater), walk keeps, allows false
eldritch knight #1 level 9 unfilled: class_options 6 winter-witch (asks advances): picker Err(BreaksLater), walk keeps, allows false
```

What the walk says of the raw edit (cleric #2 level 5, Winter Witch at level 4 with no advancement): `Slot(Feat { level: 5, source: General, index: 0 }): Unmet(CasterLevel(5), has 4)`; `advances_options` keeps `{cleric}`. All 88 are Winter Witch (the test world's easiest prestige entry: arcane *or* divine 2nd-level spells) at levels 4–7 of bard, cleric, eldritch knight and mystic theurge Builds. The real data's Builds (the bench's and the proof's 2,003) show none, though nothing in the code keeps the case out of the real data: their Builds never replace a caster's level with a prestige class that would need to advance it. **Ask the user** whether the class picker should mark such a class open when any advancement keeps the Build (and the advancement picker list only those), or keep today's answer.

**The class-entry split: a hazard, apart from the advancement (a bug, disagreement 1).** Evidence from a probe over the test-world corpus (2,002 Builds, levels 1–14, every level's class picker and one past the end; 543,376 class options):

- Entry prerequisites, the class level cap (64 options capped) and the archetype: no disagreement. `class_status` and `check_class_entry` both run `Catalog::entry_prerequisites` against the same state (the probe at `Probe::Level` is taken just before `check_class_entry`), and the walk's `ClassLevelCap` (`walk/level.rs:154`) fails the same `Choice::Class { level }`.
- The walk's `UnknownKey` and `NotAllowed` (an archetype of another class) can't come from a picker: it offers only the data's classes with their own archetypes.
- **The hazard:** `Recheck::later` excuses the edit's own choices, which for a class first taken at the level include `Choice::Advances`. `class_status` doesn't check the advancement, so a class option whose edit carries an advancement that doesn't fit at that level (the advanced class not yet taken there) would be marked open with an edit the walk refuses. Of 80,986 class options that ask advances, **0** were open with a raw edit the walk refuses: the UI and MCP never apply that edit (they go through `advances_options`, which filters by `is_possible`), and in the corpus an entry that's met already holds the advanced class. So today it's a hazard; the cursor's one class-entry check (with `check_advances`) removes it.
- **The bug** is the reverse direction (disagreement 1): the advancement belongs to class entry in the walk (`check_class_entry` calls `check_advances`), but the picker judges a single advancement, not the best one.

**Q1, the seam (A, B or C).** *Recommendation: A, over today's walk, with `try` level-granular (as `Recheck` is).*
- Slot picks already share their check: the walk's `fill` calls the same `slots::check_pick` the picker calls (`walk/hold.rs:61`, `pickers.rs` through `check_pick_with`), so the slot pickers can't disagree on a pick's own check; no slot disagreement anywhere. The second implementations are `class_status` (class entry) and `Recheck` (the later choices), plus the shortcuts (`named_feats`' left-empty answer, `met_later`'s reasons).
- **Option C, timed** (`libs/seeds/tests/game_data/agreement.rs`, `a_walk_per_candidate_against_the_feat_picker`, release, M-series Mac): one feat slot's 749–896 candidates.

  | Build, slot | Picker (`options`) | A walk per candidate (`is_possible`) | `allows` per candidate |
  | --- | --- | --- | --- |
  | fighter20, level 1 class slot | 1.0 ms | 7.9 ms | 31 ms |
  | fighter20, level 20 class slot | 0.5 ms | 28 ms | 70 ms |
  | multi_prestige20, level 1 race slot | 1.0 ms | 11 ms | 54 ms |
  | multi_prestige20, level 20 option feat slot | 0.7 ms | 44 ms | 108 ms |
  | legend40, level 1 class slot | 3.4 ms | 14 ms | 62 ms |
  | legend40, level 39 general slot | 0.7 ms | 66 ms | 195 ms |

  C costs 8× to 90× the picker (a walk is 10–75 µs, but a slot has 750–900 candidates); in the browser's wasm, slower again, against ticket 48's 5 ms. C is too slow as the pickers' path, but it's the oracle: this test is C.
- B keeps the parts assembled in the pickers (the check mode, `held_advances` with `Recheck`), which is where disagreement 1 lives.
- A's `try` can't resume mid-level: `Walker::level` walks a whole level in one call (`walk.rs:538–603`; the slots of the level are worked out at its start, `level_slots`, `grant_selections`, `pending`). So `until(slot)` gives the state (today's probe) and `try(edit)` resumes from the level's start on the edited Build (today's `Recheck::walks`). A slot-granular resume would need `level` as a resumable state machine: a much larger change, not needed for the speed.

**Q2, what the cursor returns.** *Recommendation as written: the state stays readable, verdicts through `try`.* What the pickers read off the pause is only `CharacterState` and the slot's `SlotKind` (`Probed`), plus the checkpoints' states for `met_later` and `Walker::skill_points` for the skill steppers (`Recheck::skill_points`, `removable.rs:302`). Presentation reads the state: `slot_lists`, `pools`, `reason_for`, `class_options`' `state.classes` (a class taken keeps its archetype). None of the walk's other ~22 fields (holdings, pending, rank grants…) is read outside `walk/`.

**Q3, the walk's modes.** *Recommendation: the cursor over today's `Walker`; no split.* `Recheck` uses five things of `Walker` (`removable.rs:308–356`): `Walker::new(..).without_sheet()`, `start()`, `level()`, `holds()`, a `clone()` per level, and two fields: it **writes `walker.build = edited`** (`removable.rs:352`) to resume a paused walk on the edited Build, and reads `walker.skill_points` (`removable.rs:304`). That is a level-granular cursor already: `until(level)` is `Recheck::paused`, `try(edit)` is `Recheck::walks`. Moving those into `walk/` behind `until`/`try` (and a `skill_points` getter) meets "Recheck reads no Walker field" without splitting the `Walker`. The probe hooks (`hold.rs:40–59`, `walk.rs:539–590`) stay as they are for `until(slot)`.

**Q4, tests and visibility.** *Recommendation: `pub(crate)` now, `state_before` kept.* `state_before` has 7 calls in 3 test files (`backgrounds.rs`, `path_rules.rs`, `first_ascension.rs`). Neither the UI nor the MCP judges a single candidate: both list a picker (`slot_entries`, the MCP's `offers`) and apply an option's edit; the MCP's `choose` lists the whole slot to take one value, then reopens the result as its final gate (`choose.rs`), which is option C for one candidate. A public `try` could save `choose` that listing, but that's a speed-up nobody has asked for; the agreement test needs only the public pickers and `is_possible`.

## Build

Once the design is decided. A refactor: the engine's answers don't change.
1. First, a test that pins today's agreement. For every test-world Build and every slot and level, each option a picker marks open must make an edit `Engine::allows`, and each one it marks closed must not.
   - Run it on the real data too (`libs/seeds/tests/game_data/`) over ticket 48's Builds.
   - Every disagreement it finds is a bug: record it in the Answer and ask the user before changing behaviour.
   - This test is the oracle the refactor keeps green.
2. Then the cursor, one picker family at a time: slots, then classes, then paths, first ascension and switch, then the start pickers ([ticket 96](96-start-and-level-pickers.md)).
- No schema, data or link-format change. `Engine`'s public interface stays the same, apart from what the design adds. If something has to move, say why.
- [Ticket 151](151-bonuses-in-force.md) also changes the walk: run the two one after the other, not in parallel.

## Done when

- Class entry is judged in one place, and a grep finds no prerequisite check (`state.check*`) outside `walk/` and `state.rs`.
- `Recheck` reads no `Walker` field.
- The agreement test from step 1 passes on the test world and the real data.
- `cargo test --workspace -- --list` lists the same tests, plus the new ones.
- `scripts/proof.sh rules` gives an empty comparison (or only the disagreements the user agreed to fix).
- Ticket 48's benchmark shows an edit and the level-1 picker no slower, within noise.
- `scripts/check.sh` passes.

## Progress log

- 2026-11-11, Build step 1 (an agent in its own worktree, branch `worktree-agent-ac3a9c9d58a333ac4`, fast-forwarded to `main`'s `4b1f164`; the design questions still wait on the user). The agreement test, test-first:
  - `libs/rules/src/agreement.rs` (`rules::agreement`, behind the `testing` feature): `judge(&Engine, &Build) -> Agreement` judges every picker of a possible Build against the walk, through the public interface only; `judge_one` judges one option. See Findings for what the verdict is and why it isn't `allows` alone.
  - `libs/rules/tests/engine/agreement.rs` (`mod agreement;`): a corpus grown through the pickers (23 plans: each base class to 20, the five prestige classes, multiclass and Legend Builds; Build start, class, Mythic choices, ability increase, skill ranks and every slot, each a choice the pickers open and the walk keeps). The suite judges a sample (seed 1, Build start and levels 1, 2, 6, 9: about 7 s in a debug build); `every_picker_agrees_with_the_walk_on_the_whole_test_world_corpus` (three seeds, every level: 2,387 Builds, about 22 s in release on every core) is `#[ignore]`d. `the_check_names_an_open_option_the_walk_refuses` tests the check itself.
  - `libs/seeds/tests/game_data/agreement.rs` (`mod agreement;`): ticket 48's six bench Builds, cut to their first ten levels in the suite (about 5 s in a debug build); whole, `#[ignore]`d (about 40 s in a debug build, a few seconds in release); and the option C timing, `#[ignore]`d. The bench Builds' loader moved from `build_sheets.rs` to the tests' `support.rs`.
  - `tools/proof/tests/agreement.rs`: every Build `proof record` records (`builds::all`, 2,003 possible), `#[ignore]`d (about 7 minutes in release on 8 threads); `AGREEMENT_ONLY` and `AGREEMENT_THREADS` narrow it. `proof`'s dev-dependencies turn on `rules`' `testing`.
  - Each test pins the disagreements it finds in a `KNOWN` list (`// ticket 150: disagreement 1`), so it stays green until the user decides.
  - Findings: one disagreement family (the class picker and a prestige class's advancement), the class-entry verdict, the option C timing and evidence for the four design questions, under the Question.
- 2026-11-11, Build step 1's review fixes (an agent in the same worktree; nothing in the engine changed):
  - The advancement oracle is the check's own: every set of at most two of the Build's classes, each judged by `is_possible`, rather than `advances_options`' own candidates. A prestige class that asks advances is open exactly when one keeps the Build; `advances_options` must offer exactly those (a kept advancement it misses is "picker doesn't offer it"). The redundant walk per offered advancement is gone: an answer is judged by its own edit only when that edit makes another Build than the check's.
  - An open option whose edit leaves the Build unchanged is a disagreement unless it's the pick the Build holds there (each picker's held value: the start's fields, the level's class with its archetype, its ability increase, the slot's pick, the paths; a step is never held). The pickers mark the held pick `Ok` with a no-op edit; none marks any other no-op open. New test: `the_check_names_an_open_option_whose_edit_changes_nothing`.
  - `KNOWN` is pinned by content: the test world's Builds are keyed by their plan's name, not by seed and corpus position, so the 88 cases of disagreement 1 are 7 lines (plan, `class_options` level, option, verdicts); `judge_all` returns a `Sweep` (options judged, disagreements, deduplicated lines). The suite's sample checks that each line it finds is pinned; the whole corpus, that it finds exactly the pins. The proof's sweep filters its pins with the shared `known_for`, by the names of the Builds it sweeps.
  - Counts: no new disagreement. Test world 2,387 Builds, 2,878,430 options, 88 disagreements (7 pins), 40–53 s in release on a machine at load 130–250 (other agents building; about 30 s expected unloaded, from 22 s and the 24% more options); bench Builds whole, 0 (6 s in release); the proof's 2,003 Builds, 24,180,268 options, 0 disagreements, 372 s in release on 8 threads (same load). `scripts/check.sh`: ALL-OK.
- 2026-11-11, Build step 2, the cursor (an agent in its own worktree, branch `worktree-agent-a9b2f2f44e2b66570`, fast-forwarded to `main`'s `32a1ab0`; ticket 152 was built at the same time in another worktree, in `apps/mcp`, `labels.rs` and `build_sheet.rs`, which this step doesn't touch). The user said not to stop, so the Findings' recommendations were taken (Answer). Baseline: the workspace's test list (`cargo test --workspace -- --list`, 1,121 lines) and a `git archive 32a1ab0` copy for the benchmark, both in the scratchpad (`t150/`).
  - Built in one pass and committed as one (`c3d9a9d`), since the families share the cursor's walks: the cursor and the walk's own checks (`walk/cursor.rs`, `walk/judge.rs`), then the slot pickers, the class pickers (with class entry judged once), the paths, the start pickers and `Recheck`'s users moved onto it. The agreement suite was green after it; the whole test-world sweep then found no disagreement, so the 7 pins of disagreement 1 came off.
  - New test: `advances::the_class_picker_opens_a_prestige_class_that_some_advancement_keeps` (a cleric 5 with Spell Penetration at level 5: Winter Witch at level 4 is open, and only advancing the cleric keeps the Build). It fails on `32a1ab0` (`Err(BreaksLater)`).
  - Checks, the proof and the benchmark: see the Answer.

## Answer

**Decided 2026-11-11: the recommendations, taken under the user's "don't stop until all tasks are done".**
1. Q1: option A, a cursor over today's walk. `until(slot | level)` pauses the walk, and `try(edit)` gives a candidate's verdict, `Result<(), Issue>`. The check is level by level, since `Walker::level` can't resume mid-level. It covers the slot's own check, in the mode the slot needs, and whether the edit breaks a later choice.
2. Q2: the paused `&CharacterState` stays readable for presentation (slot lists, pools, the class picker's taken classes, the skill steppers' points). Only verdicts go through `try`.
3. Q3: no `Walker` split. What `Recheck` did moved behind the cursor.
4. Q4: the cursor is `pub(crate)`, and `Engine::state_before` stays (now `until_slot`'s state). `GLOSSARY.md` is unchanged: the cursor isn't a public engine term.
5. Disagreement 1 is fixed: a prestige class that asks for advances is open when some advancement keeps the Build. This is the one behaviour change.

**Done.** Every verdict a picker gives is the walk's, asked of the walk paused at the picker's slot or level. The pickers list candidates and present them. Class entry has one implementation. `Recheck` is gone; its walks are the cursor's. The agreement test finds no disagreement anywhere, and its pins are gone.

### The cursor (`engine/walk/cursor.rs`, `pub(crate)`)

- **`Cursor<'a> { engine, build, paused: RefCell<Vec<Walker>> }`**: the Build's walk paused before each level, filled as far as the questions asked need. This is `Recheck`'s store, moved. Build one per picker.
  - `until_slot(&SlotId) -> Option<AtSlot>` clones the walk paused before the slot's level, then walks that level with the slot probe. The old probe was a separate full walk with the sheet.
  - `until_level(u8) -> Option<AtLevel>`: the state off the paused walk. One past the last level gives the state after it.
  - `try_edit(&Edit) -> Result<(), Issue>`: an edit's own check, then `BreaksLater`. The own checks are: a start's `faith_misfit`; the first ascension's, path's and switch's `*_fit` (so the switch picker's `NotFromPath` is now `switch_fit`'s); and a pick or a class, routed through `until_slot`/`until_level`.
  - `keeps(&Edit, allowed)`: what `allows` and the withdrawals ask.
  - `skill_points(level)`.
  - Private: `later`, `later_walked`, `walks`, `first_level` (moved from `removable.rs`), `named_feats` (from `pickers.rs`) and `level_of`.
- **`AtSlot`**: `state()`, `kind()`, `lists()` for presentation.
  - `try_pick(&Pick)`: `check_pick_with` against the paused state, with `MetLater` on an unmet prerequisite, then the later walk. A feat no check names shares the slot-left-empty walk.
  - Private: `met_later`, `later_pick` and `later_picks`. The "without this pick" probe now walks only the slot's level, from the paused walk.
- **`AtLevel`**: `state()`, and `try_class(&Edit)`. It checks, in order:
  - the cap (`ClassLevelCap`);
  - on first entry, `class_entry(..).issue()`;
  - past the last level, nothing more;
  - for a prestige class that asks advances, whether any of `advancements(..)` keeps the Build (the edit's own advancement first), walked with nothing excused, so the walk's `check_advances` judges it;
  - otherwise, whether the edit breaks a later choice.

### What moved where

| From | To |
| --- | --- |
| `slots.rs`: `check_pick`, `check_pick_with`, `check_repeat`, `check_deity_domain` | `walk/judge.rs` (`pub(super)`). `slots.rs` keeps the `impl Catalog` of what a slot accepts. |
| `walk/level.rs` `check_class_entry`'s prerequisite part, and `pickers.rs` `class_status` | One `judge::class_entry(&Catalog, &state, &Class, archetype) -> Entry { class, archetype }`. The walk fails `Choice::Class`/`Choice::Archetype` with its parts, and `Entry::issue` joins them for a class option. |
| `pickers.rs`: `advances_options`' candidate sets and `subsets` | `walk/level.rs` `advancements(build, level, prestige)`, shared by the cursor and `advances_options` |
| `pickers.rs`: `later_picks`, `met_later`, `named_feats` | `AtSlot` and `Cursor` |
| `removable.rs`: `Recheck`, `Engine::first_level` | `Cursor` |
| `walk.rs`: `Engine::probe`, `Probe::Level`, `Walk::probed`, `Walker::finish` | Gone. `Probe::{Slot, Later}` and `Probed` stay, private to `walk`, set only by the cursor. `Walker`, its fields (`build`, `skill_points`) and its methods are private to `walk/`. |

`Engine`'s public interface is unchanged. Only docs changed: `class_options` and `ClassOption::status` describe decision 5.

### Done when

- **Class entry in one place:** `judge::class_entry`. A grep for `state.check*` outside `walk/` and `state.rs` finds only `engine/caster.rs:218`: an item bonus's `requires`, which is a bonus's condition, not a choice's prerequisite. Ticket 151 moves the bonuses in force into the walk.
- **`Recheck` reads no `Walker` field:** `Recheck` is gone, and only `walk/` reads `Walker`.
- **Agreement:**
  - The suite passes.
  - Test world, whole corpus, release: 2,387 Builds, 2,878,430 options, **0 disagreements** (was 88 in 7 pins), 28 s.
  - Real data, the bench Builds whole and the option C timing, release: 0 disagreements.
  - The proof's Builds (`tools/proof`'s sweep), release on 8 threads: 2,003 Builds, 24,180,268 options, 0 disagreements, 218 s.
  - Every test's `KNOWN` is empty.
- **Test list:** the same as `32a1ab0`'s, plus `advances::the_class_picker_opens_a_prestige_class_that_some_advancement_keeps`.
- **Proof:** `scripts/proof.sh rules <out> 32a1ab0` (with `CARGO_TARGET_DIR=$TMPDIR/wotr-proof-150b`). The comparison is empty: 2,163 Builds, none differing, and `A.jsonl` and `B.jsonl` are byte-identical (17,599,587 B). So the real data's Builds show no difference of any kind. Decision 5's change shows only where an asks-advances prestige class replaces a caster's level that a later pick needs, which no recorded real-data Build does: the Findings' 0 disagreements there, now confirmed. Its one kind of difference, a class option `Err(BreaksLater)` → `Ok(())` for a prestige class that asks advances when an advancement keeps the Build, is pinned by the test world: the 88 cases in 7 lines that the agreement test no longer finds, and the new `advances` test. A class option open with an advancement the walk refuses (the Findings' hazard) could also turn closed. It had 0 cases before and has 0 now.
- **Benchmark:** ticket 48's browser benchmark (`t48/bench`) no longer exists. The scratchpad's `bench/` holds only ticket 149's load timing (`check-timing.js`). So the engine calls an edit and the pickers make were timed natively instead: `t150/bench150.rs`, a `seeds` example built in release on each side and not committed. It runs on ticket 48's bench Builds, 31 runs a call, median. The machine was heavily loaded (load average 60–156, other agents building), so the table gives the best median of 4–5 runs per side, run alternately. Raw runs are in `t150/bench-runs.txt`. Times are in ms, `32a1ab0` → now:

  | Build | level-1 feat picker (`options`) | class picker, level 1 | class picker, level 7 | edit (`allows`, `apply`, `evaluate`, `removable_classes`) |
  | --- | --- | --- | --- | --- |
  | fighter20 | 0.783 → 0.784 | 1.169 → 1.186 | 1.455 → 1.451 | 0.385 → 0.390 |
  | multi_prestige20 | 0.930 → 0.908 | 1.692 → 1.730 | 7.987 → 8.015 | 0.887 → 0.872 |
  | legend40 | 0.879 → 0.879 | 1.733 → 1.762 | 1.970 → 1.986 | 1.029 → 1.004 |

  The start pickers (race, point buy, skill steps) are within ±3%. The class picker is up to 2% slower, and on the blank-level Build `start` it went 0.040 → 0.051 ms. That is `try_class` looking each class up by key (one index lookup per option, about 230 options), where `class_status` was handed the class. It is within the runs' spread on the long Builds, and far under ticket 48's budget.
- **`scripts/check.sh`:** ALL-OK (fmt, clippy, wasm clippy, tests, the rules wasm build, the seed's check).
