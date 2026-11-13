# Four small web-app fixes from a QA session

Type: bugfix
Status: resolved
Blocked by: none
Model: Opus to build (Leptos UI, checked in the running app at 360 and 320 px).
Map: [WotR Build Planner](../map.md)

## Problems

Found in a QA session on the dev server (2026-11-13), on a narrow phone viewport (the Fold's cover screen).

1. **A picker with no options says "Nothing matches."** A Human Sorcerer 7 with no bloodline chosen at level 1; tap level 7's "Bloodline feat" chip. The picker opens with no search box and only "Nothing matches.", which reads like a failed search and doesn't say why the list is empty.
2. **"at most 1 ranks in a skill".** The skill ranks dialog at level 1. The same slip was in "1 of 1 skill points spent" (a level with one skill point) and in the rules' prerequisite text "Needs 1 ranks in Persuasion" (Frightening Ambush, Mounted Combat).
3. **A tab's label is cut short on a narrow phone.** At 360 px wide the three tabs read "Levels", "✦ Mythic tra…", "Gear"; at 412 px they fit.
4. **A locked class's reason is hover-only.** A level's class button whose later choices depend on it (sorcerer 1 with a bloodline chosen) was `disabled` with the reason in its `title`. On a touch screen a tap did nothing and the reason never showed, which the map's Mobile note forbids.

## Answer

Done 2026-11-13 (branch `qa/ui-fixes`).

1. `ItemList` (`libs/ui/src/picker.rs`) tells a list with no items at all from a search or chip that hides every item: only the second says "Nothing matches."; the first says its `empty` text, by default "No options here yet.". Its old `nothing` prop was never passed, so it was repurposed. The slot picker passes a reason for a slot whose list comes from a held option (`FeatSlot::OptionList`, the sorcerer's bloodline feat): "Nothing to choose yet: choose the Bloodline first.", with the selection's name from `Engine::selection_name`. Every other picker gets the default.
2. `libs/ui/src/start.rs`: "skill point" and "rank" in the singular for 1. `libs/rules/src/engine/labels.rs`: the skill ranks prerequisite says "1 rank in …". No other count in `libs/ui` takes a plural noun after a number that can be 1 ("Show N locked" has no noun; the game data's problems already have a one-problem arm).
3. `apps/frontend/style/main.css`: under 600 px the tabs are `flex: 1 1 auto`, so each is as wide as its label and they share what's left. They no longer split the width equally there. At 360 px they're 88, 149 and 82 px wide, and at 320 px 74, 135 and 68 px, all 44 px tall, with no ellipsis and no horizontal scroll. From 600 px up they stay equal widths.
4. `libs/ui/src/grid.rs`: the locked class button isn't `disabled` any more. It is `aria-disabled` and stays tappable. A tap shows "Later choices depend on this class" under the class name (`.class-locked`), and another tap hides it. It doesn't open the class picker. The `title` stays for a pointer. An unlocked class still opens the picker.

Other `disabled` controls, checked and left as they are, since none hides a reason in a `title`:

- The Remove level × (`grid.rs`): `disabled` when the level can't be removed. Its `title` is only its label ("Remove level 7"). No reason is shown anywhere, and the engine's `withdrawal` gives none, so showing one needs more than this change.
- The Mythic track's first ascension, path and switch chips (`mythic.rs`): `disabled` while the Build doesn't reach the rank's level. They have no `title`, and the row's greyed level already shows it.
- The alignment wheel's nodes (`picker.rs`): locked nodes are `disabled`, with the alignment's name in `title`. They show its short name, and the reason is said once under the wheel.
- The racial ability segments and the point-buy and skill steppers (`start.rs`): `disabled` without a `title`, at the end of a range or an option the engine refuses.

**Checks.** Seen fixed in the running app (its own dev server on port 3950) at 360 × 840 and 320 × 840. No unit tests: screens get end-to-end smoke tests only, and the rules change is one label. `scripts/check.sh` passes with the snapshot files untouched.

**Review (2026-11-13).**

- The reason span lost its `role="status"`: a button's children are presentational, so the role did nothing. A screen reader gets the reason from the button's `title` (its description) without the tap.
- The MCP server's Todo label had the same plural slip, "1 of 1 skill points unspent" (`apps/mcp/src/code.rs`); it says "skill point" for 1 now.
- Left as they are:
  - A pool selection with nothing held (an Oracle's Revelation before the Mystery) says the default "No options here yet.", not which choice to make first: the engine gives the sheet no way to ask what a slot's list depends on.
  - The reason names the slot's own selection, so a Dragon Disciple without a bloodline reads "choose the Bloodline first" where its chip is "Blood of Dragons".
  - Under 600 px the tabs aren't equal widths even where equal ones would fit (412 px).
