# The engine's pickers cover the Build start and the level controls

Type: task
Status: open
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
