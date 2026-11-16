# 248: no order among one level's picks

**What the data says now.** A Feature selection carries `band` (`FeatureSelection::band`, `rules::Band`): where its
picks come among a level-up's. The game applies a level-up's picks band by band (`Spellbook`, `MythicAbility`,
`Path`, `Features`; the game's Deity band, just before Path, has no selection here) and the picks of one band in
the order the player makes them, each checked with the earlier ones applied. `Features` is the default and is left
out of the files; the others are written:

- `band: Path` on the selections whose blueprint's group the game puts in the Path band: the domains
  (`cleric/domain`, `inquisitor/domain`, `inquisitor/sacred-huntsmaster/domain`, `hunter/divine-hunter/domain`,
  `arcanist/magic-deceiver/domain`, `druid/domain`, `druid/blight-druid/domain`, `druid/winter-child/domain`), the
  druid's `druid/nature-bond`, the bloodlines (`sorcerer/bloodline` and the Empyreal, Sage, Seeker and Sylvan ones,
  `magus/eldritch-scion/bloodline`), the mysteries (`oracle/mystery`, `oracle/second-mystery`, the Lone Strider's and
  the Wind Whisperer's), the curses (`oracle/oracles-curse`, `witch/stigmatized-witch/curse`), the spirits
  (`shaman/spirit`, `shaman/wildland-shaman/spirit`), `warpriest/blessings`, the schools (`wizard/school`,
  `wizard/elemental-specialist/school`) and `lich/skeleton-upgrade`.
- `band: Spellbook` on `wizard/thassilonian-specialist/school` (group `ThassilonianSpellbook`).

Game: `UnitLogic/Class/LevelUp/Actions/SelectFeature.cs`, `CalculatePriority` (the group to the band) and `Check`;
`UnitLogic/Class/LevelUp/LevelUpController.cs`, `AddAction` (an action of the last action's band or a later one is
appended and checked against the preview; an earlier band's is inserted before the later bands); WrathDecompiled
6ce4a6b. Each selection's group is its blueprint's `Group`, cited in the test below.

**Not every game group has a selection here.** The bloodrager's bloodlines (`BloodragerBloodline`), the Separatist's
second domain (`SeparatistSecondaryDomain`) and the Draconic bloodline choices are groups the game leaves in the
Features band, as it does a feat, a talent and a bonus feat list. The planner's selections with no blueprint (the
Possessed Oracle's curse, the Spell Master's school, the slayer's combat style) stay in the Features band: the first
two offer one option and take no pick.

**What the engine does with it.** `fill_bands` in `rules/src/engine/walk/level.rs`: a level's slots are filled band by
band; inside a band a pick refused for an unmet prerequisite at its turn is tried again after the band's other picks,
until no more holds. A Mythic rank's slots keep the planner's order (its picks come after the level's, and its general
pick before its path's slots).

**Not modelled.** Trying every order: a pick refused for what another pick of its band holds (a `Not`, a
`NoOtherOptionOfGroup`) is refused when that pick's slot comes first, though the game's player could take it first.

**Test.** `libs/seeds/tests/game_data/pick_order.rs`: `the_selections_outside_the_features_band_are_the_games` and
`greater_spell_focus_in_the_sorcerers_magic_slot_leans_on_spell_focus_in_the_same_levels_general_slot`.
