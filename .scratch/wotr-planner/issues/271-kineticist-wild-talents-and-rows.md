# The kineticist's wild talents and rows

Type: build (data)
Status: open
Blocked by: [270](270-kineticist-blasts-and-infusions.md) (the element and blast blueprints), [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: chained talent lists and rows read from the progressions.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half B part 2, `wild_talents.rs`) differ:
- **Wild talents' prerequisites** name the game's element blueprints (`SecondaryElement<Element>`, `ThirdElement<Element>`, `KineticKnightElementalFocus<Element>`, a blast), where the planner names `ElementalFocus<Element>`; several ask kineticist 2 the game doesn't. Flame Shield (`FlameShieldFeature` 733bbf836ec575b44b3c88b61a26282e) needs kineticist 10, not 2.
- **The bonus-feat talent is a chain:** per element, four selections (water six), `WildTalentBonusFeatFire` (4d14baf0ee4da2a4cb05fb4312921ee4) then `…Fire1`, `…Fire2` (cd107bf355f84b64f9f472ca288c208b), …, each opening once the previous one's feats are held; the first offers only the entry feats (fire: Iron Will, Persuasive, Dazzling Display). The planner's one list ignores prerequisites and can't be taken twice.
- **Expanded Defense** (`ExpandedDefense` d741f298dfae8fc40b4615aaf83b6548, `Ranks` 1) picks one defensive talent; an element taken later gains none.
- **Rows:** Infusion Specialization (`InfusionSpecialization` 5f64f446aef387d499a396b626d5fc51) is granted again at its steps (ranks); Elemental Overflow's bonus feature (`ElementalOverflowBonusFeature` 2496916d8465dbb4b9ddeafdf28c67d8) at 3; the Kinetic Knight's blade at 1 and its proficiencies; the Kinetic Sharpshooter's capstone, the Overwhelming Soul's and the Psychokineticist's own features; the Knight's Resolve and Mental Prowess repeat ([254](254-a-features-steps-follow-the-class-level.md)).

## Findings (the game check, `known.txt` section 271)

| The check's group (area / kind) | Findings |
| --- | --- |
| progression rows / a further rank in game only | 40 |
| prerequisites / missing in planner, a blueprint it doesn't map | 39 |
| option lists / in game only | 18 |
| prerequisites / not in game | 14 |
| progression rows / a further rank in planner only | 11 |
| progression rows / in game only | 11 |
| progression rows / in planner only | 3 |
| prerequisites / value differs | 1 |
| **all** | **137** |

The 28 entries: `archetype kineticist/blood-kineticist`, `archetype kineticist/dark-elementalist`, `archetype kineticist/elemental-engine`, `archetype kineticist/kinetic-knight`, `archetype kineticist/kinetic-sharpshooter`, `archetype kineticist/overwhelming-soul`, `archetype kineticist/psychokineticist`, `class kineticist`, `selection kineticist/wild-talents`, `selection option kineticist/wild-talents/aerial-adaptation`, `selection option kineticist/wild-talents/aerial-evasion`, `selection option kineticist/wild-talents/celerity`, `selection option kineticist/wild-talents/cold-adaptation`, `selection option kineticist/wild-talents/elemental-whispers`, `selection option kineticist/wild-talents/enduring-earth`, `selection option kineticist/wild-talents/fires-fury`, `selection option kineticist/wild-talents/flame-shield`, `selection option kineticist/wild-talents/foxfire`, `selection option kineticist/wild-talents/healing-burst`, `selection option kineticist/wild-talents/heat-adaptation`, `selection option kineticist/wild-talents/jagged-flesh`, `selection option kineticist/wild-talents/kinetic-healer`, `selection option kineticist/wild-talents/kinetic-revivification`, `selection option kineticist/wild-talents/skill-focus`, `selection option kineticist/wild-talents/skilled-kineticist`, `selection option kineticist/wild-talents/slick`, `selection option kineticist/wild-talents/tidal-wave`, `selection option kineticist/wild-talents/tremor-sense`.

The report lists each finding under its group, marked `[known: 271]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- Wild talents' prerequisites, the chained bonus-feat lists, Expanded Defense's single pick and the rows become the game's; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `wild_talents.rs`: `wild_talents_are_a_selection_of_22_talents_with_their_level_and_elements` (Flame Shield 10), `a_fire_kineticist_2_is_offered_its_elements_and_universal_level_1_talents` (less Flame Shield), `a_talent_needing_a_defensive_talent_waits_for_it` (the Knight at 10), `the_bonus_feat_talent_grants_one_slot_from_the_held_elements_lists` (the chain; repeatable while a variant is open), `expanded_defense_unlocks_the_expanded_elements_defensive_talent` (one talent; none at 15).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 271 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
