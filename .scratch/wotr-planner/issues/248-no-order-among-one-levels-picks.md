# No order among one level's picks

Type: build (engine rule)
Status: open
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

## Done when

- The pick-order test holds the game's answer, with the citation above.
- A fixture test in `rules` shows two picks of one level leaning on each other in either slot order, and a pick still refused when it leans on a later level's pick.
- The scoped checks pass, the self-checks' short runs among them.
