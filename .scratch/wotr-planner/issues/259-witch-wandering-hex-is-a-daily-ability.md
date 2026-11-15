# Witch Wandering Hex is a daily-use ability, not a pick

Type: build (engine and data)
Status: open
Blocked by: none
Model: Opus to build: a mythic ability's slot goes and its tests are rewritten.
Map: [WotR Build Planner](../map.md)

## Problem

Found by [242](242-the-tests-expected-values-from-the-game.md)'s research (2026-11-15), half B part 3 (`mythic.rs`), and the game check.
- **The game:** `WitchWanderingHex` (fb8490d1721bd5a4883c3b6559310351) is a feature, not a selection. It grants an ability with a daily resource that applies one of 12 basic witch hexes the caster lacks (`WitchWanderingHexAbility` b209beab784d93546b40a2fa2a09ffa8, `AbilityApplyFact`): no permanent pick, no Agony, no major hex.
- **Its prerequisite** is one of witch level 1, the Sylvan Trickster's rogue level 2 (`SylvanTricksterArchetype` 490394869f666c141bf8647b1a365220) or the Hexcrafter's magus level 4 (`HexcrafterArchetype` 79ccf7a306a5d5547bebd97299f6fc89). A shaman doesn't qualify.
- **The planner** gives a slot of `witch/hex`, needs what Extra Hex needs (any witch or shaman hex held: the 49 "not in game" leaves) and lists it among the abilities that give a slot.

## Findings (the game check, `known.txt` section 259)

| The check's group (area / kind) | Findings |
| --- | --- |
| prerequisites / not in game | 49 |
| prerequisites / missing in planner | 5 |
| **all** | **54** |

The 1 entries: `mythic ability witch-wandering-hex`.

The report lists each finding under its group, marked `[known: 259]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- Witch Wandering Hex is a mythic ability with no slot; its summary says it applies one of the 12 hexes once a day (the list in our own words, by blueprint name). Its prerequisite is the game's.
- The engine's list of "mythic abilities that give a pick" loses it.

## Tests 242 marks as waiting on this ticket

- `mythic.rs`: `witch_wandering_hex_and_beneficial_curse_give_a_slot_of_their_selection` (the Witch Wandering Hex half), `witch_wandering_hex_is_locked_without_a_hex` (a shaman isn't offered it), `witch_wandering_hex_gives_a_witch_hex_slot_offering_hexes_whose_prerequisites_are_met` (rewritten around the ability), `removing_witch_wandering_hex_or_beneficial_curse_is_refused_while_its_slot_holds_a_pick` (its half goes).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 259 (exit 0, run by the main session).
- The tests above hold the game's answers, cited; a data note `libs/game-data/notes/259-…` records the change.
- The scoped checks pass.
