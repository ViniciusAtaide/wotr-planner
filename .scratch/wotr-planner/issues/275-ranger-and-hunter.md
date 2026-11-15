# Ranger and hunter rows and lists

Type: build (data)
Status: open
Blocked by: [254](254-a-features-steps-follow-the-class-level.md), [253](253-a-progression-counts-its-class-levels-with-offsets.md) (the Divine Hunter's domains)
Model: Opus to build: rows read from the progressions; lists from the selections.
Map: [WotR Build Planner](../map.md)

## Problem

The game check and 242's research (half A `bonus_feat_choices.rs`, `archetype_losses.ron`; half B `scaling_features.rs`) differ on `ranger.ron` and `hunter.ron`:
- **The ranger's rows** (`RangerProgression` 97261d609529d834eba4fd4da1bc44dc): the favoured enemy and terrain picks are `FavoriteEnemySelection` (16cc2c937ea8d714193017780e7d4fc6) and `FavoriteTerrainSelection` (a6ea422d7308c0d428a541562faedefd) at the game's levels, with their rank-ups, for the class and each archetype; the combat style's later stages are `RangerStyleSelection6` (61f82ba786fe05643beb3cd3910233a8) and `RangerStyleSelection10` (78177315fc63b474ea3cbb8df38fafcd) at 6/10, 14, 18. The planner's own favoured-enemy and terrain rows differ.
- **Archetypes:** the Demonslayer (`DemonslayerArchetype` fd8c915590dcc5b41b73b80c3c0da8ee) keeps Quarry and Improved Quarry and adds its own at 11 and 19, and its favoured enemies are the three demon kinds; the Flamewarden (`FlamewardenArchetype` 300917c3479d27d47b4b4b52b1762e8d) removes Evasion at 9 and Improved Evasion at 16; the Freebooter's bane and the Flamewarden's and Stormwalker's weapon features follow [254](254-a-features-steps-follow-the-class-level.md).
- **The Stormwalker's lists are the ranger's archery lists** (`RangerStyleArcherySelection2` 3cf94df3f1cf9a443a880bc1ae9be3c6, 6, 10), with the five extras (Dodge, Great Fortitude, Iron Will, Lightning Reflexes, Toughness), all ignoring prerequisites.
- **The hunter** (`HunterProgression` f4943530be50c3c4da841a7b96cfae98): the teamwork list at 3 (`HunterTeamworkFeatSelection` 01046afc774beee48abde8e35da0f4ba) holds 13 feats, not Precise Shot; the Divine Hound's judgement uses are `JudgmentAdditionalUse` (ee50875819478774b8968701893b52f5); the archetypes' studied target, tactician, scapegoat and planned distraction repeats follow [254](254-a-features-steps-follow-the-class-level.md); the Forester's animal focus and the Urban Hunter's Captor are the game's features; the Divine Hunter's companions need their alignments.

## Findings (the game check, `known.txt` section 275)

| The check's group (area / kind) | Findings |
| --- | --- |
| progression rows / in game only | 114 |
| progression rows / a further rank in planner only | 24 |
| progression rows / in planner only | 17 |
| option lists / in game only | 15 |
| prerequisites / missing in planner | 2 |
| option lists / in planner only | 1 |
| **all** | **173** |

The 21 entries: `archetype hunter/colluding-scoundrel`, `archetype hunter/divine-hound`, `archetype hunter/divine-hunter`, `archetype hunter/forester`, `archetype hunter/tandem-executioner`, `archetype hunter/urban-hunter`, `archetype hunter/wandering-marksman`, `archetype ranger/demonslayer`, `archetype ranger/espionage-expert`, `archetype ranger/flamewarden`, `archetype ranger/freebooter`, `archetype ranger/nomad`, `archetype ranger/sable-company-marine`, `archetype ranger/stormwalker`, `class ranger`, `feat list hunter/bonus-feats-3`, `feat list ranger/bonus-feats-10b`, `feat list ranger/bonus-feats-2b`, `feat list ranger/bonus-feats-6b`, `selection option hunter/divine-hunter/otherworldly-companion/otherworldly-companion-celestial`, `selection option hunter/divine-hunter/otherworldly-companion/otherworldly-companion-fiendish`.

The report lists each finding under its group, marked `[known: 275]`, with the entry, the blueprint's name and GUID, and both values.

## Change

- The rows and lists become the game's; `archetype_losses.ron`'s ranger entries follow; a data note lists every change.

## Tests 242 marks as waiting on this ticket

- `bonus_feat_choices.rs`: `the_bonus_feat_lists_are_the_guides` (the Stormwalker's lists are the archery lists, extras included), `a_bonus_feat_choice_offers_a_held_feat_again_only_when_it_repeats` (the hunter's list has no Precise Shot; the test uses another feat), `the_hunters_and_the_stormwalkers_choices_offer_their_lists_ignoring_prerequisites` (the Stormwalker's extras).
- `archetype_losses.rs` (`archetype_losses.ron`'s demonslayer and flamewarden entries).
- `scaling_features.rs`: `the_ranger_archetypes_step_the_features_the_guide_gives_steps_to`, `the_hunter_archetypes_step_the_features_the_guide_gives_steps_to` (Touch of Flame and Wind Treader: [254](254-a-features-steps-follow-the-class-level.md)'s code pass).
- `option_levels.rs`: `a_rangers_style_slot_picks_from_the_held_styles_list` (re-read with the game's 6 and 10 selections).

## Done when

- The game check reports none of this ticket's findings and `known.txt` holds no line naming 275 (exit 0, run by the main session).
- The tests above hold the game's values, cited; the scoped checks pass.
