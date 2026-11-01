//! The game data the planner works from: races, skills, feats, classes, mythic paths and the progression facts.
//!
//! These types are the single shape of the data. The `game-data/*.ron` files deserialise straight into them,
//! the database is loaded from and read back into them, and the browser receives them as JSON.
//!
//! Order carries no meaning anywhere in the data: catalogs are ordered by key, level and rank rows by their
//! number, and set-like children either are sets or get sorted. [`GameData::canonicalize`] puts a value in that
//! one order, so two `GameData`s with the same content compare equal however their files were written.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::keys::{
    ArchetypeKey, ClassKey, FeatKey, FeatureKey, FeatureSelectionKey, MythicAbilityKey,
    MythicFeatKey, MythicPathKey, RaceKey, SelectionOptionKey, SkillKey, SubraceKey,
};
use crate::prerequisite::Prerequisite;

/// The guide every [`GuideRef`] points into.
pub const GUIDE_BASE_URL: &str =
    "https://gamefaqs.gamespot.com/ps4/324475-pathfinder-wrath-of-the-righteous/faqs/80843/";

/// A link into the guide as `page#anchor`, for example `rogue#dark-lurker`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GuideRef(pub String);

impl GuideRef {
    pub fn new(page_and_anchor: impl Into<String>) -> Self {
        Self(page_and_anchor.into())
    }

    /// The full guide URL.
    pub fn url(&self) -> String {
        format!("{GUIDE_BASE_URL}{}", self.0)
    }
}

/// Everything the planner knows about the game.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameData {
    pub progression: Progression,
    pub races: Vec<Race>,
    pub skills: Vec<Skill>,
    pub feats: Vec<Feat>,
    /// Base classes, each with its archetypes.
    pub classes: Vec<Class>,
    /// Prestige classes. They share [`Class`]'s shape but have no archetypes.
    pub prestige_classes: Vec<Class>,
    pub mythic_abilities: Vec<MythicAbility>,
    pub mythic_feats: Vec<MythicFeat>,
    pub mythic_paths: Vec<MythicPath>,
}

impl GameData {
    /// Every archetype of every base class.
    pub fn archetypes(&self) -> impl Iterator<Item = &Archetype> {
        self.classes.iter().flat_map(|class| &class.archetypes)
    }

    /// How much of each kind of thing the data holds, labelled, as a quick check that seeding worked.
    pub fn counts(&self) -> [(&'static str, usize); 9] {
        [
            ("Races", self.races.len()),
            ("Skills", self.skills.len()),
            ("Feats", self.feats.len()),
            ("Classes", self.classes.len()),
            ("Archetypes", self.archetypes().count()),
            ("Prestige classes", self.prestige_classes.len()),
            ("Mythic abilities", self.mythic_abilities.len()),
            ("Mythic feats", self.mythic_feats.len()),
            ("Mythic paths", self.mythic_paths.len()),
        ]
    }

    /// Sorts every list into its one canonical order: catalogs by key, class level rows by level, mythic rank rows
    /// and ascensions by rank, and multisets (granted features and selections, feat slots, ability modifiers,
    /// prerequisite children) by value.
    pub fn canonicalize(&mut self) {
        self.progression.mythic_ascensions.sort_by_key(|a| a.rank);
        sort_by_key(&mut self.races, |r| r.key.clone());
        for race in &mut self.races {
            race.ability_modifiers.sort();
            race.feat_slots.sort();
            sort_by_key(&mut race.subraces, |s| s.key.clone());
            for subrace in &mut race.subraces {
                subrace.ability_modifiers.sort();
            }
        }
        sort_by_key(&mut self.skills, |s| s.key.clone());
        sort_by_key(&mut self.feats, |f| f.key.clone());
        for feat in &mut self.feats {
            canonicalize_prerequisite(&mut feat.prerequisite);
        }
        for class in self.classes.iter_mut().chain(&mut self.prestige_classes) {
            class.canonicalize();
        }
        sort_by_key(&mut self.classes, |c| c.key.clone());
        sort_by_key(&mut self.prestige_classes, |c| c.key.clone());
        sort_by_key(&mut self.mythic_abilities, |a| a.key.clone());
        for ability in &mut self.mythic_abilities {
            canonicalize_prerequisite(&mut ability.prerequisite);
        }
        sort_by_key(&mut self.mythic_feats, |f| f.key.clone());
        for feat in &mut self.mythic_feats {
            canonicalize_prerequisite(&mut feat.prerequisite);
        }
        sort_by_key(&mut self.mythic_paths, |p| p.key.clone());
        for path in &mut self.mythic_paths {
            sort_by_key(&mut path.features, |f| f.key.clone());
            canonicalize_selections(&mut path.selections);
            path.ranks.sort_by_key(|r| r.rank);
            for row in &mut path.ranks {
                sort_grants(&mut row.features, &mut row.feat_slots, &mut row.selections);
            }
        }
    }

    /// [`canonicalize`](Self::canonicalize), by value.
    pub fn canonical(mut self) -> Self {
        self.canonicalize();
        self
    }
}

fn sort_by_key<T, K: Ord>(items: &mut [T], key: impl FnMut(&T) -> K) {
    items.sort_by_cached_key(key);
}

fn canonicalize_prerequisite(prerequisite: &mut Option<Prerequisite>) {
    if let Some(prerequisite) = prerequisite {
        prerequisite.canonicalize();
    }
}

fn canonicalize_selections(selections: &mut [FeatureSelection]) {
    sort_by_key(selections, |s| s.key.clone());
    for selection in selections {
        sort_by_key(&mut selection.options, |o| o.key.clone());
        for option in &mut selection.options {
            canonicalize_prerequisite(&mut option.prerequisite);
        }
    }
}

fn canonicalize_rows(rows: &mut [ClassLevelRow]) {
    rows.sort_by_key(|r| r.level);
    for row in rows {
        sort_grants(&mut row.features, &mut row.feat_slots, &mut row.selections);
    }
}

/// The grants of a class level row or a mythic rank row are multisets: sorted by value.
fn sort_grants(
    features: &mut [FeatureKey],
    feat_slots: &mut [FeatSlot],
    selections: &mut [FeatureSelectionKey],
) {
    features.sort();
    feat_slots.sort();
    selections.sort();
}

impl Class {
    fn canonicalize(&mut self) {
        canonicalize_prerequisite(&mut self.prerequisite);
        sort_by_key(&mut self.features, |f| f.key.clone());
        canonicalize_selections(&mut self.selections);
        canonicalize_rows(&mut self.levels);
        sort_by_key(&mut self.archetypes, |a| a.key.clone());
        for archetype in &mut self.archetypes {
            canonicalize_prerequisite(&mut archetype.prerequisite);
            sort_by_key(&mut archetype.features, |f| f.key.clone());
            canonicalize_selections(&mut archetype.selections);
            canonicalize_rows(&mut archetype.levels);
        }
    }
}

/// The progression table that applies to every Build (`progression.ron`): when each Mythic rank is reached. The
/// rest of how a character progresses is rules in code ([`PATH_CHOICE_RANK`](crate::PATH_CHOICE_RANK),
/// [`PATH_SWITCH_RANK`](crate::PATH_SWITCH_RANK), [`FeatCategory::is_combat`], the skill rank cap of one per
/// Character level).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progression {
    pub mythic_ascensions: Vec<MythicAscension>,
}

/// One Mythic ascension: the Mythic rank reached at a fixed Character level.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MythicAscension {
    pub rank: u8,
    pub character_level: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Ability {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SpellType {
    Arcane,
    Divine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Alignment {
    LawfulGood,
    NeutralGood,
    ChaoticGood,
    LawfulNeutral,
    TrueNeutral,
    ChaoticNeutral,
    LawfulEvil,
    NeutralEvil,
    ChaoticEvil,
}

/// The guide's feat groups on the `feats` page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FeatCategory {
    Weapon,
    Ranged,
    UnarmedCombat,
    ArmourAndShield,
    Magic,
    Miscellaneous,
    MountedCombat,
    CombatManoeuvres,
    Teamwork,
    Skill,
    Class,
    Racial,
}

impl FeatCategory {
    /// Whether feats of this category are Combat feats: every category but class, racial, skill, teamwork and
    /// magic.
    pub fn is_combat(self) -> bool {
        use FeatCategory::*;
        !matches!(self, Class | Racial | Skill | Teamwork | Magic)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Race {
    pub key: RaceKey,
    pub name: String,
    #[serde(default)]
    pub summary: String,
    pub guide: GuideRef,
    #[serde(default)]
    pub ability_modifiers: Vec<AbilityModifier>,
    /// Extra Feat slots at character level 1 (a human's bonus feat is a `General` slot).
    #[serde(default)]
    pub feat_slots: Vec<FeatSlot>,
    #[serde(default)]
    pub subraces: Vec<Subrace>,
}

/// A subrace. Its ability modifiers replace the race's.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subrace {
    pub key: SubraceKey,
    pub name: String,
    #[serde(default)]
    pub summary: String,
    pub guide: GuideRef,
    #[serde(default)]
    pub ability_modifiers: Vec<AbilityModifier>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AbilityModifier {
    Fixed {
        ability: Ability,
        amount: i8,
    },
    /// The player picks the ability at Build start ("+2 to one ability score").
    Chosen {
        amount: i8,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Skill {
    pub key: SkillKey,
    pub name: String,
    pub ability: Ability,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feat {
    pub key: FeatKey,
    pub name: String,
    pub category: FeatCategory,
    #[serde(default)]
    pub summary: String,
    pub guide: GuideRef,
    /// `None`: the guide lists no prerequisites, so there are none.
    #[serde(default)]
    pub prerequisite: Option<Prerequisite>,
}

/// A Feat slot granted by a race, a class level or a mythic rank. Every slot checks the picked feat's
/// prerequisites, except a bonus-list slot.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FeatSlot {
    /// A general slot: any feat whose prerequisites are met.
    General,
    /// A bonus slot for any Combat feat.
    CombatFeat,
    /// A bonus slot for any feat of one category ("Bonus Teamwork Feat").
    Category(FeatCategory),
    /// A bonus slot for one feat from a fixed list.
    OneOf {
        feats: BTreeSet<FeatKey>,
        /// Set on a bonus-list slot, one that draws from the class's own bonus-feat list (a monk's bonus feats, a
        /// ranger's combat style): the pick ignores the feat's prerequisites. It still has to be on the list and
        /// not already held, and it counts as held afterwards. Unset, the prerequisites are checked.
        #[serde(default)]
        ignores_prerequisites: bool,
    },
}

impl FeatSlot {
    /// Whether a pick here ignores the feat's prerequisites: only a bonus-list slot does.
    pub fn ignores_prerequisites(&self) -> bool {
        matches!(
            self,
            FeatSlot::OneOf {
                ignores_prerequisites: true,
                ..
            }
        )
    }
}

/// A base class or a prestige class.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Class {
    pub key: ClassKey,
    pub name: String,
    #[serde(default)]
    pub summary: String,
    pub guide: GuideRef,
    pub skill_points: u8,
    #[serde(default)]
    pub class_skills: BTreeSet<SkillKey>,
    /// Entry prerequisites: a prestige class's, or a base class's alignment rule.
    #[serde(default)]
    pub prerequisite: Option<Prerequisite>,
    #[serde(default)]
    pub features: Vec<Feature>,
    #[serde(default)]
    pub selections: Vec<FeatureSelection>,
    /// One row per Class level, 1 up to the class's last level with none missing. The number of rows is the
    /// class level cap. What taking the class grants is in row 1.
    pub levels: Vec<ClassLevelRow>,
    #[serde(default)]
    pub archetypes: Vec<Archetype>,
}

/// An archetype. Its level rows replace the base class's in full.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Archetype {
    pub key: ArchetypeKey,
    pub name: String,
    #[serde(default)]
    pub summary: String,
    pub guide: GuideRef,
    #[serde(default)]
    pub prerequisite: Option<Prerequisite>,
    /// `None`: same as the base class.
    #[serde(default)]
    pub skill_points: Option<u8>,
    /// `None`: same as the base class. `Some`: replaces the base class's list (possibly with none).
    #[serde(default)]
    pub class_skills: Option<BTreeSet<SkillKey>>,
    /// Features this archetype adds (it can also grant the base class's).
    #[serde(default)]
    pub features: Vec<Feature>,
    #[serde(default)]
    pub selections: Vec<FeatureSelection>,
    pub levels: Vec<ClassLevelRow>,
}

/// What one Class level of a class, archetype or prestige class gives.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassLevelRow {
    pub level: u8,
    /// Base attack bonus at this Class level (the first number of "+6 / +1").
    pub bab: u8,
    pub fort: u8,
    pub reflex: u8,
    pub will: u8,
    /// Highest arcane spell level castable at this Class level; 0 when none.
    #[serde(default)]
    pub arcane_spell_level: u8,
    /// Highest divine spell level castable at this Class level; 0 when none.
    #[serde(default)]
    pub divine_spell_level: u8,
    /// Prestige classes: this level advances the chosen arcane class's spellcasting.
    #[serde(default)]
    pub advances_arcane: bool,
    /// Prestige classes: this level advances the chosen divine class's spellcasting.
    #[serde(default)]
    pub advances_divine: bool,
    #[serde(default)]
    pub features: Vec<FeatureKey>,
    #[serde(default)]
    pub feat_slots: Vec<FeatSlot>,
    /// One Feature selection slot per entry.
    #[serde(default)]
    pub selections: Vec<FeatureSelectionKey>,
}

/// A named ability that a class, archetype or mythic path grants, such as Sneak Attack.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feature {
    pub key: FeatureKey,
    pub name: String,
    #[serde(default)]
    pub summary: String,
    pub guide: GuideRef,
}

/// The fixed list a Feature selection picks from, such as rogue talents or a path's picks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureSelection {
    pub key: FeatureSelectionKey,
    pub name: String,
    #[serde(default)]
    pub summary: String,
    pub guide: GuideRef,
    pub options: Vec<SelectionOption>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionOption {
    pub key: SelectionOptionKey,
    pub name: String,
    #[serde(default)]
    pub summary: String,
    pub guide: GuideRef,
    /// The feat this option grants ("This is the feat of the same name").
    #[serde(default)]
    pub feat: Option<FeatKey>,
    #[serde(default)]
    pub prerequisite: Option<Prerequisite>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MythicAbility {
    pub key: MythicAbilityKey,
    pub name: String,
    /// The guide's group heading (Melee, Ranged, Magic, ...). Display only.
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub summary: String,
    pub guide: GuideRef,
    #[serde(default)]
    pub prerequisite: Option<Prerequisite>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MythicFeat {
    pub key: MythicFeatKey,
    pub name: String,
    /// The guide's group heading. Display only.
    #[serde(default)]
    pub group: String,
    #[serde(default)]
    pub summary: String,
    pub guide: GuideRef,
    #[serde(default)]
    pub prerequisite: Option<Prerequisite>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MythicPath {
    pub key: MythicPathKey,
    pub name: String,
    #[serde(default)]
    pub summary: String,
    pub guide: GuideRef,
    #[serde(default)]
    pub features: Vec<Feature>,
    /// The path's own picks.
    #[serde(default)]
    pub selections: Vec<FeatureSelection>,
    /// The path's rank table: its own picks on top of the general ones in [`MythicAscension`].
    #[serde(default)]
    pub ranks: Vec<MythicRankRow>,
    /// Set when a Build can switch to this path at the switch rank.
    #[serde(default)]
    pub switch: Option<PathSwitch>,
}

/// What one Mythic rank of a path gives beyond the general mythic picks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MythicRankRow {
    pub rank: u8,
    #[serde(default)]
    pub features: Vec<FeatureKey>,
    #[serde(default)]
    pub feat_slots: Vec<FeatSlot>,
    #[serde(default)]
    pub selections: Vec<FeatureSelectionKey>,
}

/// What happens when a Build switches to this path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathSwitch {
    /// The earlier Mythic ranks whose picks are kept (Legend: 1 and 2).
    pub kept_ranks: BTreeSet<u8>,
    /// Whether later Mythic ascensions still happen (Legend: no).
    pub further_ascensions: bool,
    /// The Character level cap after switching, when the switch raises the usual one (Legend: 40).
    #[serde(default)]
    pub level_cap: Option<u8>,
}

impl PathSwitch {
    /// The Mythic rank right after the switch: one above the highest rank kept (Legend: 3).
    pub fn rank_after_switch(&self) -> u8 {
        self.kept_ranks
            .last()
            .map_or(1, |rank| rank.saturating_add(1))
    }
}
