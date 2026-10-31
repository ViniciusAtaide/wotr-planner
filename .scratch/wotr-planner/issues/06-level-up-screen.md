# Level-up screen behaviour

Type: prototype
Status: open
Blocked by: 04
Map: [WotR Build Planner](../map.md)

## Question

How should the level-by-level screen behave?

- **Layout:** one level at a time as a wizard, or all levels on a timeline you can scroll and jump through?
- **Unavailable options:** hidden, or greyed out with the prerequisite that blocks them?
- **Editing earlier levels:** if you change level 4 and a level 12 feat stops being legal, does the planner clear it, flag it, or block the edit?
- **Mythic ranks:** shown on the same timeline as character levels, or in their own track?
- **Fold 7 layouts:**
  - Folded (cover screen, tall and narrow, ~21:9): probably one column, with the level navigation as a compact strip or drawer.
  - Unfolded (inner screen, nearly square, ~6:5): probably two panes, with the level timeline on one side and that level's choices on the other.
  - Desktop: the unfolded layout, wider.
  - Decide where the layout switches between these, and whether either layout avoids the hinge.
  - Folding or unfolding the phone must keep the selected level and any open choice in place.
- **Choice lists on a phone:** a choice like "pick a feat" can have 250+ options. How do search, filtering by category, and showing an option's prerequisites work on the narrow cover screen?

Make the prototype viewable on the user's phone over the local network, and check it at both Fold sizes. The browser pane's resize tool can emulate them.

Build a rough UI prototype with the `prototype` skill and work through it with the user.
