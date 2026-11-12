//! The pickers and steppers of the Build start and a level: race and subrace, faith, alignment, background, the
//! racial ability, the point buy, and a level's ability increase and skill ranks. Each option and step carries the
//! edit that takes it and whether it can be taken now.

use super::walk::Cursor;
use super::{ChoiceOption, Engine, RaceChoice, Step, Stepper};
use crate::build::{AbilityScores, Build, BuildStart, Edit, Faith, has_ability_increase};
use crate::game_data::{Ability, Alignment};
use crate::keys::{BackgroundKey, SkillKey};

impl Engine {
    /// The race picker: every race in key order, each followed by its subraces in key order. A race (or subrace)
    /// that doesn't choose where a racial bonus goes also clears the start's racial ability. One that would break a
    /// later choice (a pick in its race's feat slot, an ability score a prerequisite needs) is
    /// [`Issue::BreaksLater`](crate::Issue::BreaksLater).
    ///
    /// Costs a walk of the Build for each race or subrace but the Build's.
    pub fn race_options(&self, build: &Build) -> Vec<ChoiceOption<RaceChoice>> {
        let cursor = Cursor::new(self, build);
        let mut options = Vec::new();
        for race in self.races() {
            let subraces = race
                .subraces
                .iter()
                .map(|subrace| Some(subrace.key.clone()));
            for subrace in std::iter::once(None).chain(subraces) {
                let mut start = BuildStart {
                    race: Some(race.key.clone()),
                    subrace: subrace.clone(),
                    ..build.start.clone()
                };
                if !self.chooses_racial_ability(&start) {
                    start.racial_ability = None;
                }
                let value = RaceChoice {
                    race: race.key.clone(),
                    subrace,
                };
                options.push(self.start_option(&cursor, value, start));
            }
        }
        options
    }

    /// The faith picker: atheism, then every deity in key order. A deity that doesn't allow the start's alignment
    /// is [`Issue::DeityAlignment`](crate::Issue::DeityAlignment); one that would break a later choice (a class
    /// serving a deity, a domain it doesn't offer) is [`Issue::BreaksLater`](crate::Issue::BreaksLater).
    ///
    /// Costs a walk of the Build for each faith that fits but isn't the Build's.
    pub fn faith_options(&self, build: &Build) -> Vec<ChoiceOption<Faith>> {
        let cursor = Cursor::new(self, build);
        std::iter::once(Faith::Atheism)
            .chain(
                self.deities()
                    .iter()
                    .map(|deity| Faith::Deity(deity.key.clone())),
            )
            .map(|faith| {
                let start = BuildStart {
                    deity: Some(faith.clone()),
                    ..build.start.clone()
                };
                self.start_option(&cursor, faith, start)
            })
            .collect()
    }

    /// The alignment picker, in [`Alignment::ALL`]'s order. The alignment comes after the faith: with none chosen
    /// every alignment is [`Issue::NeedsFaith`]; a deity locks the alignments it doesn't allow with
    /// [`Issue::DeityAlignment`]. One that would break a later choice (a class's alignment) is
    /// [`Issue::BreaksLater`](crate::Issue::BreaksLater). The options share their faith's issue, which the wheel
    /// says once.
    ///
    /// Costs a walk of the Build for each alignment that fits but isn't the Build's.
    pub fn alignment_options(&self, build: &Build) -> Vec<ChoiceOption<Alignment>> {
        let cursor = Cursor::new(self, build);
        Alignment::ALL
            .into_iter()
            .map(|alignment| {
                let start = BuildStart {
                    alignment: Some(alignment),
                    ..build.start.clone()
                };
                self.start_option(&cursor, alignment, start)
            })
            .collect()
    }

    /// The background picker: every background in key order; one that would break a later choice (a pick needing
    /// its feat) is [`Issue::BreaksLater`](crate::Issue::BreaksLater).
    ///
    /// Costs a walk of the Build for each background but the Build's.
    pub fn background_options(&self, build: &Build) -> Vec<ChoiceOption<BackgroundKey>> {
        let cursor = Cursor::new(self, build);
        self.backgrounds()
            .iter()
            .map(|background| {
                let start = BuildStart {
                    background: Some(background.key.clone()),
                    ..build.start.clone()
                };
                self.start_option(&cursor, background.key.clone(), start)
            })
            .collect()
    }

    /// Where the racial bonus goes ("+2 to one ability score"): each ability in Str, Dex, Con, Int, Wis, Cha order,
    /// or none when the start's race (or subrace) doesn't choose one. One that would break a later choice (an
    /// ability score a prerequisite needs) is [`Issue::BreaksLater`](crate::Issue::BreaksLater).
    ///
    /// Costs a walk of the Build for each ability but the Build's.
    pub fn racial_ability_options(&self, build: &Build) -> Vec<ChoiceOption<Ability>> {
        if !self.chooses_racial_ability(&build.start) {
            return Vec::new();
        }
        let cursor = Cursor::new(self, build);
        AbilityScores::ABILITIES
            .into_iter()
            .map(|ability| {
                let start = BuildStart {
                    racial_ability: Some(ability),
                    ..build.start.clone()
                };
                self.start_option(&cursor, ability, start)
            })
            .collect()
    }

    /// A level's ability increase picker: +1 to each ability in Str, Dex, Con, Int, Wis, Cha order. One that would
    /// break a later choice (an ability score a later prerequisite needs) is
    /// [`Issue::BreaksLater`](crate::Issue::BreaksLater). `None` for a level the Build doesn't have or that has no
    /// increase (only every 4th does).
    ///
    /// Costs one walk of the Build up to the level, then one from there for each ability but the Build's.
    pub fn ability_increase_options(
        &self,
        build: &Build,
        level: u8,
    ) -> Option<Vec<ChoiceOption<Ability>>> {
        if level == 0 || !has_ability_increase(level) || usize::from(level) > build.levels.len() {
            return None;
        }
        let cursor = Cursor::new(self, build);
        let options = AbilityScores::ABILITIES
            .into_iter()
            .map(|ability| {
                let edit = Edit::AbilityIncrease {
                    level,
                    ability: Some(ability),
                };
                ChoiceOption {
                    value: ability,
                    status: cursor.try_edit(&edit),
                    edit,
                }
            })
            .collect();
        Some(options)
    }

    /// The point buy's steppers, one per score in Str, Dex, Con, Int, Wis, Cha order: a step down and a step up of
    /// one, each `None` past the point buy's range (a score from 7 to 18, within the budget). A step that would break
    /// a later choice (a score a prerequisite needs) is [`Issue::BreaksLater`](crate::Issue::BreaksLater).
    ///
    /// The steps are judged with the Build's gear, as the walk judges the Build: a step the gear keeps legal is
    /// open. One that is open only because of the gear says so ([`Step::with_gear`]), so it never looks safe.
    ///
    /// Costs a walk of the Build for each step in the range; with gear, four more for each open step.
    pub fn point_buy_steps(&self, build: &Build) -> Vec<Stepper<Ability>> {
        let cursor = Cursor::new(self, build);
        let point_buy = build.start.ability_scores;
        let step = |ability: Ability, delta: i8| {
            let next = point_buy.get(ability).checked_add_signed(delta)?;
            let edit = Edit::Start(BuildStart {
                ability_scores: point_buy.with(ability, next)?,
                ..build.start.clone()
            });
            let status = cursor.try_edit(&edit);
            // Allowed with the gear: whether it's allowed only because of it.
            let with_gear = status.is_ok()
                && !build.gear.is_empty()
                && !self.newly_gear_dependent(build, &edit).is_empty();
            Some(Step {
                status,
                edit,
                with_gear,
            })
        };
        AbilityScores::ABILITIES
            .into_iter()
            .map(|ability| Stepper {
                value: ability,
                down: step(ability, -1),
                up: step(ability, 1),
            })
            .collect()
    }

    /// A level's skill rank steppers, one per skill in key order: a rank fewer and a rank more added at that level,
    /// each `None` past the range: no ranks below 0, none above the level's skill points, and no skill's total above
    /// the character level, at that level or a later one (the Build refuses that edit). A step that would break a
    /// later choice (ranks a prerequisite needs) is [`Issue::BreaksLater`](crate::Issue::BreaksLater). `None` for a
    /// level the Build doesn't have.
    ///
    /// Costs one walk of the Build to the end of the level, then one from the level for each step in the range.
    pub fn skill_steps(&self, build: &Build, level: u8) -> Option<Vec<Stepper<SkillKey>>> {
        let level_up = build.levels.get(usize::from(level).checked_sub(1)?)?;
        let cursor = Cursor::new(self, build);
        let points = cursor.skill_points(level)?;
        let step = |skill: &SkillKey, ranks: u8| {
            let edit = Edit::SkillRanks {
                level,
                skill: skill.clone(),
                ranks,
            };
            let mut after = build.clone();
            after.apply(&edit);
            (&after != build).then(|| Step {
                status: cursor.try_edit(&edit),
                edit,
                with_gear: false,
            })
        };
        let steppers = self
            .skills()
            .iter()
            .map(|skill| {
                let here = level_up.skill_ranks.get(&skill.key).copied().unwrap_or(0);
                let up = (points.spent < points.available)
                    .then(|| here.checked_add(1))
                    .flatten();
                Stepper {
                    value: skill.key.clone(),
                    down: here
                        .checked_sub(1)
                        .and_then(|ranks| step(&skill.key, ranks)),
                    up: up.and_then(|ranks| step(&skill.key, ranks)),
                }
            })
            .collect();
        Some(steppers)
    }

    /// A Build start picker's option for a new start, with the cursor's verdict: its faith and alignment's misfit,
    /// else whether it breaks a later choice (the walk of the Build with it, unless it changes nothing).
    fn start_option<T>(&self, cursor: &Cursor<'_>, value: T, start: BuildStart) -> ChoiceOption<T> {
        let edit = Edit::Start(start);
        ChoiceOption {
            value,
            status: cursor.try_edit(&edit),
            edit,
        }
    }
}
