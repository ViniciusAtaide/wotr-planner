# Witch Wandering Hex and Beneficial Curse give their pick

Type: task
Status: open
Blocked by: 61
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 55](55-mythic-extra-picks.md)'s decisions 1 and 2 (user, 2026-11-02):

1. **Witch Wandering Hex** ("You can select any hex for which you meet the prerequisites") gives one slot of `witch/hex`, as Second Spirit gives a spirit, and needs a hex held (the witch's or the shaman's): its "Prerequisites: Hex" is missing from the data today.
2. **Beneficial Curse** ("You gain an additional Curse with no penalties") gives one slot of the oracle's curses (`oracle/oracles-curse`), offering a curse not already held, as Second Mystery gives a mystery.

Reuse ticket 55's machinery where it fits; otherwise say what was added and why.

## Build

AFK, test-first (`tdd`); the only agent in the repo; schema change and DB reset allowed per `CLAUDE.md` if the types need one; `V1_LINK` regenerated with a comment if the link format changes. Same method as tickets 45, 54 and 55: an extractor module asserting the guide's sentences, a one-off scratchpad rewrite of `libs/game-data/` (never re-run the extractor over it), real-data tests red first, a fresh extraction equal to HEAD's run through the rewrite, and a before/after proof in which only Builds that take Witch Wandering Hex or Beneficial Curse differ. Ticket 48's picker shortcut must stay exact. Engine code reaches game data through ticket 56's type.

Data from the saved guide pages in `guide-cache/` only. If the guide doesn't settle something, write a question in this ticket with a recommendation and go on.

**Done when:** every check from [ticket 53](53-rename-packages.md)'s Done when passes, plus real-data tests: Witch Wandering Hex is locked without a hex and, once taken, gives one witch hex slot offering hexes whose prerequisites are met; Beneficial Curse gives one oracle curse slot offering only curses not held; removing either is refused while its slot holds a pick and frees once it's cleared; the sheet labels the new slots by name (checked in the browser).

Leave the user's uncommitted edit in `apps/backend/src/main.rs` exactly as it is.

## Progress log

## Answer
