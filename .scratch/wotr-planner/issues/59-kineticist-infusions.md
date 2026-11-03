# Kineticist infusions become a selection

Type: task
Status: open
Blocked by: 58
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 54](54-mercies-and-enforcer-exploits.md)'s decision 2 (user, 2026-11-02): the kineticist's infusions (`kineticist#infusion`, at 3, 5, 9, 11, 13, 17 and 19, with each infusion's level and element in the guide's Infusions section) are a feature in the data, not picks. Make them a selection, as [ticket 45](45-kineticist-wild-talents.md) did the wild talents (level limit, element conditions, Expanded Element). Kineticist archetypes that change infusions follow their own sections.

## Build

AFK, test-first (`tdd`); the only agent in the repo; schema change and DB reset allowed per `CLAUDE.md` if the types need one; `V1_LINK` regenerated with a comment if the link format changes. Same method as tickets 45, 54 and 55: an extractor module asserting the guide's sentences, a one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, and a before/after proof in which only Builds that have a kineticist of 3 or more differ. Ticket 48's picker shortcut must stay exact. Engine code reaches game data through ticket 56's type.

Data from the saved guide pages in `guide-cache/` only. If the guide doesn't settle something, write a question in this ticket with a recommendation and go on.

**Done when:** every check from [ticket 53](53-rename-packages.md)'s Done when passes, plus real-data tests: a kineticist 3 has one infusion slot offering only its elements' and universal infusions of allowed level; an infusion of a higher level is locked with a reason and opens at its level; a Fire kineticist isn't offered a Water-only infusion; every kineticist archetype's infusion levels match its table; the sheet labels the new slots by name (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer
