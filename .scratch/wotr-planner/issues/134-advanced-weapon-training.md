# Advanced Weapon Training picks its option

Type: design
Status: open
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

Ticket [125](125-favorite-metamagic-picks-its-metamagic.md)'s audit found that the guide backs Advanced Weapon Training, but the planner doesn't build it. The feat's summary says it takes one of seven Weapon Training options: Armed Bravery, Combat Maneuver Defense, Effortless Dual Wielding, Fighter's Finesse, Fighter's Reflexes, Fighter's Tactics and Trained Initiative. Three things stand in the way:

- The seven options aren't in the data.
- A feat can't give a selection slot. `extra_pick_of` gives a slot only of a selection the Build already holds.
- The fighter's own Weapon Training picks no weapon group. It is one of ticket [54](54-mercies-and-enforcer-exploits.md)'s "choices that gate no pick", which stay labels.

So a fighter can take Advanced Weapon Training, but the Build can't say which option it took, and no prerequisite can check one.

Options:
- **A.** A selection `fighter/advanced-weapon-training` with the seven options in `fighter.ron`, plus a new grant shape: a feat that gives one slot of a selection at the level it's taken.
- **B.** A parameter on the feat whose values are the seven options: a new value kind, as Favorite Metamagic got in ticket 125.
- **C.** Leave it unbuilt, and record it as checked in `pick_values`' audit list.

*Recommend:* B if no option has picks or prerequisites of its own, because a parameter needs no new grant shape. Otherwise A. Before choosing, read each option's prerequisites on the saved `fighter` page. Weapon Training's groups stay labels (ticket 54) unless an option's prerequisite names a group.

## Done when

A fighter who takes Advanced Weapon Training chooses one of the seven options. The choice round-trips through the link with an additive tag (`V1_LINK` unchanged), and a link without it opens with the value in `todo`. Real-data tests cover the options and their prerequisites. The item leaves `pick_values::every_item_whose_summary_chooses_has_its_pick_or_was_checked`'s list of unbuilt items. `scripts/proof.sh` shows that only Builds holding the feat differ. `scripts/check.sh --schema` passes.

## Answer

**Recommendation taken** (user, 2026-11-10): B, a parameter, if no option has picks or prerequisites of its own; A, a selection, otherwise. Weapon Training's groups stay labels.
