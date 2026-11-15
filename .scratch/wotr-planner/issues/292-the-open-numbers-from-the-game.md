# The numbers the guide lacks, re-answered from the game

Type: research
Status: open
Blocked by: none. Its findings feed the stopped queue's tickets 215, 217 and 219–224, which stay stopped until the user restarts them.
Model: Opus: each number is read from the blueprints and the code, and judged.
Map: [WotR Build Planner](../map.md)

## Problem

Ticket [234](234-game-blueprints-as-a-source.md)'s Answer, item 14, sent a background agent to do two things:
- re-answer the open numbers of [research/numbers-the-guide-lacks-2026-11-14.md](../research/numbers-the-guide-lacks-2026-11-14.md) from the game;
- file a ticket per area of difference.

The agent stopped without writing anything. Neither `research/game-source-check-2026-11-14.md` nor any of its tickets exists. Its session was archived on 2026-11-15 (user: "Yes, both").

Part of that work is now covered:
- the game check ([238](238-the-game-check.md)) compares the data with the blueprints;
- 242's research compares the tested facts with them;
- tickets 247–291 hold every difference found.

Not covered: the research file's own list of numbers the guide never had, which the planner doesn't hold yet. They belong to these tickets:
- Aivu and the Lich's Skeletal Champion (219);
- animal companions (217);
- martial rules where the guide is silent (215);
- gear (221–223);
- relics and artifacts (224);
- riding (220).

## Change

For each open number and rule in that file:
- read the blueprints (`blueprints/`, through the game check's reader or the scratch tools) and the decompiled code (`decompiled/`, WrathDecompiled `6ce4a6b`);
- write the game's value with its citation (blueprint name and GUID, or file and method);
- say whether it agrees with the file's answer, the user's or the online research's, or differs from it.

The write-up goes in `research/open-numbers-from-the-game-2026-11-xx.md`, in our own words, with no game text. Each blocked ticket among 215, 217 and 219–224 gets a line pointing to its numbers.

A difference with one of the user's facts goes to the user first (234, item 5): the Aivu table, the Skeletal Champion table, and the user's Answers in the research file.

## Done when

- Every open number in the research file has the game's value and a citation, or a stated reason it can't be read from the game. One such reason: a value the game computes only at runtime from something not in the blueprints.
- Each difference with the user's facts is listed for the user.
- Tickets 215, 217 and 219–224 each point to their numbers.
