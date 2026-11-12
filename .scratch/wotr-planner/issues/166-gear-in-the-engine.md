# Gear in the engine: the Build, the link, scores and prerequisites

Type: task
Status: done
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
- **Tomes are gear** (user, 2026-11-11: "yes tomes are gear, add the Tomes row"). The six manuals and tomes (`slot: Usable`, a permanent inherent bonus each) don't compete for a place: a Build holds any number of them, each at most once.
- **A robe and an armour share one body place** (user, 2026-11-11).
- **Inherent bonuses, confirmed:** two inherent bonuses to one score count as the higher one, so the Tome of Ancient Perfection's +4 hides a +2 manual. The user asked (2026-11-11, "are you sure inherent bonuses don't stack?") and then confirmed it the same day: "inherent bonuses don't stack, keep highest only". Built as highest-only like every other type, with no switch.

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

## Answer

**Built**, 2026-11-11 on branch `ticket-166`, from `main` at `995564e`. Not merged. `scripts/check.sh` passes, and the snapshot check passes with the snapshot files untouched.

### What was built

- **The Build** (`build.rs`): `Build::gear`, a `Gear` of `worn` (a place to an item key) and `used` (a set of item keys), `#[serde(default)]` and empty by default.
  - **Places** (`GearPlace`, 12): belt, boots, bracers, cloak, glasses, gloves, headwear, necklace, the first and second ring, the body, the weapon. Both ring places take the guide's Ring slot; the body takes a Robe or an Armour.
  - **Usable items** are in `used`, not in a place: a Build holds any number, each once, by construction.
  - **Edits:** `Edit::Wear { place, item }` (an item replaces the one there; `None` takes it off; a ring worn in the other ring place moves) and `Edit::Use { item, used }`.
- **The link** (`codec.rs`): a last section after the first ascension, `2, count, count × (place, key)`, written only when the Build holds gear. A worn item's place is its index in `GearPlace::ALL` (0 to 11); a used item's is 12; worn items come in place order, then used ones in key order. The decoder refuses anything else. `version_1_body_layout_is_frozen` and `V1_LINK` are untouched; `V1_GEAR_LINK` is pinned beside them.
- **Addresses** (`engine/address.rs`): `gear.belt`, `gear.boots`, `gear.bracers`, `gear.cloak`, `gear.glasses`, `gear.gloves`, `gear.headwear`, `gear.necklace`, `gear.ring.0`, `gear.ring.1`, `gear.body`, `gear.weapon`, and `gear.usable.<item key>`. `Place::Gear` is their place ("Gear: Belt"). `Choice::Gear(GearSpot)` is the engine's choice.
- **The `Usable` addresses, and why:** each usable item is its own address, named by its key (`gear.usable.manual-of-bodily-health-2`). A numbered address (`gear.usable.0`) would shift when another item comes off, and would let one item sit at two numbers; the key can do neither. To list them all, the engine has `usable_options`, since no single address stands for "the usable items".
- **The walk** (`engine/walk/gear.rs`): the gear is checked once at the Build start (an unknown key, an item where its slot doesn't go, one item held twice), then counted at every level.
- **Stacking** (`bonus.rs`, `stacking`): the highest bonus of each type per number; different types add; `Untyped` always adds; a penalty always applies. Inherent follows the rule (confirmed by the user, above). It is applied among gear's bonuses only (`counted`); a feat's or feature's bonus adds as before, until 144. Dodge bonuses follow the rule too, though the tabletop game stacks them; no first-stage item gives one to a number the planner computes.
- **Caster level:** `GearTarget::CasterLevel` adds to `CasterLine::caster_level`, and so to spell penetration and concentration. Prerequisites still read `CharacterState::caster_level`, which gear doesn't change.
- **Bonuses in force:** `BonusInForce` has `source: BonusSource` (`Item(BonusItem)` or `Gear(GearKey)`) in place of `item`, a `kind: BonusType` (`Untyped` for today's Item bonuses) and `target: InForceTarget` (`Number(BonusTarget)` or `CasterLevel`). `BonusItem` and `BonusTarget` are unchanged, so there is no schema change. Gear's entries follow the Item bonuses at every level.

### What each reader of the scores sees

`CharacterState::ability_scores` is still written only by `Walker::refresh_ability_scores`, now as: base scores, then gear's stacked bonuses and penalties, then the features' bonuses (`with_bonuses`).

| Reader | Sees gear | Why |
| --- | --- | --- |
| Prerequisites (`state.rs`), class and archetype entry | yes | The user's decision: a belt's Dexterity meets a feat's Dex 19, at every level. |
| "Highest score" features (`with_bonuses`) | yes, before them | The user chose that gear counts first, so a belt can move Perfect Body's bonus. |
| Attack line, caster line, totals | yes | Everything derived from a score. |
| Item bonuses of an ability's modifier (Divine Grace) | yes | Derived from the score. |
| The sheet's states, the web grid, the MCP's `sheet` scores | yes | They read the same field. `Engine::naked_sheet` gives the other number. |
| A level's skill points (`walk/level.rs`) | **no** | The user's rule: ranks stay valid whatever is worn. They read `Walker::skill_point_scores`. |
| The point buy's steppers (`BreaksLater`) | yes | They are judged as the walk judges the Build, so a step the gear keeps legal is open. A step open only because of the gear sets `Step::with_gear`, so it never looks safe. |
| `removable`, `is_removable`, `withdrawal`, `allows` | yes | The same walk. A withdrawal the gear alone keeps legal is removable; `Engine::newly_gear_dependent(build, edit)` names the choices it would make lean on the gear. |
| The text export's "Point buy" line | no | It prints the point buy, as before. |

### The API for tickets 167 and 168

- **The data:** `Engine::gear()`, `gear_item(key)`, `gear_name(key)`, `gear_summary(key)`.
- **A place's options:** `Engine::gear_options(build, place)`, every item the place takes, each with `held` and its `edit`. None is ever locked. `Engine::usable_options(build)` does the same for the usable items.
- **Taking an item off:** `Engine::withdrawal(build, &Choice::Gear(spot))`, which gives the edit and whether anything needs the item.
- **The naked numbers:** `Engine::naked_sheet(build)`, the sheet of the Build without its gear; `None` for a Build without gear, at no cost. Pass its levels to `totals`, `caster_line` and `attack_line`.
- **Gear-dependent choices:** `Engine::gear_dependents(build)`, each choice with a `GearNeed` (`items`, and `all`: every item needed, or any one enough). `Engine::gear_note(&need)` words it: "With Belt of Incredible Dexterity +2".
- **In a slot's picker:** `SlotOption::with_gear` (the `GearNeed` of an option open only because of gear) and `SlotEntry::gear_note`.
- **Before offering an edit as safe:** `Engine::newly_gear_dependent(build, edit)`; `Step::with_gear` carries it for the point buy.

### What tickets 167 and 168 must decide

- **A Build whose item was removed is not possible, so its link opens nothing.** The user decided the pick stays and is refused; `Engine::open_link` refuses every Build that isn't possible, and the web app writes the link at once. Until the user fixes the pick or wears the item again, a reload shows the stale-link error. The MCP's `withdraw` refuses a choice that isn't removable, so today it refuses a belt a feat needs.
- **While a pick is refused, every other picker locks its options** (`BreaksLater`), because each recheck stops at the refused pick. Only clearing that pick or wearing an item again is open.
- So `withdrawal` reports honestly whether anything needs the item, and the engine applies the edit either way. Whether the app warns, removes the dependent picks as well, or lets the Build go impossible is the caller's choice, and probably the user's.
- **The text export has no gear lines yet** (167 lists them). A Build with gear exports its link, which holds the gear, and nothing else about it.
- **The MCP** parses gear addresses now; `options` on one answers `no_such_choice` until 168.

### Judgement calls

- **One item is held once:** the same ring in both ring places fails (`AlreadyHeld`), and wearing it in the second place moves it.
- **A choice that needs another gear-dependent choice** is gear-dependent too, with the same items.
- **When no single item is needed** (a belt or a manual, either enough), the note says "With A or B".
- **The naked numbers** are those of the same Build with no gear, exactly: a feat only gear allows isn't held in that walk, so what it gave is missing too.
- **The test world** has 12 invented gear items, for these tests and for 168's.
- **`Step` and `SlotOption` gained a field,** and `BonusInForce` changed shape; nothing outside `rules` built them by hand.

### For the queue's proof

- **The leaves-out texts changed,** as this ticket asks, and the proof hashes them inside `totals` and `caster_lines`. Those two fields will differ for every Build with a level, for that reason alone. The snapshot check, which prints the numbers and not the texts, is unchanged.
- The sheet's Debug text is unchanged for a Build without gear: no state, sheet or slot type gained a field, and the bonuses in force, which did, are left out of the sheet's hash.

### Edits outside `libs/rules`

- `apps/mcp/src/options.rs`: one match arm for `Address::Gear`.
- `GLOSSARY.md`: Gear, Stacking, Naked numbers, Gear-dependent, and four sentences that said gear never counts.
- `libs/game-data/README.md`: one sentence that said the same.
