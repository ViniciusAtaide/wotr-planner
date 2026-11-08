# The engine's pickers cover the Build start and the level controls

Type: task
Status: resolved
Blocked by: [95](95-public-withdrawals.md) (same files, and its withdrawals)
Map: [WotR Build Planner](../map.md)

## Question

The architecture review of 2026-11-07 (candidate 1) found the engine's pickers (`libs/rules/src/engine/pickers.rs`) stop at slots, class, path and switch. For everything else the UI works out each option's status itself (`libs/ui/src/picker.rs:837–1194`, `libs/ui/src/start.rs:133–330`), and `libs/ui` has no tests:

- race/subrace, alignment, faith and background, through `allows_many` and `Engine::faith_fits`;
- the racial +2 and the level's ability increase, through `allows_many`;
- the point-buy and skill-rank steppers, through the budget the UI checks itself and, in `SkillsPanel`, by applying the edit and comparing to see whether the Build refused it.

The UI also applies a rule itself: `RacePicker` clears the racial +2 when the new race doesn't choose one. And the faith/alignment rule lives twice: the walk fails `Choice::Deity` (`libs/rules/src/engine/walk.rs` `faith`) and `faith_fits` returns `Issue::DeityAlignment`; the UI ANDs them.

The user settled the design in a grilling round (2026-11-07, all recommendations taken).

## Build

- **Every control gets an engine picker:** race and subrace, alignment, faith, background, the racial +2, each level's ability increase, the point-buy scores and the skill ranks.
- **An option carries its edit.** Each option is what it is, its `status: Result<(), Issue>` (as today: a misfit's own issue, or `Issue::BreaksLater`) and the `Edit` that takes it. The engine does the side effects (a race without a racial choice clears the +2), so the UI never builds a `BuildStart`. The existing slot, class, path, first-ascension and switch options gain their edit too, and `class_edit` leaves what callers need to know.
- **Steppers:** for each point-buy score and each skill at a level, a "down" and an "up" step, each an option with its edit and status, or none at the end of the range. The budget, the 7–18 range and the refused-edit check live in the engine.
- **One faith/alignment rule:** one private engine function decides whether a faith and an alignment fit (including ticket 94's "no alignment without a faith") and returns the `Issue`. The walk and the pickers both call it, and `faith_fits` leaves the public interface. The alignment wheel's one reason line comes from the options' shared issue.
- **The UI only renders** these options; it has no rules left. `libs/ui` gets no tests: the rules move to where the tests are, and the browser check covers rendering.
- **The proof** builds its battery from the engine's start options and steps, so new start choices are covered without editing it.
- Test-first in `rules` (`tdd`), on the small world: each picker's statuses and reasons, the racial +2 reset, the steppers' ends and budget, and the faith/alignment issue.

**Done when:**
- The rules tests above pass, and `libs/ui` calls neither `allows_many`, `faith_fits` nor `Build::apply` to decide a status.
- `scripts/proof.sh rules` against the parent commit shows no behaviour difference.
- The browser check (a release build on its own port): every start control and stepper locks and unlocks as before, and each is no slower than today by the `timed(...)` logs; no console errors.
- `scripts/check.sh` passes, and `--full` once at the end.

## Choices settled

1. **The interface.** A generic `ChoiceOption<T> { value, status, edit }` for every Build start and level picker: `Engine::race_options(&Build) -> Vec<ChoiceOption<RaceChoice>>` (`RaceChoice { race, subrace }`: each race in key order, then its subraces), `faith_options` (atheism, then each deity in key order; the UI sorts the deities by name, as it showed them), `alignment_options` (`Alignment::ALL`'s order), `background_options`, `racial_ability_options` (empty when the race or subrace doesn't choose one; the start panel shows the "+2 to" row by that) and `ability_increase_options(&Build, level) -> Option<…>` (`None` for a level the Build doesn't have or without an increase). A generic `Stepper<T> { value, down: Option<Step>, up: Option<Step> }`, `Step { status, edit }`: `point_buy_steps(&Build) -> Vec<Stepper<Ability>>` and `skill_steps(&Build, level) -> Option<Vec<Stepper<SkillKey>>>` (`None` for a level the Build doesn't have). They live in a new `libs/rules/src/engine/controls.rs`.
2. **Existing options gain their edit:** `SlotOption.edit` (`Edit::Pick`), `ClassOption.edit` (a class first taken there gets the option's archetype and keeps the Build's advancement; one taken before keeps its choice), `PathOption.edit` (`MythicPath`, `PathSwitch` or `FirstAscension`). `class_edit` is private now: `advances_options` returns `AdvancesOption { advances, edit }`, so the UI never builds a class edit.
3. **A status's order:** a faith and alignment that don't fit is the option's status without a walk; otherwise the walk's `BreaksLater`; the option the Build holds is always `Ok` (its edit changes nothing). The UI never compares an option with the Build to unlock it.
4. **A step's range ends are `None`, not an error:** a point-buy score outside 7–18 or over the budget (no `PointBuy` holds it, so there's no edit to carry), a skill below 0 ranks, over the level's skill points, or above the character level in total at that level or a later one (the Build refuses that edit: the refused-edit check is the engine's now). A step in range is `Ok` or `BreaksLater`. The controls lock exactly as before.
5. **One faith/alignment rule:** the private `Engine::faith_misfit(&BuildStart) -> Option<(Choice, Issue)>` gives the choice the walk fails (`Alignment` for `NeedsFaith`, `Deity` for `DeityAlignment`) and the issue a picker shows. `Walker::faith` and every start picker call it; `faith_fits` is gone. The wheel's one reason line is the first option's issue that has a text.
6. **`allows_many` is gone** from the engine and the planner: nothing asks it. `Planner::ask(label, |engine, build| …)` is the panels' untracked, timed engine call. Its consistency check moved: the small-world test and the proof check each start and level option's and step's status against `allows` of its edit.
7. **Batching kept:** each picker makes one `Recheck` and gets every candidate's status from it, as `allows_many` did (start choices walk from the start, as before; a level's from that level); a misfit skips its walk. The skill steppers read the level's skill points off the `Recheck`'s walk paused after the level (the walker keeps the last level's skill points), so no sheet walk is added.
8. **The proof:** `battery` is `fingerprint::controls` (every start and level option and step's edit, from the engine) plus the first ascensions' edits, the held choices' withdrawals, Remove level and a name. The slot, class and path options hash without their edits through mirror structs with the same Debug text, so records from before this ticket still compare. `fingerprint` and `record` lost their unused game-data argument.
9. **Running the proof across this change:** as ticket 95: the new harness can't compile against the parent's crates (no start pickers), so the rules comparison ran from a scratch clone holding the working tree's crates and the parent's `tools/proof`, whose `fingerprint.rs` was changed only to hash the options without their edits and to drop the `allows_many` check, on both sides. The added coverage is the new harness's record against that side B (`tools/proof` crate docs say so).
10. **Test fallout outside the new tests:** the real-data tests' `SlotOption` patterns take `..`; the Extra Feat test compares the two slots' options without their edits (each has its own slot's); `deities.rs`'s `faith_fits` tests ask the alignment and faith pickers.

## Progress log

- 2026-11-07: started at HEAD 1683337, clean tree. Read `CLAUDE.md`, the map's Notes, `GLOSSARY.md`, this ticket and 94–95.
- 2026-11-07: test-first on the small world, one slice at a time (outputs in the scratchpad's `t96/`): the faith and alignment pickers (`start_pickers.rs`, 4 red at their assertions against stubs, then green with `faith_misfit`); race, background and racial ability (5 red, green); the point-buy steps and the level pickers (`level_pickers.rs`; 10 red, green); each existing option's edit (`pickers.rs`, `advances.rs`; 6 red, green). `removable.rs`'s `allows_many` test became `the_start_and_level_pickers_lock_what_allows_refuses`. `cargo test -p rules --test engine`: 458 passed.
- 2026-11-07: UI onto the pickers (`picker.rs`, `start.rs`, `planner.rs`); `libs/ui` calls no `allows_many`, `faith_fits` or `Build::apply`. Proof harness (choice 8). `scripts/check.sh`: ALL-OK.
- 2026-11-08: rules proof, parent 1683337 against the working tree, 2,162 Builds (scratch clone, choice 9): no difference (`A: 2162 Builds, B: 2162; only in A: 0, only in B: 0, differing: 0`).
- 2026-11-08: added coverage, new harness against the old one on the same crates and Builds: only `allows` differs (2,161 Builds; the blank one has no controls). The battery went from 1,276,499 edits to 673,455: it drops the edits no control offers (an ability increase at a level without one, skill steps past their range, a racial ability for a race that chooses none). Every start and level option and step's status agreed with `allows` (no panic); held-choice counts unchanged.
- 2026-11-08: browser check on release builds made with `--lib-features timings` (working tree on 127.0.0.1:3913, parent on 3914; both servers stopped after). The pane was hidden, so timers were throttled; the pickers filled after a screenshot forced a frame.

## Answer

Done 2026-11-08. Every Build start and level control has an engine picker whose options carry their status and edit; the UI only renders them.

- **Rules.** `race_options`, `faith_options`, `alignment_options`, `background_options`, `racial_ability_options`, `ability_increase_options` (`ChoiceOption<T>`), `point_buy_steps` and `skill_steps` (`Stepper<T>` of `Step`s) in `engine/controls.rs`; slot, class and path options carry their edit, and `advances_options` gives `AdvancesOption`s. One private `faith_misfit` decides the faith and alignment for the walk and the pickers; `faith_fits`, `allows_many` and the public `class_edit` are gone.
- **UI.** No rule left: the start pickers, the wheel, the racial +2, the point buy, the ability increase and the skill ranks render the engine's options and apply their edits.
- **Proof.** No behaviour difference over 2,162 Builds; the battery comes from the engine's controls, and their statuses are checked against `allows`.
- **Browser** (working tree, release): the wheel locked all nine with "Choose a deity or atheism first"; Erastil opened LG, NG and LN with his reason under it; at Lawful Good the deity picker locked 17 deities each with its alignments and hid the faith's remove control; Half-Elf showed the +2 row, +2 on Str gave 12, Dwarf hid the row and dropped it, Human brought it back unset; Str stepped up to 18 then locked, Dex down to 7 then locked, Con stopped at 16 with 2 points left, and at 0 left every + locked; a background chosen (38 open); with Power Attack at level 1 on Str 11 + 2, Str − and the +2 elsewhere locked, and the races without the +2 or with a Str penalty locked; Int − locked while level 1 spent 3 skill points; the level-4 increase offered all six, then showed Str as current with a remove control; skill ranks locked at the level cap (1 at level 1, 2 by level 2) and once the 3 points were spent. No console errors.
- **Timings** (`timed(...)` medians in ms on `full fighter` (20 levels), parent → working tree; the engine calls were `allows_many` before): race 0.6 → 0.7 (within the 0.1 ms timer step; the engine now also builds the starts, which the UI built outside the timer), deity 3.7 → 1.4, background 3.3 → 3.3, alignment 2.6 → 0.6, racial +2 0.9 → 0.5, point buy 1.4 → 0.8, ability increase 2.0 → 0.4, skill ranks 0.8 → 0.2.
- **Tests:** `libs/rules/tests/engine/start_pickers.rs`, `level_pickers.rs`; edits in `pickers.rs`, `advances.rs`, `deities.rs`, `removable.rs`.
