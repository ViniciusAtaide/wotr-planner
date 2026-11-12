# The gear item's shape and `gear.ron`

The spec for ticket [165](../issues/165-gear-data.md), and the surface ticket [166](../issues/166-gear-in-the-engine.md) builds on. Designed read-only on 2026-11-11 against `main` at `34143b5` and the three saved item pages; nothing was built. Where this file and a ticket disagree on the shape, this file holds.

## Corrections to the tickets

- **The data types are serde-only, not serde-free.** Every type in `libs/rules/src/game_data.rs` derives `Serialize, Deserialize` with `#[serde(deny_unknown_fields)]`, and the RON files deserialise straight into them. The gear types do the same.
- **About 67 distinct items, not 77.** A script over the table rows found 70 rows with an always-on ability score bonus: 58 on `items`, 4 on `items-inevitable-excess`, 8 on `items-lord-of-nothing`. Three repeat a main-page row (Flawless Headband of Mental Perfection +8, Hammer Fists, Soaring Fist). The extractor recounts independently.

## The Rust types

`BonusType` goes in `libs/rules/src/bonus.rs`, since ticket 144 will reuse it. The rest goes in a new `libs/rules/src/gear.rs`, re-exported from `lib.rs` as `bonus.rs` and `spells.rs` are. `game_data.rs` then gains only the field, the `counts()` entry and the canonical sort.

```rust
// keys.rs
text_key!(
    /// `belt-of-physical-perfection-6`: a gear item.
    GearKey
);

// bonus.rs
/// A bonus's type, as the guide words it. Two bonuses of one type to one number don't stack: the higher counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BonusType {
    Alchemical, Armour, Circumstance, Competence, Deflection, Dodge, Enhancement, Inherent, Insight,
    Luck, Morale, NaturalArmour, Profane, Racial, Resistance, Sacred, Shield, Size,
    /// The guide gives no type, or says "untyped". Always adds. Every penalty is untyped.
    Untyped,
}
impl BonusType {
    pub const ALL: [Self; 19] = [/* in the order above */];
    /// Lower case, as the summary prints it ("enhancement", "natural armour"); "" for `Untyped`.
    pub fn name(self) -> &'static str;
}

// gear.rs
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GearItem {
    pub key: GearKey,
    pub name: String,
    /// The guide section it is listed in.
    pub slot: GearSlot,
    /// `page#section`: the guide has no anchor per item.
    pub guide: GuideRef,
    /// Its always-on bonuses, one per number changed.
    pub bonuses: Vec<GearBonus>,
    /// Everything else it does, one short line each, in the planner's own words.
    #[serde(default)]
    pub other_effects: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GearSlot {
    Belt, Boots, Bracers, Cloak, Glasses, Gloves, Headwear, Necklace, Ring, Robe,
    Armour, Weapon,
    /// The guide's "Equipable and Consumable" section.
    Usable,
}
impl GearSlot { pub const ALL: [Self; 13]; pub fn name(self) -> &'static str; }

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GearBonus {
    pub target: GearTarget,
    /// Never 0. Negative for a penalty, which is always `Untyped`.
    pub amount: i8,
    #[serde(default = "untyped")] // left out in the file: Untyped
    pub kind: BonusType,
}

/// Declared in the order the summary lists them.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GearTarget {
    Ability(Ability),
    AllSaves,
    Save(Save),
    Initiative,
    Skill(SkillKey),
    /// Never `SpellScope::Value`.
    SpellDc(SpellScope),
    SpellPenetration,
    Concentration,
}
impl GearTarget {
    /// The `BonusTarget` it adds to; `None` for an ability score, which goes through the scores.
    pub fn bonus_target(&self) -> Option<BonusTarget>;
}

impl GearItem {
    /// The generated summary (below). `skills` gives a skill target its name.
    pub fn summary(&self, skills: &[Skill]) -> String;
}

// game_data.rs, on GameData
#[serde(default)]
pub gear: Vec<GearItem>,
```

Why:

- **Slots** are the guide's sections (`items.html` h4 ids `belts`, `boots`, `bracers`, `cloaks`, `glasses`, `gloves`, `headwear`, `necklaces`, `rings`, `robes`, `equipable-and-consumable`), plus `Armour` and `Weapon`, which hold first-stage items. No first-stage item is a shield or a rod. The item records only its section; how many of a slot a Build holds is ticket 166's concern.
- **No escape variant for an unknown type:** the RON parse refuses it with file and line. The enum is the standard list, so ticket 144 and the second stage need no migration for the table's CHECK. Ten occur in the first stage: enhancement, inherent, luck, profane, circumstance, competence, resistance, armour, alchemical, untyped.
- **A gear-specific target, not an extended `BonusTarget`:** `BonusTarget` has `Held` and `AllSkills`, which no item means; adding `Ability` to it would let `bonuses.ron` give a feat a score bonus, which the engine applies by another route (`AbilityBonus`); and it would force a new arm into the seed's `target_columns` and the `item_bonus` CHECK. `bonus_target()` is the bridge 166 uses.
- **A signed `i8` amount:** no item scales, and six first-stage items carry always-on penalties (Bookworm's Headband: -2 Charisma, -3 Persuasion; Goggles of Ferocious Pact: -5 Persuasion, -2 Fortitude).
- **One bonus per target:** "Strength, Dexterity and Constitution" is three entries; the summary regroups them.
- **Other effects** are a `Vec<String>`, sorted by `canonicalize` like every other child list.

Additive later, with no reshaping: `GearTarget::CasterLevel`, AC, attack and damage targets, `GearSlot::Shield` and `Rod`, a `weapon: Option<WeaponKey>` field, a set-bonus field.

## The RON format

`libs/game-data/gear.ron` is one `Vec<GearItem>`, one item per block, grouped by slot with a comment per guide section. `kind` is left out for an untyped bonus, and `other_effects` when empty.

- **Key:** the name in lower case, apostrophes dropped, every other run of non-alphanumerics a single hyphen (`assailants-belt`, `belt-of-physical-perfection-6`).
- **A row repeated on a DLC page with the same properties** is one entry with the main page's reference.
- **Guide reference:** `<page>#<section>`: `items#belts`, `items-inevitable-excess#headwear`, `items-lord-of-nothing#weapons`. Main-page weapons are `items#weapon-properties` and armour `items#unique-armour`.

```ron
// Belts.
(
    key: "assailants-belt",
    name: "Assailant's Belt",
    slot: Belt,
    guide: "items#belts",
    bonuses: [
        (target: Ability(Dexterity), amount: 6, kind: Enhancement),
    ],
    other_effects: [
        "After a critical hit: +2 circumstance to attack and damage rolls for 3 rounds.",
    ],
),
(
    key: "belt-of-physical-perfection-6",
    name: "Belt of Physical Perfection +6",
    slot: Belt,
    guide: "items#belts",
    bonuses: [
        (target: Ability(Strength), amount: 6, kind: Enhancement),
        (target: Ability(Dexterity), amount: 6, kind: Enhancement),
        (target: Ability(Constitution), amount: 6, kind: Enhancement),
    ],
),
// Cloaks.
(
    key: "grand-festival-cloak",
    name: "Grand Festival Cloak",
    slot: Cloak,
    guide: "items#cloaks",
    bonuses: [
        (target: AllSaves, amount: 7, kind: Resistance),
        (target: Ability(Charisma), amount: 2, kind: Circumstance),
    ],
    other_effects: [
        "Bluffing with Persuasion: +5 circumstance.",
    ],
),
```

A penalty is written `(target: Ability(Charisma), amount: -2)`.

## The summary rule

1. Sort the bonuses by `GearTarget`'s derived order: abilities Str to Cha, all saves, Fortitude, Reflex, Will, initiative, skills, spell DC, spell penetration, concentration.
2. Group the bonuses that share amount and type. Groups keep the order of their first target; targets keep their order inside a group.
3. Print a group as its target names joined by ", ", a space, the signed amount (`+6`, `-2`), then a space and the type's lower-case name. An untyped bonus prints no type.
4. Join groups with "; ". No full stop.

Target names: `Str`, `Dex`, `Con`, `Int`, `Wis`, `Cha`; `Saves`, `Fortitude`, `Reflex`, `Will`; `Initiative`; a skill's `name` from `skills.ron`; `Spell DC`, or `Spell DC (Evocation)` when scoped; `Spell penetration`; `Concentration`.

- Assailant's Belt: `Dex +6 enhancement`
- Belt of Physical Perfection +6: `Str, Dex, Con +6 enhancement`
- Grand Festival Cloak: `Cha +2 circumstance; Saves +7 resistance`
- Gloves of the Ambassador: `Cha +2 enhancement; Knowledge (World), Persuasion +5 competence`
- Bookworm's Headband: `Int +2 enhancement; Cha -2; Persuasion -3`

## What fits and what becomes text

Every item's ability score bonuses fit, including the odd wordings: Timely Assistance ("Constitution + Wisdom"); Life Infuser ("+8 armour bonus to Constitution and Strength", typed `Armour` as written); Half-Plate of Vigor and Dirge for Sarkoris (no type: `Untyped`); Sin Mage's Staff ("+6 untyped bonus to Intelligence").

Typed bonuses to existing targets also fit: Belt of Alacrity (Reflex +4 competence), Soaring Fist (Mobility +4 competence), Earth's Embrace (Lore (Nature) +4 competence), Gloves of the Ambassador and Trusted Friend (Persuasion, Knowledge (World)), Goggles of Ferocious Pact (Will +3, Fortitude -2, Persuasion -5), Sin Mage's Staff (Spell DC +1).

An other-effect line in the first stage:

| Kind | Examples | Later |
| --- | --- | --- |
| AC | Hammer Fists +2 morale; Heart of Ice +4 armour | a target once AC is computed |
| Attack and damage | Goggles +4 damage; Midnight Hunter's Ring vs lycanthropes | second-stage target |
| CMD, critical range, DR, speed, immunities | Ustlavic Pride +4 CMD | text |
| Caster level | Sin Mage's Staff +3; Headband of Reshaping +1 for Transmutation | `GearTarget::CasterLevel` |
| A weapon's or armour's own enhancement and base type | Sin Mage's Staff is a +5 quarterstaff | second-stage fields; one line now |
| One use of a skill | Persuasion (Bluff) +5 | text, as `bonuses.ron` already leaves these out |
| Saves against one thing | vs mind-affecting, fire, poison | text |
| Set bonuses | Cat's Boots with Owl's Cowl | a field only if the user wants them counted |
| Temporary or triggered bonuses | on kill; charges per day | text |
| Who may wear it | main character only; not good | text |

Calls the extractor records and doesn't re-decide:

- **Items whose only score change is conditional or a penalty are out of the first stage:** Glasses of Undeniable Truth, Bestial Rags, Robe of False Death, Mutilated Angel.
- **Caster level and the manuals and tomes:** see "Open, for the user".

## Tables and checks

A new migration; the names are free since `20261111152919_148-drop-gear.sql`.

- `gear_item`: `key` (primary key, the `text_key` CHECK), `name`, `slot` (CHECK on the 13 values), `guide`.
- `gear_bonus`: `id`, `gear_item_key` (FK), `target` (CHECK: `ability`, `all_saves`, `save`, `initiative`, `skill`, `spell_dc`, `spell_penetration`, `concentration`), `ability`, `save`, `skill_key` (FK `skill`), `scope`, `spell_school`, `subschool`, `energy`, `amount` (CHECK -128 to 127 and not 0), `bonus_type` (CHECK on the 19 values). "Column present exactly with its target" constraints as on `item_bonus`; a CHECK that a negative amount is `untyped`; an index on `gear_item_key`.
- `gear_other_effect`: `id`, `gear_item_key` (FK), `text` (CHECK not empty), UNIQUE (`gear_item_key`, `text`).

`GameData::check`, under a new `DataAt::Gear(GearKey)`, refuses: a duplicate key, and two items with one name; an item with no bonus; an amount of 0, or a negative amount with a type other than `Untyped`; two bonuses on one item with the same target and type; a `Skill` target naming no skill; `SpellDc(Value)`; an empty or repeated other-effect line. An unknown slot, target or type never reaches the check: the RON parse refuses it.

Real-data tests (first-stage rules, not check rules, since the second stage breaks them): every item has a positive `Ability` bonus; every type is one of the ten; every guide reference is one of the three pages and its section matches the slot; no other-effect line is over about 140 characters; the three hand-picked items; every summary is non-empty.

## Notes for the builders

- **`gear.ron` is optional in the reader,** as `interactions.ron` is (`libs/seeds/src/files.rs`), because the proof reads data folders from before it. Add it to `SINGLE_FILES`.
- **The link's key kind:** add `KeyKind::Gear` and a loop in `LinkKeys::new` (`codec.rs`), and update the list in the module docs. The collision test needs no edit: it is `LinkKeys::new(&DATA).collisions()`.
- **`GameData::counts()`** grows from 14 to 15 entries; `libs/rules/tests/engine/canonical.rs` and `libs/seeds/src/bin/seed.rs` read it.
- **165 adds no gear variant to `BonusItem`.** For 166: `BonusInForce.item` is the serde enum the seed matches into eight foreign-key columns, so a gear source there would force a migration. Give `BonusInForce` (not serde) a source type of its own, such as `enum BonusSource { Item(BonusItem), Gear(GearKey) }`, and a `kind: BonusType` that is `Untyped` for today's bonuses.
- **Penalties in 166:** `with_bonuses` takes unsigned `AbilityBonus` amounts, so gear's signed, type-stacked amounts go onto the base scores before it. That matches "gear counts before highest score features".
- **The count in 165's Answer:** rows per page, with the three repeats named, so the totals reconcile with the file.

## Open, for the user

1. **Manuals and tomes** ("1 charge: +2 inherent bonus"): six, each a different score, and a character can use all of them. **Answered** (user, 2026-11-11): "yes tomes are gear, add the Tomes row". They are in the first stage: `slot: Usable`, one `Inherent` bonus each, with their guide reference. A character can use all six at once, so `Usable` is not a one-item place; that is ticket 166's concern.
2. **Robe and armour:** separate guide sections. **Answered** (user, 2026-11-11): "same slot. only cloth armor can be worn with an armor." A robe and an armour share one body place in the Build; `GearSlot` keeps both sections, and ticket 166 maps both to that place.
3. **Caster level:** **Answered** (user, 2026-11-11): "counted now". Add `GearTarget::CasterLevel` in the first stage, after `SpellDc` in the summary's order and named `Caster level`; the table's `target` CHECK gains `caster_level`. Only an always-on, unscoped bonus is a `CasterLevel` bonus (Sin Mage's Staff +3); one for a single school stays text. Ticket 166 adds it to the caster level the caster line reports, and so to spell penetration and concentration.
4. **The Enforcer's Dex +2:** **Answered** (user, 2026-11-11): "belt of incredible dexterity 2". That belt is on none of the saved pages: the guide lists unique items, not the plain stat belts and headbands. So the catalogue holds items the guide doesn't back; see "The generic stat items".

## The generic stat items

The user's list (2026-11-11), added by hand. The guide has none of them, so the user's word is the source:

- Belt of Incredible Dexterity +2, +4, +6
- Belt of Giant Strength +2, +4, +6
- Belt of the Mighty Constitution +2, +4, +6
- Headband of Alluring Charisma +2, +4, +6
- Headband of Vast Intelligence +2, +4, +6
- Headband of Inspired Wisdom +2, +4, +6

Eighteen single-score items, each one enhancement bonus to its score and no other effect. Belts are `slot: Belt`, headbands `slot: Headwear`. Keys follow the rule (`belt-of-incredible-dexterity-2`). The user then added the three-score ones (2026-11-11): Belt of Physical Perfection (Str, Dex, Con) and Headband of Mental Perfection (Int, Wis, Cha), each an enhancement bonus to its three scores at +2, +4 and +6. The guide already lists the +6 of each (`items#belts`, `items#headwear`), so those two keep the guide's entry and reference; the +2 and +4 of each are added by hand with no guide reference (four more items). The two-score variants are not on the list: don't add them.

What this changes in the shape:

- `GearItem.guide` becomes `Option<GuideRef>`, left out in the file for these eighteen (`#[serde(default)]`); the table's `guide` column is nullable.
- `gear.ron` puts them first, under a comment saying they come from the user and not from the guide.
- The real-data test on guide references applies to the items that have one, and a second test pins that exactly these eighteen and the four hand-added three-score items have none.
- An item with no guide reference shows no guide link in the app or the MCP.
