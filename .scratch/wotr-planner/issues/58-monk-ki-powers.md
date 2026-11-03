# Monk ki powers become a selection

Type: task
Status: open
Blocked by: 57
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 54](54-mercies-and-enforcer-exploits.md)'s decision 2 (user, 2026-11-02): the monk's ki powers (`monk#ki-power`, at 4 and every 2 levels after) are a feature in the data, not picks. Make them a selection from the guide's list, each with its level and prerequisites, as [ticket 45](45-kineticist-wild-talents.md) did the wild talents. Monk archetypes that change or replace ki powers follow their own sections. If a feat (such as an Extra feat) names ki powers, check whether its guide entry gives a pick and wire it per ticket 49.

## Build

AFK, test-first (`tdd`); the only agent in the repo; schema change and DB reset allowed per `CLAUDE.md` if the types need one; `V1_LINK` regenerated with a comment if the link format changes. Same method as tickets 45, 54 and 55: an extractor module asserting the guide's sentences, a one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, and a before/after proof in which only Builds that have a monk of 4 or more differ. Ticket 48's picker shortcut must stay exact. Engine code reaches game data through ticket 56's type.

Data from the saved guide pages in `guide-cache/` only. If the guide doesn't settle something, write a question in this ticket with a recommendation and go on.

**Done when:** every check from [ticket 53](53-rename-packages.md)'s Done when passes, plus real-data tests: a monk 4 has one ki power slot offering only the powers of level ≤ 4; a power of a higher level is locked with a reason and opens at its level; every monk archetype's ki power levels match its table; the sheet labels the new slots by name (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer
