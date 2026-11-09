# Favorite Metamagic picks its metamagic, and the other in-game choices with no pick

Type: task
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

In the Max DC run (2026-11-09, see [119](119-caster-line.md)) Favorite Metamagic ("The chosen metamagic costs one spell level less") was taken at MR 3 and the Build asked nothing more. `libs/game-data/mythic/abilities.ron` gives it no selection and no parameter, so the link can't say which metamagic, and the agent had to tell the user "choose Persistent in the game". Second Bloodline, beside it in the same file, does open its pick (`selections: ["sorcerer/bloodline"]`), so the gap is per item.

## Scope

1. **Favorite Metamagic** takes its metamagic, from the guide's list: a parameter of the metamagic feats (or a selection of them), each held once. Whether it needs the metamagic feat taken is the guide's call; check the saved page.
2. **An audit** of every feat, mythic ability, mythic feat and option whose summary says "chosen", "choose", "pick" or "of your choice" and that has neither a parameter nor a selection, listed in the ticket with what the guide says each chooses. Each one the guide backs gets its pick; the rest are listed as checked.
3. A link with Favorite Metamagic and no metamagic (every older one) still opens, with the pick open in `todo`.

## Open questions (with recommendations)

1. **Parameter or selection?** *Recommend:* a parameter over the metamagic feats, as mythic feats already take one (ticket 32), so `options {values: true}` lists them and the link format already holds it. A selection would need option entries that duplicate the feats.
2. **Repeatable?** The guide may let it be taken once per metamagic. *Recommend:* follow the saved page; if it says nothing, once (the run found it "already held" at MR 7).

## Done when

Favorite Metamagic opens a metamagic choice in `todo` and `options`; the audit's list is in this ticket; each fixed item has a real-data test in `libs/seeds/tests/game_data/` and a note in `libs/game-data/README.md`; an older link still opens; `scripts/proof.sh` shows no other change; `scripts/check.sh --schema` passes.
