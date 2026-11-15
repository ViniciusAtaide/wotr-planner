# Druid and shifter rows and options

Type: build (data)
Status: superseded by 293 (the user, 2026-11-15): the importer writes its mechanical data; what it can't express becomes a gap ticket
Blocked by: [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: wild shapes and steps read from the progressions.
Map: [WotR Build Planner](../map.md)

## Problem

The game check, 242's research (half B part 1 `scaling_features.rs`; half A `archetype_losses.ron`, `borrowed_selections.rs`) and [243](243-ranked-features-and-filtered-pools.md)'s leftovers differ on `druid.ron` and `shifter.ron`:
- **Wild Shape (Wolf)** (`WildShapeIWolfFeature` 19bb148cb92db224abb431642d10efeb, `DruidProgression` 01006f2ac8866764fb7af135e73be81c): granted at 4 with its extra uses from 6 to 18 as the game's own features; the planner grants it again. The Blight Druid, the Defender of the True World, the Elemental Rampager and the Primal Druid hold the class's own feature, not a copy; the Feyspeaker (`FeyspeakerArchetype` da69747aa3dd0044ebff5f3d701cdde3) moves the class's Wolf to 6 and keeps the extra uses from 6, and keeps the Huge Elemental at 12; the Drovier and the Winter Child remove both.
- **Rows at 1:** Detect Magic (`DetectMagic` ee0b69e90bac14446a4cf9a050f87f2e) and the druid's alignment restriction (`DruidAlignmentRestriction` d00ff3791359311449c481126fbf71ce) for the class and its archetypes; the Blight Druid's elemental and Shambling Mound shapes; the Primal Druid (`PrimalDruidArchetype` c1c86e2997fd4257a13ef5601b5dc6dd) removes Resist Nature's Lure (`ResistNaturesLure` ad6a5b0e1a65c3540986cf9a7b006388) at 4; the Elemental Rampager's claws and the planner's elemental damage and rampage rows.
- **The Winter Child** (`WinterChildArchetype` 9f835c539f6e4a96a7b64461f4267d06): its domain selection (`WinterChildDomainSelection` 79922e274bf4444b93ea77ae0f876bd2) offers the eight druid domains, not Ice alone; Arctic Native (`ArcticNativeFeature` 776190c771e2487886ab1de9dfa8f2f1) steps at 4, 8, 12 on character level.
- **[243](243-ranked-features-and-filtered-pools.md)'s leftovers:** Fey Magic's spell parameter (`FeyspeakerMagicSelection` 237efc6a131c8174bbd466ed239f21fc chooses a spell, which the planner doesn't hold) and the Elemental Rampager's element progressions.
- **The shifter:** Manticore Shape (`ShifterWildShapeManticoreFeatureLevelUp` 719be33c87f94ed58414ba3eb5a4b664) steps at 4, 8, 14; Lycanthropic Aspect (`WereTouchedShifter` a30313b4b01a4049ae2f50884917e443) at 1, 8, 15; the Fiendflesh, Griffonheart and Rageshaper aspects and defences follow [254](254-a-features-steps-follow-the-class-level.md); the final aspect rows; the Feyform's talent is the rogue's selection (its list in [276](276-rogue-and-slayer-talents.md)); the Dragonblood Shifter's final form (`ShifterDragonFormFeatureFinal` c676ba5eb744492583d989244c81f127) is a feature, not a choice.
- The druid's domain copies (`<Domain>DomainProgressionDruid`) are [244](244-the-game-data-laid-out-by-blueprint.md)'s.

## Findings (the game check, `known.txt` section 277)

| The check's group (area / kind) | Findings |
| --- | --- |
| progression rows / a further rank in planner only | 82 |
| progression rows / in game only | 46 |
| option lists / in game only | 26 |
| progression rows / in planner only | 12 |
| option lists / a choice the game doesn't offer | 1 |
| option lists / in planner only | 1 |
| **all** | **168** |

The 17 entries: `archetype druid/blight-druid`, `archetype druid/defender-of-the-true-world`, `archetype druid/drovier`, `archetype druid/elemental-rampager`, `archetype druid/feyspeaker`, `archetype druid/primal-druid`, `archetype druid/winter-child`, `archetype shifter/child-of-the-manticore`, `archetype shifter/fiendflesh-shifter`, `archetype shifter/griffonheart-shifter`, `archetype shifter/rageshaper`, `archetype shifter/weretouched`, `archetype shifter/wild-effigy`, `class druid`, `selection druid/winter-child/domain`, `selection shifter/dragonblood-shifter/greater-wyrmshifter`, `selection shifter/feyform-shifter/rogue-talent`.

The report lists each finding under its group, marked `[known: 277]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The rows and options become the game's with [254](254-a-features-steps-follow-the-class-level.md)'s model; `archetype_losses.ron`'s feyspeaker and primal-druid entries follow; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `scaling_features.rs`: `the_druids_wild_shape_wolf_steps_every_two_levels_and_in_each_archetypes_own_copy` (the four archetypes hold the class's; the Feyspeaker's at 6), `the_druid_archetypes_step_the_features_the_guide_gives_steps_to` (Arctic Native 4, 8, 12; Primal Size: [254](254-a-features-steps-follow-the-class-level.md)'s pass), `the_shifters_aspects_and_defences_step` (Lycanthropic Aspect 1, 8, 15), `the_shifter_archetypes_step_the_features_the_guide_gives_steps_to` (Manticore 4, 8, 14).
- `option_families.rs`: `a_blight_druid_skips_the_animal_domains_companion_and_a_winter_child_holds_ice_with_no_pick` (the Winter Child picks among eight).
- `archetype_losses.rs` (feyspeaker, primal-druid).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 277 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
