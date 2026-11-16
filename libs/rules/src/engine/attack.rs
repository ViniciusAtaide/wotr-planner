//! The attack line: one base weapon's attacks at a point of a Build (ticket 114).
//!
//! Unconditional numbers only, from what the Build fixes: the iterative attacks from BAB; the character's
//! Strength, its gear's bonuses included (Dexterity to attack for a ranged weapon, or with Weapon Finesse's flag and a
//! finessable weapon when higher; 1½ times the Strength bonus to damage two-handed, none for a weapon that shoots unless
//! the game gives it the Strength trait, as the sling staff); and the components of the feats held for the weapon
//! (ticket 297): Weapon Focus and Greater Weapon Focus ([`WeaponFocus`]), Weapon Specialization and its greater
//! ([`WeaponSpecialization`]), Improved Critical's doubled threat range and the Trickster's critical feats after it
//! ([`ImprovedCritical`], [`CriticalEdge`], [`CriticalMultiplier`]; ticket 131); and Power Attack's trade beside it
//! ([`PowerAttack`]). Everything else is [`ATTACK_LINE_LEAVES_OUT`].
//!
//! [`WeaponFocus`]: crate::Component::WeaponFocus
//! [`WeaponSpecialization`]: crate::Component::WeaponSpecialization
//! [`ImprovedCritical`]: crate::Component::ImprovedCritical
//! [`CriticalEdge`]: crate::Component::CriticalEdge
//! [`CriticalMultiplier`]: crate::Component::CriticalMultiplier
//! [`PowerAttack`]: crate::Component::PowerAttack

use std::collections::BTreeSet;

use super::LeftOut;
use crate::build::BuildStart;
use crate::component::{Component, MechanicsFlag};
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
    pub leaves_out: &'static [LeftOut],
}

/// What an attack line leaves out, the same for every line: it holds only the numbers a Build fixes whatever happens
/// in a fight.
pub const ATTACK_LINE_LEAVES_OUT: &[LeftOut] = &[
    LeftOut {
        short: "rage, stances and other abilities switched on in a fight",
        full: "rage, stances and other abilities switched on in a fight",
    },
    LeftOut {
        short: "mythic feats and mythic abilities",
        full: "mythic feats and mythic abilities",
    },
    LeftOut {
        short: "class features",
        full: "class features (Weapon Training, sneak attack, a monk's unarmed damage, …)",
    },
    LeftOut {
        short: "gear but for its ability score bonuses",
        full: "gear but for its ability score bonuses: a weapon's enhancement and properties, armour, masterwork, \
               and attack and damage bonuses from items",
    },
    LeftOut {
        short: "buffs and spells",
        full: "buffs and spells",
    },
    LeftOut {
        short: "Deadly Aim and Piranha Strike",
        full: "Deadly Aim and Piranha Strike",
    },
    LeftOut {
        short: "a second weapon and two-weapon fighting",
        full: "a second weapon, off-hand attacks and two-weapon fighting",
    },
    LeftOut {
        short: "size, proficiency and situational bonuses",
        full: "size, proficiency and situational bonuses (flanking, charging, …)",
    },
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

    /// The weapons a character is proficient with, of all the data has ([`Catalog::proficient`]). `start` is the
    /// Build's and `state` a point of it.
    ///
    /// [`Catalog::proficient`]: super::catalog::Catalog
    pub fn proficient_weapons(
        &self,
        start: &BuildStart,
        state: &CharacterState,
    ) -> BTreeSet<WeaponKey> {
        (self.catalog.data().weapons.iter())
            .filter(|weapon| self.catalog.proficient(start, state, weapon))
            .map(|weapon| weapon.key.clone())
            .collect()
    }

    /// The weapon an attack line is drawn for when nobody has chosen one: that of the Build's first Weapon Focus (a feat
    /// with a [`WeaponFocus`](Component::WeaponFocus) requiring no other, not Greater Weapon Focus) with stats, by level
    /// (in key order within a level); `None` without one.
    pub fn default_attack_weapon(&self, sheet: &Sheet) -> Option<WeaponKey> {
        let empty = CharacterState::default();
        let mut before = &empty;
        for level in &sheet.levels {
            let state = &level.state;
            let focus = (state.feat_values.keys()).filter(|feat| {
                self.feat_kind(feat) == Some(Kind::Focus) && self.feat_depth(feat) == 0
            });
            let first = focus.into_iter().find_map(|feat| {
                (state.feat_values[feat].keys())
                    .filter(|value| !before.has_feat_for(feat, value))
                    .find_map(|value| match value {
                        Parameter::Weapon(key) => self
                            .catalog
                            .weapon(key)
                            .is_some_and(|weapon| weapon.stats.is_some())
                            .then(|| key.clone()),
                        _ => None,
                    })
            });
            if first.is_some() {
                return first;
            }
            before = state;
        }
        None
    }

    /// One base weapon's attack line for a character, from the character's scores (its gear's bonuses included):
    /// `None` for a weapon the data lacks or that has no stats.
    pub fn attack_line(&self, state: &CharacterState, weapon: &WeaponKey) -> Option<AttackLine> {
        let scores = &state.ability_scores;
        let stats = self.catalog.weapon(weapon)?.stats?;
        let finesse = stats.finesse
            && state.flags.contains_key(&MechanicsFlag::WeaponFinesse)
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
        // The feats held for the weapon with a kind the line reads, focus before specialization, a feat after those it
        // requires.
        let value = Parameter::Weapon(weapon.clone());
        let mut feats: Vec<(&FeatKey, Kind)> = (state.feat_values.iter())
            .filter(|(_, values)| values.contains_key(&value))
            .filter_map(|(feat, _)| Some((feat, self.feat_kind(feat)?)))
            .collect();
        feats.sort_by_cached_key(|(feat, kind)| (*kind, self.feat_depth(feat), *feat));
        let (mut improved, mut widened, mut multiplier) = (false, 0u8, stats.multiplier);
        for (feat, _) in &feats {
            for component in self.catalog.feat_components(feat) {
                let (attack, damage) = match component {
                    Component::WeaponFocus => (1, 0),
                    Component::WeaponSpecialization => (0, 2),
                    Component::ImprovedCritical => {
                        improved = true;
                        continue;
                    }
                    Component::CriticalEdge => {
                        widened = widened.saturating_add(1);
                        continue;
                    }
                    Component::CriticalMultiplier => {
                        multiplier = multiplier.saturating_add(1);
                        continue;
                    }
                    _ => continue,
                };
                parts.push(AttackPart {
                    name: self.feat_name(feat).to_owned(),
                    attack,
                    damage,
                });
            }
        }
        let attack: i16 = parts.iter().map(|part| part.attack).sum();
        let power_attack = (state.feats.keys()).any(|feat| {
            (self.catalog.feat_components(feat).iter())
                .any(|component| matches!(component, Component::PowerAttack))
        });
        Some(AttackLine {
            weapon: weapon.clone(),
            attacks: iteratives(state.bab).map(|base| base + attack).collect(),
            damage: stats.damage,
            damage_bonus: parts.iter().map(|part| part.damage).sum(),
            threat: {
                let improved = if improved {
                    // As many rolls again below the range: 19–20 (two rolls) becomes 17–20.
                    stats.threat.saturating_sub(21 - stats.threat)
                } else {
                    stats.threat
                };
                improved.saturating_sub(widened).max(1)
            },
            multiplier,
            power_attack: (stats.ranged.is_none() && power_attack)
                .then(|| power_attack_at(state.bab, stats.handedness)),
            parts,
            leaves_out: ATTACK_LINE_LEAVES_OUT,
        })
    }

    /// The attack-line kind a feat's components give, when they give one: its parts' order.
    fn feat_kind(&self, feat: &FeatKey) -> Option<Kind> {
        (self.catalog.feat_components(feat).iter()).find_map(|component| match component {
            Component::WeaponFocus => Some(Kind::Focus),
            Component::WeaponSpecialization => Some(Kind::Specialization),
            Component::ImprovedCritical
            | Component::CriticalEdge
            | Component::CriticalMultiplier => Some(Kind::Critical),
            _ => None,
        })
    }

    /// How far down its chain of attack-line feats a feat is: 0 for one whose prerequisite names none, else one more
    /// than the furthest it names (Weapon Focus 0, Greater Weapon Focus and Weapon Specialization 1, Greater Weapon
    /// Specialization 2), so a feat comes after those it requires.
    fn feat_depth(&self, feat: &FeatKey) -> usize {
        let mut named = std::collections::HashSet::new();
        if let Some(prerequisite) = self
            .catalog
            .feat(feat)
            .and_then(|f| f.prerequisite.as_ref())
        {
            prerequisite.collect_feats(&mut named);
        }
        (named.into_iter())
            .filter(|key| *key != feat && self.feat_kind(key).is_some())
            .map(|key| self.feat_depth(key) + 1)
            .max()
            .unwrap_or(0)
    }
}

/// An attack-line kind of a feat, in the order the line lists its parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Focus,
    Specialization,
    Critical,
}

/// Power Attack at a BAB, as the guide has it: −1 to attack for +2 damage, one step more at BAB 4 and every four
/// after; +3 damage a step with a two-handed weapon.
fn power_attack_at(bab: u8, handedness: Handedness) -> PowerAttack {
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

/// An ability score's modifier: −1 for 8 or 9, +4 for 18 or 19.
fn modifier(score: u8) -> i16 {
    (i16::from(score) - 10).div_euclid(2)
}

/// The base of each attack of a full attack: the BAB, then 5 less for each further attack while it stays above 0:
/// four at BAB 16 to 20, one more for each 5 past 20, and eight at most (`other-mythic-paths#playing-a-legend`).
fn iteratives(bab: u8) -> impl Iterator<Item = i16> {
    let bab = i16::from(bab);
    (0..8)
        .map(move |n| bab - 5 * n)
        .enumerate()
        .take_while(|(n, base)| *n == 0 || *base > 0)
        .map(|(_, base)| base)
}
