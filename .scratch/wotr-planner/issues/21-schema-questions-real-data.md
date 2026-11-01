# Answer the schema questions from the real data

Type: research
Status: open
Blocked by: 17, 18, 19, 20
Map: [WotR Build Planner](../map.md)

## Question

With the hand-fixed real `game-data/`, answer each item of the map's **Schema questions for the real guide data** with numbers. This is read-only: change no files except this ticket.

- How many archetype level rows differ from their base class, and in which columns?
- Do all classes follow full/¾/½ BAB and good/poor saves? List the exceptions.
- Are kept ranks always 1..n?
- Are any grants duplicated within one row?
- Are selection options shared across selections?
- `one_of` slot vs feat-granting selection: what does the data show?
- Are `level_cap` and `further_ascensions` independent?
- Is mythic `group_name` a closed set?

For each, recommend keep or change, and the schema or type change it implies. The user decides; this ticket only reports.
