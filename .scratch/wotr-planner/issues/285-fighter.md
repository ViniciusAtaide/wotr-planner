# Fighter rows and lists

Type: build (data)
Status: open
Blocked by: [254](254-a-features-steps-follow-the-class-level.md)
Model: Opus to build: weapon training, the Titan's lists and the Dragonheir's dragons.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half A `bonus_feat_choices.rs`, `class_rules.rs`, `bloodline_ascendance.rs`, `archetype_losses.ron`) differ on `fighter.ron`:
- **Weapon Training's ranks:** at 9, 13 and 17 the fighter also picks a rank-up (`WeaponTrainingRankUpSelection` 5f3cc7b9a46b880448275763fe70c0b0), for the class and every archetype that keeps weapon training (`FighterProgression` b50e94b57be32f74892f381ae2a8905a).
- **The Titan Fighter** (`TitanFighterArchetype` 88a9dbb92a674ed995bae800896795f5): its Massive Attacks lists (`TitanFighterStyleTwoWeaponSelection2` 9c3e4b49b9f243f080c0432d2a1e5cbf, 6, 10) are the ranger's two-weapon lists with the five extras (Dodge, Great Fortitude, Iron Will, Lightning Reflexes, Toughness); it keeps Armor Mastery (`ArmorMastery` ae177f17cfb45264291d4d7c2cb64671) at 19. **The user's decision (2026-11-15):** Incredible Heft replaces Armor Training (added at 3, 7, 11, 15, 19 where Armor Training is removed), not Weapon Training as ticket 131 said; applied without asking again.
- **The Dragonheir Scion** (`DragonheirScionArchetype` 8dff97413c63c1147be8a5ca229abefc): its dragon (`DragonheirDragonSelection` 729411185291d704696e58316420fe38) offers the ten `Dragonheir<Colour>Progression`, its own, not the sorcerer's bloodlines (they grant no Draconic requisite feature, so the sorcerer's rows don't apply); its Fearful Might and defences step ([254](254-a-features-steps-follow-the-class-level.md)); the Wings feat row.
- **Other rows:** the Tower Shield Specialist's Tower Shield Training (`TowerShieldTraining` d2dbc71b45efa8749bd9c1924534b2bb) ranks; the Defender of the Hearth's Shield Block ([254](254-a-features-steps-follow-the-class-level.md)'s pass); the Aldori Defender's, the Armiger's and the Mutation Warrior's rows.

## Findings (the game check, `known.txt` section 285)

| The check's group (area / kind) | Findings |
| --- | --- |
| option lists / in game only | 25 |
| progression rows / in game only | 19 |
| option lists / in planner only | 10 |
| progression rows / a further rank in planner only | 8 |
| progression rows / a further rank in game only | 3 |
| progression rows / in planner only | 1 |
| **all** | **66** |

The 12 entries: `archetype fighter/aldori-defender`, `archetype fighter/armiger`, `archetype fighter/defender-of-the-hearth`, `archetype fighter/dragonheir-scion`, `archetype fighter/mutation-warrior`, `archetype fighter/titan-fighter`, `archetype fighter/tower-shield-specialist`, `class fighter`, `feat list fighter/titan-fighter/bonus-feats-10`, `feat list fighter/titan-fighter/bonus-feats-2`, `feat list fighter/titan-fighter/bonus-feats-6`, `selection fighter/dragonheir-scion/bloodline`.

The report lists each finding under its group, marked `[known: 285]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The rows and lists become the game's; the Dragonheir's dragons become its own progressions; `class_rules.rs`' doc comment says Armor Training; `archetype_losses.ron`'s titan-fighter entry follows; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `bonus_feat_choices.rs`: `the_bonus_feat_lists_are_the_guides` (Massive Attacks with the extras), `a_titan_fighter_picks_massive_attacks_from_the_two_weapon_style_feats_ignoring_prerequisites` (the offered set).
- `class_rules.rs`: `the_titan_fighter_has_no_weapon_training` (its doc comment).
- `scaling_features.rs`: `the_dragonheir_scions_defences_and_fearful_might_step`, `a_dragonheir_scion_of_each_colour_never_holds_the_bloodlines_powers`, `the_fighter_archetypes_step_the_features_the_guide_gives_steps_to`.
- `bloodline_ascendance.rs`: `a_dragonheir_scions_bloodline_gets_nothing_from_bloodline_ascendance` (agrees; re-read with the new dragons).
- `archetype_losses.rs` (titan-fighter).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 285 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
