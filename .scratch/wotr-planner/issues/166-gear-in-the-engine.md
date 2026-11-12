# Gear in the engine: the Build, the link, scores and prerequisites

Type: task
Status: open
Blocked by: 158, 159, 165
Model: Fable to build (the walk, a link format pinned byte for byte, stacking, and no change for a Build without gear); review on Opus.
Map: [WotR Build Planner](../map.md)

## Question

The second gear ticket from the equipment plan ([163](163-equipment.md), user, 2026-11-11): the rules. It reverses the map's rule that prerequisites never see a gear bonus.

Decided in 163:

- **Gear is part of the Build and its link.** A Build has no gear unless the user adds it.
- **Independent of level.** "We can select any gear we want since we know we will not get it at level 1. It's like another screen/tab overall independant on the level screen" (user). A gear bonus counts at every level.
- **What it changes:** an item's always-on bonuses change the numbers the engine already computes. A score bonus counts for prerequisites and for everything derived from the score, **except skill points per level**, which keep using the score without gear, so ranks stay valid whatever is worn.
- **Stacking:** the highest bonus of each type per target, and different types add. This is ticket [144](144-bonus-types-and-stacking.md)'s rule, applied to gear first; 144 extends it to feats and features.
- **Order:** gear counts before "highest score" features (Perfect Body and its like), so a belt can move such a feature's bonus (user, Q23).
- **Both numbers:** every number is available with gear and without.
- **Gear-dependent picks:** a pick that is allowed only because of gear is marked with the item that allows it. If the item is removed, the pick stays in the Build and is refused with the usual reason.
- **One item per slot,** two rings.

## Build

AFK. `libs/rules` only, plus its tests. No schema change. Behaviour without gear must not change.

**The item's shape** is in [research/gear-item-shape.md](../research/gear-item-shape.md), landed by 165. Its "Notes for the builders" apply here: `BonusInForce` gets a source type of its own (no gear variant on `BonusItem`, which would force a migration), and gear's signed amounts go onto the base scores before `with_bonuses`. Items carry always-on penalties too; they count.

**Order:** after [165](165-gear-data.md), which lands the item's shape and the real items; the two share `libs/rules/src/game_data.rs` and the codec. It can run beside [160](160-gold-builds.md), which touches only `libs/seeds/tests/`. Ticket [164](164-cheap-key-clones.md) changes the text keys across the engine: never beside it.

- **The Build:** a gear field, slot to item key, `#[serde(default)]` and empty by default. Two ring places.
- **The link** (`codec.rs`): a last section after the first ascension, written only when the Build has gear, as ticket 91 added its own. Every existing link decodes as before and a Build without gear writes the link it always did: `version_1_body_layout_is_frozen` and `V1_LINK` stay untouched. Add a pinned link with gear beside them.
- **Addresses** (`engine/address.rs`): `gear.<slot>`, with `gear.ring.0` and `gear.ring.1`. Gear needs a place of its own beside Start, Level and Rank, since it belongs to no level. The label, `Display` and `parse` follow the others.
- **Options for a slot:** every item of that slot, none ever locked. An item whose slot is taken replaces the one there.
- **Scores:** the one writer of `CharacterState::ability_scores` is `Walker::refresh_ability_scores` (`engine/walk/hold.rs`). Add gear's score bonuses there, stacked by type, before `with_bonuses` picks the highest score. Every reader then sees gear: prerequisites, the attack line, the caster line, the totals, the sheet.
  - **Skill points:** `walk/level.rs` reads the Int modifier from that field. Give it the score without gear.
  - **The point-buy steppers' `BreaksLater`** and the removable logic read the same field: decide for each whether it should see gear, and say so in the Answer. A point-buy change that only gear keeps legal must not look safe.
- **Other bonuses:** gear's bonuses to saves, skills, initiative, spell DC, spell penetration and concentration join `bonuses_in_force` (ticket 151) with their type and their item as the source. Apply the stacking rule among gear's bonuses only; a feat's or feature's bonus still adds as today until 144.
- **Both numbers:** `Engine` gives the naked numbers by evaluating the Build without its gear. Offer it as one call, so the web app and the MCP don't each strip the gear themselves. A Build without gear costs nothing extra.
- **Gear-dependent picks:** a pick is gear-dependent when it is possible with the Build's gear and refused without it. Report, per such pick, the item or items whose removal refuses it. The picker's entry for an option open only because of gear carries the same note; `Unmet::MetLater` and `story_note` are the nearest precedents.
- **The docs that promise the opposite:** `state.rs`'s "Gear never counts" and the leaves-out constants (`ATTACK_LINE_LEAVES_OUT`, `CASTER_LINE_LEAVES_OUT`, `TOTALS_LEAVE_OUT`) say what is still left out after this ticket: weapon and armour bonuses, circumstance bonuses, other effects.
- **Tests,** in `libs/rules/tests/engine/`:
  - a feat needing Dex 19 is refused at Dex 17, allowed with a Dex +2 belt, marked gear-dependent, and refused again when the belt is withdrawn, with "Needs Dex 19 (have 17)";
  - two enhancement bonuses to one score give the higher; an enhancement and an inherent bonus add;
  - a belt that makes another score the highest moves a "highest score" feature's bonus;
  - an Int item changes the Int score and no level's skill points;
  - the link: round trip with gear, an old link unchanged, a Build without gear byte-identical to before;
  - the naked numbers of a Build with gear equal the numbers of the same Build without it.

## Done when

- Each test above is listed by `cargo test -p rules -- --list` and passes.
- No behaviour changes for a Build without gear: the snapshot check ([159](159-snapshot-check.md)) is unchanged, and the queue's proof ([161](161-proof-once-per-queue.md)) is empty.
- The Answer says what each reader of the scores sees (with or without gear) and why.
- `scripts/check.sh` passes.
