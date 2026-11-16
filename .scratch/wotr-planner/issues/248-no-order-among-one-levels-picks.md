# No order among one level's picks

Type: build (engine rule)
Status: resolved
Blocked by: none
Model: Opus to build: it changes how the walk checks a level's picks.
Map: [WotR Build Planner](../map.md)

## Problem

Found by [242](242-the-tests-expected-values-from-the-game.md)'s research (2026-11-15): the game's code contradicts a rule of the engine.
- **The game:** inside one level-up, every selection of the same priority (feats, talents and class options are all `Features`) is applied in the order the player makes them, each checked against the preview with the earlier ones applied (`UnitLogic/Class/LevelUp/Actions/SelectFeature.cs`, `CalculatePriority` and `Check`; `UnitLogic/Class/LevelUp/LevelUpController.cs`, `AddAction`; WrathDecompiled 6ce4a6b). The bands keep an order: race, class, archetype, heritage, …, deity, path (domains, bloodlines, mysteries), features, spells; a mythic ability before features.
- **The planner** checks a level's slots in a fixed order, so a pick in an earlier slot can't lean on a pick in a later slot of the same level (the `MetLater` refusals).
- **Example:** at sorcerer 1, Greater Spell Focus in the bonus feat slot (`SorcererBonusFeat` d6dd06f454b34014ab0903cb1ed2ade3, group `WizardFeat`) with Spell Focus in the general feat slot is legal in the game; the planner refuses it.

## Change

- A level's picks of one band are legal when some order of them is: each pick is checked with the level's other picks of its band that it may follow, never with itself or with a later level's.
- The bands keep the game's order. The reasons stop naming a later slot of the same level.
- The self-checks (picker against walk, `allows`) follow the rule.

## Tests 242 marks as waiting on this rule

- `pick_order.rs`: `greater_spell_focus_in_the_sorcerers_magic_slot_names_spell_focus_in_the_later_general_slot` (the game allows the pair in either slot).
- `athletics_iii.rs`: `a_wizard_with_athletics_iii_opens_critical_focus_from_the_next_level_it_couldnt_before` asserts the slot order inside a level; 242 found it the planner's model, not a game fact. Its expected values are re-read against this rule.

## Answer

**Built** (2026-11-15). The game applies a level-up's picks band by band and the picks of one band in the order the player makes them, each checked with the earlier ones applied (`UnitLogic/Class/LevelUp/Actions/SelectFeature.cs`, `CalculatePriority` and `Check`; `UnitLogic/Class/LevelUp/LevelUpController.cs`, `AddAction`; WrathDecompiled 6ce4a6b). The planner now walks a level the same way.

- **The data.** `FeatureSelection::band` (`rules::Band`: `Spellbook`, `MythicAbility`, `Path`, `Features`, in the game's order; `Features` by default). `band: Path` on the 27 selections whose blueprint's group the game puts in the Path band (domains, the druid's bond, bloodlines, mysteries, curses, spirits, blessings, schools, the Lich's skeletal upgrade), `band: Spellbook` on the Thassilonian school; note `libs/game-data/notes/248-no-order-among-one-levels-picks.md`. A feat slot is the Features band, a mythic ability or feat slot the MythicAbility band (`Catalog::band`).
- **The walk** (`Walker::fill_bands`, `engine/walk/level.rs`): a level's slots, then the slots its picks grant, band by band; inside a band a pick refused for an unmet prerequisite at its turn is deferred (`Walker::deferred`, `fill` in `walk/hold.rs`) and tried again after the band's other picks and the slots they grant (`retry`), until no more holds; what still fails fails then, with its last reason. The sheet keeps the level's slot order. A Mythic rank's slots keep the planner's order (the rank after its level's slots, its general pick before its path's slots).
- **The pickers** (`AtSlot::try_pick`, `walk/cursor.rs`): a pick refused at its turn for an unmet prerequisite that the states after its band's later picks at its level open (`band_states`: from the walk without the pick, the checkpoints of its band and the band's end, `Probed::closure`) gets the walk's verdict of the Build with it (`Cursor::later_held`: open, `BreaksLater`, or its unmet prerequisite), so the picker and the walk can't disagree. One a later band's or a Mythic rank's pick meets keeps `Unmet::MetLater`, naming that slot. The gear check's "open only because of gear" reads the same states (`SlotTurn::fits`, computed only for a pick its turn refuses).
- **The MCP's guide** (`wotr-build` skill): the pick order paragraph says the bands, that picks of one band may need each other either way round, and keeps the refusal's "comes after this slot" for a later band's or a rank's pick.

**Decisions.**
- Ours: the band is data on the selection, read from its blueprint's group; the planner's three selections with no blueprint stay Features (two take no pick).
- Ours: trying again after the band's other picks rather than trying every order. It finds an order whenever prerequisites only grow as picks are held; a pick refused for what another pick of its band holds (a `Not`, `NoOtherOptionOfGroup`) is still refused when that pick's slot comes first, though the game's player could click it first. Recorded in the note and in `fill_bands`'s doc.
- Ours: a Mythic rank's slots are left in their order (its general pick is the MythicAbility band, before its path's slots, which agrees with the game; the order among its path's slots is the planner's).
- The game's Deity band has no selection in the data, so `Band` has no Deity variant.

**Tests.**
- Rules, hand-made data, `libs/rules/tests/engine/pick_order.rs`: `two_picks_of_one_level_lean_on_each_other_in_either_slot_order` (Cleave in the fighter's class slot with Power Attack in the general slot, and swapped: both possible, picker open, `agreement::judge` clean), `a_chain_of_three_picks_holds_whatever_slots_they_are_in` (Crane Wing, Crane Style, Dodge last first), `a_pick_is_never_met_by_itself_nor_by_a_later_levels_pick` (Power Attack at fighter 2 refuses Cleave at 1 with the plain reason; two picks that need each other both fail), `a_path_bands_pick_still_comes_before_its_levels_feats` (a Path-band bloodline needing Toughness in the general slot is refused with `MetLater`; the pickers agree), `a_mythic_ranks_pick_comes_after_its_levels_slots` (unchanged). `options.rs` `options_see_the_other_picks_of_their_level` (was `…_but_not_later_ones`).
- Real data, `libs/seeds/tests/game_data/pick_order.rs`: `greater_spell_focus_in_the_sorcerers_magic_slot_leans_on_spell_focus_in_the_same_levels_general_slot` off ignore (game: `SorcererProgression` 997665565ca80a649aedd72455c4df1f, `SorcererBonusFeat` d6dd06f454b34014ab0903cb1ed2ade3 group `WizardFeat`, Features; the code above), with `agreement::judge` clean; `the_selections_outside_the_features_band_are_the_games` (every selection's band, each cited by blueprint name, GUID and group).
- MCP: `choose.rs` `a_feat_whose_prerequisite_another_slot_of_its_level_holds_is_taken_in_either_slot` (replaces the refusal test of 126), `listing.rs` `the_guide_gives_the_pick_order_within_a_level` (new phrases).
- `athletics_iii.rs` re-read: its tests are about a Mythic rank's place at its level, the planner's model, which this ticket keeps; no change.
- The snapshot check and the agreement sweeps' short runs pass unchanged.

## Done when

- The pick-order test holds the game's answer, with the citation above.
- A fixture test in `rules` shows two picks of one level leaning on each other in either slot order, and a pick still refused when it leans on a later level's pick.
- The scoped checks pass, the self-checks' short runs among them.
