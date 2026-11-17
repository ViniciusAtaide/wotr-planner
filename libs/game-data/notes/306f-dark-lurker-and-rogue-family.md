# 306f: the Dark Lurker's talent list, Evasion, and the rogue family's memberships

## What the data says now

- `selections/rogue/dark-lurker/rogue-talents.ron` (`DarkLurkerRogueTalentSelection` 1edbb68422c046f98a86f4726f6721b2,
  `DarkLurkerArchetype` 33eb002078e5437b8c25db4b80b4a4fe `AddFeatures` at 4, 6, 10, 12, 16, 18 and 20): the rogue's 38
  talents (shared) and the new option `rogue/rogue-talents/evasion` (`Evasion` 576933720c440aa4d8d42b0c54b77e80, owned
  by this selection, granting the `evasion` feature). The game removes `RogueTalentSelection` at every even level
  and Evasion at 2.
- `archetypes/rogue/dark-lurker.ron` has rows at 4, 6, 10, 12, 16, 18 and 20 naming it; each carries what the class's
  row there gives (Debilitating Injury and Uncanny Dodge at 4, Danger Sense at 6, 12 and 18, Advanced Talents at 10).
  The rows at 2, 8 and 14 hold no talent slot.
- `rogue/underground-chemist/discovery` offers the game's 29 (the rogue's 23 and 6 discoveries), written by the
  writer; its rows end at 10: from 12 on the chemist takes the rogue's own talents, so rows 12 to 20 are the class's.
- `shifter/feyform-shifter/rogue-talent` offers the rogue's 38 (the game's `RogueTalentSelection`), written.
- `rogue/master-of-all/skill-focus` no longer ignores prerequisites (`MasterOfAllSkillFocus` `IgnorePrerequisites` false).
- Written with `game-check --write option-lists --only <the six selections>`.

## Decisions

- The Evasion option carries no prerequisite: the blueprint's `PrerequisiteNoFeature` of Evasion is a feature's own
  NoFeature (the game check leaves it out), and a Dark Lurker has none to hold. Summary ours (main session's slice plan).
- `rogue/sylvan-trickster/fey-tricks` stays as it was: the game's 64 include the Hex Channeler's channel, which has
  no entry until 306j, so the writer leaves the selection unwritten. `a_sylvan_tricksters_fey_tricks_are_the_games_64`
  stays ignored, now on 306j.
- The Underground Chemist's rows 12 to 20 were removed (the check refuses a row equal to the class's) to make the
  discovery at 10 only, as the game has it; its `progression rows` known line (276) stays for the feature granted at 10.
- A key sorts where the one it replaces did: the Dark Lurker's rows hold one selection each.

## Tests

`importer_lists::the_dark_lurker_offers_its_own_list_with_evasion`,
`importer_lists::the_master_of_alls_skill_focus_checks_prerequisites`; un-ignored
`borrowed_selections::a_feyform_shifters_rogue_talent_is_the_rogues_selection` and
`data::an_underground_chemist_takes_a_discovery_at_10_only`.
