# 251: proficiency holds where a slot ignores prerequisites

**What the data says now.**

- `requires_proficiency: true` (`Feat::requires_proficiency`, `MythicFeat::requires_proficiency`) on `weapon-focus`,
  `greater-weapon-focus`, `weapon-specialization`, `greater-weapon-specialization`, `improved-critical`,
  `improved-improved-critical`, `improved-improved-improved-critical`,
  `improved-improved-improved-critical-improved` (`feats.ron`) and `improved-critical-mythic` (`mythic/feats.ron`):
  the weapon such a feat is taken for must be one the character is proficient with (`Catalog::proficient`, ticket
  198's rule), in every slot, whatever the slot's list ignores. An option whose chosen feat is one of them (the rogue
  talent Weapon Focus, the Battle oracle's Weapon Mastery) checks it for the option's weapon too.
- `skips_proficiency: true` (`FeatList::skips_proficiency`) on `magus/sword-saint/chosen-weapon`: its pick is not
  the game's Weapon Focus but a feature that gives Weapon Focus and proficiency with the weapon, so any weapon the list
  offers is taken, and the pick makes the character proficient with it (`CharacterState::weapon_proficiencies`, which
  `Catalog::proficient` reads): the feats that require proficiency take the chosen weapon afterwards (Improved
  Critical at magus 11), and the sheet's weapons list it.

**The game.** `Blueprints/Classes/Selection/BlueprintParametrizedFeature.cs`, `CanSelect` (WrathDecompiled 6ce4a6b):
the weapon must be among the unit's proficiencies when the feature sets `RequireProficiency`, unless the global
`IgnorePrerequisites` context flag (a debug cheat) is on; the selection's own `IgnorePrerequisites` skips only the
feature's prerequisites. The flag is set on `WeaponFocus` 1e1f627d26ad36f43bbd26cc2bf8ac7e, `WeaponFocusGreater`
09c9e82965fb4334b984a1e9df3bd088, `WeaponSpecialization` 31470b17e8446ae4ea0dacd6c5817d86,
`WeaponSpecializationGreater` 7cf5edc65e785a24f9cf93af987d66b3, `ImprovedCritical` f4201c85a991369408740c6888362e20,
`TricksterImprovedImprovedCritical` 56f94badbba018b4b8277ce6e2e79e72, `TricksterImprovedImprovedImprovedCritical`
006a966007802a0478c9e21007207aac, `TricksterImprovedImprovedImprovedCriticalImproved`
319c882ab3cc51544ad2f3f43633d5b1 and `ImprovedCriticalMythicFeat` 8bc0190a4ec04bd489eec290aeaa6d07. The Sword
Saint's `SwordSaintChosenWeaponSelection` 3130e3083845b7b4fa2ac5b6f1a1ae6e offers `SwordSaintChosenWeaponFeature`
c0b4ec0175e3ff940a45fc21f318a39a, which doesn't set it; its `KensaiChosenWeapon` adds the weapon's category to the
unit's proficiencies (`Owner.Proficiencies.Add`) and gives Weapon Focus in it
(`Designers/Mechanics/Facts/KensaiChosenWeapon.cs`, `OnActivate`). No other blueprint carries that component.

**Not modelled.**

- `FencingGrace`, `SlashingGrace`, `PointBlankMaster`, `WeaponFocusMythicFeat` and `WeaponSpecializationMythicFeat`
  set the flag too, but the planner takes them for no weapon (they read the Weapon Focus or Weapon Specialization
  held), so there is nothing to check.
- `CanSelect` also refuses a weapon of the `Disabled` subcategory (`Enums/WeaponCategoryExtension.cs`: the sling,
  siangham, hand crossbow, repeating crossbows, shuriken, other natural weapons). None of them is in `weapons.ron`.

**Tests.** `libs/seeds/tests/game_data/parameterised_feats.rs`:
`a_warpriests_weapon_focus_slot_opens_only_the_weapons_he_is_proficient_with`,
`the_feats_that_require_proficiency_are_the_games`, `a_sword_saint_chooses_a_weapon_he_isnt_proficient_with_yet`,
`a_sword_saint_is_proficient_with_his_chosen_weapon`; `bonus_feat_choices.rs`:
`the_hunters_and_the_stormwalkers_choices_offer_their_lists_ignoring_prerequisites` (the Stormwalker's archery list
from 10 refuses Improved Critical in the exotic weapons).
