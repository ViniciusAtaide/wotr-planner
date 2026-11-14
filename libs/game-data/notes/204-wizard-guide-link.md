# Ticket 204: the wizard's School selection links to the wizard page

## What the data says now

`libs/game-data/classes/wizard.ron`, the selection `wizard/school`: `guide: "wizard"` (it was `"wizard#wizard"`).

## What it rests on

The guide's wizard page (`guide-cache/txt/wizard.txt`) has its title heading, "Wizard", with the page's own link `wizard` and no `#wizard` anchor. The guide-text index found this the only one of 4,351 guide links in the data that doesn't resolve to a heading. The class page is the right target: the school choice is made on it, under the class's own text.

## Decisions

The main session, 2026-11-14: the page link, with no anchor.

## Not modelled

Nothing: only the link changed. No game rule or number moved.

## Test

None: the guide cache is git-ignored, so a test can't read it. The snapshot check is unaffected (links aren't projected).
