//! Gear: each item's slot, and its always-on bonuses with their type and value.
//!
//! The game's: the item's enchantments (`m_Enchantments`) and their components, and the features an enchantment or
//! the item gives its wearer, read for flat bonuses: `AddStatBonusEquipment`, `AllSavesBonusEquipment`,
//! `AddStatBonus`, `AddContextStatBonus` of a simple value, and the spell DC, caster level, spell penetration and
//! concentration bonuses. A bonus to a number the planner doesn't count (AC, speed) is left out, as are manuals and
//! tomes, which give theirs through an ability (ticket 238's decisions: a later ticket).

use rules::{BonusType, DataAt, GearSlot, GearTarget, Save, SpellScope};
use serde_json::Value;

use super::{Ctx, ability_stat, is_ability_stat, skill_stat};
use crate::dump::{component_kind, components, int, reference, references};
use crate::report::{Area, Out};

const AREA: Area = Area::Items;

/// One bonus: what it adds to, its type, its amount, in the game's names.
type Bonus = (String, String, i64);

pub fn compare(ctx: &Ctx, out: &mut Out) {
    for item in &ctx.planner.data.gear {
        let Some((guid, json)) = ctx.blueprint(out, AREA, item.blueprint.as_ref(), "gear item")
        else {
            continue;
        };
        if item.slot == GearSlot::Usable {
            out.skip(AREA, "a manual or tome (deferred)");
            continue;
        }
        let kind = ctx.dump.kind(guid).unwrap_or("");
        let name = ctx.name(guid);
        let at = DataAt::Gear(item.key.clone());
        let mark = out.mark();
        if !slot_fits(item.slot, kind) {
            out.finding(
                AREA,
                "slot differs",
                &at,
                guid,
                &name,
                kind,
                format!("{:?}", item.slot),
            );
        }

        let mut game = Vec::new();
        for part in components(json) {
            wearer_fact(ctx, out, part, &mut game);
        }
        for enchantment in references(json.get("m_Enchantments")) {
            let Some(enchantment) = ctx.dump.json(&enchantment) else {
                out.skip(AREA, "an enchantment not in the dump");
                continue;
            };
            for part in components(enchantment) {
                match component_kind(part) {
                    "AddStatBonusEquipment" => {
                        stat_bonus(out, part, int(&part["Value"]), &mut game)
                    }
                    "AllSavesBonusEquipment" => {
                        game.push(("AllSaves".to_owned(), descriptor(part), int(&part["Value"])));
                    }
                    "AddUnitFeatureEquipment" | "AddUnitFactEquipment" => {
                        wearer_fact(ctx, out, part, &mut game)
                    }
                    other => out.skip(AREA, format!("an enchantment's {other}, not compared")),
                }
            }
        }
        let planner: Vec<Bonus> = item
            .bonuses
            .iter()
            .map(|bonus| {
                (
                    target(&bonus.target),
                    bonus_type(bonus.kind),
                    i64::from(bonus.amount),
                )
            })
            .collect();
        bonuses(out, &at, guid, &name, game, planner);
        out.compared(AREA, mark);
    }
}

/// A feature given to the wearer (`m_Feature`, `m_Blueprint`, `m_Fact`): its flat bonuses.
fn wearer_fact(ctx: &Ctx, out: &mut Out, part: &Value, game: &mut Vec<Bonus>) {
    let kind = component_kind(part);
    if !matches!(
        kind,
        "AddUnitFeatureEquipment" | "AddUnitFactEquipment" | "AddFactToEquipmentWielder"
    ) {
        return;
    }
    let fact = ["m_Feature", "m_Blueprint", "m_Fact"]
        .iter()
        .find_map(|field| reference(&part[*field]));
    let Some(feature) = fact.and_then(|fact| ctx.dump.json(&fact)) else {
        out.skip(AREA, "a wearer's fact not in the dump");
        return;
    };
    for part in components(feature) {
        let simple = || {
            (part["Value"]["ValueType"].as_str() == Some("Simple"))
                .then(|| int(&part["Value"]["Value"]))
        };
        match component_kind(part) {
            "AddStatBonus" => stat_bonus(out, part, int(&part["Value"]), game),
            "AddContextStatBonus" => match simple() {
                Some(value) => stat_bonus(out, part, value * int(&part["Multiplier"]).max(1), game),
                None => out.skip(AREA, "a bonus of a value worked out in play"),
            },
            "IncreaseAllSpellsDC" => context_bonus(out, part, "SpellDC", simple(), game),
            "IncreaseSpellSchoolDC" => {
                let school = part["School"].as_str().unwrap_or("");
                game.push((
                    format!("SpellDC {school}"),
                    descriptor(part),
                    int(&part["BonusDC"]),
                ));
            }
            "IncreaseCasterLevel" => context_bonus(out, part, "CasterLevel", simple(), game),
            "SpellPenetrationBonus" => context_bonus(out, part, "SpellPenetration", simple(), game),
            "ConcentrationBonus" => context_bonus(out, part, "Concentration", simple(), game),
            other => out.skip(AREA, format!("a wearer's fact's {other}, not compared")),
        }
    }
}

fn context_bonus(
    out: &mut Out,
    part: &Value,
    target: &str,
    value: Option<i64>,
    game: &mut Vec<Bonus>,
) {
    match value {
        Some(value) => game.push((target.to_owned(), descriptor(part), value)),
        None => out.skip(AREA, "a bonus of a value worked out in play"),
    }
}

fn stat_bonus(out: &mut Out, part: &Value, value: i64, game: &mut Vec<Bonus>) {
    let stat = part["Stat"].as_str().unwrap_or("");
    let counted = is_ability_stat(stat)
        || matches!(
            stat,
            "SaveFortitude" | "SaveReflex" | "SaveWill" | "Initiative"
        )
        || stat.starts_with("Skill");
    if counted {
        // A penalty is untyped in the planner's words, whatever the game calls it (`Penalty`).
        let kind = if value < 0 {
            "Untyped".to_owned()
        } else {
            descriptor(part)
        };
        game.push((stat.to_owned(), kind, value));
    } else {
        out.skip(
            AREA,
            format!("a bonus to {stat}, which the planner doesn't count"),
        );
    }
}

/// The planner's bonus type in the game's words (`ModifierDescriptor`).
fn descriptor(part: &Value) -> String {
    match part["Descriptor"].as_str().unwrap_or("None") {
        "None" | "UntypedStackable" => "Untyped".to_owned(),
        "Armor" => "Armour".to_owned(),
        "NaturalArmor" => "NaturalArmour".to_owned(),
        other => other.to_owned(),
    }
}

fn bonus_type(kind: BonusType) -> String {
    format!("{kind:?}")
}

fn target(target: &GearTarget) -> String {
    match target {
        GearTarget::Ability(ability) => ability_stat(*ability).to_owned(),
        GearTarget::AllSaves => "AllSaves".to_owned(),
        GearTarget::Save(Save::Fortitude) => "SaveFortitude".to_owned(),
        GearTarget::Save(Save::Reflex) => "SaveReflex".to_owned(),
        GearTarget::Save(Save::Will) => "SaveWill".to_owned(),
        GearTarget::Initiative => "Initiative".to_owned(),
        GearTarget::Skill(skill) => skill_stat(skill),
        GearTarget::SpellDc(SpellScope::All) => "SpellDC".to_owned(),
        GearTarget::SpellDc(SpellScope::School(school)) => format!("SpellDC {school:?}"),
        GearTarget::SpellDc(scope) => format!("SpellDC {scope:?}"),
        GearTarget::CasterLevel => "CasterLevel".to_owned(),
        GearTarget::SpellPenetration => "SpellPenetration".to_owned(),
        GearTarget::Concentration => "Concentration".to_owned(),
    }
}

/// The two sides' bonuses: the same target with another amount or type is one difference.
fn bonuses(
    out: &mut Out,
    at: &DataAt,
    guid: &rules::BlueprintGuid,
    name: &str,
    mut game: Vec<Bonus>,
    mut planner: Vec<Bonus>,
) {
    game.sort();
    planner.sort();
    game.retain(
        |bonus| match planner.iter().position(|other| other == bonus) {
            Some(index) => {
                planner.remove(index);
                false
            }
            None => true,
        },
    );
    let show = |(target, kind, amount): &Bonus| format!("{target} {amount:+} {kind}");
    for bonus in game {
        match planner.iter().position(|other| other.0 == bonus.0) {
            Some(index) => {
                let other = planner.remove(index);
                let kind = if other.1 == bonus.1 {
                    "amount differs"
                } else {
                    "type differs"
                };
                out.finding(AREA, kind, at, guid, name, show(&bonus), show(&other));
            }
            None => out.finding(
                AREA,
                "bonus missing in planner",
                at,
                guid,
                name,
                show(&bonus),
                "-",
            ),
        }
    }
    for bonus in planner {
        out.finding(AREA, "bonus not in game", at, guid, name, "-", show(&bonus));
    }
}

/// Whether an item blueprint's type is the planner's slot.
fn slot_fits(slot: GearSlot, kind: &str) -> bool {
    let wanted: &[&str] = match slot {
        GearSlot::Belt => &["BlueprintItemEquipmentBelt"],
        GearSlot::Boots => &["BlueprintItemEquipmentFeet"],
        GearSlot::Bracers => &["BlueprintItemEquipmentWrist"],
        GearSlot::Cloak => &["BlueprintItemEquipmentShoulders"],
        GearSlot::Glasses => &["BlueprintItemEquipmentGlasses"],
        GearSlot::Gloves => &["BlueprintItemEquipmentGloves"],
        GearSlot::Headwear => &["BlueprintItemEquipmentHead"],
        GearSlot::Necklace => &["BlueprintItemEquipmentNeck"],
        GearSlot::Ring => &["BlueprintItemEquipmentRing"],
        GearSlot::Robe => &["BlueprintItemEquipmentShirt"],
        GearSlot::Armour => &["BlueprintItemArmor", "BlueprintItemShield"],
        GearSlot::Weapon => &["BlueprintItemWeapon"],
        GearSlot::Usable => &["BlueprintItemEquipmentUsable", "BlueprintItem"],
    };
    wanted.contains(&kind)
}
