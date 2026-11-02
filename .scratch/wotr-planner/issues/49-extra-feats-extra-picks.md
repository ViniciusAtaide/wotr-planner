# Extra feats give their extra pick

Type: task
Status: open
Blocked by: 48
Map: [WotR Build Planner](../map.md)

## Question

[Ticket 45](45-kineticist-wild-talents.md)'s decision 2 (user, 2026-11-02): the "Extra …" feats that grant another pick of a class selection give that pick. Today none does. From `game-data/feats.ron`: Extra Arcana, Extra Arcanist Exploit, Extra Discovery, Extra Hex, Extra Mercy, Extra Rage Power, Extra Rogue Talent, Extra Slayer Talent and Extra Wild Talent (check each against its guide entry; the resource ones, such as Extra Ki, Extra Rage or Extra Channel, grant no pick and stay as they are). Each time the feat is taken (they repeat: `Times(3)` or `Unlimited`), the character gets one more slot of that selection, at the level the feat is taken.

Data from the saved guide pages only. If a feat's selection isn't clear (Extra Hex for a witch vs a shaman, Extra Rogue Talent for a slayer's or an Eldritch Scoundrel's talents), write a question in this ticket with a recommendation and go on.

## Build

AFK, test-first (`tdd`); the only agent in the repo. A feat that grants a selection slot is probably new to the types (feats grant feats and features today): add it test-first in `rules`, the schema (one migration, DB reset per `CLAUDE.md`) and the loader, then the data by the extractor-module-plus-rewrite method of tickets 43–45. The new slot needs a `SlotId` (the link format from [ticket 46](46-shorter-links.md) must carry it; no link has shipped, so regenerate the frozen vector if the format changes) and a label on the sheet.

**Done when:** every check from [ticket 22](22-drop-key-type-prefix.md)'s Verification passes, plus real-data tests: a barbarian with Extra Rage Power at level 3 gets a rage power slot there and its option list is the barbarian's rage powers; taking it a second time gives a second slot; removing the feat removes the slot (and, by the removal rules, isn't offered while the slot's pick depends on it); a witch's Extra Hex gives a hex slot.

## Progress log

## Answer
