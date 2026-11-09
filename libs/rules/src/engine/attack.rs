//! The attack line: one base weapon's attacks at a point of a Build (ticket 114).
//!
//! Unconditional numbers only, from what the Build fixes: the iterative attacks from BAB; the character's own
//! Strength (Dexterity to attack for a ranged weapon, or with Weapon Finesse and a finessable weapon when higher;
//! 1½ times the Strength bonus to damage two-handed, none for a weapon that shoots); Weapon Focus, Greater Weapon
//! Focus, Weapon Specialization and Greater Weapon Specialization held for the weapon; Improved Critical's doubled
//! threat range; and Power Attack's trade beside it. Everything else is [`ATTACK_LINE_LEAVES_OUT`].

use crate::game_data::{Ability, Dice, Handedness, Ranged, Weapon};
use crate::keys::{FeatKey, WeaponKey};
use crate::parameter::Parameter;
use crate::state::CharacterState;

use super::{Engine, Sheet};

/// One base weapon's full attack at one point of a Build ([`Engine::attack_line`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttackLine {
    pub weapon: WeaponKey,
    /// The attack bonus of each attack of a full attack, from the first at the full BAB.
    pub attacks: Vec<i16>,
    /// The damage dice of a hit.
    pub damage: Dice,
    /// What adds to each hit's dice.
    pub damage_bonus: i16,
    /// The lowest d20 roll that threatens a critical hit.
    pub threat: u8,
    /// The critical multiplier.
    pub multiplier: u8,
    /// Power Attack's trade, when the character has it and the weapon is a melee weapon.
    pub power_attack: Option<PowerAttack>,
    /// What the attack bonuses and the damage bonus count besides the BAB, in order: the ability modifiers, then
    /// each feat held for the weapon.
    pub parts: Vec<AttackPart>,
    /// What no attack line counts: [`ATTACK_LINE_LEAVES_OUT`].
    pub leaves_out: &'static [&'static str],
}

/// What an attack line leaves out, the same for every line: it holds only the numbers a Build fixes whatever happens
/// in a fight.
pub const ATTACK_LINE_LEAVES_OUT: &[&str] = &[
    "rage, stances and other abilities switched on in a fight",
    "mythic feats and mythic abilities",
    "class features (Weapon Training, sneak attack, a monk's unarmed damage, …)",
    "gear: enhancement, masterwork and magic items",
    "buffs and spells",
    "Deadly Aim and Piranha Strike",
    "a second weapon, off-hand attacks and two-weapon fighting",
    "size, proficiency and situational bonuses (flanking, charging, …)",
];

/// One thing an attack line counts, with what it adds to each attack and to each hit's damage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttackPart {
    /// "Strength", "Dexterity", or the feat's name ("Weapon Focus").
    pub name: String,
    pub attack: i16,
    pub damage: i16,
}

/// Power Attack's trade at a BAB: a penalty to every attack for a bonus to each hit's damage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PowerAttack {
    pub attack_penalty: i16,
    pub damage_bonus: i16,
}

impl AttackLine {
    /// Each attack's bonus, as the game lists a full attack: "+15/+10/+5".
    pub fn attacks_text(&self) -> String {
        self.attacks
            .iter()
            .map(|bonus| signed(*bonus))
            .collect::<Vec<_>>()
            .join("/")
    }

    /// A hit's damage: "1d8+4", or "1d8" with no bonus.
    pub fn damage_text(&self) -> String {
        match self.damage_bonus {
            0 => self.damage.to_string(),
            bonus => format!("{}{}", self.damage, signed(bonus)),
        }
    }

    /// The threat range and multiplier: "19–20/×2", or "20/×3" for a weapon that threatens on a 20 only.
    pub fn critical_text(&self) -> String {
        match self.threat {
            20 => format!("20/×{}", self.multiplier),
            threat => format!("{threat}–20/×{}", self.multiplier),
        }
    }
}

/// A bonus with its sign: "+4", "+0", "-1".
fn signed(value: i16) -> String {
    format!("{value:+}")
}

impl Engine {
    /// The weapons an attack line can be drawn for (those with stats), by name: the weapon picker's list.
    pub fn attack_weapons(&self) -> Vec<&Weapon> {
        let mut weapons: Vec<&Weapon> = self
            .catalog
            .data()
            .weapons
            .iter()
            .filter(|weapon| weapon.stats.is_some())
            .collect();
        weapons.sort_by(|a, b| a.name.cmp(&b.name));
        weapons
    }

    /// The weapon an attack line is drawn for when nobody has chosen one: that of the Build's first Weapon Focus
    /// with stats, by level (in key order within a level); `None` without one.
    pub fn default_attack_weapon(&self, sheet: &Sheet) -> Option<WeaponKey> {
        let focus = FeatKey::new(WEAPON_FOCUS);
        let empty = CharacterState::default();
        let mut before = &empty;
        for level in &sheet.levels {
            let state = &level.state;
            let first = state
                .feat_values
                .get(&focus)
                .into_iter()
                .flat_map(|values| values.keys())
                .filter(|value| !before.has_feat_for(&focus, value))
                .find_map(|value| match value {
                    Parameter::Weapon(key) => self
                        .catalog
                        .weapon(key)
                        .is_some_and(|weapon| weapon.stats.is_some())
                        .then(|| key.clone()),
                    _ => None,
                });
            if first.is_some() {
                return first;
            }
            before = state;
        }
        None
    }

    /// One base weapon's attack line for a character: `None` for a weapon the data lacks or that has no stats.
    pub fn attack_line(&self, state: &CharacterState, weapon: &WeaponKey) -> Option<AttackLine> {
        let stats = self.catalog.weapon(weapon)?.stats?;
        let scores = &state.ability_scores;
        let finesse = stats.finesse
            && state.feat_count(&FeatKey::new(WEAPON_FINESSE)) > 0
            && scores.dexterity > scores.strength;
        let to_attack = if stats.ranged.is_some() || finesse {
            Ability::Dexterity
        } else {
            Ability::Strength
        };
        let strength = modifier(scores.strength);
        let to_damage = match (stats.ranged, stats.handedness) {
            (Some(Ranged::Projectile), _) => None,
            (None, Handedness::TwoHanded) if strength > 0 => Some(strength * 3 / 2),
            _ => Some(strength),
        };
        let mut parts = Vec::new();
        if to_attack != Ability::Strength {
            parts.push(AttackPart {
                name: to_attack.name().to_owned(),
                attack: modifier(scores.get(to_attack)),
                damage: 0,
            });
        }
        if let Some(damage) = to_damage {
            parts.push(AttackPart {
                name: Ability::Strength.name().to_owned(),
                attack: if to_attack == Ability::Strength {
                    strength
                } else {
                    0
                },
                damage,
            });
        }
        let value = Parameter::Weapon(weapon.clone());
        for (feat, attack, damage) in WEAPON_FEATS {
            if state.has_feat_for(&FeatKey::new(feat), &value) {
                parts.push(AttackPart {
                    name: self.feat_name(&FeatKey::new(feat)).to_owned(),
                    attack,
                    damage,
                });
            }
        }
        let attack: i16 = parts.iter().map(|part| part.attack).sum();
        Some(AttackLine {
            weapon: weapon.clone(),
            attacks: iteratives(state.bab).map(|base| base + attack).collect(),
            damage: stats.damage,
            damage_bonus: parts.iter().map(|part| part.damage).sum(),
            threat: if state.has_feat_for(&FeatKey::new(IMPROVED_CRITICAL), &value) {
                // As many rolls again below the range: 19–20 (two rolls) becomes 17–20.
                stats.threat.saturating_sub(21 - stats.threat)
            } else {
                stats.threat
            },
            multiplier: stats.multiplier,
            power_attack: (stats.ranged.is_none()
                && state.feat_count(&FeatKey::new(POWER_ATTACK)) > 0)
                .then(|| power_attack(state.bab, stats.handedness)),
            parts,
            leaves_out: ATTACK_LINE_LEAVES_OUT,
        })
    }
}

const WEAPON_FOCUS: &str = "weapon-focus";
const POWER_ATTACK: &str = "power-attack";
const IMPROVED_CRITICAL: &str = "improved-critical";
const WEAPON_FINESSE: &str = "weapon-finesse";

/// Power Attack at a BAB, as the guide has it: −1 to attack for +2 damage, one step more at BAB 4 and every four
/// after; +3 damage a step with a two-handed weapon.
fn power_attack(bab: u8, handedness: Handedness) -> PowerAttack {
    let steps = 1 + i16::from(bab) / 4;
    let per_step = match handedness {
        Handedness::TwoHanded => 3,
        Handedness::Light | Handedness::OneHanded => 2,
    };
    PowerAttack {
        attack_penalty: steps,
        damage_bonus: steps * per_step,
    }
}

/// The feats that count when held for the weapon, each with what it adds to attack and to damage.
const WEAPON_FEATS: [(&str, i16, i16); 4] = [
    ("weapon-focus", 1, 0),
    ("greater-weapon-focus", 1, 0),
    ("weapon-specialization", 0, 2),
    ("greater-weapon-specialization", 0, 2),
];

/// An ability score's modifier: −1 for 8 or 9, +4 for 18 or 19.
fn modifier(score: u8) -> i16 {
    (i16::from(score) - 10).div_euclid(2)
}

/// The base of each attack of a full attack: the BAB, then 5 less for each further attack while it stays above 0,
/// four at most.
fn iteratives(bab: u8) -> impl Iterator<Item = i16> {
    let bab = i16::from(bab);
    (0..4)
        .map(move |n| bab - 5 * n)
        .enumerate()
        .take_while(|(n, base)| *n == 0 || *base > 0)
        .map(|(_, base)| base)
}
