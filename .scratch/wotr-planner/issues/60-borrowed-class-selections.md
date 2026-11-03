# Talents, hexes and discoveries taken through another class become picks

Type: task
Status: open
Blocked by: 59
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 54](54-mercies-and-enforcer-exploits.md)'s decision 2 (user, 2026-11-02): several classes and archetypes pick from another class's list, but the data holds a feature with no slot. Give each a slot of an existing selection (the rogue's, the slayer's, the witch's, the alchemist's), or of one sharing their options as [ticket 25](25-shared-selection-options.md) did, whichever the guide's list matches:

- bard talents (`bard#bard-talents`) and skald talents;
- the Archaeologist's advanced talents;
- the Feyform Shifter's rogue talent at 8 and 14;
- the Vivisectionist's medical discoveries;
- the Sanctified Slayer's slayer talents at 8, 16, 17 and 20;
- the Hexcrafter's hex arcana and Hex Magus.

Where the slot is of a selection an Extra feat names, ticket 49's rule gives that feat's pick there too.

## Build

AFK, test-first (`tdd`); the only agent in the repo; schema change and DB reset allowed per `CLAUDE.md` if the types need one; `V1_LINK` regenerated with a comment if the link format changes. Same method as tickets 45, 54 and 55: an extractor module asserting the guide's sentences, a one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, and a before/after proof in which only Builds that have one of the listed classes or archetypes differ. Ticket 48's picker shortcut must stay exact. Engine code reaches game data through ticket 56's type.

Data from the saved guide pages in `guide-cache/` only. If the guide doesn't settle something, write a question in this ticket with a recommendation and go on.

**Done when:** every check from [ticket 53](53-rename-packages.md)'s Done when passes, plus real-data tests: each listed class or archetype has its slots at its table's levels, offering the guide's list; an option taken through one class is not offered again through another where the selection is shared; an Extra feat that names the slot's selection gives its extra pick there; the sheet labels the new slots by name (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer
