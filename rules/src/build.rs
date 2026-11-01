//! A Build: the choices a player makes, and nothing derived from them.
//!
//! Everything is stored as game-data text keys, so a Build survives re-seeds and can name things that no longer
//! exist (the engine reports those rather than failing). The URL codec serialises this type as is.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::game_data::{Ability, Alignment};
use crate::keys::{
    ArchetypeKey, ClassKey, FeatKey, MythicAbilityKey, MythicFeatKey, MythicPathKey, RaceKey,
    SelectionOptionKey, SkillKey, SubraceKey,
};
use crate::parameter::Parameter;
use crate::problem::Choice;

/// A plan for one character: the Build start, one Level-up per character level, each class's archetype and
/// spellcasting choice, and every pick, keyed by the slot it fills.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Build {
    /// An optional name, carried in the link.
    pub name: Option<String>,
    pub start: BuildStart,
    /// `levels[0]` is character level 1.
    pub levels: Vec<LevelUp>,
    /// The archetype and spellcasting advancement of each class the Build takes. They belong to the class, not to
    /// a level: the game fixes them when the class is first taken, wherever that level ends up.
    pub class_choices: BTreeMap<ClassKey, ClassChoice>,
    /// The mythic path chosen at the path choice rank (MR 3).
    pub mythic_path: Option<MythicPathKey>,
    /// The path switched to at the switch rank (MR 8), if any.
    pub path_switch: Option<MythicPathKey>,
    /// Feats, Feature selections, mythic abilities and mythic feats, keyed by the slot they fill. A slot's id says
    /// which Level-up or Mythic ascension it belongs to.
    pub picks: BTreeMap<SlotId, Pick>,
}

/// The choices made before level 1.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildStart {
    pub race: Option<RaceKey>,
    pub subrace: Option<SubraceKey>,
    /// The ability a "+2 to one ability score" racial modifier goes to.
    pub racial_ability: Option<Ability>,
    pub alignment: Option<Alignment>,
    /// The point-buy scores, before racial modifiers.
    pub ability_scores: AbilityScores,
}

/// The six ability scores.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AbilityScores {
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub intelligence: u8,
    pub wisdom: u8,
    pub charisma: u8,
}

impl Default for AbilityScores {
    /// All 10s: an unspent point buy.
    fn default() -> Self {
        Self {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        }
    }
}

impl AbilityScores {
    pub const ABILITIES: [Ability; 6] = [
        Ability::Strength,
        Ability::Dexterity,
        Ability::Constitution,
        Ability::Intelligence,
        Ability::Wisdom,
        Ability::Charisma,
    ];

    pub fn get(&self, ability: Ability) -> u8 {
        match ability {
            Ability::Strength => self.strength,
            Ability::Dexterity => self.dexterity,
            Ability::Constitution => self.constitution,
            Ability::Intelligence => self.intelligence,
            Ability::Wisdom => self.wisdom,
            Ability::Charisma => self.charisma,
        }
    }

    pub fn get_mut(&mut self, ability: Ability) -> &mut u8 {
        match ability {
            Ability::Strength => &mut self.strength,
            Ability::Dexterity => &mut self.dexterity,
            Ability::Constitution => &mut self.constitution,
            Ability::Intelligence => &mut self.intelligence,
            Ability::Wisdom => &mut self.wisdom,
            Ability::Charisma => &mut self.charisma,
        }
    }

    /// The ability modifier, `(score - 10) / 2` rounded down.
    pub fn modifier(&self, ability: Ability) -> i8 {
        (i16::from(self.get(ability)) - 10).div_euclid(2) as i8
    }
}

/// The choices made once for a class, when it's first taken.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassChoice {
    /// At most one archetype per class.
    pub archetype: Option<ArchetypeKey>,
    /// Prestige classes: the classes whose spellcasting this one advances (one arcane and/or one divine).
    pub advances: BTreeSet<ClassKey>,
}

/// Everything chosen at one character level except its picks (those live in [`Build::picks`]) and the class's
/// archetype and advancement (in [`Build::class_choices`]).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelUp {
    pub class: ClassKey,
    /// The +1 at every 4th character level.
    pub ability_increase: Option<Ability>,
    /// Ranks added at this level.
    pub skill_ranks: BTreeMap<SkillKey, u8>,
}

impl LevelUp {
    pub fn new(class: ClassKey) -> Self {
        Self {
            class,
            ability_increase: None,
            skill_ranks: BTreeMap::new(),
        }
    }
}

/// Where a Feat slot at a character level comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FeatSource {
    /// The race's extra slots at level 1 (a human's bonus feat).
    Race,
    /// The class level row taken at this level (class or archetype bonus slots).
    Class,
    /// The general slot at every odd character level.
    General,
}

/// A stable name for one slot in a Build. Ids are positional within their source, so they stay put when other
/// levels change.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SlotId {
    /// A Feat slot at a character level; `index` counts within the source.
    Feat {
        level: u8,
        source: FeatSource,
        index: u8,
    },
    /// A Feature selection slot from the class level row taken at a character level.
    Selection { level: u8, index: u8 },
    /// The general mythic pick of a Mythic rank: a mythic ability at odd ranks, a mythic feat at even ones (see
    /// [`mythic_pick_kind`](crate::mythic_pick_kind)).
    Mythic { rank: u8 },
    /// A Feat slot from the mythic path's rank table.
    PathFeat { rank: u8, index: u8 },
    /// A Feature selection slot from the mythic path's rank table.
    PathSelection { rank: u8, index: u8 },
    /// A Feature selection slot from the rows of options held, reached at a character level (the Arcane
    /// bloodline's school at option level 15), in the order the level reaches them.
    OptionSelection { level: u8, index: u8 },
    /// A Feature selection slot granted by the general mythic pick of a Mythic rank (Second Bloodline).
    MythicSelection { rank: u8, index: u8 },
    /// A Feat slot from the rows of options held, reached at a character level (the War domain's bonus Combat
    /// feat at option level 8), in the order the level reaches them.
    OptionFeat { level: u8, index: u8 },
}

impl SlotId {
    /// The Mythic rank, for slots that belong to a Mythic ascension.
    pub fn rank(&self) -> Option<u8> {
        match *self {
            SlotId::Feat { .. }
            | SlotId::Selection { .. }
            | SlotId::OptionSelection { .. }
            | SlotId::OptionFeat { .. } => None,
            SlotId::Mythic { rank }
            | SlotId::PathFeat { rank, .. }
            | SlotId::PathSelection { rank, .. }
            | SlotId::MythicSelection { rank, .. } => Some(rank),
        }
    }

    /// The character level, for slots that belong to a Level-up (an option row's slot belongs to the level that
    /// reaches the row).
    pub fn level(&self) -> Option<u8> {
        match *self {
            SlotId::Feat { level, .. }
            | SlotId::Selection { level, .. }
            | SlotId::OptionSelection { level, .. }
            | SlotId::OptionFeat { level, .. } => Some(level),
            _ => None,
        }
    }
}

/// What fills a slot. A pick of a parameterised feat or mythic feat names what it's taken for (Weapon Focus
/// (Longsword)), and so does a pick of an option that takes a value (the rogue talent Weapon Focus).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Pick {
    Feat {
        feat: FeatKey,
        parameter: Option<Parameter>,
    },
    /// An option of a Feature selection.
    SelectionOption {
        option: SelectionOptionKey,
        parameter: Option<Parameter>,
    },
    MythicAbility(MythicAbilityKey),
    /// A mythic feat, with its value when it's parameterised (Improved Critical (Mythic) (Longsword)).
    MythicFeat {
        feat: MythicFeatKey,
        parameter: Option<Parameter>,
    },
}

impl Pick {
    /// A feat taken for nothing in particular.
    pub fn feat(feat: FeatKey) -> Self {
        Pick::Feat {
            feat,
            parameter: None,
        }
    }

    /// A parameterised feat taken for `value`.
    pub fn feat_for(feat: FeatKey, value: Parameter) -> Self {
        Pick::Feat {
            feat,
            parameter: Some(value),
        }
    }

    /// An option with no chosen feat.
    pub fn option(option: SelectionOptionKey) -> Self {
        Pick::SelectionOption {
            option,
            parameter: None,
        }
    }

    /// A mythic feat taken for nothing in particular.
    pub fn mythic_feat(feat: MythicFeatKey) -> Self {
        Pick::MythicFeat {
            feat,
            parameter: None,
        }
    }

    /// A parameterised mythic feat taken for `value`.
    pub fn mythic_feat_for(feat: MythicFeatKey, value: Parameter) -> Self {
        Pick::MythicFeat {
            feat,
            parameter: Some(value),
        }
    }

    /// An option whose chosen feat is taken for `value`.
    pub fn option_for(option: SelectionOptionKey, value: Parameter) -> Self {
        Pick::SelectionOption {
            option,
            parameter: Some(value),
        }
    }

    /// The text key, for display and for "removed from game data" messages.
    pub fn key(&self) -> &str {
        match self {
            Pick::Feat { feat, .. } => feat.as_str(),
            Pick::SelectionOption { option, .. } => option.as_str(),
            Pick::MythicAbility(key) => key.as_str(),
            Pick::MythicFeat { feat, .. } => feat.as_str(),
        }
    }

    /// What the pick is taken for, if anything.
    pub fn parameter(&self) -> Option<&Parameter> {
        match self {
            Pick::Feat { parameter, .. }
            | Pick::SelectionOption { parameter, .. }
            | Pick::MythicFeat { parameter, .. } => parameter.as_ref(),
            Pick::MythicAbility(_) => None,
        }
    }
}

/// One change to a Build, as the sheet makes them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edit {
    /// Fill (or refill) a slot.
    Pick {
        slot: SlotId,
        pick: Pick,
    },
    /// Empty a slot.
    Clear(SlotId),
    /// Set the class taken at a character level. A level one past the end adds a Level-up. `choice` replaces the
    /// class's archetype and advancement choice; `None` keeps whatever the class already has.
    Class {
        level: u8,
        class: ClassKey,
        choice: Option<ClassChoice>,
    },
    AbilityIncrease {
        level: u8,
        ability: Option<Ability>,
    },
    /// Set the ranks added to a skill at a level (0 removes them).
    SkillRanks {
        level: u8,
        skill: SkillKey,
        ranks: u8,
    },
    Start(BuildStart),
    MythicPath(Option<MythicPathKey>),
    PathSwitch(Option<MythicPathKey>),
    /// Remove the last Level-up, with the picks of its slots.
    RemoveLevel,
    SetName(Option<String>),
}

impl Edit {
    /// The choices this edit sets directly. Their own problems are the picker's business, not a would-break.
    pub fn choices(&self) -> Vec<Choice> {
        match self {
            Edit::Pick { slot, .. } | Edit::Clear(slot) => vec![Choice::Slot(slot.clone())],
            Edit::Class {
                level,
                class,
                choice,
            } => {
                let mut choices = vec![Choice::Class { level: *level }];
                if choice.is_some() {
                    choices.push(Choice::Archetype {
                        class: class.clone(),
                    });
                    choices.push(Choice::Advances {
                        class: class.clone(),
                    });
                }
                choices
            }
            Edit::AbilityIncrease { level, .. } => vec![Choice::AbilityIncrease { level: *level }],
            Edit::SkillRanks { level, skill, .. } => vec![
                Choice::SkillRanks {
                    level: *level,
                    skill: skill.clone(),
                },
                Choice::SkillPoints { level: *level },
            ],
            Edit::Start(_) => vec![
                Choice::Race,
                Choice::Subrace,
                Choice::Alignment,
                Choice::AbilityScores,
            ],
            Edit::MythicPath(_) => vec![Choice::MythicPath],
            Edit::PathSwitch(_) => vec![Choice::PathSwitch],
            // Nothing after the last level can depend on it, and a name breaks nothing.
            Edit::RemoveLevel | Edit::SetName(_) => vec![],
        }
    }
}

impl Build {
    /// Applies an edit. Edits naming a level the Build doesn't have (other than the next one, for a class) do
    /// nothing.
    pub fn apply(&mut self, edit: &Edit) {
        match edit {
            Edit::Pick { slot, pick } => {
                self.picks.insert(slot.clone(), pick.clone());
            }
            Edit::Clear(slot) => {
                self.picks.remove(slot);
            }
            Edit::Class {
                level,
                class,
                choice,
            } => {
                let index = usize::from(*level).wrapping_sub(1);
                if index == self.levels.len() {
                    self.levels.push(LevelUp::new(class.clone()));
                }
                let Some(level_up) = self.levels.get_mut(index) else {
                    return;
                };
                level_up.class = class.clone();
                if let Some(choice) = choice {
                    self.class_choices.insert(class.clone(), choice.clone());
                }
                self.drop_unused_class_choices();
            }
            Edit::AbilityIncrease { level, ability } => {
                if let Some(level_up) = self.level_mut(*level) {
                    level_up.ability_increase = *ability;
                }
            }
            Edit::SkillRanks {
                level,
                skill,
                ranks,
            } => {
                if let Some(level_up) = self.level_mut(*level) {
                    if *ranks == 0 {
                        level_up.skill_ranks.remove(skill);
                    } else {
                        level_up.skill_ranks.insert(skill.clone(), *ranks);
                    }
                }
            }
            Edit::Start(start) => self.start = start.clone(),
            Edit::MythicPath(path) => self.mythic_path = path.clone(),
            Edit::PathSwitch(path) => self.path_switch = path.clone(),
            Edit::RemoveLevel => {
                if self.levels.pop().is_some() {
                    // Mythic picks stay: a rank no level reaches is ignored, and comes back with the level.
                    let removed = u8::try_from(self.levels.len() + 1).unwrap_or(u8::MAX);
                    self.picks.retain(|slot, _| slot.level() != Some(removed));
                    self.drop_unused_class_choices();
                }
            }
            Edit::SetName(name) => self.name = name.clone(),
        }
    }

    fn level_mut(&mut self, level: u8) -> Option<&mut LevelUp> {
        self.levels.get_mut(usize::from(level).checked_sub(1)?)
    }

    /// Forgets the choices of classes no level takes any more, so a link carries no dead choices.
    fn drop_unused_class_choices(&mut self) {
        let levels = &self.levels;
        self.class_choices
            .retain(|class, _| levels.iter().any(|level| &level.class == class));
    }
}
