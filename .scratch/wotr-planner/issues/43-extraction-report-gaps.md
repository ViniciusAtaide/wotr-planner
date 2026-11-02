# Close the extraction report's gaps

Type: task
Status: open
Blocked by: 12
Map: [WotR Build Planner](../map.md)

## Question

`game-data/EXTRACTION_REPORT.md` still lists 12 items (Feats 1, Base classes 11). Most let the sheet hold a Build the game doesn't allow, against the rule that the sheet only ever holds a possible Build ([ticket 39](39-impossible-states-not-validation.md)). User, 2026-11-02: close these before the end-to-end tests.

The items:

1. **Sorcerer archetypes' bonus feats** (`sorcerer#crossblooded`, `#empyreal-sorcerer`, `#geomancer`, `#nine-tailed-heir`, `#sage-sorcerer`, `#sylvan-sorcerer`): stored as general slots, so any feat fits. Find each one's list in the saved pages (the bloodline's bonus-feat list, the archetype's own, or Crossblooded's two bloodlines' lists together), and give the slot that list.
2. **Barbarian totem rage powers** (`barbarian#rage-powers`): one totem line per character. Only the Lesser → plain → Greater chain is checked. Make the totems exclusive with the `Not` prerequisite (ticket 32) or an option group, whichever the data already does for similar cases.
3. **Kineticist wild talents' bonus feats** (`kineticist#kineticist-bonus-feat`): see the item; model what the saved pages state, if the current types can hold it.
4. **Eldritch Scoundrel's sneak attack** (`rogue#sneak-attack-eldritch-scoundrel`): read from prose as dice at 1, 3, 7, 11, 15, 19. Check the reading against the saved page.
5. **Shaman's Wandering Hex** (`shaman`): a hex from another spirit. Model it if the saved pages say enough.
6. **Extra Slayer Talent** (`feats#extra-slayer-talent`): "Slayer Talents or Talented Slayer"; nothing named Talented Slayer is on a saved page.
7. **Cleric's deity alignment** (`cleric`): "within 1 place of their chosen deity". Deities aren't data.

Data from the saved guide pages only (offline; never fetch GameFAQs or fextralife). For each item: if the saved pages settle it, build it; if they don't, don't guess: write a numbered question for the user in this ticket with what the pages do say and a recommendation, and go on to the next item. Items 6 and 7 are likely questions (7 may well stay out of scope: deities aren't modelled).

## Build

AFK, test-first (`tdd`). The only agent in the repo, so it may edit the one migration and reset the DB per `CLAUDE.md` if a schema change is needed. As in tickets 30–38: the rules go in an extractor module, applied to `game-data/` by a one-off scratchpad program (never re-run the extractor over `game-data/`); a fresh extraction must be byte-identical to the rewrite; real-data tests first (red on the current data); a before/after proof that nothing else moved; each item settled is removed from `EXTRACTION_REPORT.md` by the extractor.

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes; every item is either built (with a real-data test showing the slot, lock or row now matches the guide: e.g. an Empyreal Sorcerer's bonus feat slot no longer offers Power Attack, a barbarian with one totem's power can't take another totem's) or written up as a question; the report's counts match.

## Progress log

## Answer
