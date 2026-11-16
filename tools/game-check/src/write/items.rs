//! The items target (ticket 293c): the game's always-on bonuses of each gear item, as the `GearStat` components the
//! entry holds, written where they differ from the entry's.

use rules::{Component, DataAt, Descriptor, GearSlot, GearTarget, SpellScope};

use super::span::{Value, ron_text};
use super::{Change, Unexpressed, entry_file};
use crate::areas::items::{Bonus, bonus_type, game_bonuses, stacked, target};
use crate::areas::{Ctx, game_skill};
use crate::report::{Area, Out};

/// The changes that bring every non-usable gear item with a blueprint to the game's bonuses, in key order, and the
/// items a bonus with no inverse leaves unwritten.
pub(crate) fn changes(ctx: &Ctx) -> (Vec<Change>, Vec<Unexpressed>) {
    let mut changes = Vec::new();
    let mut unexpressed = Vec::new();
    let mut items: Vec<_> = ctx.planner.data.gear.iter().collect();
    items.sort_by(|a, b| a.key.cmp(&b.key));
    for item in items {
        if item.slot == GearSlot::Usable {
            continue;
        }
        // The check's skips are its own business; the target only needs what the game gives.
        let mut skipped = Out::default();
        let Some((_, json)) = ctx.blueprint(
            &mut skipped,
            Area::Items,
            item.blueprint.as_ref(),
            "gear item",
        ) else {
            continue;
        };
        let mut game = stacked(game_bonuses(ctx, &mut skipped, json));
        game.sort();
        let mut planner: Vec<Bonus> = item
            .stats()
            .map(|(stat, value, descriptor)| {
                (target(stat), bonus_type(descriptor), i64::from(value))
            })
            .collect();
        planner.sort();
        if game == planner {
            continue;
        }
        let at = DataAt::Gear(item.key.clone());
        let mut stats = Vec::new();
        let mut missing = None;
        for bonus in &game {
            match gear_stat(bonus) {
                Some(stat) => stats.push(stat),
                None => {
                    missing = Some(bonus);
                    break;
                }
            }
        }
        if let Some(bonus) = missing {
            unexpressed.push(Unexpressed {
                at,
                what: format!("{} {:+} {}", bonus.0, bonus.2, bonus.1),
            });
            continue;
        }
        stats.sort();
        let show = |bonuses: &[Bonus]| {
            let each: Vec<String> = bonuses
                .iter()
                .map(|(target, kind, amount)| format!("{target} {amount:+} {kind}"))
                .collect();
            each.join("; ")
        };
        changes.push(Change {
            file: entry_file(ctx.planner.data, &at),
            at,
            field: "components",
            kind: "bonuses",
            value: Some(Value::List(stats.iter().map(ron_text).collect())),
            old: show(&planner),
            new: show(&game),
        });
    }
    (changes, unexpressed)
}

/// The `GearStat` component for one of the game's bonuses: the inverse of the check's `target` and `bonus_type`.
/// `None` for a target or a type the planner has no word for.
pub(crate) fn gear_stat(bonus: &Bonus) -> Option<Component> {
    let (stat, kind, amount) = bonus;
    let descriptor: Descriptor = match kind.as_str() {
        "Untyped" => Descriptor::Untyped,
        other => from_name(other)?,
    };
    let stat = match stat.as_str() {
        "AllSaves" => GearTarget::AllSaves,
        "SaveFortitude" => GearTarget::Save(rules::Save::Fortitude),
        "SaveReflex" => GearTarget::Save(rules::Save::Reflex),
        "SaveWill" => GearTarget::Save(rules::Save::Will),
        "Initiative" => GearTarget::Initiative,
        "CasterLevel" => GearTarget::CasterLevel,
        "SpellPenetration" => GearTarget::SpellPenetration,
        "Concentration" => GearTarget::Concentration,
        "SpellDC" => GearTarget::SpellDc(SpellScope::All),
        other => {
            if let Some(school) = other.strip_prefix("SpellDC ") {
                GearTarget::SpellDc(SpellScope::School(from_name(school)?))
            } else if other.starts_with("Skill") {
                GearTarget::Skill(game_skill(other)?)
            } else {
                GearTarget::Ability(from_name(other)?)
            }
        }
    };
    // An ability name that isn't an ability, or a scope that isn't a school, found no inverse above.
    Some(Component::GearStat {
        stat,
        value: i8::try_from(*amount).ok()?,
        descriptor,
    })
}

/// A unit variant of a rules enum by its name.
fn from_name<T: serde::de::DeserializeOwned>(name: &str) -> Option<T> {
    ron::from_str(name).ok()
}
