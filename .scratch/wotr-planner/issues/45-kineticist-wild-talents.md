# Kineticist wild talents

Type: task
Status: open
Blocked by: 44
Map: [WotR Build Planner](../map.md)

## Question

Build [ticket 43](43-extraction-report-gaps.md)'s decision 2 (user, 2026-11-02): wild talents become a selection. Today the even-level "Wild Talents" is a feature, not a pick.

- From `kineticist#wild-talents`: picked at 2nd and every even level, from talents whose level is at most half the kineticist level, each limited to its elements. The guide lists 26 talents with their level and element conditions. The four defensive talents stay each element's 2nd-level grant, as now.
- The bonus-feat wild talent (Universal, level 1) is taken once (the guide doesn't say it repeats) and grants a bonus feat slot, ignoring prerequisites, from the held elements' wild-talent bonus-feat lists (`OptionList(selection: "kineticist/element")`, so Expanded Element offers both elements' lists).
- Kineticist archetypes that change wild talents follow their own sections.

Data from the saved guide pages only. If the guide doesn't settle a talent's condition, write a question in this ticket with a recommendation and go on.

## Build

AFK, test-first (`tdd`); the only agent in the repo; schema change and DB reset allowed per `CLAUDE.md` if the types need one (a new prerequisite kind, for example); `V1_LINK` regenerated with a comment if `Build` changes. Same method as tickets 30–38 and 43: extractor module, one-off rewrite of `game-data/`, real-data tests first, fresh extraction matching the rewrite, before/after proof. The kineticist report item is closed: [ticket 44](44-sorcerer-feat-slayer-deities.md) moved it here (`kineticist#kineticist-bonus-feat`, "a wild talent that grants a bonus feat slot; a selection option can't grant a slot"), so `extract::report_gaps::moved` leaves it out of an extraction's report and `game-data/EXTRACTION_REPORT.md` has 0 items; this ticket settles it instead (`moved` goes, and the rule that builds the talent settles the item).

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a kineticist 2 picks a wild talent of level ≤ 1 from its element's and Universal talents only; a Fire kineticist isn't offered a Water-only talent; a talent of level 3 opens at kineticist 6; the bonus-feat talent grants one slot offering only the held elements' bonus feats, ignoring prerequisites, and can't be taken twice.

## Progress log

## Answer
