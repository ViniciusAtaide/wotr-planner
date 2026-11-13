# The previous level's class first in the class picker

Type: enhancement
Status: resolved
Blocked by: none
Model: Opus to build (Leptos UI, checked in the running app).
Map: [WotR Build Planner](../map.md)

## Problem

The "Add a class" picker lists the classes in the engine's order (`Engine::class_options`: base classes with their archetypes, then prestige classes, in key order). Levelling the same class again is the most common pick, yet a Hunter 1 (Tandem Executioner) who clicks "+ Level 2" finds Hunter (Tandem Executioner) below Alchemist and the other classes before it.

## Answer

Done 2026-11-13 (branch `ticket-186`).

`ClassPicker` (`libs/ui/src/picker.rs`) moves the option for the class the Build took at the level before (level N − 1, for the picker on level N) to the top of its options; the rest keep the engine's order. It reorders the options themselves, not only the rendered items, so the index `ItemList` passes back on pick still names the same option. A small pure helper, `lift`, moves the first matching item to the front (`[..=index].rotate_right(1)`); unit tests in the same file cover it.

- Level 1 has no level before it, so nothing moves; nor when the engine offers no option of that class. A class already taken is offered only with its fixed archetype, so there is one such option.
- The same applies when re-picking an existing level's class: the picker on level N puts level N − 1's class first.
- `ItemList` filters by chip and by locked without reordering, so the lifted option is also first under its own chip (Base classes or Prestige classes), once. A locked one goes first among the locked.
- `Engine::class_options` is unchanged: its order is documented and the MCP server's `options` tool relies on it.
- No marker or label: only the order was asked for.

**Checks.** Seen in the running app (this worktree's dev server on port 3950), with a Human Hunter 1 (Tandem Executioner):

- "+ Level 2" lists Tandem Executioner, Alchemist, Chirurgeon first under All, and Tandem Executioner first (once) under Base classes. It isn't under Prestige classes.
- Picking it adds Hunter 2.
- Re-opening level 2's class puts Tandem Executioner first, marked current.

`scripts/check.sh` passes, with the snapshot files untouched (a UI-only change).
