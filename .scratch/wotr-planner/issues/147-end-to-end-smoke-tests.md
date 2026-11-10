# End-to-end smoke tests

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The map says screens get end-to-end smoke tests only, but its "Not yet specified" leaves the tooling and the flows open, and none exist. The web app has been checked by hand: ticket [88](88-qa-full-builds.md)'s QA run, and browser checks in single tickets. So a regression in hydration, the link round-trip or the Fold layouts shows only when someone looks.

**Tooling** options:
- **A.** Playwright (Node) against a release build served on its own port (ticket 88's setup), with device emulation for the two Fold screens. It adds Node to a Rust-only repo.
- **B.** A Rust WebDriver client (`fantoccini` or `thirtyfour`) against chromedriver, which keeps the tests in cargo but has no device emulation built in.

**Flows** (recommended set):
1. `/` opens a blank Build and hydrates.
2. The Build start (race, faith, alignment, point buy) and level 1 with its picks: the link updates at each edit.
3. Reloading the link restores the same sheet.
4. A stale link opens a blank Build with the generic error (ticket [40](40-stale-links-generic-error.md)).
5. "Copy build sheet" gives the text (ticket [87](87-build-sheet-export.md)).
6. Flows 1–3 on the Fold cover screen (~21:9) and inner screen (~6:5), with no horizontal scroll.
7. The static site from `scripts/build-site.sh` runs flow 1 under a sub-path, as GitHub Pages serves it.

*Recommend:* A for its device emulation, run by `scripts/check.sh --full` and the Pages workflow, with flows 1–7. Adding Node is the user's call.

## Done when

The user has chosen the tooling. The flows run from one script, `scripts/check.sh --full` runs it, and it fails on a broken flow. The map's "Not yet specified" entry is gone.

## Answer

**Decided** (user, 2026-11-10): A, Playwright (adding Node is accepted), with flows 1–7, run by `scripts/check.sh --full` and the Pages workflow.
