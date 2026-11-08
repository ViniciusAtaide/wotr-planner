# Feat type icons

Type: task
Status: resolved
Blocked by: none
Map: [WotR Build Planner](../map.md)

## Question

The user asked for icons on feats by type, "like combat feats can have a sword symbol" (2026-11-08), while the design language is being explored (the throwaway branch `prototype/design-language`).

## Choices settled

1. **The type is the feat's category in the data**, the game's own 12 groups: Weapon, Ranged, Unarmed Combat, Armour and Shield, Combat Manoeuvres, Mounted Combat, Magic, Skill, Teamwork, Racial, Class, Miscellaneous. "Combat feats" map to Weapon, which gets the sword.
2. **The engine says the type, the UI draws it.**
   - `Engine::feat_category(&Pick) -> Option<FeatCategory>` returns a feat's category, or the chosen feat's for an option that grants one (a rogue talent's Weapon Focus is a Weapon feat). It's `None` for every other pick, mythic feats and abilities included.
   - `SlotEntry::feat_category` carries it into the picker.
   - The UI matches each `FeatCategory` to its icon (`libs/ui/src/icons.rs`), so a new category won't compile without one.
3. **The icons are my own line drawings** on a 24×24 grid, inline SVG in the text's colour (`stroke: currentColor`). They follow any theme and need no licence, credit or download. The Ranged bow was redrawn after the first version read as a pickaxe.
4. **Where they show:**
   - **Picker:** before each feat's name, locked ones included, and on each category filter.
   - **Grid chip:** in the small label line ("⚔ FEAT"), in the accent colour. Before the name, the icon's 14 px pushed the Skills chip onto its own line on a 375 px screen. The label line never sets a chip's width, so the grid stays as dense as before.
5. **Tests** in `libs/rules/tests/engine/slot_entries.rs`:
   - an entry names its feat type: a feat's, a granting option's, and none for an option granting no feat;
   - `feat_category` agrees with the entries for every option.

## Answer

Done 2026-11-08. Feats show their type's icon in the picker (rows and filters) and on the grid's chips. Checked at 375 px on a level 20 Slayer/Rogue/Fighter: Skill Focus shows the quill, Two-Weapon Fighting the sword, Shield Bash the shield, and the picker's 12 filters show the 12 icons. There are no console errors.
