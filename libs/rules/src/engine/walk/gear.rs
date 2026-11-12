//! The gear a Build holds (ticket 166), as the walk counts it: checked once at the Build start, since an item counts
//! at every level, then added to the ability scores and to the bonuses in force.
//!
//! - **Scores:** the items' ability score bonuses and penalties, stacked by type ([`stacked`]), go onto the base
//!   scores before the features' bonuses ([`Walker::own_scores`]), so a belt can move a "highest score" feature's
//!   bonus. Every reader of [`CharacterState::ability_scores`] then sees gear, but for a level's skill points, which
//!   read the scores without it ([`Walker::skill_point_scores`]).
//! - **Other numbers:** the items' other bonuses join the bonuses in force at each level's end
//!   (`Walker::record_bonuses`), each with its type and its item, after the Item bonuses.
//!
//! A Build without gear takes none of these paths: the walk asks [`Gear::is_empty`] first.
//!
//! [`CharacterState::ability_scores`]: crate::CharacterState::ability_scores
//! [`Gear::is_empty`]: crate::Gear::is_empty

use super::Walker;
use crate::bonus::{BonusInForce, BonusSource, BonusType, InForceTarget, stacked};
use crate::build::AbilityScores;
use crate::choice::{Choice, Issue};
use crate::gear::{GearItem, GearTarget};

/// What the gear a Build holds adds, worked out once at the Build start.
#[derive(Clone, Debug, Default)]
pub(super) struct WornGear {
    /// What the items add to each ability score, in [`AbilityScores::ABILITIES`]' order, already stacked.
    scores: [i16; 6],
    /// The items' other bonuses, in the order the Build holds its items, each item's in its own order: the tail of
    /// every level's bonuses in force. Only a walk that keeps the sheet has them.
    pub(super) bonuses: Vec<BonusInForce>,
}

impl WornGear {
    /// Whether the gear changes any ability score.
    pub(super) fn changes_scores(&self) -> bool {
        self.scores.iter().any(|amount| *amount != 0)
    }

    /// The scores with the gear's bonuses and penalties: never below 0 nor above 255.
    pub(super) fn onto(&self, mut scores: AbilityScores) -> AbilityScores {
        for (ability, amount) in AbilityScores::ABILITIES.into_iter().zip(self.scores) {
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
        for (at, ability) in AbilityScores::ABILITIES.into_iter().enumerate() {
            let bonuses: Vec<(BonusType, i16)> = held
                .iter()
                .flat_map(|item| &item.bonuses)
                .filter(|bonus| bonus.target == GearTarget::Ability(ability))
                .map(|bonus| (bonus.kind, i16::from(bonus.amount)))
                .collect();
            gear.scores[at] = stacked(&bonuses);
        }
        if self.sheet {
            for item in &held {
                for bonus in &item.bonuses {
                    let target = match &bonus.target {
                        GearTarget::Ability(_) => continue,
                        GearTarget::CasterLevel => InForceTarget::CasterLevel,
                        target => match target.bonus_target() {
                            Some(target) => InForceTarget::Number(target),
                            None => continue,
                        },
                    };
                    gear.bonuses.push(BonusInForce {
                        source: BonusSource::Gear(item.key.clone()),
                        value: None,
                        target,
                        amount: i16::from(bonus.amount),
                        kind: bonus.kind,
                        name: item.name.clone(),
                    });
                }
            }
        }
        self.gear = gear;
    }

    /// The scores a level's skill points read: the character's own without its gear (the user's rule, ticket 163:
    /// ranks stay valid whatever is worn).
    pub(super) fn skill_point_scores(&self) -> AbilityScores {
        if self.gear.changes_scores() {
            self.own_scores(false)
        } else {
            self.state.ability_scores
        }
    }
}
