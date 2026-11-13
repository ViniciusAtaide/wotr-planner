//! The walk's failures worded: each one's address, label and reason from the declared Build and its sheet, what it
//! would have given, and what it can't be reached without. The issue is the engine's alone; the game data is read
//! here only to name the closest keys after an unknown one.

use rules::{
    Address, ArchetypeKey, BackgroundKey, Build, Choice, ClassKey, DeityKey, Engine, Failure,
    Faith, GameData, GearKey, Issue, MythicPathKey, Pick, RaceKey, RankStatus, Sheet, Slot, SlotId,
    SubraceKey,
};

use super::closest;
use super::tree::{At, Given, Node, Why, pick_gives};
use crate::options::issue_reason;
use crate::planner::{Planner, all_selections};

/// What a walk failure is worded from.
pub(super) struct Wording<'a> {
    pub(super) planner: &'a Planner,
    pub(super) build: &'a Build,
    pub(super) sheet: &'a Sheet,
}

impl Wording<'_> {
    fn engine(&self) -> &Engine {
        &self.planner.engine
    }

    fn data(&self) -> &GameData {
        &self.planner.data
    }

    pub(super) fn node(&self, failure: &Failure) -> Node {
        let Failure { choice, issue } = failure;
        // A level past the cap is declared by its class.
        let address = match choice {
            Choice::CharacterLevel { level } => Address::Class(*level),
            choice => Address::from(choice),
        };
        Node {
            label: self.label(&address),
            reason: self.reason(choice, issue),
            gives: self.gives(choice),
            under: match issue {
                Issue::NotReached => self.reached_through(choice),
                _ => None,
            },
            why: Why::Walk(issue.clone()),
            at: At::Address(address),
        }
    }

    /// A failure's label at an address, after its place, with the sheet's slot at a slot address
    /// ([`Engine::placed_label`]): the walk's failures and the declaration's refusals alike.
    pub(super) fn label(&self, address: &Address) -> String {
        self.engine().placed_label(address, self.slot_of(address))
    }

    /// The sheet's slot at a slot address, when the walk reached it.
    fn slot_of(&self, address: &Address) -> Option<&Slot> {
        let Address::Slot(id) = address else {
            return None;
        };
        self.sheet
            .levels
            .iter()
            .flat_map(|level| &level.slots)
            .chain(self.sheet.ranks.iter().flat_map(|rank| &rank.slots))
            .find(|slot| &slot.id == id)
    }

    /// What the failed choice would have given: its pick's ([`pick_gives`]), its class's levels, its archetype.
    fn gives(&self, choice: &Choice) -> Vec<Given> {
        let build = self.build;
        match choice {
            Choice::Slot(slot) => build
                .picks
                .get(slot)
                .map(|pick| pick_gives(self.data(), pick))
                .unwrap_or_default(),
            Choice::Class { level } => class_at(build, *level)
                .map(|class| vec![Given::Class(class.clone())])
                .unwrap_or_default(),
            Choice::Archetype { class } => build
                .class_choices
                .get(class)
                .and_then(|choice| choice.archetype.clone())
                .map(|archetype| vec![Given::Archetype(archetype)])
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    /// The choice an unreached choice can't be reached without: its slot's level's class, its path slot's path
    /// choice (the first ascension's at MR 1 and 2, the switch's from MR 8 when there is one), the general mythic
    /// pick that gives its rank's extra or selection slot.
    fn reached_through(&self, choice: &Choice) -> Option<Address> {
        let Choice::Slot(slot) = choice else {
            return None;
        };
        if let Some(level) = slot.level() {
            return Some(Address::Class(level));
        }
        Some(match *slot {
            SlotId::PathFeat { rank, .. } | SlotId::PathSelection { rank, .. } => {
                if rank < rules::PATH_CHOICE_RANK && self.build.first_ascension.is_some() {
                    Address::FirstAscension
                } else if rank >= rules::PATH_SWITCH_RANK && self.build.path_switch.is_some() {
                    Address::Switch
                } else {
                    Address::Path
                }
            }
            SlotId::MythicSelection { rank, .. } | SlotId::MythicExtra { rank } => {
                Address::Slot(SlotId::Mythic { rank })
            }
            _ => return None,
        })
    }

    /// The reason: the engine's own text, or, where the issue alone says too little (a key the data lacks, a choice
    /// not taken there or not reached, overspent points), what the declaration and the sheet add.
    fn reason(&self, choice: &Choice, issue: &Issue) -> String {
        match issue {
            Issue::UnknownKey => self.unknown(choice),
            Issue::NotAllowed => self.not_allowed(choice),
            Issue::NotReached => self.not_reached(choice),
            issue => issue_reason(self.engine(), issue),
        }
    }

    /// A key the game data lacks, with the closest of its kind.
    fn unknown(&self, choice: &Choice) -> String {
        let data = self.data();
        let build = self.build;
        let start = &build.start;
        let paths = || {
            data.mythic_paths
                .iter()
                .map(|path| (path.name.as_str(), path.key.as_str()))
        };
        let (given, noun, near) = match choice {
            Choice::Race => {
                let given = start.race.as_ref().map_or("", RaceKey::as_str);
                let races = data.races.iter().flat_map(|race| {
                    std::iter::once((race.name.as_str(), race.key.as_str())).chain(
                        race.subraces
                            .iter()
                            .map(|subrace| (subrace.name.as_str(), subrace.key.as_str())),
                    )
                });
                (given, "a race", closest(given, races))
            }
            Choice::Subrace => {
                let given = start.subrace.as_ref().map_or("", SubraceKey::as_str);
                let subraces = data.races.iter().flat_map(|race| {
                    race.subraces
                        .iter()
                        .map(|subrace| (subrace.name.as_str(), subrace.key.as_str()))
                });
                (given, "a subrace", closest(given, subraces))
            }
            Choice::Deity => {
                let given = start
                    .deity
                    .as_ref()
                    .and_then(Faith::deity)
                    .map_or("", DeityKey::as_str);
                let deities = data
                    .deities
                    .iter()
                    .map(|deity| (deity.name.as_str(), deity.key.as_str()));
                let near = closest(given, deities);
                (given, "a deity", format!(" (`atheism` for none){near}"))
            }
            Choice::Background => {
                let given = start.background.as_ref().map_or("", BackgroundKey::as_str);
                let backgrounds = data
                    .backgrounds
                    .iter()
                    .map(|background| (background.name.as_str(), background.key.as_str()));
                (given, "a background", closest(given, backgrounds))
            }
            Choice::Class { level } => {
                let given = class_at(build, *level).map_or("", ClassKey::as_str);
                let classes = data
                    .all_classes()
                    .map(|class| (class.name.as_str(), class.key.as_str()));
                (given, "a class", closest(given, classes))
            }
            Choice::Archetype { class } => {
                let given = archetype_of(build, class).map_or("", ArchetypeKey::as_str);
                let archetypes = data.class(class).into_iter().flat_map(|class| {
                    class
                        .archetypes
                        .iter()
                        .map(|archetype| (archetype.name.as_str(), archetype.key.as_str()))
                });
                (given, "an archetype", closest(given, archetypes))
            }
            Choice::Advances { class } => {
                let unknown: Vec<&str> = build
                    .class_choices
                    .get(class)
                    .map(|choice| {
                        choice
                            .advances
                            .iter()
                            .filter(|key| data.class(key).is_none())
                            .map(ClassKey::as_str)
                            .collect()
                    })
                    .unwrap_or_default();
                return format!(
                    "The {}'s advanced classes name {}, which the planner's data lacks.",
                    self.engine().class_name(class),
                    unknown.join(" and ")
                );
            }
            Choice::SkillRanks { skill, .. } => {
                let skills = data
                    .skills
                    .iter()
                    .map(|skill| (skill.name.as_str(), skill.key.as_str()));
                (skill.as_str(), "a skill", closest(skill.as_str(), skills))
            }
            Choice::FirstAscension => {
                let given = build
                    .first_ascension
                    .as_ref()
                    .map_or("", MythicPathKey::as_str);
                (given, "a path", closest(given, paths()))
            }
            Choice::MythicPath => {
                let given = build.mythic_path.as_ref().map_or("", MythicPathKey::as_str);
                (given, "a path", closest(given, paths()))
            }
            Choice::PathSwitch => {
                let given = build.path_switch.as_ref().map_or("", MythicPathKey::as_str);
                (given, "a path", closest(given, paths()))
            }
            Choice::Slot(slot) => {
                let Some(pick) = build.picks.get(slot) else {
                    return issue_reason(self.engine(), &Issue::UnknownKey);
                };
                let (noun, items) = items_like(data, pick);
                let near = closest(pick.key(), items.into_iter());
                (pick.key(), noun, near)
            }
            Choice::Gear(spot) => {
                let given = build.gear.at(spot).map_or("", GearKey::as_str);
                let items = (data.gear.iter()).map(|item| (item.name.as_str(), item.key.as_str()));
                (given, "a gear item", closest(given, items))
            }
            _ => return issue_reason(self.engine(), &Issue::UnknownKey),
        };
        format!("`{given}` isn't {noun} in the planner's data{near}.")
    }

    /// A choice the data has, not taken there.
    fn not_allowed(&self, choice: &Choice) -> String {
        let engine = self.engine();
        let data = self.data();
        let build = self.build;
        let start = &build.start;
        match choice {
            Choice::Subrace => {
                let given = start.subrace.as_ref().map_or("", SubraceKey::as_str);
                let race = start
                    .race
                    .as_ref()
                    .and_then(|race| data.races.iter().find(|candidate| &candidate.key == race));
                match race {
                    Some(race) => {
                        let subraces = race
                            .subraces
                            .iter()
                            .map(|subrace| (subrace.name.as_str(), subrace.key.as_str()));
                        format!(
                            "`{given}` isn't a subrace of the {}{}.",
                            race.name,
                            closest(given, subraces)
                        )
                    }
                    None => format!("`{given}` isn't a subrace of the Build's race."),
                }
            }
            Choice::Archetype { class } => {
                let given = archetype_of(build, class).map_or("", ArchetypeKey::as_str);
                let archetypes = data.class(class).into_iter().flat_map(|class| {
                    class
                        .archetypes
                        .iter()
                        .map(|archetype| (archetype.name.as_str(), archetype.key.as_str()))
                });
                format!(
                    "`{given}` isn't an archetype of the {}{}.",
                    engine.class_name(class),
                    closest(given, archetypes)
                )
            }
            Choice::Advances { class } => {
                let advanced: Vec<&str> = build
                    .class_choices
                    .get(class)
                    .map(|choice| choice.advances.iter().map(ClassKey::as_str).collect())
                    .unwrap_or_default();
                format!(
                    "The {} can't advance {}: it advances classes the Build takes before it that cast the spells it \
                     advances, at most one arcane and one divine (`options` on `level.N.class` lists its answers).",
                    engine.class_name(class),
                    advanced.join(" and ")
                )
            }
            Choice::SkillPoints { level } => match self.sheet.levels.get(usize::from(*level) - 1) {
                Some(row) => format!(
                    "Level {level} spends {} skill points of its {}.",
                    row.skill_points.spent, row.skill_points.available
                ),
                None => format!("Level {level} spends more skill points than it has."),
            },
            Choice::FirstAscension => {
                let given = build
                    .first_ascension
                    .as_ref()
                    .map_or("", MythicPathKey::as_str);
                let paths = data
                    .mythic_paths
                    .iter()
                    .filter(|path| {
                        path.switch.is_none() && path.row(rules::FIRST_ASCENSION_RANK).is_some()
                    })
                    .map(|path| (path.name.as_str(), path.key.as_str()));
                format!(
                    "`{given}` isn't a path whose path-linked ability the first ascension takes{}.",
                    closest(given, paths)
                )
            }
            Choice::MythicPath => {
                let given = build.mythic_path.as_ref().map_or("", MythicPathKey::as_str);
                let paths = data
                    .mythic_paths
                    .iter()
                    .filter(|path| path.switch.is_none())
                    .map(|path| (path.name.as_str(), path.key.as_str()));
                format!(
                    "`{given}` is a Path switch, not a path chosen at MR 3{}.",
                    closest(given, paths)
                )
            }
            Choice::PathSwitch => {
                let given = build.path_switch.as_ref().map_or("", MythicPathKey::as_str);
                let paths = data
                    .mythic_paths
                    .iter()
                    .filter(|path| path.switch.is_some())
                    .map(|path| (path.name.as_str(), path.key.as_str()));
                format!("`{given}` isn't a Path switch{}.", closest(given, paths))
            }
            Choice::Gear(spot) => {
                let item = build.gear.at(spot).and_then(|key| engine.gear_item(key));
                match item {
                    Some(item) => format!(
                        "{} ({}) doesn't go at `{}`: its place is {}.",
                        item.name,
                        item.slot.name(),
                        Address::Gear(spot.clone()),
                        crate::gear::places(item).join(" or ")
                    ),
                    None => issue_reason(engine, &Issue::NotAllowed),
                }
            }
            Choice::Slot(slot) => match build.picks.get(slot) {
                Some(pick) => {
                    let address = Address::Slot(slot.clone());
                    let label = self.label(&address);
                    format!(
                        "{} isn't a pick this slot takes ({label}), or not with that value: `options` lists what it \
                         takes.",
                        engine.pick_name(pick)
                    )
                }
                None => issue_reason(engine, &Issue::NotAllowed),
            },
            Choice::RankLevel { rank } => match engine.rank_table_level(*rank) {
                Some(earliest) => {
                    // Where the rank comes without this declaration: the table's level, or the rank below's.
                    let mut without = self.build.clone();
                    without.rank_levels.remove(rank);
                    let comes_at = engine.rank_floor(&without, *rank).unwrap_or(earliest);
                    if !engine.rank_reached(&without, *rank) {
                        let levels = without.levels.len();
                        return match comes_at > u8::try_from(levels).unwrap_or(u8::MAX) {
                            true => format!(
                                "The Build doesn't reach MR {rank}, so `mythic.{rank}.level` has no level to take it at: \
                                 MR {rank} comes at level {comes_at} at the earliest, and the Build has {levels} \
                                 level(s)."
                            ),
                            false => format!(
                                "The Build doesn't reach MR {rank}, so `mythic.{rank}.level` has no level to take it at: \
                                 its Path switch (Legend) ends the ascensions at MR {}.",
                                rules::PATH_SWITCH_RANK
                            ),
                        };
                    }
                    let why = if comes_at > earliest {
                        format!("MR {} is taken at level {comes_at}", rank - 1)
                    } else {
                        "the rank table's level".to_owned()
                    };
                    format!(
                        "MR {rank} comes at level {comes_at} at the earliest ({why}): `mythic.{rank}.level` takes \
                         level {comes_at} (where it comes) or a later level, up to the Build's last. To take the rank \
                         at level {comes_at}, leave `mythic.{rank}.level` out."
                    )
                }
                None => format!("There is no MR {rank}: the Mythic ranks are 1 to 10."),
            },
            _ => issue_reason(engine, &Issue::NotAllowed),
        }
    }

    /// A choice the Build doesn't reach, with what it does have there.
    fn not_reached(&self, choice: &Choice) -> String {
        let engine = self.engine();
        let sheet = self.sheet;
        let levels = self.build.levels.len();
        let rank_line = |rank: u8| sheet.ranks.iter().find(|line| line.rank == rank);
        let rank_text = |rank: u8, what: &str| match rank_line(rank) {
            Some(line) => format!(
                "The Build doesn't reach MR {rank}, where {what}: its ascension comes at character level {}, and the \
                 Build has {levels} level(s).",
                line.character_level
            ),
            None => format!("There is no MR {rank}."),
        };
        let addresses = |slots: &[Slot]| -> Vec<String> {
            slots
                .iter()
                .map(|slot| Address::Slot(slot.id.clone()).to_string())
                .collect()
        };
        match choice {
            Choice::Slot(slot) => {
                if let Some(level) = slot.level() {
                    let Some(row) = sheet.levels.get(usize::from(level) - 1) else {
                        return format!(
                            "The Build has {levels} level(s), so it doesn't reach level {level}."
                        );
                    };
                    return format!(
                        "Level {level} ({} {}) has no such slot. Its slots: {}.",
                        engine.class_name(&row.class),
                        row.class_level,
                        or_none(&addresses(&row.slots))
                    );
                }
                let rank = slot.rank().unwrap_or(0);
                match rank_line(rank) {
                    Some(line) if line.status != RankStatus::NotReached => format!(
                        "MR {rank} has no such slot. Its slots: {}.",
                        or_none(&addresses(&line.slots))
                    ),
                    _ => rank_text(rank, "this slot is"),
                }
            }
            Choice::Archetype { class } => format!(
                "No level takes the {}: a class's archetype and advanced classes come with its levels.",
                engine.class_name(class)
            ),
            Choice::FirstAscension => {
                rank_text(rules::FIRST_ASCENSION_RANK, "the first ascension is chosen")
            }
            Choice::MythicPath => rank_text(rules::PATH_CHOICE_RANK, "the path is chosen"),
            Choice::PathSwitch => rank_text(rules::PATH_SWITCH_RANK, "the Path switch is chosen"),
            Choice::CharacterLevel { level } => format!(
                "Level {level} is past the level cap of {} (40 after a Legend Path switch).",
                sheet.level_cap
            ),
            Choice::RankLevel { rank } => match self.build.rank_levels.get(rank) {
                Some(level) if usize::from(*level) > levels => format!(
                    "MR {rank} can't be taken at level {level}: the Build has {levels} level(s), and a rank is taken \
                     at a level the Build has."
                ),
                Some(_) if engine.rank_table_level(*rank).is_some() => format!(
                    "MR {rank} never comes: the Path switch ends the ascensions before it, so it has no level."
                ),
                _ => format!("There is no MR {rank}: the Mythic ranks are 1 to 10."),
            },
            _ => issue_reason(engine, &Issue::NotReached),
        }
    }
}

/// The class a level takes, when the Build has the level.
fn class_at(build: &Build, level: u8) -> Option<&ClassKey> {
    let index = usize::from(level).checked_sub(1)?;
    build.levels.get(index).map(|level_up| &level_up.class)
}

/// The archetype the Build gives a class, if any.
fn archetype_of<'a>(build: &'a Build, class: &ClassKey) -> Option<&'a ArchetypeKey> {
    build
        .class_choices
        .get(class)
        .and_then(|choice| choice.archetype.as_ref())
}

/// The kind of item a pick names, with its article, and every item of that kind by (name, key): the candidates for
/// the closest after an unknown key.
fn items_like<'a>(data: &'a GameData, pick: &Pick) -> (&'static str, Vec<(&'a str, &'a str)>) {
    match pick {
        Pick::Feat { .. } => (
            "a feat",
            data.feats
                .iter()
                .map(|feat| (feat.name.as_str(), feat.key.as_str()))
                .collect(),
        ),
        Pick::SelectionOption { .. } => (
            "an option of a Feature selection",
            all_selections(data)
                .flat_map(|selection| &selection.options)
                .map(|option| (option.name.as_str(), option.key.as_str()))
                .collect(),
        ),
        Pick::MythicAbility { .. } => (
            "a mythic ability",
            data.mythic_abilities
                .iter()
                .map(|ability| (ability.name.as_str(), ability.key.as_str()))
                .collect(),
        ),
        Pick::MythicFeat { .. } => (
            "a mythic feat",
            data.mythic_feats
                .iter()
                .map(|feat| (feat.name.as_str(), feat.key.as_str()))
                .collect(),
        ),
    }
}

/// A list of addresses joined for a reason, or "none" when it's empty.
fn or_none(list: &[String]) -> String {
    match list.is_empty() {
        true => "none".to_owned(),
        false => list.join(", "),
    }
}
