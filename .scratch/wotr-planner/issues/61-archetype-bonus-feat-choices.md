# Archetype bonus-feat choices become feat slots

Type: task
Status: open
Blocked by: 60
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 54](54-mercies-and-enforcer-exploits.md)'s decision 2 (user, 2026-11-02): some archetypes choose a bonus feat from a fixed list, but the data holds a feature with no slot. Make each a bonus feat slot from its list (`OneOf` feat lists, the shape of the Divine Guardian's bonus feats), with prerequisites as the guide says (ignored or checked):

- the Titan Fighter's Massive Attacks: a feat from the ranger's two-weapon list, at 2 and every 4 levels;
- the Urban Hunter's Captor: a bonus feat at 6 and every 3 levels;
- the Hunter's Precise Companion: Precise Shot or Outflank;
- the Stormwalker's Archery: the ranger's archery style feats.

## Build

AFK, test-first (`tdd`); the only agent in the repo; schema change and DB reset allowed per `CLAUDE.md` if the types need one; `V1_LINK` regenerated with a comment if the link format changes. Same method as tickets 45, 54 and 55: an extractor module asserting the guide's sentences, a one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, and a before/after proof in which only Builds that have one of the listed archetypes differ. Ticket 48's picker shortcut must stay exact. Engine code reaches game data through ticket 56's type.

Data from the saved guide pages in `guide-cache/` only. If the guide doesn't settle something, write a question in this ticket with a recommendation and go on.

**Done when:** every check from [ticket 53](53-rename-packages.md)'s Done when passes, plus real-data tests: each listed archetype has its bonus feat slots at its table's levels, offering exactly its list; a feat already held is not offered again unless it repeats; prerequisites are checked or ignored as the guide says; the sheet labels the new slots by name (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer
