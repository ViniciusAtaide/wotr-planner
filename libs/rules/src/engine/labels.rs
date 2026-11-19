//! What the sheet calls things: names by key, slot labels, the deity picker's domain names, and the reasons a picker
//! shows for an [`Issue`], in the game's and the glossary's words ("Needs Power Attack", "Needs BAB +4 (have +1)").
//!
//! A negated condition that fails says what rules the option out ("Not for the Kinetic Sharpshooter archetype",
//! "Not with Beast Totem, Lesser", "Only below Slayer level 6"), never "Needs not …".
//!
//! A key the game data lacks is named by its key: the sheet never holds one (a link naming one opens nothing), but
//! a label should never fail.

use std::collections::BTreeSet;

use super::catalog::GrantedBy;
use super::{Engine, SlotKind};
use crate::build::{Faith, FeatSource, Pick, SlotId};
use crate::choice::{Issue, LaterPick, Unmet};
use crate::component::Fact;
use crate::engine::Slot;
use crate::game_data::{
    Ability, Alignment, Archetype, BASIC_FEAT_SELECTION, Class, DeityDomains, FeatCategory,
    MythicPath, SelectionOption, SpellType,
};
use crate::keys::{
    ArchetypeKey, BackgroundKey, BlueprintGuid, ClassKey, DeityKey, FeatKey, FeatureKey,
    FeatureSelectionKey, MythicAbilityKey, MythicFeatKey, MythicPathKey, RaceKey,
    SelectionOptionKey, SkillKey, SubraceKey, WeaponKey,
};
use crate::parameter::{Parameter, ParameterKind};
use crate::prerequisite::Prerequisite;
use crate::state::CharacterState;

/// What a feature belongs to.
enum Holder<'a> {
    Option(&'a SelectionOption),
    Archetype(&'a Archetype),
    Class(&'a Class),
    Path(&'a MythicPath),
}

/// A holder's name.
fn holder_name(holder: Holder<'_>) -> &str {
    match holder {
        Holder::Option(option) => &option.name,
        Holder::Archetype(archetype) => &archetype.name,
        Holder::Class(class) => &class.name,
        Holder::Path(path) => &path.name,
    }
}

/// How a negated condition's held side reads: "for the X archetype", "with Power Attack", "as Domain (Separatist)",
/// "from Slayer level 6". Refused, the first three read "Not …", the last "Only below …".
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lead {
    For,
    With,
    As,
    From,
}

impl Lead {
    fn word(self) -> &'static str {
        match self {
            Lead::For => "for",
            Lead::With => "with",
            Lead::As => "as",
            Lead::From => "from",
        }
    }

    /// The refusal of these, lower case: "not with A or B", "below A and B".
    fn refusal(self, objects: &[String]) -> String {
        match self {
            Lead::From => format!("below {}", objects.join(" and ")),
            lead => format!("not {} {}", lead.word(), objects.join(" or ")),
        }
    }

    /// The refusal as a clause of a reason, lower case: "not with A or B", "only below A and B".
    fn clause(self, objects: &[String]) -> String {
        match self {
            Lead::From => format!("only {}", self.refusal(objects)),
            lead => lead.refusal(objects),
        }
    }
}

/// The condition a failing `Not` holds, when the unmet condition is one.
fn negated(unmet: &Unmet) -> Option<&Prerequisite> {
    match unmet {
        Unmet::Condition {
            needs: Prerequisite::Not(held),
            ..
        } => Some(held),
        _ => None,
    }
}

/// The first letter in upper case.
fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

impl Engine {
    pub fn race_name<'a>(&'a self, key: &'a RaceKey) -> &'a str {
        self.catalog
            .race(key)
            .map_or(key.as_str(), |race| &race.name)
    }

    pub fn subrace_name<'a>(&'a self, key: &'a SubraceKey) -> &'a str {
        self.catalog
            .subrace(key)
            .map_or(key.as_str(), |(_, subrace)| &subrace.name)
    }

    pub fn skill_name<'a>(&'a self, key: &'a SkillKey) -> &'a str {
        self.catalog
            .skill(key)
            .map_or(key.as_str(), |skill| &skill.name)
    }

    pub fn weapon_name<'a>(&'a self, key: &'a WeaponKey) -> &'a str {
        self.catalog
            .weapon(key)
            .map_or(key.as_str(), |weapon| &weapon.name)
    }

    pub fn feat_name<'a>(&'a self, key: &'a FeatKey) -> &'a str {
        self.catalog
            .feat(key)
            .map_or(key.as_str(), |feat| &feat.name)
    }

    pub fn class_name<'a>(&'a self, key: &'a ClassKey) -> &'a str {
        self.catalog
            .class(key)
            .map_or(key.as_str(), |(class, _)| &class.name)
    }

    /// A class as a class choice's value names it: "Fighter", "Armiger (Fighter)", and with `advances` (a prestige
    /// class that asks which classes it advances) "Eldritch Knight, advancing Wizard" or "Eldritch Knight, advancing
    /// nothing".
    pub fn class_value_name(
        &self,
        class: &ClassKey,
        archetype: Option<&ArchetypeKey>,
        advances: Option<&BTreeSet<ClassKey>>,
    ) -> String {
        let name = match archetype {
            Some(archetype) => format!(
                "{} ({})",
                self.archetype_name(archetype),
                self.class_name(class)
            ),
            None => self.class_name(class).to_owned(),
        };
        match advances {
            Some(advances) => name + &self.advancing(advances),
            None => name,
        }
    }

    /// What a prestige class's levels advance, after its name: ", advancing Cleric and Wizard", or ", advancing
    /// nothing".
    pub fn advancing(&self, advances: &BTreeSet<ClassKey>) -> String {
        if advances.is_empty() {
            return ", advancing nothing".to_owned();
        }
        let names: Vec<&str> = advances.iter().map(|key| self.class_name(key)).collect();
        format!(", advancing {}", names.join(" and "))
    }

    /// Whether a class is a prestige class.
    pub fn is_prestige(&self, key: &ClassKey) -> bool {
        self.catalog
            .class(key)
            .is_some_and(|(_, prestige)| prestige)
    }

    pub fn archetype_name<'a>(&'a self, key: &'a ArchetypeKey) -> &'a str {
        self.catalog
            .archetype(key)
            .map_or(key.as_str(), |archetype| &archetype.name)
    }

    pub fn feature_name<'a>(&'a self, key: &'a FeatureKey) -> &'a str {
        self.catalog.feature_name(key).unwrap_or(key.as_str())
    }

    /// A feature's name, with whose it is when another feature shares the name: "Weapon Training (Sohei)",
    /// "Animal Companion (Animal Domain)". Without a Build, whose it is only when one holder grants it (a class with
    /// its archetypes counts as the class): a feature several classes grant (Uncanny Dodge) is named alone, never for
    /// a holder the reader may not have.
    pub fn feature_name_apart(&self, key: &FeatureKey) -> String {
        self.feature_name_held(key, None)
    }

    /// [`Self::feature_name_apart`] for a Build at `state`: whose it is, of the classes the Build has taken far enough
    /// that their rows grant it, its path and the options it holds; as without a Build when none of them grants it.
    /// A Rogue's Uncanny Dodge reads "(Rogue)", a Barbarian's "(Barbarian)" (ticket 244, review).
    pub fn feature_name_for(&self, key: &FeatureKey, state: &CharacterState) -> String {
        self.feature_name_held(key, Some(state))
    }

    fn feature_name_held(&self, key: &FeatureKey, state: Option<&CharacterState>) -> String {
        let name = self.feature_name(key);
        if !self.catalog.shares_name(key) {
            return name.to_owned();
        }
        let owner = state
            .and_then(|state| self.held_holder(key, state))
            .or_else(|| self.sole_holder(key));
        match owner {
            Some(owner) => format!("{name} ({})", holder_name(owner)),
            None => name.to_owned(),
        }
    }

    /// What grants a feature to the Build at `state`: the first class taken whose rows up to its level grant it (its
    /// archetype when the class's own rows don't), else the path or an option held whose rows grant it.
    fn held_holder(&self, key: &FeatureKey, state: &CharacterState) -> Option<Holder<'_>> {
        let by = self.catalog.feature_holders(key);
        for taken in &state.classes {
            let Some((class, _)) = self.catalog.class(&taken.class) else {
                continue;
            };
            let archetype = self.catalog.archetype_of(class, taken.archetype.as_ref());
            let reached = (self.catalog.rows(class, archetype))
                .take_while(|row| row.level <= taken.level)
                .any(|row| row.features.contains(key));
            if !reached {
                continue;
            }
            return Some(match archetype {
                Some(archetype) if !by.contains(&GrantedBy::Class(class.key.clone())) => {
                    Holder::Archetype(archetype)
                }
                _ => Holder::Class(class),
            });
        }
        self.holders(by).find(|holder| match holder {
            Holder::Path(path) => state.mythic_path.as_ref() == Some(&path.key),
            Holder::Option(option) => state.option_count(&option.key) > 0,
            Holder::Class(_) | Holder::Archetype(_) => false,
        })
    }

    /// A feature's one holder: the only one, or the class whose archetypes are the others; `None` when several
    /// classes, paths or options grant it.
    fn sole_holder(&self, key: &FeatureKey) -> Option<Holder<'_>> {
        let holders: Vec<Holder<'_>> = self.feature_holders(key).collect();
        fn class_of<'a>(holder: &Holder<'a>) -> Option<&'a ClassKey> {
            match holder {
                Holder::Class(class) => Some(&class.key),
                Holder::Archetype(archetype) => Some(&archetype.class),
                Holder::Path(_) | Holder::Option(_) => None,
            }
        }
        match holders.as_slice() {
            [] => None,
            [_] => holders.into_iter().next(),
            [first, rest @ ..] => {
                let class = class_of(first)?;
                rest.iter()
                    .all(|holder| class_of(holder) == Some(class))
                    .then(|| {
                        self.catalog
                            .class(class)
                            .map(|(class, _)| Holder::Class(class))
                    })
                    .flatten()
            }
        }
    }

    /// The name of what names a Feature selection in its rows: the first of its holders, a class before an
    /// archetype, a path and an option.
    fn selection_owner(&self, key: &FeatureSelectionKey) -> Option<&str> {
        self.holders(self.catalog.selection_holders(key))
            .next()
            .map(holder_name)
    }

    /// The classes, archetypes, paths and options whose rows grant a feature, in that order.
    fn feature_holders(&self, key: &FeatureKey) -> impl Iterator<Item = Holder<'_>> {
        self.holders(self.catalog.feature_holders(key))
    }

    fn holders<'a>(&'a self, by: &'a [GrantedBy]) -> impl Iterator<Item = Holder<'a>> {
        by.iter().filter_map(|by| match by {
            GrantedBy::Class(class) => self
                .catalog
                .class(class)
                .map(|(class, _)| Holder::Class(class)),
            GrantedBy::Archetype(archetype) => {
                self.catalog.archetype(archetype).map(Holder::Archetype)
            }
            GrantedBy::Path(path) => self.catalog.path(path).map(Holder::Path),
            GrantedBy::Option(option) => self.catalog.option(option).map(Holder::Option),
        })
    }

    /// A picker's reason for this character: of each "or", only the alternatives the character's classes give
    /// (Advanced Weapon Training for a fighter needs the Fighter's Weapon Training, not the Sohei's), or every
    /// alternative when none does. The pick is locked either way; this only keeps the reason to the point.
    pub fn reason_for(&self, issue: Issue, state: &CharacterState) -> Issue {
        match issue {
            Issue::Unmet(unmet) => Issue::Unmet(self.unmet_for(unmet, state)),
            issue => issue,
        }
    }

    fn unmet_for(&self, unmet: Unmet, state: &CharacterState) -> Unmet {
        match unmet {
            Unmet::Condition { .. } => unmet,
            // An "or" left with one alternative may be an "and", which joins this one.
            Unmet::All(children) => Unmet::all(
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
            Unmet::MetLater { unmet, later } => Unmet::MetLater {
                unmet: Box::new(self.unmet_for(*unmet, state)),
                later,
            },
            Unmet::Any(children) => {
                let children: Vec<Unmet> = children
                    .into_iter()
                    .map(|child| self.unmet_for(child, state))
                    .collect();
                let (reachable, others): (Vec<Unmet>, Vec<Unmet>) = children
                    .into_iter()
                    .partition(|child| self.within_reach(child, state));
                Unmet::any(if reachable.is_empty() {
                    others
                } else {
                    reachable
                })
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
                // A class's or an archetype's feature is within reach for a character with one of the classes or
                // archetypes whose rows grant it; a path's or an option's (a domain's, which options several classes
                // share), or one no row grants, always is.
                Prerequisite::Feature { feature, .. } => {
                    let mut holders = self.feature_holders(feature).peekable();
                    holders.peek().is_none()
                        || holders.any(|holder| match holder {
                            Holder::Class(class) => has(&class.key),
                            Holder::Archetype(archetype) => has_archetype(&archetype.key),
                            Holder::Path(_) | Holder::Option(_) => true,
                        })
                }
                _ => true,
            },
            Unmet::All(children) => children.iter().all(|child| self.within_reach(child, state)),
            Unmet::Any(children) => children.iter().any(|child| self.within_reach(child, state)),
            Unmet::AtLeast { .. } => true,
            Unmet::MetLater { unmet, .. } => self.within_reach(unmet, state),
        }
    }

    pub fn selection_name<'a>(&'a self, key: &'a FeatureSelectionKey) -> &'a str {
        self.catalog
            .selection(key)
            .map_or(key.as_str(), |selection| &selection.name)
    }

    pub fn option_name<'a>(&'a self, key: &'a SelectionOptionKey) -> &'a str {
        self.catalog
            .option(key)
            .map_or(key.as_str(), |option| &option.name)
    }

    pub fn mythic_ability_name<'a>(&'a self, key: &'a MythicAbilityKey) -> &'a str {
        self.catalog
            .mythic_ability(key)
            .map_or(key.as_str(), |ability| &ability.name)
    }

    pub fn mythic_feat_name<'a>(&'a self, key: &'a MythicFeatKey) -> &'a str {
        self.catalog
            .mythic_feat(key)
            .map_or(key.as_str(), |feat| &feat.name)
    }

    /// A fact's name, as its kind's own name helper gives it.
    pub fn fact_name<'a>(&'a self, fact: &'a Fact) -> &'a str {
        match fact {
            Fact::Feat(key) => self.feat_name(key),
            Fact::Feature(key) => self.feature_name(key),
            Fact::Option(key) => self.option_name(key),
            Fact::Selection(key) => self.selection_name(key),
            Fact::MythicAbility(key) => self.mythic_ability_name(key),
            Fact::MythicFeat(key) => self.mythic_feat_name(key),
            Fact::Class(key) => self.class_name(key),
            Fact::Archetype(key) => self.archetype_name(key),
            Fact::Race(key) => self.race_name(key),
            Fact::Deity(key) => self.deity_name(key),
        }
    }

    pub fn path_name<'a>(&'a self, key: &'a MythicPathKey) -> &'a str {
        self.catalog
            .path(key)
            .map_or(key.as_str(), |path| &path.name)
    }

    pub fn background_name<'a>(&'a self, key: &'a BackgroundKey) -> &'a str {
        self.catalog
            .background(key)
            .map_or(key.as_str(), |background| &background.name)
    }

    /// What the guide says about how an item combines with others (`interactions.ron`, ticket 124); `None` for an
    /// item without a note.
    pub fn interactions(&self, item: &crate::bonus::BonusItem) -> Option<&str> {
        self.catalog
            .data()
            .interactions
            .iter()
            .find(|note| &note.item == item)
            .map(|note| note.interactions.as_str())
    }

    pub fn deity_name<'a>(&'a self, key: &'a DeityKey) -> &'a str {
        self.catalog
            .deity(key)
            .map_or(key.as_str(), |deity| &deity.name)
    }

    /// A faith as the sheet shows it: the deity's name, or "Atheism".
    pub fn faith_name<'a>(&'a self, faith: &'a Faith) -> &'a str {
        match faith {
            Faith::Deity(key) => self.deity_name(key),
            Faith::Atheism => "Atheism",
        }
    }

    /// A background's note (what else it gives, in our own words); empty when it has none.
    pub fn background_note(&self, key: &BackgroundKey) -> &str {
        self.catalog
            .background(key)
            .map_or("", |background| &background.note)
    }

    /// A class's short summary, base or prestige, in our own words; empty when the data has none.
    pub fn class_summary(&self, key: &ClassKey) -> &str {
        self.catalog
            .class(key)
            .map_or("", |(class, _)| &class.summary)
    }

    /// An archetype's short summary, in our own words; empty when the data has none.
    pub fn archetype_summary(&self, key: &ArchetypeKey) -> &str {
        self.catalog
            .archetype(key)
            .map_or("", |archetype| &archetype.summary)
    }

    /// A path's short summary, in our own words; empty when the data has none.
    pub fn path_summary(&self, key: &MythicPathKey) -> &str {
        self.catalog.path(key).map_or("", |path| &path.summary)
    }

    /// A first ascension's option as its picker names it: its path-linked ability (Bit of Fun), the features of its
    /// path's MR 1 row that not every option's row has by name (each has Bypass Epic Damage Reduction). The path's
    /// name when nothing sets the row apart.
    pub fn first_ascension_name(&self, key: &MythicPathKey) -> String {
        let row_names = |path: &MythicPathKey| -> Vec<&str> {
            self.path_rank_features(path, super::FIRST_ASCENSION_RANK)
                .into_iter()
                .map(|feature| self.feature_name(feature))
                .collect()
        };
        let options: Vec<Vec<&str>> = self
            .catalog
            .data()
            .mythic_paths
            .iter()
            .filter(|path| self.first_ascension_option(&path.key).is_some())
            .map(|path| row_names(&path.key))
            .collect();
        let own: Vec<&str> = row_names(key)
            .into_iter()
            .filter(|name| !options.iter().all(|names| names.contains(name)))
            .collect();
        if own.is_empty() {
            self.path_name(key).to_owned()
        } else {
            own.join(" and ")
        }
    }

    /// The features a rank's path row gives, in the row's order: the row of the path the rank shows
    /// ([`RankSheet::path`](super::RankSheet::path)), so MR 1 and 2 give the first ascension's. None at a rank not
    /// reached.
    pub fn rank_features(&self, rank: &super::RankSheet) -> Vec<&FeatureKey> {
        if rank.status == super::RankStatus::NotReached {
            return Vec::new();
        }
        rank.path
            .as_ref()
            .map(|key| self.path_rank_features(key, rank.rank))
            .unwrap_or_default()
    }

    /// The features a path's row at a rank gives, in the row's order (the first ascension picker's follow-ups at
    /// MR 2).
    pub fn path_rank_features(&self, key: &MythicPathKey, rank: u8) -> Vec<&FeatureKey> {
        self.catalog
            .path(key)
            .and_then(|path| path.row(rank))
            .map(|row| row.features.iter().collect())
            .unwrap_or_default()
    }

    /// A Parameter as the sheet shows it: the skill's or weapon's name, or the school, energy or armour type.
    pub fn parameter_name<'a>(&'a self, value: &'a Parameter) -> &'a str {
        match value {
            Parameter::Skill(key) => self.skill_name(key),
            Parameter::Weapon(key) => self.weapon_name(key),
            Parameter::SpellSchool(school) => school.name(),
            Parameter::Energy(energy) => energy.name(),
            Parameter::ArmourType(armour) => armour.name(),
            // "Persistent Spell" for Metamagic (Persistent Spell).
            Parameter::Metamagic(key) => self.metamagic_name(key),
            Parameter::Stat(stat) => stat.name(),
            Parameter::Element(element) => element.name(),
        }
    }

    /// A metamagic feat's name as a value: what its "Metamagic (…)" name has in brackets ("Persistent Spell"), or
    /// the whole name (or key) when it has no brackets.
    fn metamagic_name<'a>(&'a self, key: &'a FeatKey) -> &'a str {
        let name = self.feat_name(key);
        name.strip_prefix("Metamagic (")
            .and_then(|rest| rest.strip_suffix(')'))
            .unwrap_or(name)
    }

    /// The name of what a pick takes, without its value: the feat, option, mythic ability or mythic feat.
    fn pick_base_name<'a>(&'a self, pick: &'a Pick) -> &'a str {
        self.catalog
            .pick_item(pick)
            .map_or(pick.key(), |item| item.name)
    }

    /// A pick as the sheet shows it, with its value: "Weapon Focus (Longsword)".
    pub fn pick_name(&self, pick: &Pick) -> String {
        let name = self.pick_base_name(pick);
        match pick.parameter() {
            Some(value) => format!("{name} ({})", self.parameter_name(value)),
            None => name.to_owned(),
        }
    }

    /// The category of the feat a pick takes, which picks its icon: a feat's own, or the chosen feat's for an option
    /// granting one (a rogue talent's Weapon Focus is a Weapon feat). `None` for any other pick, mythic ones
    /// included.
    pub fn feat_category(&self, pick: &Pick) -> Option<FeatCategory> {
        self.catalog.pick_item(pick)?.feat_category
    }

    /// A domain as the deity picker lists it: its option's name without " Domain" or " Subdomain" ("Fire" for the
    /// Fire Domain).
    pub fn domain_name<'a>(&'a self, key: &'a SelectionOptionKey) -> &'a str {
        let name = self.option_name(key);
        name.strip_suffix(" Domain")
            .or_else(|| name.strip_suffix(" Subdomain"))
            .unwrap_or(name)
    }

    /// What a slot takes, as its chip says: "Feat" for the general slot and Extra Feat's, "Bonus feat" for a race's
    /// or class's general bonus slot, a selection's name ("Combat feat", "Teamwork feat", a bonus list's, an option
    /// list slot's "Wild talent feat"; Extra Mythic Ability's "Mythic ability"), "Mythic ability" or "Mythic feat".
    pub fn slot_label(&self, slot: &Slot) -> String {
        match &slot.kind {
            SlotKind::Selection(key) | SlotKind::MythicAbilities(key) => {
                self.selection_name(key).to_owned()
            }
            SlotKind::MythicAbility => "Mythic ability".to_owned(),
            SlotKind::MythicFeat => "Mythic feat".to_owned(),
            // The general slot and Extra Feat's read the general selection's own name, "Feat".
            SlotKind::Feat(key) => match slot.id {
                SlotId::Feat {
                    source: FeatSource::General,
                    ..
                }
                | SlotId::MythicExtra { .. } => self.selection_name(key).to_owned(),
                _ => self.feat_slot_label(key),
            },
        }
    }

    /// What a race's, class row's, archetype row's or path row's selection of feats takes, as
    /// [`slot_label`](Self::slot_label) says for the slot it makes: the selection's name, but "Bonus feat" for the
    /// general one ([`BASIC_FEAT_SELECTION`]) there.
    pub fn feat_slot_label(&self, key: &FeatureSelectionKey) -> String {
        if key.as_str() == BASIC_FEAT_SELECTION {
            "Bonus feat".to_owned()
        } else {
            self.selection_name(key).to_owned()
        }
    }

    /// Why a picker's option can't be taken, as one line: "Needs Power Attack, BAB +4 (have +1)". `None` for an
    /// option that would break a later choice: the sheet leaves it grey, with no reason.
    pub fn issue_text(&self, issue: &Issue) -> Option<String> {
        Some(match issue {
            Issue::Unmet(unmet) => self.unmet_reason(unmet),
            Issue::AlreadyHeld => "Already taken".to_owned(),
            Issue::MaxTimes { times } => format!("Already taken {times} times"),
            Issue::NoOptionLeft(selection) => {
                format!("No {} left to take", self.selection_name(selection))
            }
            Issue::NotAllowed => "Not allowed here".to_owned(),
            Issue::UnknownKey => "Not in the game data".to_owned(),
            Issue::NotReached => "Not reached: the Build doesn't get this far".to_owned(),
            Issue::ClassLevelCap { cap } => format!("All {cap} levels taken"),
            Issue::NotFromPath { from } => {
                let paths: Vec<&str> = from.iter().map(|path| self.path_name(path)).collect();
                format!("Only from {}", paths.join(" or "))
            }
            Issue::DeityAlignment { deity } => {
                let alignments: Vec<&str> = self
                    .catalog
                    .deity(deity)
                    .map(|deity| deity.alignments.iter().map(|a| a.name()).collect())
                    .unwrap_or_default();
                format!("{} allows {}", self.deity_name(deity), or_list(&alignments))
            }
            Issue::NeedsFaith => "Choose a deity or atheism first".to_owned(),
            Issue::DeityDomain { deity, limit } => match limit {
                DeityDomains::Offered => format!("Not a domain of {}", self.deity_name(deity)),
                DeityDomains::NotOffered => format!("A domain of {}", self.deity_name(deity)),
            },
            Issue::NotProficient { weapon } => {
                format!("Not proficient with {}", self.weapon_name(weapon))
            }
            Issue::BreaksLater => return None,
        })
    }

    /// An unmet prerequisite as a reason. What rules the option out leads, each kind once ("Not for the Kinetic
    /// Sharpshooter archetype; not with A or B; needs …"); an "or" whose negated branches fail reads as the case its
    /// other branches answer ("As Domain (Separatist), needs Lawful Good alignment or …").
    fn unmet_reason(&self, unmet: &Unmet) -> String {
        if let Unmet::MetLater { unmet, later } = unmet {
            return format!("{}: {}", self.unmet_reason(unmet), self.later_text(later));
        }
        if let Some(reason) = self.one_case_reason(unmet) {
            return reason;
        }
        if let Unmet::Any(children) = unmet
            && children.iter().any(|child| negated(child).is_some())
        {
            let (held, needed): (Vec<&Unmet>, Vec<&Unmet>) =
                children.iter().partition(|child| negated(child).is_some());
            let held: Vec<(Lead, String)> = held
                .into_iter()
                .filter_map(negated)
                .map(|held| self.held_text(held))
                .collect();
            let cases: Vec<String> = held
                .iter()
                .map(|(lead, object)| format!("{} {object}", lead.word()))
                .collect();
            return match (held.as_slice(), needed.as_slice()) {
                ([(lead, object)], []) => capitalized(&lead.clause(std::slice::from_ref(object))),
                (_, []) => format!("Not {} together", cases.join(" and ")),
                _ => format!(
                    "{}, needs {}",
                    capitalized(&cases.join(" and ")),
                    needed
                        .iter()
                        .map(|child| self.nested_unmet_text(child))
                        .collect::<Vec<_>>()
                        .join(" or ")
                ),
            };
        }
        let (refused, needed): (Vec<&Unmet>, Vec<&Unmet>) = unmet
            .and_parts()
            .into_iter()
            .partition(|child| negated(child).is_some());
        if refused.is_empty() {
            return format!("Needs {}", self.unmet_text(unmet));
        }
        // Each kind once, in the order they come.
        let mut leads: Vec<(Lead, Vec<String>)> = Vec::new();
        for held in refused.into_iter().filter_map(negated) {
            let (lead, object) = self.held_text(held);
            match leads.iter_mut().find(|(seen, _)| *seen == lead) {
                Some((_, objects)) => objects.push(object),
                None => leads.push((lead, vec![object])),
            }
        }
        let mut clauses: Vec<String> = leads
            .iter()
            .map(|(lead, objects)| lead.clause(objects))
            .collect();
        match needed.as_slice() {
            [] => {}
            [one] => clauses.push(format!("needs {}", self.unmet_text(one))),
            more => clauses.push(format!(
                "needs {}",
                distinct(
                    more.iter()
                        .map(|child| self.nested_unmet_text(child))
                        .collect()
                )
                .join(", ")
            )),
        }
        capitalized(&clauses.join("; "))
    }

    /// Where the later pick that would meet a prerequisite is, and what to do (ticket 126): "Spell Focus (Enchantment)
    /// is in level 1's Feat slot, which comes after this slot in the level's pick order; swap them", or for a Mythic
    /// rank's slot "Perception II is in Mythic rank 4's Improved Mythic Trick slot, which comes after this slot in level
    /// 11's pick order".
    fn later_text(&self, later: &LaterPick) -> String {
        let slot = Slot {
            id: later.slot.clone(),
            kind: later.kind.clone(),
            pick: Some(later.pick.clone()),
        };
        let (place, order) = match later.slot.rank() {
            Some(rank) => (
                format!("Mythic rank {rank}'s"),
                format!("level {}'s", later.level),
            ),
            None => (format!("level {}'s", later.level), "the level's".to_owned()),
        };
        let swap = if later.swap { "; swap them" } else { "" };
        format!(
            "{} is in {place} {} slot, which comes after this slot in {order} pick order{swap}",
            self.pick_name(&later.pick),
            self.slot_label(&slot)
        )
    }

    /// An "and" of "or"s that each name the same negated case and something else, read as that case once: "As
    /// Chosen Alignment Blessing, needs Lawful Evil alignment or …, and a deity offering the Evil Domain" (the Champion
    /// of the Faith's alignment blessing, ticket 86). `None` for anything else.
    fn one_case_reason(&self, unmet: &Unmet) -> Option<String> {
        let Unmet::All(children) = unmet else {
            return None;
        };
        let mut case: Option<Vec<String>> = None;
        let mut needs = Vec::new();
        for child in children {
            let Unmet::Any(alternatives) = child else {
                return None;
            };
            let (held, needed): (Vec<&Unmet>, Vec<&Unmet>) = alternatives
                .iter()
                .partition(|alternative| negated(alternative).is_some());
            if held.is_empty() || needed.is_empty() {
                return None;
            }
            let cases: Vec<String> = held
                .into_iter()
                .filter_map(negated)
                .map(|held| {
                    let (lead, object) = self.held_text(held);
                    format!("{} {object}", lead.word())
                })
                .collect();
            if case.get_or_insert_with(|| cases.clone()) != &cases {
                return None;
            }
            needs.push(
                needed
                    .iter()
                    .map(|need| self.nested_unmet_text(need))
                    .collect::<Vec<_>>()
                    .join(" or "),
            );
        }
        Some(format!(
            "{}, needs {}",
            capitalized(&case?.join(" and ")),
            needs.join(", and ")
        ))
    }

    /// What a negated condition rules out, as its lead and object: (For, "the Kinetic Sharpshooter archetype"),
    /// (With, "Power Attack"), (As, "Domain (Separatist)"), (From, "Slayer level 6").
    fn held_text(&self, held: &Prerequisite) -> (Lead, String) {
        let text = || self.prerequisite_text(held, None);
        match held {
            Prerequisite::Archetype(_) => (Lead::For, text()),
            Prerequisite::Race(race) => (Lead::For, format!("the {} race", self.race_name(race))),
            Prerequisite::Alignment(alignment) => {
                (Lead::For, format!("a {} character", alignment.name()))
            }
            Prerequisite::SpellLevel { spell_type, level } => (
                Lead::For,
                format!("a caster of {}", spell_text(*spell_type, *level)),
            ),
            Prerequisite::SpontaneousSpellLevel { spell_type, level } => (
                Lead::For,
                format!(
                    "a spontaneous caster of {}",
                    spell_text(*spell_type, *level)
                ),
            ),
            Prerequisite::ClassSpellLevel { class, level } => (
                Lead::For,
                format!(
                    "a caster of {}-level {} spells",
                    ordinal(*level),
                    self.class_name(class)
                ),
            ),
            Prerequisite::PickedThrough(selection) => {
                (Lead::As, self.selection_name(selection).to_owned())
            }
            Prerequisite::NoOptionFrom(selection) => (
                Lead::With,
                format!("a {} pick", self.selection_name(selection)),
            ),
            Prerequisite::NoOtherOptionOfGroup(group) => {
                (Lead::With, format!("another {} option", group.name()))
            }
            // Any of what the character holds, each named once: the game's own NoFeature of a blueprint several
            // entries share (Second Mystery's, ticket 308d) reads "with Second Mystery", not "with (A or A)"; a
            // selection the planner keeps beside an option of its blueprint is that option (Pentamic Faith's, ticket
            // 329: "with Pentamic Faith", not "with Pentamic Faith or the Pentamic Faith selection").
            Prerequisite::Any(members)
                if members
                    .iter()
                    .all(|member| self.held_text(member).0 == Lead::With) =>
            {
                let options: Vec<&BlueprintGuid> = (members.iter())
                    .filter_map(|member| match member {
                        Prerequisite::SelectionOption(option) => {
                            self.catalog.option(option)?.blueprint.as_ref()
                        }
                        _ => None,
                    })
                    .collect();
                let mut names: Vec<String> = Vec::new();
                for member in members {
                    if let Prerequisite::Selection(selection) = member
                        && (self.catalog.selection(selection))
                            .and_then(|selection| selection.blueprint.as_ref())
                            .is_some_and(|blueprint| options.contains(&blueprint))
                    {
                        continue;
                    }
                    let name = self.held_text(member).1;
                    if !names.contains(&name) {
                        names.push(name);
                    }
                }
                (Lead::With, names.join(" or "))
            }
            Prerequisite::All(_) | Prerequisite::Any(_) | Prerequisite::AtLeast(..) => {
                (Lead::With, format!("({})", text()))
            }
            Prerequisite::Feature { count, .. } if *count > 1 => (Lead::From, text()),
            Prerequisite::Rank { .. } => (Lead::From, text()),
            Prerequisite::ClassLevel { .. }
            | Prerequisite::SkillRanks { .. }
            | Prerequisite::AbilityScore { .. }
            | Prerequisite::FullAbilityScore { .. }
            | Prerequisite::BaseAttackBonus(_)
            | Prerequisite::CasterLevel(_)
            | Prerequisite::CharacterLevel(_)
            | Prerequisite::MythicRank(_) => (Lead::From, text()),
            Prerequisite::Not(_)
            | Prerequisite::Feat(_)
            | Prerequisite::FeatFor { .. }
            | Prerequisite::FeatForSameValue(_)
            | Prerequisite::FeatIfValue { .. }
            | Prerequisite::MythicFeat(_)
            | Prerequisite::MythicAbility(_)
            | Prerequisite::Feature { .. }
            | Prerequisite::SelectionOption(_)
            | Prerequisite::Selection(_)
            | Prerequisite::CasterType(_)
            | Prerequisite::Story(_) => (Lead::With, text()),
            Prerequisite::Deity(deity) => (
                Lead::For,
                format!("a follower of {}", self.deity_name(deity)),
            ),
            Prerequisite::Atheist => (Lead::For, "an atheist".to_owned()),
            Prerequisite::Worships => (Lead::For, "a follower of a deity".to_owned()),
            Prerequisite::DeityOffers(_) => (Lead::For, text()),
        }
    }

    /// What an unmet prerequisite needs, without the leading "Needs".
    pub fn unmet_text(&self, unmet: &Unmet) -> String {
        match unmet {
            Unmet::Condition { needs, has } => self.prerequisite_text(needs, *has),
            Unmet::All(children) => self.unmet_all_texts(children).join(", "),
            Unmet::Any(children) => {
                let texts = children
                    .iter()
                    .map(|child| self.nested_unmet_text(child))
                    .collect();
                let options = children.iter().map(|child| match child {
                    Unmet::Condition {
                        needs: Prerequisite::SelectionOption(option),
                        ..
                    } => Some(option),
                    _ => None,
                });
                self.told_apart(texts, options).join(" or ")
            }
            Unmet::AtLeast {
                needed,
                met,
                failed,
            } => {
                let children: Vec<String> = failed
                    .iter()
                    .map(|child| self.nested_unmet_text(child))
                    .collect();
                // "More" only when some are held (ticket 330 item 3).
                let count = match met {
                    0 => format!("{needed} of"),
                    met => format!("{} more of", needed.saturating_sub(*met)),
                };
                format!("{count}: {}", children.join(", "))
            }
            Unmet::MetLater { unmet, .. } => self.unmet_text(unmet),
        }
    }

    /// The failing children of an "and", each once: two that read alike (both "no Bloodline yet") are one.
    fn unmet_all_texts(&self, children: &[Unmet]) -> Vec<String> {
        distinct(
            children
                .iter()
                .map(|child| self.nested_unmet_text(child))
                .collect(),
        )
    }

    /// A group inside another reads in brackets, unless it is an "and" left with one text.
    fn nested_unmet_text(&self, unmet: &Unmet) -> String {
        match unmet {
            Unmet::Condition { .. } => self.unmet_text(unmet),
            Unmet::MetLater { unmet, .. } => self.nested_unmet_text(unmet),
            Unmet::All(children) => match self.unmet_all_texts(children).as_slice() {
                [one] => one.clone(),
                _ => format!("({})", self.unmet_text(unmet)),
            },
            _ => format!("({})", self.unmet_text(unmet)),
        }
    }

    /// One prerequisite as the sheet says it, with what the character has for a numeric one ("BAB +4 (have
    /// +1)"). It reads a canonical tree, as the data holds ([`Prerequisite::canonicalize`]): a tree in another shape
    /// reads with its nesting shown (an "or" in an "or" in brackets, a double negation as one).
    pub fn prerequisite_text(&self, prerequisite: &Prerequisite, has: Option<u8>) -> String {
        self.prerequisite_text_within(prerequisite, has, None)
    }

    /// [`Self::prerequisite_text`] with every group of more than `most` alternatives shortened to how many there are,
    /// what they are and the first few: "one of 49 hexes (Air Barrier, Battle Master, Battle Ward, …)", "2 of 27
    /// alternatives (…)". For a long
    /// prerequisite in the MCP server's `search` (ticket 128), whose `describe` gives the whole text.
    pub fn short_prerequisite_text(&self, prerequisite: &Prerequisite, most: usize) -> String {
        self.prerequisite_text_within(prerequisite, None, Some(most))
    }

    /// The texts of a group's children, each group among them in brackets (an "and" left with one text, not), with
    /// the options of one name told apart ([`Self::told_apart`]).
    fn group_texts(&self, children: &[&Prerequisite], most: Option<usize>) -> Vec<String> {
        let texts = children
            .iter()
            .map(|child| match child {
                Prerequisite::All(inner) => match self.all_texts(inner, most).as_slice() {
                    [one] => one.clone(),
                    parts => format!("({})", parts.join(", ")),
                },
                Prerequisite::Any(_) | Prerequisite::AtLeast(..) => {
                    format!("({})", self.prerequisite_text_within(child, None, most))
                }
                _ => self.prerequisite_text_within(child, None, most),
            })
            .collect();
        let options = children.iter().map(|child| match child {
            Prerequisite::SelectionOption(option) => Some(option),
            _ => None,
        });
        self.told_apart(texts, options)
    }

    /// The texts of an "and"'s children, each once: "no Bloodline yet" twice (the Sorcerer's and the Bloodrager's
    /// Bloodline selections share a name) is one. What the "and" refuses comes after what it needs ("Order of the
    /// Godclaw, not with Pentamic Faith", ticket 329). An "or" keeps every alternative.
    fn all_texts(&self, children: &[Prerequisite], most: Option<usize>) -> Vec<String> {
        let mut parts = refs(children);
        parts.sort_by_key(|part| matches!(part, Prerequisite::Not(_)));
        distinct(self.group_texts(&parts, most))
    }

    /// The text of a prerequisite, its groups of more than `most` alternatives shortened when `most` is given.
    fn prerequisite_text_within(
        &self,
        prerequisite: &Prerequisite,
        has: Option<u8>,
        most: Option<usize>,
    ) -> String {
        /// How many alternatives a shortened group names.
        const SHOWN: usize = 3;
        let have = |text: String| match has {
            Some(value) => format!("{text} (have {value})"),
            None => text,
        };
        match prerequisite {
            Prerequisite::All(children) => self.all_texts(children, most).join(", "),
            Prerequisite::Any(children) | Prerequisite::AtLeast(_, children)
                if most.is_some_and(|most| children.len() > most) =>
            {
                let texts = self.group_texts(&refs(children), most);
                let mut shown = texts[..SHOWN.min(texts.len())].join(", ");
                if texts.len() > SHOWN {
                    shown.push_str(", …");
                }
                let needed = match prerequisite {
                    Prerequisite::AtLeast(n, _) => n.to_string(),
                    _ => "one".to_owned(),
                };
                format!(
                    "{needed} of {} {} ({shown})",
                    children.len(),
                    self.alternatives_noun(children)
                )
            }
            Prerequisite::Any(children) => self.group_texts(&refs(children), most).join(" or "),
            Prerequisite::AtLeast(n, children) => {
                format!(
                    "{n} of: {}",
                    self.group_texts(&refs(children), most).join(", ")
                )
            }
            Prerequisite::Not(held) => {
                let (lead, object) = self.held_text(held);
                lead.refusal(&[object])
            }
            Prerequisite::Feat(feat) => self.feat_name(feat).to_owned(),
            Prerequisite::FeatFor { feat, value } => {
                format!("{} ({})", self.feat_name(feat), self.parameter_name(value))
            }
            Prerequisite::FeatForSameValue(feat) => {
                let kind = self
                    .catalog
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
            Prerequisite::Rank { fact, rank } => have(format!("{} ×{rank}", self.fact_name(fact))),
            Prerequisite::SelectionOption(option) => self.option_name(option).to_owned(),
            Prerequisite::Selection(selection) => self.selection_text(selection),
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
                let noun = if *ranks == 1 { "rank" } else { "ranks" };
                have(format!("{ranks} {noun} in {}", self.skill_name(skill)))
            }
            Prerequisite::AbilityScore { ability, score } => {
                have(format!("{} {score}", ability.short_name()))
            }
            Prerequisite::BaseAttackBonus(bab) => match has {
                Some(value) => format!("BAB +{bab} (have +{value})"),
                None => format!("BAB +{bab}"),
            },
            Prerequisite::SpellLevel { spell_type, level } => {
                spell_need(*spell_type, *level, false, has)
            }
            Prerequisite::SpontaneousSpellLevel { spell_type, level } => {
                spell_need(*spell_type, *level, true, has)
            }
            Prerequisite::FullAbilityScore { ability, score } => have(format!(
                "{} {score} with its feat-type bonuses",
                ability.short_name()
            )),
            Prerequisite::ClassSpellLevel { class, level } => format!(
                "to cast {}-level {} spells{}",
                ordinal(*level),
                self.class_name(class),
                spells_had(has)
            ),
            Prerequisite::CasterType(spell_type) => {
                format!("a class casting {} spells", spell_type_name(*spell_type))
            }
            Prerequisite::CasterLevel(level) => have(format!("caster level {level}")),
            Prerequisite::CharacterLevel(level) => have(format!("character level {level}")),
            Prerequisite::MythicRank(rank) => have(format!("Mythic rank {rank}")),
            Prerequisite::Alignment(alignment) => format!("{} alignment", alignment.name()),
            Prerequisite::Race(race) => self.race_name(race).to_owned(),
            Prerequisite::Story(story) => story.name().to_owned(),
            Prerequisite::Deity(deity) => format!("{} as deity", self.deity_name(deity)),
            Prerequisite::Atheist => "atheism".to_owned(),
            Prerequisite::Worships => "a deity".to_owned(),
            Prerequisite::DeityOffers(domain) => {
                format!("a deity offering the {}", self.option_name(domain))
            }
        }
    }

    /// A Feature selection as a prerequisite names it: "the Fey Tricks selection", or, when another selection shares
    /// its name, with what names it in its rows ([`Self::selection_owner`]): "the Barbarian's Rage Power selection or
    /// the Skald's Rage Power selection" (ticket 329). Like a feature's ([`Self::feature_name_apart`]), whether a name
    /// is shared is the game data's, not one reason's.
    fn selection_text(&self, key: &FeatureSelectionKey) -> String {
        let name = self.selection_name(key);
        let owner = (self.catalog.selection_shares_name(key))
            .then(|| self.selection_owner(key))
            .flatten();
        match owner {
            Some(owner) => format!("the {owner}'s {name} selection"),
            None => format!("the {name} selection"),
        }
    }

    /// A group's texts, with each option whose text another of the group shares named with its Feature selection:
    /// "Animal Companion (Nature Bond) or Animal Companion (Nature Revelations)", not "Animal Companion or Animal
    /// Companion" (ticket 128); and when that is shared too, with the selection's owner as well: "Arcane (Sorcerer
    /// Bloodline) or Arcane (Bloodrager Bloodline)". A feature already reads with its owner when another shares its
    /// name ([`Self::feature_name_apart`]). `options` holds each text's option, if it is one.
    fn told_apart<'a>(
        &self,
        mut texts: Vec<String>,
        options: impl Iterator<Item = Option<&'a SelectionOptionKey>>,
    ) -> Vec<String> {
        let shared = |texts: &[String]| -> Vec<bool> {
            texts
                .iter()
                .map(|text| texts.iter().filter(|other| *other == text).count() > 1)
                .collect()
        };
        let options: Vec<_> = options.collect();
        let named = |option: &SelectionOptionKey, with_owner: bool| {
            let selection = self.catalog.selection_of(option)?;
            let owner = match with_owner {
                true => self
                    .selection_owner(&selection.key)
                    .map(|owner| format!("{owner} ")),
                false => None,
            };
            Some(format!(
                "{} ({}{})",
                self.option_name(option),
                owner.unwrap_or_default(),
                selection.name
            ))
        };
        for with_owner in [false, true] {
            let flags = shared(&texts);
            for ((text, shared), option) in texts.iter_mut().zip(flags).zip(&options) {
                if let (true, Some(option)) = (shared, option)
                    && let Some(named) = named(option, with_owner)
                {
                    *text = named;
                }
            }
        }
        texts
    }

    /// What a shortened group's alternatives are, in the plural: their one name when they share it, an ending in
    /// brackets aside ("Animal Companion (Beast Rider)" → "animal companions"), their Feature selections' one name ("hexes"), "feats", or else "alternatives".
    fn alternatives_noun<'a>(&'a self, children: &'a [Prerequisite]) -> String {
        let name = |child: &'a Prerequisite| -> Option<&'a str> {
            match child {
                Prerequisite::Feat(feat) | Prerequisite::FeatFor { feat, .. } => {
                    Some(self.feat_name(feat))
                }
                Prerequisite::Feature { feature, .. } => Some(self.feature_name(feature)),
                Prerequisite::SelectionOption(option) => Some(self.option_name(option)),
                Prerequisite::MythicFeat(feat) => Some(self.mythic_feat_name(feat)),
                Prerequisite::MythicAbility(ability) => Some(self.mythic_ability_name(ability)),
                _ => None,
            }
        };
        let selection = |child: &'a Prerequisite| match child {
            Prerequisite::SelectionOption(option) => self
                .catalog
                .selection_of(option)
                .map(|selection| selection.name.as_str()),
            _ => None,
        };
        let one = |names: Vec<Option<&str>>| {
            let first = names.first().copied().flatten()?;
            names
                .iter()
                .all(|name| *name == Some(first))
                .then(|| plural(&first.to_lowercase()))
        };
        let base = |name: &'a str| name.split(" (").next().unwrap_or(name);
        one(children.iter().map(|child| name(child).map(base)).collect())
            .or_else(|| one(children.iter().map(selection).collect()))
            .unwrap_or_else(|| {
                let feats = children.iter().all(|child| {
                    matches!(child, Prerequisite::Feat(_) | Prerequisite::FeatFor { .. })
                });
                match feats {
                    true => "feats".to_owned(),
                    false => "alternatives".to_owned(),
                }
            })
    }
}

/// A group's children by reference, as [`Engine::group_texts`] takes them.
fn refs(children: &[Prerequisite]) -> Vec<&Prerequisite> {
    children.iter().collect()
}

/// A noun's plural: "hexes", "talents", "abilities"; one that ends in "s" is taken as plural already.
fn plural(noun: &str) -> String {
    if noun.ends_with('s') {
        noun.to_owned()
    } else if noun.ends_with('x') || noun.ends_with("ch") || noun.ends_with("sh") {
        format!("{noun}es")
    } else if let Some(stem) = noun.strip_suffix('y')
        && !stem.ends_with(['a', 'e', 'i', 'o', 'u'])
    {
        format!("{stem}ies")
    } else {
        format!("{noun}s")
    }
}

/// "A, B or C".
fn or_list(items: &[&str]) -> String {
    match items {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [rest @ .., last] => format!("{} or {last}", rest.join(", ")),
    }
}

/// "to cast 3rd-level arcane spells (have 1st)", or "(can't cast any)".
fn spell_need(spell_type: SpellType, level: u8, spontaneous: bool, has: Option<u8>) -> String {
    let how = if spontaneous { " spontaneously" } else { "" };
    format!(
        "to cast {}{how}{}",
        spell_text(spell_type, level),
        spells_had(has)
    )
}

/// " (have 1st)", " (can't cast any)", or nothing without a value.
fn spells_had(has: Option<u8>) -> String {
    match has {
        Some(0) => " (can't cast any)".to_owned(),
        Some(value) => format!(" (have {})", ordinal(value)),
        None => String::new(),
    }
}

/// "3rd-level arcane spells".
fn spell_text(spell_type: SpellType, level: u8) -> String {
    format!(
        "{}-level {} spells",
        ordinal(level),
        spell_type_name(spell_type)
    )
}

/// "arcane", "divine".
fn spell_type_name(spell_type: SpellType) -> &'static str {
    match spell_type {
        SpellType::Arcane => "arcane",
        SpellType::Divine => "divine",
    }
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
            ParameterKind::Metamagic => "metamagic",
            ParameterKind::Stat => "stat",
            ParameterKind::Element => "element",
        }
    }

    /// The noun with its article, as a picker asks for a value: "a weapon", "an energy type".
    pub fn with_article(self) -> &'static str {
        match self {
            ParameterKind::Skill => "a skill",
            ParameterKind::Weapon => "a weapon",
            ParameterKind::ExoticWeapon => "an exotic weapon",
            ParameterKind::SpellSchool => "a school",
            ParameterKind::Energy => "an energy type",
            ParameterKind::ArmourType => "an armour type",
            ParameterKind::Metamagic => "a metamagic",
            ParameterKind::Stat => "a stat",
            ParameterKind::Element => "an element",
        }
    }
}

/// The texts without a repeat, in order.
fn distinct(texts: Vec<String>) -> Vec<String> {
    let mut seen: Vec<String> = Vec::with_capacity(texts.len());
    for text in texts {
        if !seen.contains(&text) {
            seen.push(text);
        }
    }
    seen
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
