# Slot options come back presented

Type: task
Status: open
Blocked by: none (runs alongside [98](98-check-validates-references.md), in its own worktree)
Map: [WotR Build Planner](../map.md)

## Question

The architecture review of 2026-11-07 (candidate 3) found the slot picker's presentation split between the UI and the engine's labels:

- The UI groups a slot's options by item (`groups`, `libs/ui/src/picker.rs` ~349), builds the "(Story: …)" note, picks a group's reason (`group_reason` ~522: anything over "Already taken"), and makes about five engine label calls per option.
- `value_noun` (~536) repeats `ParameterKind::noun` with articles added.
- The deity picker strips " Domain"/" Subdomain" from option names itself (~1017).
- Inside `rules`, five matches over `Pick`'s four variants in `libs/rules/src/engine/labels.rs` and `pickers.rs` (plus one in `slots.rs`, for checking) only reach the picked item's name, category, summary or prerequisite.

Tickets 95–96 moved every other control's rules into the engine; this is the slot picker's turn. The user chose to build it (2026-11-08) without a design round: settle small choices yourself, record them, and bring a real open question back.

## Build

- **One lookup for "the item a pick takes":** the catalog (or engine) resolves a `Pick` to its item once, and the label and picker code read the item instead of matching `Pick` each time. The checker's match in `slots.rs` may stay if it checks something the item can't say.
- **Slot options come back presented:** the engine returns a slot's options already grouped by item, each group with its name, category chip, summary, story note and the one reason it shows, plus each member's own status and edit (from ticket 96). The UI renders them. Pick the shape (a new type beside `SlotOption`, or `options` itself changing) and record why; `SlotOption` keeps its status and edit.
- **The reason rule** (`group_reason`'s priority) and the story note move into `rules` and get tests.
- **`value_noun` goes:** the UI uses the engine's noun (add the article form to `ParameterKind` or labels if the UI needs it).
- **Domain names:** the engine gives the name the deity picker shows, so the UI strips nothing.
- Test-first in `rules` (`tdd`), on the small world: grouping, the reason priority, the story note, the noun, and the domain name.

**Done when:**
- The rules tests above pass; `libs/ui` has no `groups`, `group_reason` or `value_noun`, and makes no per-option label calls to build a slot list.
- `scripts/proof.sh rules` against the parent commit shows no behaviour difference.
- The browser check (a release build on its own port): slot pickers show the same groups, chips, notes and reasons as before, no slower by the `timed(...)` logs, and no console errors.
- `scripts/check.sh` passes, and `--full` once at the end.
