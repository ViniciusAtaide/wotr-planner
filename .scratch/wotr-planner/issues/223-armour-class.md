# Armour class

Type: feature
Status: open
Blocked by: [144](144-bonus-types-and-stacking.md) (stacking by type), and [217](217-animal-companions-stage-1.md) for the companion's AC
Model: Opus to build; Opus to review.
Map: [WotR Build Planner](../map.md)

## Build

The last of [169](169-gear-second-stage.md)'s build tickets (its Answer, items 2 and 4).

- **Data:** armour and shields (`items#armour`, `#shields`: armour bonus, max Dex, armour check penalty, category), the unique armour and shields with their properties (`items#unique-armour`, `#armor-properties`, `#unique-shields`), and generic +1 to +5 armour and shields, planner-made as 221's weapons are. The other items' AC bonuses (deflection rings, natural armour amulets) are 222's, typed.
- **The number:** AC, flat-footed AC and touch AC at every level: 10, armour, shield, natural, deflection, dodge, Dexterity capped by the armour's max Dex, and every other typed AC bonus the planner counts (class features where the data has them, the horse's rider bonus from 217), stacked by 144's rule. Each source its own row; conditional ones beside the headline.
- **The companion's AC** from its animal row, levelling table and evolved form.
- **MCP:** `sheet`'s `totals` gain AC; `compare` and `improve` take `by: {totals: "ac"}` or the reviewer's preferred form. `totals_leave_out` and `wotr://leaves-out` say what AC leaves out (buffs, switched-on abilities).
- **Web app:** AC on the Sheet beside the saves.

## Done when

A fighter in full plate with a heavy shield and a Dex 14 shows the guide's numbers, with a row per source; a Dex 20 character in the same plate shows Dexterity capped. A companion's AC shows in its section. Real-data and MCP tests cover each. `scripts/check.sh` passes.
