//! The gear a Build holds (ticket 166), as the walk counts it: checked once at the Build start, since an item counts
//! at every level. Its [`GearStat`](crate::Component::GearStat) components (ticket 297) join every other source's stat
//! bonuses, stacked together by type:
//!
//! - **Scores:** the items' ability score bonuses and penalties go into each score's bonuses
//!   ([`Walker::ability_amounts`]), so a belt can move a "highest score" feature's bonus. Every reader of
//!   [`CharacterState::ability_scores`] then sees gear, but for a level's skill points, which read the scores without
//!   it ([`Walker::skill_point_scores`]), and ability prerequisites, which read
//!   [`CharacterState::permanent_ability_scores`]: the scores with only the items' enhancement bonuses among the gear's
//!   (ticket 252).
//! - **Other numbers:** the items' other bonuses join the bonuses in force at each level's end
//!   (`Walker::record_bonuses`), each with its type and its item, after the entries'.
//!
//! A Build without gear takes none of these paths: the walk asks [`Gear::is_empty`] first.
//!
//! [`CharacterState::ability_scores`]: crate::CharacterState::ability_scores
//! [`CharacterState::permanent_ability_scores`]: crate::CharacterState::permanent_ability_scores
//! [`Gear::is_empty`]: crate::Gear::is_empty

use super::Walker;
use crate::bonus::{BonusInForce, BonusSource, BonusTarget, InForceTarget};
use crate::build::AbilityScores;
use crate::choice::{Choice, Issue};
use crate::component::Descriptor;
use crate::game_data::Save;
use crate::gear::{GearItem, GearTarget};

/// What the gear a Build holds adds, worked out once at the Build start.
#[derive(Clone, Debug, Default)]
pub(super) struct WornGear {
    /// Each always-on bonus of the items held, in the order the Build holds its items, each item's in its own order.
    pub(super) stats: Vec<WornStat>,
    /// The items' bonuses to a number other than an ability score, as bonuses in force, the same at every level: the
    /// tail of every level's list. Only a walk that keeps the sheet has them.
    pub(super) in_force: Vec<BonusInForce>,
}

/// One always-on bonus of an item held.
#[derive(Clone, Debug)]
pub(super) struct WornStat {
    pub target: GearTarget,
    pub descriptor: Descriptor,
    pub amount: i16,
}

impl WornGear {
    /// Whether the gear changes any ability score.
    pub(super) fn changes_scores(&self) -> bool {
        (self.stats.iter()).any(|stat| matches!(stat.target, GearTarget::Ability(_)))
    }

    /// The scores with `amounts` added (what the bonuses add to each, [`AbilityAmounts`](super::components::AbilityAmounts)):
    /// never below 0 nor above 255.
    pub(super) fn onto(amounts: [i16; 6], mut scores: AbilityScores) -> AbilityScores {
        for (ability, amount) in AbilityScores::ABILITIES.into_iter().zip(amounts) {
            let score = scores.get_mut(ability);
            *score = u8::try_from((i16::from(*score) + amount).clamp(0, i16::from(u8::MAX)))
                .unwrap_or(u8::MAX);
        }
        scores
    }
}

impl<'a> Walker<'a> {
    /// The Build's gear, before the ability scores are first worked out: each item must be one the data has
    /// ([`Issue::UnknownKey`]), in a place that takes its slot, a usable one used and no other
    /// ([`Issue::NotAllowed`]; both [`Engine::gear_fit`](crate::Engine)), and worn once ([`Issue::AlreadyHeld`]: one ring in both ring places). An item that
    /// fails counts for nothing.
    pub(super) fn wear_gear(&mut self) {
        let (engine, build) = (self.engine, self.build);
        if build.gear.is_empty() {
            return;
        }
        let mut held: Vec<&'a GearItem> = Vec::new();
        for (spot, key) in build.gear.items() {
            let issue = match engine.gear_fit(&spot, key) {
                Err(issue) => Some(issue),
                Ok(item) if held.iter().any(|other| other.key == item.key) => {
                    Some(Issue::AlreadyHeld)
                }
                Ok(item) => {
                    held.push(item);
                    None
                }
            };
            if let Some(issue) = issue {
                self.fail(Choice::Gear(spot), issue);
            }
        }
        let mut gear = WornGear::default();
        for item in &held {
            for (target, value, descriptor) in item.stats() {
                gear.stats.push(WornStat {
                    target: target.clone(),
                    descriptor,
                    amount: i16::from(value),
                });
                if self.sheet {
                    let bonus = |target| BonusInForce {
                        source: BonusSource::Gear(item.key.clone()),
                        value: None,
                        target,
                        amount: i16::from(value),
                        descriptor,
                        name: item.name.clone(),
                    };
                    let number = |target| bonus(InForceTarget::Number(target));
                    match target {
                        GearTarget::Ability(_) => {}
                        GearTarget::AllSaves => (gear.in_force)
                            .extend(Save::ALL.map(|save| number(BonusTarget::Save(save)))),
                        GearTarget::Save(save) => {
                            gear.in_force.push(number(BonusTarget::Save(*save)));
                        }
                        GearTarget::Initiative => {
                            gear.in_force.push(number(BonusTarget::Initiative))
                        }
                        GearTarget::Skill(skill) => {
                            gear.in_force
                                .push(number(BonusTarget::Skill(skill.clone())));
                        }
                        GearTarget::SpellDc(scope) => {
                            gear.in_force.push(number(BonusTarget::SpellDc(*scope)));
                        }
                        GearTarget::CasterLevel => {
                            gear.in_force.push(bonus(InForceTarget::CasterLevel))
                        }
                        GearTarget::SpellPenetration => {
                            gear.in_force.push(number(BonusTarget::SpellPenetration));
                        }
                        GearTarget::Concentration => {
                            gear.in_force.push(number(BonusTarget::Concentration));
                        }
                    }
                }
            }
        }
        self.gear = gear;
    }

    /// The scores a level's skill points read: the character's own without its gear (the user's rule, ticket 163:
    /// ranks stay valid whatever is worn).
    pub(super) fn skill_point_scores(&self) -> AbilityScores {
        if self.gear.changes_scores() {
            self.own_scores()
        } else {
            self.state.ability_scores
        }
    }
}
