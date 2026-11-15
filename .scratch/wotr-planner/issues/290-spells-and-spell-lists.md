# Spells and spell lists

Type: build (data)
Status: open
Blocked by: none
Model: Sonnet to build: each list is compared with its blueprint's levels, spell by spell.
Map: [WotR Build Planner](../map.md)

## Problem

The game check compares spells and spell lists for their mapping only (238's decision), so these differences come from 242's research (half B part 4, `spells.rs`) alone:
- **Descriptors:** Hold Monster (`HoldMonster` 41e8a952da7a5c247b3ec1c2dbb73018) and Hold Person (`HoldPerson` c7104f7526c4c524f91474614054547e) also carry Paralysis, where the planner's descriptor set has it.
- **The class lists against each list blueprint's `SpellsByLevel`:** the wizard's 280 agree, as do the demon's and the merged angel's. The others differ by name and level: the inquisitor's game list has four more (Keen Edge 3, Prayer 3, Ignition 0, Stabilize 0); the cleric's two (Curse of Magic Negation 5, Stabilize 0); the druid's three (Bone Fists 2, Nature's Exile 3, Stabilize 0), and Thirsting Entanglement isn't on it; the witch's has Haze of Dreams 1 and Stabilize 0, and the data's four footnoted cantrips aren't on it; the bloodrager's 14 more and the hunter's 18 more, Call Lightning at 3; the ranger's Cure Light Wounds at 2; the magus's, paladin's, shaman's, warpriest's, azata's, lich's, angel's, aeon's and trickster's lists differ too.
- **Archetype and path lists:** the Magic Deceiver's Dispel Magic is the targeted one; the paladin's Protection from Chaos and from Evil; the Trickster's Beer Elementals.

## Change

- Each list in `libs/game-data/spells/` becomes its blueprint's, level by level; the descriptors become the game's where the planner's set has them; a data note lists every change, with each list's blueprint and GUID.
- A data-driven comparison (a real-data test over every list, or a check in `tools/game-check` beside the mapping) keeps them from drifting; which one is the builder's choice, recorded in the Answer.

## Tests 242 marks as waiting on this ticket

- `spells.rs`: `hold_monster_is_a_will_negates_compulsion_on_the_wizard_and_sorcerer_list_at_5` (Paralysis), `every_list_table_is_a_spell_list` (the entries per list), `footnoted_entries_are_only_for_their_archetype_or_story_branch` (the deceiver's, paladin's and trickster's entries).

## Done when

- Every list agrees with its blueprint, shown by the comparison above; the tests above hold the game's values, cited.
- The scoped checks pass.
