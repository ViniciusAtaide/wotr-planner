# Sorcerer and bloodrager bloodlines and rows

Type: build (data)
Status: superseded by 293 (the user, 2026-11-15): the importer writes its mechanical data; what it can't express becomes a gap ticket
Blocked by: [254](254-a-features-steps-follow-the-class-level.md), [257](257-bloodline-ascendance-is-one-pick.md) (it changes `sorcerer.ron` first)
Model: Opus to build: bloodline copies, rows and lists read from the progressions.
Map: [WotR Build Planner](../map.md)

## Problem

The game check, 242's research (half A `class_skills.rs`, `archetype_losses.ron`; half B `report_gaps.rs`, `option_levels.rs`, `option_follow_ups.rs`, `scaling_features.rs`) and [243](243-ranked-features-and-filtered-pools.md)'s leftovers differ on `sorcerer.ron` and `bloodrager.ron`:
- **The sorcerer's level-1 bonus feat** (`SorcererBonusFeat` d6dd06f454b34014ab0903cb1ed2ade3) offers 21 feats (Combat Casting, Spell Penetration and Greater, Augment and Superior Summoning, 11 metamagics without Completely Normal, Elemental Focus and Greater, Spell Focus and Greater, Spell Specialization), not the 35 Magic feats; the Seeker's (`SeekerFeatSelection` c6b609279cc3174478624182ac1ad812) has its own 26. **The user's decision (2026-11-15):** the game's 21 replace ticket 92's decision, applied without asking again.
- **Archetype bloodlines:** the Crossblooded's second bloodline (`CrossbloodedSecondaryBloodlineSelection` 60c99d78a70e0b44f87ba01d02d909a6) offers its own 21 `CrossbloodedSecondaryBloodline<Name>Progression`, not the sorcerer's bloodlines; the Empyreal's, the Sage's (`SageSorcererArchetype` 00b990c8be2117e45ae6514ee4ef561c) and the Sylvan's (`SylvanSorcererArchetype` 711d5024ecc75f346b9cda609c3a1f83) `bloodline` entries name `SorcererBloodlineSelection` (24bef8d1bee12274686f6da6ccbc8914, 20 options) for a fixed bloodline: the mapping or the row is wrong. The Seeker's bloodlines are their own progressions (`SeekerBloodlineAbyssalProgression` 17b752be1e0f4a34e8914df52eebeb75, …) that keep Strength of the Abyss at 9 and 13 and the claws, and lack the 3 resistances and the 15 summons and wings.
- **Bloodline rows:** Detect Magic and the Celestial bloodline's rows for the archetypes that hold them; the Earth elemental bloodline (`BloodlineElementalEarthProgression` 32393034410fb2f4d9c8beaa5c8c8ab7) has Elemental Movement at 15 too (`BloodlineElementalEarthElementalMovementFeature` 277d9c2a2392e8940a452888aa67f32e); the Nine-Tailed Heir's Magical Tail ranks ([254](254-a-features-steps-follow-the-class-level.md)); the Geomancer's terrain rank-ups.
- **243's leftover:** Strength of the Abyss's second step (`BloodlineAbyssalStrengthAbilityAddLevel2` 48d81d94ddf0cc84abbde97718cb25f2), which the planner doesn't grant.
- **The bloodrager** (`BloodragerProgression` 69dec3bda9a94914bb159cba128300b9): the Hag-Riven (`HagRivenArchetype` 61edfe08d9e04a11844e5fcc01da3602) keeps the bloodline selection and its spells (the game's 15 other bloodlines exclude it by prerequisite; the planner's own list of four is the same set in another shape); the Mixed-Blood Rager (`MixedBloodRagerArchetype` 10d92d54d19e4a59b5bbd4259ec0974f) removes the spellbook (`ClassData.AddArchetype`, WrathDecompiled 6ce4a6b); the Steelblood's Armor Training (`SteelbloodArchetype` 32a5dff92373a9641b43e97d453b9369) at 5, 9, 13, 17; the Greenrager's summons at 7, 10, 13 (`GreenragerArchetype` 5648585af75596f4a9fa3ae385127f57); the Primalist's choices are the game's selections; the bloodline feat lists' extras.
- The bloodline feat lists' Skill Focus is [255](255-a-parametrised-feat-at-one-value.md)'s; Ascendance and the Empyreal's Ascension are [257](257-bloodline-ascendance-is-one-pick.md)'s; the Primalist's step rule [254](254-a-features-steps-follow-the-class-level.md)'s.

## Findings (the game check, `known.txt` section 281)

| The check's group (area / kind) | Findings |
| --- | --- |
| option lists / in game only | 97 |
| progression rows / in game only | 34 |
| progression rows / a further rank in planner only | 25 |
| option lists / in planner only | 22 |
| prerequisites / missing in planner | 15 |
| progression rows / in planner only | 9 |
| option lists / a choice the game doesn't offer | 1 |
| **all** | **203** |

The 39 entries: `archetype bloodrager/bloodrider`, `archetype bloodrager/greenrager`, `archetype bloodrager/hag-riven`, `archetype bloodrager/primalist`, `archetype bloodrager/reformed-fiend`, `archetype bloodrager/spelleater`, `archetype bloodrager/steelblood`, `archetype sorcerer/crossblooded`, `archetype sorcerer/empyreal-sorcerer`, `archetype sorcerer/geomancer`, `archetype sorcerer/nine-tailed-heir`, `archetype sorcerer/overwhelming-mage`, `archetype sorcerer/sage-sorcerer`, `archetype sorcerer/seeker-sorcerer`, `archetype sorcerer/sylvan-sorcerer`, `class sorcerer`, `feat list bloodrager/bloodline/arcane/feats`, `feat list bloodrager/bloodline/elemental/feats`, `selection bloodrager/hag-riven/bloodline`, `selection option bloodrager/bloodline/abyssal`, `selection option bloodrager/bloodline/celestial`, `selection option bloodrager/bloodline/draconic-black`, `selection option bloodrager/bloodline/draconic-blue`, `selection option bloodrager/bloodline/draconic-brass`, `selection option bloodrager/bloodline/draconic-bronze`, `selection option bloodrager/bloodline/draconic-copper`, `selection option bloodrager/bloodline/draconic-gold`, `selection option bloodrager/bloodline/draconic-green`, `selection option bloodrager/bloodline/draconic-red`, `selection option bloodrager/bloodline/draconic-silver`, `selection option bloodrager/bloodline/draconic-white`, `selection option bloodrager/bloodline/infernal`, `selection option bloodrager/bloodline/serpentine`, `selection option bloodrager/bloodline/undead`, `selection sorcerer/crossblooded/second-bloodline`, `selection sorcerer/empyreal-sorcerer/bloodline`, `selection sorcerer/geomancer/geomancy`, `selection sorcerer/sage-sorcerer/bloodline`, `selection sorcerer/sylvan-sorcerer/bloodline`.

The report lists each finding under its group, marked `[known: 281]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The bonus feat list, the archetype bloodlines, the rows and the lists become the game's; `archetype_losses.ron`'s hag-riven and mixed-blood-rager entries follow; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `report_gaps.rs`: `every_sorcerers_level_1_bonus_feat_is_a_magic_feat` (21; the Seeker's 26).
- `option_levels.rs`: `a_seeker_skips_the_bloodline_rows_at_3_9_and_15` (it keeps Strength of the Abyss).
- `option_follow_ups.rs`: `elemental_movement_follows_the_element_and_earth_has_none` (Earth has one).
- `class_skills.rs`: the Seeker's Trickery is [261](261-class-tables-saves-bab-and-skill-points.md)'s.
- `scaling_features.rs`: `the_bloodrager_archetypes_step_the_features_the_guide_gives_steps_to` (Furious Summoning, Abyssal Bloodrage: [254](254-a-features-steps-follow-the-class-level.md)'s pass), `the_bloodragers_infernal_resistance_and_abyssal_bloodrage_step`.
- `archetype_losses.rs` (hag-riven, mixed-blood-rager).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 281 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
