//! What the sheet calls things: names by key, slot labels, picker categories, and the reasons a picker shows for an
//! [`Issue`], in the game's and the glossary's words ("Needs Power Attack", "Needs BAB +4 (have +1)").
//!
//! A key the game data lacks is named by its key: the sheet never holds one (a link naming one opens nothing), but
//! a label should never fail.

use super::{Engine, SlotKind};
use crate::build::{FeatSource, Pick, SlotId};
use crate::choice::{Issue, Unmet};
use crate::engine::Slot;
use crate::game_data::{
    Ability, Alignment, Class, FeatCategory, FeatSlot, MythicPath, SelectionOption, SpellType,
};
use crate::keys::{
    ArchetypeKey, BackgroundKey, ClassKey, FeatKey, FeatListKey, FeatureKey, FeatureSelectionKey,
    MythicAbilityKey, MythicFeatKey, MythicPathKey, RaceKey, SelectionOptionKey, SkillKey,
    SubraceKey, WeaponKey,
};
use crate::parameter::{Parameter, ParameterKind};
use crate::prerequisite::Prerequisite;
use crate::state::CharacterState;

/// What a feature belongs to.
enum Holder<'a> {
    Option(&'a SelectionOption),
    /// A class and the index of one of its archetypes.
    Archetype(&'a Class, usize),
    Class(&'a Class),
    Path(&'a MythicPath),
}

impl Engine {
    pub fn race_name<'a>(&'a self, key: &'a RaceKey) -> &'a str {
        self.race(key).map_or(key.as_str(), |race| &race.name)
    }

    pub fn subrace_name<'a>(&'a self, key: &'a SubraceKey) -> &'a str {
        self.subraces
            .get(key)
            .map_or(key.as_str(), |&(r, s)| &self.data.races[r].subraces[s].name)
    }

    pub fn skill_name<'a>(&'a self, key: &'a SkillKey) -> &'a str {
        self.skills
            .get(key)
            .map_or(key.as_str(), |&index| &self.data.skills[index].name)
    }

    pub fn weapon_name<'a>(&'a self, key: &'a WeaponKey) -> &'a str {
        self.weapon(key).map_or(key.as_str(), |weapon| &weapon.name)
    }

    pub fn feat_name<'a>(&'a self, key: &'a FeatKey) -> &'a str {
        self.feat(key).map_or(key.as_str(), |feat| &feat.name)
    }

    pub fn feat_list_name<'a>(&'a self, key: &'a FeatListKey) -> &'a str {
        self.feat_list(key).map_or(key.as_str(), |list| &list.name)
    }

    pub fn class_name<'a>(&'a self, key: &'a ClassKey) -> &'a str {
        self.class(key)
            .map_or(key.as_str(), |(class, _)| &class.name)
    }

    /// Whether a class is a prestige class.
    pub fn is_prestige(&self, key: &ClassKey) -> bool {
        self.class(key).is_some_and(|(_, prestige)| prestige)
    }

    pub fn archetype_name<'a>(&'a self, key: &'a ArchetypeKey) -> &'a str {
        self.archetypes.get(key).map_or(key.as_str(), |&(c, a)| {
            &self.data.classes[c].archetypes[a].name
        })
    }

    pub fn feature_name<'a>(&'a self, key: &'a FeatureKey) -> &'a str {
        self.feature_names
            .get(key)
            .map_or(key.as_str(), String::as_str)
    }

    /// A feature's name, with whose it is when another feature shares the name: "Weapon Training (Sohei)",
    /// "Animal Companion (Animal Domain)".
    pub fn feature_name_apart(&self, key: &FeatureKey) -> String {
        let name = self.feature_name(key);
        if !self.shared_feature_names.contains(key) {
            return name.to_owned();
        }
        match self.feature_owner(key) {
            Some(owner) => format!("{name} ({owner})"),
            None => name.to_owned(),
        }
    }

    /// The name of what a feature belongs to (`cleric/domain/animal/animal-companion` is the Animal Domain's).
    fn feature_owner(&self, key: &FeatureKey) -> Option<&str> {
        Some(match self.feature_holder(key)? {
            Holder::Option(option) => &option.name,
            Holder::Archetype(class, archetype) => &class.archetypes[archetype].name,
            Holder::Class(class) => &class.name,
            Holder::Path(path) => &path.name,
        })
    }

    /// The nearest option, archetype, class or path a feature's key sits under.
    fn feature_holder(&self, key: &FeatureKey) -> Option<Holder<'_>> {
        let mut prefix = key.as_str();
        while let Some((owner, _)) = prefix.rsplit_once('/') {
            prefix = owner;
            if let Some(option) = self.option(&SelectionOptionKey::new(prefix)) {
                return Some(Holder::Option(option));
            }
            if let Some(&(class, archetype)) = self.archetypes.get(&ArchetypeKey::new(prefix)) {
                return Some(Holder::Archetype(&self.data.classes[class], archetype));
            }
            if let Some((class, _)) = self.class(&ClassKey::new(prefix)) {
                return Some(Holder::Class(class));
            }
            if let Some(path) = self.path(&MythicPathKey::new(prefix)) {
                return Some(Holder::Path(path));
            }
        }
        None
    }

    /// A picker's reason for this character: of each "or", only the alternatives the character's classes give
    /// (Advanced Weapon Training for a fighter needs the Fighter's Weapon Training, not the Sohei's), or every
    /// alternative when none does. The pick is locked either way; this only keeps the reason to the point.
    pub(super) fn reason_for(&self, issue: Issue, state: &CharacterState) -> Issue {
        match issue {
            Issue::Unmet(unmet) => Issue::Unmet(self.unmet_for(unmet, state)),
            issue => issue,
        }
    }

    fn unmet_for(&self, unmet: Unmet, state: &CharacterState) -> Unmet {
        match unmet {
            Unmet::Condition { .. } => unmet,
            Unmet::All(children) => Unmet::All(
                children
                    .into_iter()
                    .map(|child| self.unmet_for(child, state))
                    .collect(),
            ),
            Unmet::AtLeast {
                needed,
                met,
                failed,
            } => Unmet::AtLeast {
                needed,
                met,
                failed: failed
                    .into_iter()
                    .map(|child| self.unmet_for(child, state))
                    .collect(),
            },
            Unmet::Any(children) => {
                let children: Vec<Unmet> = children
                    .into_iter()
                    .map(|child| self.unmet_for(child, state))
                    .collect();
                let (mut reachable, others): (Vec<Unmet>, Vec<Unmet>) = children
                    .into_iter()
                    .partition(|child| self.within_reach(child, state));
                match reachable.len() {
                    0 => Unmet::Any(others),
                    1 => reachable.pop().expect("one"),
                    _ => Unmet::Any(reachable),
                }
            }
        }
    }

    /// Whether the character's classes could give this: a class feature or level of a class it has (with its
    /// archetype, for an archetype's), or anything not tied to a class.
    fn within_reach(&self, unmet: &Unmet, state: &CharacterState) -> bool {
        let has = |class: &ClassKey| state.classes.iter().any(|taken| &taken.class == class);
        let has_archetype = |archetype: &ArchetypeKey| {
            state
                .classes
                .iter()
                .any(|taken| taken.archetype.as_ref() == Some(archetype))
        };
        match unmet {
            Unmet::Condition { needs, .. } => match needs {
                Prerequisite::ClassLevel { class, .. } => has(class),
                Prerequisite::Archetype(archetype) => has_archetype(archetype),
                Prerequisite::Feature { feature, .. } => match self.feature_holder(feature) {
                    Some(Holder::Class(class)) => has(&class.key),
                    Some(Holder::Archetype(class, archetype)) => {
                        has_archetype(&class.archetypes[archetype].key)
                    }
                    // An option's features (a domain's) come with options several classes share.
                    _ => true,
                },
                _ => true,
            },
            Unmet::All(children) => children.iter().all(|child| self.within_reach(child, state)),
            Unmet::Any(children) => children.iter().any(|child| self.within_reach(child, state)),
            Unmet::AtLeast { .. } => true,
        }
    }

    pub fn selection_name<'a>(&'a self, key: &'a FeatureSelectionKey) -> &'a str {
        self.selection(key)
            .map_or(key.as_str(), |selection| &selection.name)
    }

    pub fn option_name<'a>(&'a self, key: &'a SelectionOptionKey) -> &'a str {
        self.option(key).map_or(key.as_str(), |option| &option.name)
    }

    pub fn mythic_ability_name<'a>(&'a self, key: &'a MythicAbilityKey) -> &'a str {
        self.mythic_ability(key)
            .map_or(key.as_str(), |ability| &ability.name)
    }

    pub fn mythic_feat_name<'a>(&'a self, key: &'a MythicFeatKey) -> &'a str {
        self.mythic_feat(key)
            .map_or(key.as_str(), |feat| &feat.name)
    }

    pub fn path_name<'a>(&'a self, key: &'a MythicPathKey) -> &'a str {
        self.path(key).map_or(key.as_str(), |path| &path.name)
    }

    pub fn background_name<'a>(&'a self, key: &'a BackgroundKey) -> &'a str {
        self.background(key)
            .map_or(key.as_str(), |background| &background.name)
    }

    /// A Parameter as the sheet shows it: the skill's or weapon's name, or the school, energy or armour type.
    pub fn parameter_name<'a>(&'a self, value: &'a Parameter) -> &'a str {
        match value {
            Parameter::Skill(key) => self.skill_name(key),
            Parameter::Weapon(key) => self.weapon_name(key),
            Parameter::SpellSchool(school) => school.name(),
            Parameter::Energy(energy) => energy.name(),
            Parameter::ArmourType(armour) => armour.name(),
        }
    }

    /// The name of what a pick takes, without its value: the feat, option, mythic ability or mythic feat.
    pub fn pick_base_name<'a>(&'a self, pick: &'a Pick) -> &'a str {
        match pick {
            Pick::Feat { feat, .. } => self.feat_name(feat),
            Pick::SelectionOption { option, .. } => self.option_name(option),
            Pick::MythicAbility(key) => self.mythic_ability_name(key),
            Pick::MythicFeat { feat, .. } => self.mythic_feat_name(feat),
        }
    }

    /// A pick as the sheet shows it, with its value: "Weapon Focus (Longsword)".
    pub fn pick_name(&self, pick: &Pick) -> String {
        let name = self.pick_base_name(pick);
        match pick.parameter() {
            Some(value) => format!("{name} ({})", self.parameter_name(value)),
            None => name.to_owned(),
        }
    }

    /// The group a picker files a pick under, for its category chips: a feat's category, the Feature selection
    /// that defines an option (a slayer's shared talents are "Rogue Talents"), a mythic pick's guide heading.
    pub fn pick_category(&self, pick: &Pick) -> &str {
        match pick {
            Pick::Feat { feat, .. } => self.feat(feat).map_or("", |feat| feat.category.name()),
            Pick::SelectionOption { option, .. } => self
                .options
                .get(option)
                .map_or("", |(selection, _)| self.selection_name(selection)),
            Pick::MythicAbility(key) => self
                .mythic_ability(key)
                .map_or("", |ability| ability.group.name()),
            Pick::MythicFeat { feat, .. } => {
                self.mythic_feat(feat).map_or("", |feat| feat.group.name())
            }
        }
    }

    /// The short summary of what a pick takes, in our own words; empty when the data has none.
    pub fn pick_summary(&self, pick: &Pick) -> &str {
        match pick {
            Pick::Feat { feat, .. } => self.feat(feat).map_or("", |feat| &feat.summary),
            Pick::SelectionOption { option, .. } => {
                self.option(option).map_or("", |option| &option.summary)
            }
            Pick::MythicAbility(key) => self
                .mythic_ability(key)
                .map_or("", |ability| &ability.summary),
            Pick::MythicFeat { feat, .. } => {
                self.mythic_feat(feat).map_or("", |feat| &feat.summary)
            }
        }
    }

    /// What a slot takes, as its chip says: "Feat" for the general slot, "Bonus feat" for a race's or class's
    /// general bonus slot, "Combat feat", "Teamwork feat", a feat list's or Feature selection's name, "Mythic
    /// ability" or "Mythic feat".
    pub fn slot_label(&self, slot: &Slot) -> String {
        match &slot.kind {
            SlotKind::Selection(key) => self.selection_name(key).to_owned(),
            SlotKind::MythicAbility => "Mythic ability".to_owned(),
            SlotKind::MythicFeat => "Mythic feat".to_owned(),
            SlotKind::Feat(feat_slot) => match feat_slot {
                FeatSlot::General
                    if matches!(
                        slot.id,
                        SlotId::Feat {
                            source: FeatSource::General,
                            ..
                        }
                    ) =>
                {
                    "Feat".to_owned()
                }
                FeatSlot::General => "Bonus feat".to_owned(),
                FeatSlot::CombatFeat => "Combat feat".to_owned(),
                FeatSlot::Category(category) => format!("{} feat", category.name()),
                FeatSlot::OneOf { list } => self.feat_list_name(list).to_owned(),
                FeatSlot::OptionList { selection, .. } => {
                    format!("{} feat", self.selection_name(selection))
                }
            },
        }
    }

    /// Why a picker's option can't be taken, as one line: "Needs Power Attack, BAB +4 (have +1)". `None` for an
    /// option that would break a later choice: the sheet leaves it grey, with no reason.
    pub fn issue_text(&self, issue: &Issue) -> Option<String> {
        Some(match issue {
            Issue::Unmet(unmet) => format!("Needs {}", self.unmet_text(unmet)),
            Issue::AlreadyHeld => "Already taken".to_owned(),
            Issue::MaxTimes { times } => format!("Already taken {times} times"),
            Issue::NotAllowed => "Not allowed here".to_owned(),
            Issue::ClassLevelCap { cap } => format!("All {cap} levels taken"),
            Issue::NotFromPath { from } => {
                let paths: Vec<&str> = from.iter().map(|path| self.path_name(path)).collect();
                format!("Only from {}", paths.join(" or "))
            }
            Issue::BreaksLater => return None,
        })
    }

    /// What an unmet prerequisite needs, without the leading "Needs".
    pub fn unmet_text(&self, unmet: &Unmet) -> String {
        match unmet {
            Unmet::Condition { needs, has } => self.prerequisite_text(needs, *has),
            Unmet::All(children) => children
                .iter()
                .map(|child| self.nested_unmet_text(child))
                .collect::<Vec<_>>()
                .join(", "),
            Unmet::Any(children) => children
                .iter()
                .map(|child| self.nested_unmet_text(child))
                .collect::<Vec<_>>()
                .join(" or "),
            Unmet::AtLeast {
                needed,
                met,
                failed,
            } => {
                let children: Vec<String> = failed
                    .iter()
                    .map(|child| self.nested_unmet_text(child))
                    .collect();
                format!(
                    "{} more of: {}",
                    needed.saturating_sub(*met),
                    children.join(", ")
                )
            }
        }
    }

    /// A group inside another reads in brackets.
    fn nested_unmet_text(&self, unmet: &Unmet) -> String {
        match unmet {
            Unmet::Condition { .. } => self.unmet_text(unmet),
            _ => format!("({})", self.unmet_text(unmet)),
        }
    }

    /// One prerequisite as the sheet says it, with what the character has for a numeric one ("BAB +4 (have
    /// +1)").
    pub fn prerequisite_text(&self, prerequisite: &Prerequisite, has: Option<u8>) -> String {
        let have = |text: String| match has {
            Some(value) => format!("{text} (have {value})"),
            None => text,
        };
        let group = |children: &[Prerequisite], joiner: &str| {
            children
                .iter()
                .map(|child| match child {
                    Prerequisite::All(_) | Prerequisite::Any(_) | Prerequisite::AtLeast(..) => {
                        format!("({})", self.prerequisite_text(child, None))
                    }
                    _ => self.prerequisite_text(child, None),
                })
                .collect::<Vec<_>>()
                .join(joiner)
        };
        match prerequisite {
            Prerequisite::All(children) => group(children, ", "),
            Prerequisite::Any(children) => group(children, " or "),
            Prerequisite::AtLeast(n, children) => format!("{n} of: {}", group(children, ", ")),
            Prerequisite::Not(child) => format!("not {}", self.prerequisite_text(child, None)),
            Prerequisite::Feat(feat) => self.feat_name(feat).to_owned(),
            Prerequisite::FeatFor { feat, value } => {
                format!("{} ({})", self.feat_name(feat), self.parameter_name(value))
            }
            Prerequisite::FeatForSameValue(feat) => {
                let kind = self
                    .feat(feat)
                    .and_then(|feat| feat.parameter)
                    .map_or("value", ParameterKind::noun);
                format!("{} in the same {kind}", self.feat_name(feat))
            }
            Prerequisite::FeatIfValue { value, feat } => format!(
                "{} (for {})",
                self.feat_name(feat),
                self.parameter_name(value)
            ),
            Prerequisite::MythicFeat(feat) => self.mythic_feat_name(feat).to_owned(),
            Prerequisite::MythicAbility(ability) => self.mythic_ability_name(ability).to_owned(),
            Prerequisite::ClassLevel { class, level } => {
                have(format!("{} level {level}", self.class_name(class)))
            }
            Prerequisite::Archetype(archetype) => {
                format!("the {} archetype", self.archetype_name(archetype))
            }
            Prerequisite::Feature { feature, count } if *count <= 1 => {
                self.feature_name_apart(feature)
            }
            Prerequisite::Feature { feature, count } => {
                have(format!("{} ×{count}", self.feature_name_apart(feature)))
            }
            Prerequisite::SelectionOption(option) => self.option_name(option).to_owned(),
            Prerequisite::NoOptionFrom(selection) => {
                format!("no {} yet", self.selection_name(selection))
            }
            Prerequisite::NoOtherOptionOfGroup(group) => {
                format!("no other {} option", group.name())
            }
            Prerequisite::PickedThrough(selection) => {
                format!("taken as {}", self.selection_name(selection))
            }
            Prerequisite::SkillRanks { skill, ranks } => {
                have(format!("{ranks} ranks in {}", self.skill_name(skill)))
            }
            Prerequisite::AbilityScore { ability, score } => {
                have(format!("{} {score}", ability.short_name()))
            }
            Prerequisite::BaseAttackBonus(bab) => match has {
                Some(value) => format!("BAB +{bab} (have +{value})"),
                None => format!("BAB +{bab}"),
            },
            Prerequisite::SpellLevel { spell_type, level } => {
                spell_text(*spell_type, *level, false, has)
            }
            Prerequisite::SpontaneousSpellLevel { spell_type, level } => {
                spell_text(*spell_type, *level, true, has)
            }
            Prerequisite::CasterLevel(level) => have(format!("caster level {level}")),
            Prerequisite::CharacterLevel(level) => have(format!("character level {level}")),
            Prerequisite::MythicRank(rank) => have(format!("Mythic rank {rank}")),
            Prerequisite::Alignment(alignment) => format!("{} alignment", alignment.name()),
            Prerequisite::Race(race) => self.race_name(race).to_owned(),
            Prerequisite::Story(story) => story.name().to_owned(),
        }
    }
}

/// "to cast 3rd-level arcane spells (have 1st)", or "(can't cast any)".
fn spell_text(spell_type: SpellType, level: u8, spontaneous: bool, has: Option<u8>) -> String {
    let spell_type = match spell_type {
        SpellType::Arcane => "arcane",
        SpellType::Divine => "divine",
    };
    let how = if spontaneous { " spontaneously" } else { "" };
    let have = match has {
        Some(0) => " (can't cast any)".to_owned(),
        Some(value) => format!(" (have {})", ordinal(value)),
        None => String::new(),
    };
    format!(
        "to cast {}-level {spell_type} spells{how}{have}",
        ordinal(level)
    )
}

/// 1st, 2nd, 3rd, 4th, …, 11th, 12th, 13th, 21st.
pub fn ordinal(n: u8) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

impl ParameterKind {
    /// What a value of this kind is called in a sentence: "weapon", "skill", "school".
    pub fn noun(self) -> &'static str {
        match self {
            ParameterKind::Skill => "skill",
            ParameterKind::Weapon => "weapon",
            ParameterKind::ExoticWeapon => "exotic weapon",
            ParameterKind::SpellSchool => "school",
            ParameterKind::Energy => "energy type",
            ParameterKind::ArmourType => "armour type",
        }
    }
}

impl Ability {
    pub fn name(self) -> &'static str {
        match self {
            Ability::Strength => "Strength",
            Ability::Dexterity => "Dexterity",
            Ability::Constitution => "Constitution",
            Ability::Intelligence => "Intelligence",
            Ability::Wisdom => "Wisdom",
            Ability::Charisma => "Charisma",
        }
    }

    /// The three-letter abbreviation: Str, Dex, Con, Int, Wis, Cha.
    pub fn short_name(self) -> &'static str {
        match self {
            Ability::Strength => "Str",
            Ability::Dexterity => "Dex",
            Ability::Constitution => "Con",
            Ability::Intelligence => "Int",
            Ability::Wisdom => "Wis",
            Ability::Charisma => "Cha",
        }
    }
}

impl Alignment {
    pub const ALL: [Alignment; 9] = [
        Alignment::LawfulGood,
        Alignment::NeutralGood,
        Alignment::ChaoticGood,
        Alignment::LawfulNeutral,
        Alignment::TrueNeutral,
        Alignment::ChaoticNeutral,
        Alignment::LawfulEvil,
        Alignment::NeutralEvil,
        Alignment::ChaoticEvil,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Alignment::LawfulGood => "Lawful Good",
            Alignment::NeutralGood => "Neutral Good",
            Alignment::ChaoticGood => "Chaotic Good",
            Alignment::LawfulNeutral => "Lawful Neutral",
            Alignment::TrueNeutral => "True Neutral",
            Alignment::ChaoticNeutral => "Chaotic Neutral",
            Alignment::LawfulEvil => "Lawful Evil",
            Alignment::NeutralEvil => "Neutral Evil",
            Alignment::ChaoticEvil => "Chaotic Evil",
        }
    }

    /// The two-letter abbreviation: LG, NG, …, TN, …, CE.
    pub fn short_name(self) -> &'static str {
        match self {
            Alignment::LawfulGood => "LG",
            Alignment::NeutralGood => "NG",
            Alignment::ChaoticGood => "CG",
            Alignment::LawfulNeutral => "LN",
            Alignment::TrueNeutral => "TN",
            Alignment::ChaoticNeutral => "CN",
            Alignment::LawfulEvil => "LE",
            Alignment::NeutralEvil => "NE",
            Alignment::ChaoticEvil => "CE",
        }
    }
}

impl FeatCategory {
    /// The guide's group heading on the feats page.
    pub fn name(self) -> &'static str {
        match self {
            FeatCategory::Weapon => "Weapon",
            FeatCategory::Ranged => "Ranged",
            FeatCategory::UnarmedCombat => "Unarmed Combat",
            FeatCategory::ArmourAndShield => "Armour and Shield",
            FeatCategory::Magic => "Magic",
            FeatCategory::Miscellaneous => "Miscellaneous",
            FeatCategory::MountedCombat => "Mounted Combat",
            FeatCategory::CombatManoeuvres => "Combat Manoeuvres",
            FeatCategory::Teamwork => "Teamwork",
            FeatCategory::Skill => "Skill",
            FeatCategory::Class => "Class",
            FeatCategory::Racial => "Racial",
        }
    }
}
