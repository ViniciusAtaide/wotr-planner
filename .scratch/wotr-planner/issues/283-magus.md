# Magus rows and arcana

Type: build (data)
Status: open
Blocked by: [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: rows and arcana read from the progressions.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half A `borrowed_selections.rs`, `follow_ups.rs`, `extra_picks.rs`, `archetype_losses.ron`) differ on `magus.ron`:
- **Bonus feats:** the magus's bonus feats at 5, 11, 17 are `MagusFeatSelection` (66befe7b24c42dd458952e3c47c93563, group `CombatFeat`), for the class and its archetypes; the planner gives a general feat slot.
- **Rows at 1:** the magus's cantrips and Detect Magic for the class and its archetypes; the magus proficiencies (the Armored Battlemage, `ArmoredBattlemageArchetype` 67ec8dcae6fb3d3439e5ae874ddc7b9b, keeps them and removes Greater Spell Combat at 14).
- **Arcana:** the Hexcrafter (`HexcrafterArchetype` 79ccf7a306a5d5547bebd97299f6fc89) keeps the Magus Arcana (`MagusArcanaSelection` e9dc4dfc73eaaf94aae27e0ed6cc9ada) at 3, removing it at 6 to 18; its hex arcana selection (`HexcrafterMagusHexArcanaSelection` ad6b9cecb5286d841a66e23cea3ef7bf) offers the Hex Channeler's channel too; the Bladebound (`BladeboundArchetype` 9b3915d38c7d4b6495415b38571e9d53) removes the arcana at 3, 9, 12 and True Magus at 20, and its black blade's features come at the game's levels (energy at 9, ego at 5, life drinker at 12, the true bond); the Eldritch Scion's arcana are its own selection (`EldritchScionArcana`) at the game's levels; the arcana's level prerequisites (`MagusClass level` 6, 9, 12, 15) and the Armored Battlemage exclusions are the game's.
- **The Sword Saint's chosen weapon** (`SwordSaintChosenWeaponFeature` c0b4ec0175e3ff940a45fc21f318a39a) is a weapon category filtered to melee weapons, not any Weapon Focus, and its list ignores no prerequisites.
- **The magus class** refuses Eldritch Scion levels (`not EldritchScionClass level 1`); the Arcane Rider's mounted combat feature and mount at 15.

## Findings (the game check, `known.txt` section 283)

| The check's group (area / kind) | Findings |
| --- | --- |
| progression rows / in game only | 29 |
| progression rows / a bonus pick of another group | 25 |
| prerequisites / missing in planner | 14 |
| progression rows / a further rank in planner only | 9 |
| progression rows / in planner only | 8 |
| progression rows / at another level | 4 |
| option lists / in game only | 2 |
| option lists / ignores prerequisites or not | 1 |
| option lists / in planner only | 1 |
| prerequisites / missing in planner, a blueprint it doesn't map | 1 |
| **all** | **94** |

The 21 entries: `archetype magus/arcane-rider`, `archetype magus/armored-battlemage`, `archetype magus/bladebound`, `archetype magus/eldritch-archer`, `archetype magus/eldritch-scion`, `archetype magus/hexcrafter`, `archetype magus/spell-dancer`, `archetype magus/sword-saint`, `class magus`, `feat list magus/sword-saint/chosen-weapon`, `selection magus/hexcrafter/hex-arcana`, `selection option magus/magus-arcana/bane-blade-level-15`, `selection option magus/magus-arcana/devoted-blade-level-12`, `selection option magus/magus-arcana/dimension-strike-level-9`, `selection option magus/magus-arcana/empowered-magic-level-6`, `selection option magus/magus-arcana/enduring-blade-level-6`, `selection option magus/magus-arcana/ghost-blade-level-9`, `selection option magus/magus-arcana/hasted-assault-level-9`, `selection option magus/magus-arcana/maximized-magic-level-12`, `selection option magus/magus-arcana/prescient-attack-level-6`, `selection option magus/magus-arcana/quickened-magic-level-15`.

The report lists each finding under its group, marked `[known: 283]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The rows, lists and prerequisites become the game's; `archetype_losses.ron`'s armored-battlemage, bladebound and hexcrafter entries follow; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `borrowed_selections.rs`: `a_hexcrafter_picks_a_witch_hex_at_4_and_an_arcana_or_hex_from_6` (the arcana slot at 3).
- `follow_ups.rs`: the Sword Saint's Weapon Focus test (melee weapons only).
- `scaling_features.rs`: `the_magus_archetypes_step_the_features_the_guide_gives_steps_to` (the Bladebound's).
- `archetype_losses.rs` (armored-battlemage, bladebound, hexcrafter).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 283 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
