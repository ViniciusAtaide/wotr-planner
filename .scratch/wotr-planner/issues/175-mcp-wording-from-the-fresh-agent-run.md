# The MCP's wording: what a fresh agent had to find out by trial

Type: task
Status: open
Blocked by: none
Model: Sonnet to build; a fresh Sonnet agent repeats the run to check it.
Map: [WotR Build Planner](../map.md)

## Question

Ticket [168](168-gear-in-the-mcp.md)'s last Done-when item was run on 2026-11-12: a Sonnet agent, given only the server's instructions, tool descriptions and results over HTTP, built a level 13 Fighter with the three Two-Weapon Fighting feats, first without gear and then with a Belt of Incredible Dexterity +2. Both builds were right and it reported the gear-dependent pick. It named five places where the server's text left it guessing:

1. **A skill's value in `level.N.skill`.** The guide says "a skill's ranks take the number to reach", and the agent sent `{"perception": 4}` at level 1, which was refused ("no skill's total goes above the character level"). The text should say what the number is, with the limit.
2. **Whether the Mythic entries in `todo` must be filled.** After level 13 the todo lists the first ascension, the path and ranks 1 to 4. Nothing says a Build may leave them open.
3. **"Put the item before the pick it opens"** reads as being about feats. Lowering a score or moving an ability increase under a worn item is the same case, and the text doesn't say so.
4. **One refusal, two codes.** Taking off an item a pick needs is `breaks_later` from `choose` and `not_removable` from `withdraw`. Either make them one, or say in both descriptions that they are the same case.
5. **`start.score.<ability>` and the racial bonus.** The text doesn't say the point-buy score excludes the racial +2.

The agent also guessed a feat key that doesn't exist; the refusal's "Closest:" list recovered it, so nothing to change there.

## Build

AFK. `apps/mcp` (the instructions, the tool descriptions, the `wotr://build-guide` resource) and `.claude/skills/wotr-build/`. Wording only, unless point 4 is settled by making the codes one.

- Keep the instructions under the 4,000-character limit its test holds.
- Change no tool's behaviour for points 1, 2, 3 and 5.
- The snapshot check passes with the snapshot files untouched.

## Done when

- Each of the five points is answered by the server's own text.
- A fresh agent, given only the server over HTTP, repeats both builds without a refusal on points 1 and 4, and says whether anything else was unclear.
- `scripts/check.sh` passes.
