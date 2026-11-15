# The kineticist's blasts and infusions

Type: build (data and model: the blast choice)
Status: open
Blocked by: none
Model: Opus to build: infusion prerequisites name blasts, which the planner doesn't model yet.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half B part 2, `infusions.rs`) differ on the infusions and the element choices:
- **Infusions need blasts, not elements:** each infusion's prerequisite names blast features (`FireBlastFeature`, `BlueFlameBlastFeature`, `MetalBlastFeature`, the composites …), which the planner doesn't map; the planner names elements (`ElementalFocusFire`, …). With the Air/Electric and Water/Cold blast choices and the composites counted, Pushing (`PushingInfusionFeature` fbb97f35a41b71c4cbc36c5f3995b892), Entangling, Magnetic, Rare-Metal, Synaptic, Torrent, Cyclone, Spray, Chilling, Cloud (`CloudInfusion` 3c53ee4965a13d74e81b37ae34f0861b: composites only), Fragmentation and Pure Flame (`PureFlameInfusionFeature` 1dfd9b8e1439e4a4ab4b6b11f5ea676a: Blue Flame only) differ.
- **Infusion levels are per blueprint,** not twice the infusion's level: the level-1 infusions need no kineticist level; Exploding Arrows (`ExplodingArrowsInfusion` 7c8ded7015114e4086b3dbdd16388b29) 3; Rending Arrows 11.
- **The level-1 infusion is a pick:** `KineticistProgression` (b79e92dd495edd64e90fb483c504b8df) holds `InfusionSelection` (58d6f8e9eea63f6418b107ce64f315ea) at 1, for the base class and every archetype that keeps it (Dark Elementalist, Elemental Engine, Overwhelming Soul, Psychokineticist); the planner's "Infusion" feature at 1 is that pick.
- **Elemental Focus:** 13 infusions also need `ElementalFocusSelection` (1f3a15a3ae8a5524ab8b97f469bf4e3d); the Kinetic Knight (`KineticKnightArchetype` 7d61d9b2250260a45b18c5634524a8fb) replaces it with its own selection (`KineticKnightElementalFocusSelection` b1f296f0bd16bc242ae35d0638df82eb), so it can't take them.
- **The element selections** are the game's own blueprints (Expanded Element's `SecondatyElementalFocusSelection` 4204bc10b3d5db440b1f52f0c375848b offers `SecondaryElement<Element>`; the Kinetic Knight's `KineticKnightElementalFocus<Element>`; the Blood Kineticist's), where the planner shares `kineticist/element/*`.

## Findings (the game check, `known.txt` section 270)

| The check's group (area / kind) | Findings |
| --- | --- |
| prerequisites / missing in planner, a blueprint it doesn't map | 87 |
| prerequisites / not in game | 39 |
| prerequisites / missing in planner | 16 |
| option lists / in game only | 11 |
| option lists / in planner only | 8 |
| prerequisites / value differs | 2 |
| **all** | **163** |

The 34 entries: `selection kineticist/blood-kineticist/element`, `selection kineticist/expanded-element`, `selection kineticist/kinetic-knight/element`, `selection option kineticist/element/air`, `selection option kineticist/element/earth`, `selection option kineticist/element/fire`, `selection option kineticist/infusions/bowling-infusion`, `selection option kineticist/infusions/burning-infusion`, `selection option kineticist/infusions/chilling-infusion`, `selection option kineticist/infusions/cloud`, `selection option kineticist/infusions/cyclone`, `selection option kineticist/infusions/dazzling-infusion`, `selection option kineticist/infusions/deadly-earth`, `selection option kineticist/infusions/detonation`, `selection option kineticist/infusions/entangling-infusion`, `selection option kineticist/infusions/eruption`, `selection option kineticist/infusions/exploding-arrows`, `selection option kineticist/infusions/extended-range`, `selection option kineticist/infusions/fan-of-flames`, `selection option kineticist/infusions/flash-infusion`, `selection option kineticist/infusions/foxfire-infusion`, `selection option kineticist/infusions/fragmentation`, `selection option kineticist/infusions/kinetic-blade`, `selection option kineticist/infusions/magnetic-infusion`, `selection option kineticist/infusions/pure-flame-infusion`, `selection option kineticist/infusions/pushing-infusion`, `selection option kineticist/infusions/rare-metal-infusion`, `selection option kineticist/infusions/rending-arrows`, `selection option kineticist/infusions/spindle`, `selection option kineticist/infusions/spray`, `selection option kineticist/infusions/synaptic-infusion`, `selection option kineticist/infusions/torrent`, `selection option kineticist/infusions/unraveling-infusion`, `selection option kineticist/infusions/wall`.

The report lists each finding under its group, marked `[known: 270]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- Model the blast choice (an element's blasts, the Air/Electric and Water/Cold choice, the composites from a second element) so a Build holds the game's blast features, and key the infusions' prerequisites to them.
- Infusion levels, the level-1 pick, the Elemental Focus leaf and the element selections become the game's; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `infusions.rs`: `infusions_are_a_selection_of_32_with_their_level_and_elements` (levels and blasts), `a_kineticist_3_has_one_infusion_slot_offering_its_elements_and_universal_infusions_of_level_1` (slots at 1 and 3), `a_higher_level_infusion_is_locked_with_a_reason_and_opens_at_its_level`, `a_fire_kineticist_isnt_offered_a_water_only_infusion` (Pure Flame needs Blue Flame), `an_infusion_waits_for_the_infusion_or_talent_it_needs` (Expanded Element water: Spray and Pushing only with the Water blast), `every_kineticist_archetype_picks_infusions_at_its_tables_levels` (the slot at 1).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 270 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
