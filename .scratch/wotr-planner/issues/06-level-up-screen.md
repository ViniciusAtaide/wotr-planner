# Level-up screen behaviour

Type: prototype
Status: resolved
Blocked by: 04
Map: [WotR Build Planner](../map.md)

## Question

How should the level-by-level screen behave?

- **Layout:** one level at a time as a wizard, or all levels on a timeline you can scroll and jump through?
- **Unavailable options:** hidden, or greyed out with the prerequisite that blocks them?
- **Editing earlier levels:** if you change level 4 and a level 12 feat stops being legal, does the planner clear it, flag it, or block the edit?
- **Mythic ranks:** shown on the same timeline as character levels, or in their own track? Each Mythic ascension happens at a fixed character level (see [How a character progresses](04-progression-model.md)). Level 17 has a Level-up, two Mythic ascensions and the possible Path switch.
- **Fold 7 layouts:**
  - Folded (cover screen, tall and narrow, ~21:9): probably one column, with the level navigation as a compact strip or drawer.
  - Unfolded (inner screen, nearly square, ~6:5): probably two panes, with the level timeline on one side and that level's choices on the other.
  - Desktop: the unfolded layout, wider.
  - Decide where the layout switches between these, and whether either layout avoids the hinge.
  - Folding or unfolding the phone must keep the selected level and any open choice in place.
- **Choice lists on a phone:** a choice like "pick a feat" can have 250+ options. How do search, filtering by category, and showing an option's prerequisites work on the narrow cover screen?

Make the prototype viewable on the user's phone over the local network, and check it at both Fold sizes. The browser pane's resize tool can emulate them.

Build a rough UI prototype with the `prototype` skill and work through it with the user.

## Comments

**2026-10-31, prototype:**
- **Location:** `prototypes/level-up-screen/index.html`. It's throwaway: one HTML file with fake sample data (a human Fighter 8 / Rogue 4 / Fighter 8). Serve it with `python3 -m http.server 3950 --bind 0.0.0.0` from that folder, then open `http://<mac-lan-ip>:3950/?variant=A|B|C` on the Fold.
- **Variants:** each one bundles a layout with a locked-option policy and an edit policy.

| Variant | Layout | Locked options | Editing an earlier level |
|---|---|---|---|
| **A: Wizard** | One level per screen, with a level chip strip and Back/Next. Mythic ascensions sit inline. The picker is full-screen when folded and a side sheet when unfolded. | hidden, with a count | dependent picks are **auto-cleared**, with Undo |
| **B: Timeline** | Every level and mythic ascension as rows. Folded: an accordion. Unfolded: list + detail panes. | greyed, with the reason | picks are **flagged**, with a problems banner |
| **C: Sheet** | A dense level × slot grid of chips, with a separate Mythic track tab. Bottom sheet folded, modal unfolded. | a "Show N locked" toggle | edits that would break later picks are **blocked**, showing "Would break: …" |

- Noticed while building: when edits are flagged, a broken pick still counts as held. So Great Cleave stays valid while its prerequisite Cleave is flagged, and the flag doesn't carry down the chain. The real rules engine needs to decide that.

**2026-10-31, user:** picked **Sheet** (variant C).

## Answer

The planning screen is **variant C, Sheet**, from the [prototype](../prototypes/level-up-screen/index.html):

- **Layout:** a compact grid with one row per character level. Each row has the level number, a class button, and one chip per slot (Feat, Human, F-bonus, Talent and so on). Empty slots are dashed "+" chips, and rows with a Mythic ascension carry a ✦ MR badge. Mythic ranks have their own **Mythic track** tab: rows MR 1–10 with their fixed level, showing "not reached" after a Legend switch.
- **Picking:** tapping a chip or class opens a picker. It's a bottom sheet on the folded cover screen and a centred modal on the unfolded inner screen, with search and category chips.
- **Locked options:** hidden by default, with a "Show N locked" toggle. When shown, they're greyed out with the reason ("Needs: BAB +4 (have +1)").
- **Edits that break later picks:** blocked. An option, class change or Clear that would invalidate a later pick is disabled and says "Would break: Cleave (L1), …". Edits therefore never leave the Build in a broken state, which settles the flag-cascade question. Saved Builds can still go stale when game data changes, so server-side validation still reports problems on load.
- **Fold:** the same grid on both screens, with roomier columns from 600px up. Folding or unfolding keeps the user's place because nothing navigates away.
