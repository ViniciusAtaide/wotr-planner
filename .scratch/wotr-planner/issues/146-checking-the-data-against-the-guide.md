# Checking the data against the guide

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The map's "Not yet specified": how we confirm that the data files match the guide. The hand-fix stages (tickets 17–20) checked the extracted data once. Since then, data changes have been hand edits, each with a real-data test for what it changed, but nothing re-checks the whole against the saved pages. The guide's sample-build pages weren't saved, so there are no golden builds from it. That page also holds BAB, ability-score and skill-rank prerequisite tables that would cross-check the feats. The two build write-ups used by the QA run (ticket [88](88-qa-full-builds.md)) are no longer in `.scratch/`.

Options:
- **A.** Spot checks: an independent agent samples N items per kind (feats, class rows, selection options, mythic abilities), compares each with its saved guide section, and files each mismatch as a ticket. It is repeated after large data changes.
- **B.** Golden builds: the user gives build write-ups (from the guide's sample-build page if they save it, or elsewhere), and each becomes a real-data test that builds it through the engine and checks its sheet. Ticket 88 did this by hand, once.
- **C.** Save the sample-build page and cross-check every feat's BAB, ability and skill-rank prerequisites against its tables in a real-data test.

*Recommend:* A now, as it needs nothing from the user. Add C if the user saves the page, and B for any write-ups the user gives.

## Done when

The user has chosen. The chosen checks have run once, each mismatch is a ticket, and how to repeat them is written in `libs/game-data/README.md`. The map's "Not yet specified" entry is gone.

## Answer

**Decided** (user, 2026-11-10): A (spot checks) and B (golden builds). A runs now. B waits on build write-ups from the user, and each one becomes a real-data test. C is not taken.
