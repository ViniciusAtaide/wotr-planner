# The sheet's view state as types

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The implicit state machine audit (2026-11-09, see the map's Notes) found the web app's view state held in signals whose combinations mean more than the types say. Ticket 39's decision 3 applies ("where it's cheap, make the bad state unrepresentable in the types"), and here it's cheap: `libs/ui` only, no rules, link or data change.

- **A pick closes the picker and edits by hand at nine sites.** Every pick does "close, then edit unless the pick is the current one": `libs/ui/src/picker.rs` ~113, ~391, ~518, ~635, ~692, ~745, ~778, ~871 and ~1037, each with its own `is_current` test where it has one. A new picker has to copy the rule.
- **A picker's steps are two signals each.** `SlotPicker` holds `computed: Option<Arc<Vec<SlotEntry>>>` (~374) and `open_entry: Option<usize>` (~384); `ClassPicker` holds `computed` (~491) and `advancing: Option<ClassOption>` (~500). "An entry open while the list is still computing" and "advancing while the list is still computing" are representable, and the view's `match`es have to agree on which signal wins.
- **The attack weapon is `Option<Option<WeaponKey>>`** (`planner.rs` ~56): `None` follows the engine's default, `Some(None)` is no weapon, `Some(Some(k))` a weapon. Set at `grid.rs` ~370, resolved at ~301; `Planner::replace` (commit a74e278) resets it.
- **`after_paint`'s doc comment says the same thing twice** (`picker.rs` ~312–315), two summaries from two edits.

Checked and left alone: a `SlotPicker` reads its slot once, when it opens (~360). Every Build change closes the picker (each pick, Remove, Escape, `Planner::replace`) except a step in the Skills picker, which re-reads its level on every edit (`start.rs` ~260–272), so a stale open slot isn't reachable today.

## Build

- **(a)** `Planner::pick(edit)` closes the picker and makes the edit. The nine sites call it.
- **(b)** One signal per picker for its step: `enum SlotStep { Loading, List(Arc<Vec<SlotEntry>>), Values { entries: Arc<Vec<SlotEntry>>, entry: usize } }` and `enum ClassStep { Loading, List(Arc<Vec<ClassOption>>), Advances { options: Arc<Vec<ClassOption>>, option: ClassOption } }`. The `back` callbacks return to `List`, and `after_paint` sets `List` only from `Loading`.
- **(c)** `enum AttackWeapon { Default, NoWeapon, Weapon(WeaponKey) }` in place of `Option<Option<WeaponKey>>`. `Planner::replace` sets `Default`.
- **(d)** `after_paint` keeps one doc comment.

**Done when:**
- `scripts/check.sh` passes (`--full` isn't needed: no cargo-leptos change).
- The dev server, checked in the browser: each picker kind opens, shows its loading state and list, picks, re-picks the current pick (nothing changes and the URL is unchanged), removes; the slot picker's value step and back; the class picker's advances step and back; the attack weapon: default, a weapon, no weapon, and back to the default after a pasted link. No console errors.

## Choices to settle

1. **How `pick` skips the current pick.**
   - **A (recommended):** `pick` applies the edit to a clone of the Build and sets it only if it changed, so the nine `is_current` tests go. The clone is cheap beside what follows every edit: the engine walks the whole Build again.
   - **B:** `pick(edit, is_current: bool)`, and each site keeps its own test. Smaller change; the rule stays at each site.
2. **Whether Remove goes through `pick`.** Recommended: yes. A Remove's edit always changes the Build, so `pick` behaves the same, and every close-then-edit has one home.
3. **Where the step enums live.** Recommended: beside their pickers in `picker.rs`, private. Nothing outside a picker reads its step.

## Progress log

## Answer
