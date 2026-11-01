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
    ArchetypeKey, ClassKey, FeatKey, FeatListKey, FeatureKey, FeatureSelectionKey,
    MythicAbilityKey, MythicFeatKey, MythicPathKey, RaceKey, SelectionOptionKey, SkillKey,
    SubraceKey, WeaponKey,
};
use crate::parameter::{ParameterKind, Repeat};
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
    /// The weapons a parameterised feat can be taken for.
    pub weapons: Vec<Weapon>,
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

    /// Every feat list: each class's (base, then prestige), each of its archetypes', then each path's.
    pub fn feat_lists(&self) -> impl Iterator<Item = &FeatList> {
        let classes = self
            .classes
            .iter()
            .chain(&self.prestige_classes)
            .flat_map(|class| {
                class.feat_lists.iter().chain(
                    class
                        .archetypes
                        .iter()
                        .flat_map(|archetype| &archetype.feat_lists),
                )
            });
        classes.chain(self.mythic_paths.iter().flat_map(|path| &path.feat_lists))
    }

    /// How much of each kind of thing the data holds, labelled, as a quick check that seeding worked.
    pub fn counts(&self) -> [(&'static str, usize); 10] {
        [
            ("Races", self.races.len()),
            ("Skills", self.skills.len()),
            ("Weapons", self.weapons.len()),
            ("Feats", self.feats.len()),
            ("Classes", self.classes.len()),
            ("Archetypes", self.archetypes().count()),
            ("Prestige classes", self.prestige_classes.len()),
            ("Mythic abilities", self.mythic_abilities.len()),
            ("Mythic feats", self.mythic_feats.len()),
            ("Mythic paths", self.mythic_paths.len()),
        ]
    }

    /// Sorts every list into its one canonical order: catalogs (feat lists included) by key, class level rows by
    /// level, mythic rank rows and ascensions by rank, and multisets (granted features and selections, feat slots,
    /// ability modifiers, prerequisite children) by value.
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
        sort_by_key(&mut self.weapons, |w| w.key.clone());
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
            canonicalize_features(&mut path.features);
            canonicalize_selections(&mut path.selections);
            sort_by_key(&mut path.feat_lists, |l| l.key.clone());
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

/// Features by key, each one's ability bonuses (a set in all but type) by value.
fn canonicalize_features(features: &mut [Feature]) {
    sort_by_key(features, |f| f.key.clone());
    for feature in features {
        feature.ability_bonuses.sort();
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

fn canonicalize_archetype_rows(rows: &mut [ArchetypeLevelRow]) {
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
        canonicalize_features(&mut self.features);
        canonicalize_selections(&mut self.selections);
        sort_by_key(&mut self.feat_lists, |l| l.key.clone());
        self.save_exceptions.sort();
        canonicalize_rows(&mut self.levels);
        sort_by_key(&mut self.archetypes, |a| a.key.clone());
        for archetype in &mut self.archetypes {
            canonicalize_prerequisite(&mut archetype.prerequisite);
            canonicalize_features(&mut archetype.features);
            canonicalize_selections(&mut archetype.selections);
            sort_by_key(&mut archetype.feat_lists, |l| l.key.clone());
            canonicalize_archetype_rows(&mut archetype.levels);
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

/// A weapon, as the guide's weapon lists name it (Weapon Focus is taken for one).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Weapon {
    pub key: WeaponKey,
    pub name: String,
    /// The proficiency feat whose list names it; `None` for what no proficiency list names (unarmed strike, rays).
    #[serde(default)]
    pub proficiency: Option<WeaponProficiency>,
}

/// The guide's three weapon proficiency lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WeaponProficiency {
    Simple,
    Martial,
    Exotic,
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
    /// What the feat is taken for, when the guide says to choose one (Weapon Focus: a weapon). Each pick of it then
    /// names a value of this kind.
    #[serde(default)]
    pub parameter: Option<ParameterKind>,
    /// How many times it can be taken (Weapon Focus: once per weapon; Extra Hex: three times).
    #[serde(default)]
    pub repeat: Repeat,
    /// `None`: the guide lists no prerequisites, so there are none.
    #[serde(default)]
    pub prerequisite: Option<Prerequisite>,
}

/// A Feat slot granted by a race, a class level or a mythic rank. Every slot checks the picked feat's
/// prerequisites, except a bonus-list slot (a `OneOf` whose list ignores them).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FeatSlot {
    /// A general slot: any feat whose prerequisites are met.
    General,
    /// A bonus slot for any Combat feat.
    CombatFeat,
    /// A bonus slot for any feat of one category ("Bonus Teamwork Feat").
    Category(FeatCategory),
    /// A bonus slot for one feat from a named list.
    OneOf { list: FeatListKey },
}

/// A named fixed list of feats that a [`FeatSlot::OneOf`] picks from, such as a monk's bonus feats. It belongs to
/// the class, archetype or path whose file defines it (its key starts with the owner's), and any row may name
/// it. A list that grows with level (the monk's at 1, 6 and 10) is one list per stage. A lone feat is never a
/// list: the row grants it (`feats`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatList {
    pub key: FeatListKey,
    pub name: String,
    pub guide: GuideRef,
    /// Set on a bonus list, the class's own bonus-feat list (a monk's bonus feats, a ranger's combat style): a
    /// pick from it ignores the feat's prerequisites. It still has to be on the list and not already held, and it
    /// counts as held afterwards. Unset, the prerequisites are checked.
    #[serde(default)]
    pub ignores_prerequisites: bool,
    /// Set on a list of every feat in the data (Gold Dragon's Dragon Feats: "any three feats"), which then names
    /// no feats of its own: the members would only copy the feat catalog.
    #[serde(default)]
    pub every_feat: bool,
    #[serde(default)]
    pub feats: BTreeSet<FeatKey>,
}

impl FeatList {
    /// Whether the list holds a feat: every feat on an `every_feat` list, else its own.
    pub fn contains(&self, feat: &FeatKey) -> bool {
        self.every_feat || self.feats.contains(feat)
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
    /// How the base attack bonus grows with class level. Archetypes never change it.
    pub bab: BabProgression,
    pub fort: SaveProgression,
    pub reflex: SaveProgression,
    pub will: SaveProgression,
    /// The guide's save cells that its own progression doesn't give (paladin's Will +20 at 20), kept as printed.
    #[serde(default)]
    pub save_exceptions: Vec<SaveException>,
    #[serde(default)]
    pub class_skills: BTreeSet<SkillKey>,
    /// Entry prerequisites: a prestige class's, or a base class's alignment rule.
    #[serde(default)]
    pub prerequisite: Option<Prerequisite>,
    #[serde(default)]
    pub features: Vec<Feature>,
    #[serde(default)]
    pub selections: Vec<FeatureSelection>,
    #[serde(default)]
    pub feat_lists: Vec<FeatList>,
    /// One row per Class level, 1 up to the class's last level with none missing. The number of rows is the
    /// class level cap. What taking the class grants is in row 1.
    pub levels: Vec<ClassLevelRow>,
    #[serde(default)]
    pub archetypes: Vec<Archetype>,
}

/// An archetype. It stores only the levels where it differs from its base class: see
/// [`effective_row`](crate::effective_row).
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
    #[serde(default)]
    pub feat_lists: Vec<FeatList>,
    /// The levels where the archetype differs from its base class, each at most once. A level left out is the
    /// base class's row.
    #[serde(default)]
    pub levels: Vec<ArchetypeLevelRow>,
}

/// One level where an archetype differs from its base class. Its grants replace the base row's in full (an
/// empty list takes the base row's away), and its spell levels replace the base row's when set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchetypeLevelRow {
    pub level: u8,
    /// `None`: the base class's spell levels at this level.
    #[serde(default)]
    pub spell_levels: Option<SpellLevels>,
    #[serde(default)]
    pub features: Vec<FeatureKey>,
    /// Granted feats: see [`ClassLevelRow::feats`].
    #[serde(default)]
    pub feats: BTreeSet<FeatKey>,
    #[serde(default)]
    pub feat_slots: Vec<FeatSlot>,
    /// One Feature selection slot per entry.
    #[serde(default)]
    pub selections: Vec<FeatureSelectionKey>,
}

/// The highest arcane and divine spell levels castable at a Class level; 0 when none.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpellLevels {
    pub arcane: u8,
    pub divine: u8,
}

impl SpellLevels {
    /// Whether any spells are castable.
    pub fn any(self) -> bool {
        self.arcane > 0 || self.divine > 0
    }
}

/// How a class's base attack bonus grows: the guide's full, ¾ and ½ columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BabProgression {
    Full,
    ThreeQuarters,
    Half,
}

impl BabProgression {
    /// The base attack bonus at a class level: `L`, `⌊3L/4⌋` or `⌊L/2⌋`, for base and prestige classes alike.
    pub fn bab(self, class_level: u8) -> u8 {
        let level = u16::from(class_level);
        let bab = match self {
            Self::Full => level,
            Self::ThreeQuarters => level * 3 / 4,
            Self::Half => level / 2,
        };
        u8::try_from(bab).unwrap_or(u8::MAX)
    }
}

/// How one of a class's saves grows: good or poor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SaveProgression {
    Good,
    Poor,
}

impl SaveProgression {
    /// The save bonus at a class level. Base classes: good `2 + ⌊L/2⌋`, poor `⌊L/3⌋`. Prestige classes: good
    /// `⌊(L+1)/2⌋`, poor `⌊(L+1)/3⌋`.
    pub fn save(self, class_level: u8, prestige: bool) -> u8 {
        let level = u16::from(class_level);
        let bonus = match (self, prestige) {
            (Self::Good, false) => 2 + level / 2,
            (Self::Poor, false) => level / 3,
            (Self::Good, true) => level.div_ceil(2),
            (Self::Poor, true) => (level + 1) / 3,
        };
        u8::try_from(bonus).unwrap_or(u8::MAX)
    }
}

/// One of the three saving throws.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Save {
    Fortitude,
    Reflex,
    Will,
}

impl Save {
    pub const ALL: [Save; 3] = [Save::Fortitude, Save::Reflex, Save::Will];
}

/// A save cell the guide prints differently from the class's progression, kept as printed.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveException {
    pub level: u8,
    pub save: Save,
    pub value: u8,
}

impl Class {
    /// The progression of one save.
    pub fn save_progression(&self, save: Save) -> SaveProgression {
        match save {
            Save::Fortitude => self.fort,
            Save::Reflex => self.reflex,
            Save::Will => self.will,
        }
    }

    /// The base attack bonus at a class level.
    pub fn base_attack_bonus(&self, class_level: u8) -> u8 {
        self.bab.bab(class_level)
    }

    /// One save at a class level: the guide's cell when it's an exception, else the progression's formula
    /// (`prestige` picks the prestige formula).
    pub fn save_bonus(&self, save: Save, class_level: u8, prestige: bool) -> u8 {
        self.save_exceptions
            .iter()
            .find(|exception| exception.level == class_level && exception.save == save)
            .map_or_else(
                || self.save_progression(save).save(class_level, prestige),
                |exception| exception.value,
            )
    }
}

/// What one Class level of a class or prestige class gives. BAB and saves aren't here: they follow from the
/// class's progressions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassLevelRow {
    pub level: u8,
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
    /// Granted feats: held from this level on with no slot and no pick, whatever their prerequisites (Scribe
    /// Scrolls, a class's armour and shield proficiencies). One already held adds nothing. A parameterised one is
    /// held for every value; a bonus feat of the player's choosing is a one-feat bonus list instead.
    #[serde(default)]
    pub feats: BTreeSet<FeatKey>,
    #[serde(default)]
    pub feat_slots: Vec<FeatSlot>,
    /// One Feature selection slot per entry.
    #[serde(default)]
    pub selections: Vec<FeatureSelectionKey>,
}

impl ClassLevelRow {
    pub fn spell_levels(&self) -> SpellLevels {
        SpellLevels {
            arcane: self.arcane_spell_level,
            divine: self.divine_spell_level,
        }
    }
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
    /// What the feature does to the character's own ability scores while it's held (Legend: +4 to every score).
    /// They count toward ability-score prerequisites like point buy and level increases do; gear never does.
    #[serde(default)]
    pub ability_bonuses: Vec<AbilityBonus>,
}

/// A permanent change a feature makes to the character's own ability scores, as the guide states it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AbilityBonus {
    /// `+amount` to one score (Eye of the Swarm: Strength and Dexterity +4).
    Fixed { ability: Ability, amount: u8 },
    /// `+amount` to every score (Legend: +4).
    Every { amount: u8 },
    /// `+amount` to the highest score (Perfect Body: +4).
    Highest { amount: u8 },
    /// Every score is at least `score` (Perfect Body: 14).
    AtLeast { score: u8 },
}

/// The fixed list a Feature selection picks from, such as rogue talents or a path's picks: its own options plus
/// the ones it shares from other selections (a slayer's talents include the rogue's).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureSelection {
    pub key: FeatureSelectionKey,
    pub name: String,
    #[serde(default)]
    pub summary: String,
    pub guide: GuideRef,
    /// The options this selection defines, each keyed under it.
    pub options: Vec<SelectionOption>,
    /// Other selections' options this one offers too. A shared option keeps its one key, so holding it through
    /// either selection holds it once, and its prerequisite is the same in both (where a class's own feature
    /// gates it, the data accepts each sharing class's with an `Any`).
    #[serde(default)]
    pub shared_options: BTreeSet<SelectionOptionKey>,
}

impl FeatureSelection {
    /// Whether a pick of `option` may fill this selection's slot: one of its own options, or one it shares.
    pub fn offers(&self, option: &SelectionOptionKey) -> bool {
        self.shared_options.contains(option) || self.options.iter().any(|own| &own.key == option)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionOption {
    pub key: SelectionOptionKey,
    pub name: String,
    #[serde(default)]
    pub summary: String,
    pub guide: GuideRef,
    /// The feats this option grants ("This is the feat of the same name"; All-skilled: every weapon, armour and
    /// shield proficiency). A parameterised one is held for every value (All-skilled: Exotic Weapon Proficiency in
    /// every exotic weapon).
    #[serde(default)]
    pub feats: BTreeSet<FeatKey>,
    /// A parameterised feat this option grants for the value its pick names (the rogue talent Weapon Focus: one
    /// weapon of the player's choosing).
    #[serde(default)]
    pub chosen_feat: Option<FeatKey>,
    /// How many times it can be taken (Increased Damage Reduction: three times). Never `PerValue`.
    #[serde(default)]
    pub repeat: Repeat,
    #[serde(default)]
    pub prerequisite: Option<Prerequisite>,
}

/// The guide's group headings under "Mythic Abilities".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MythicAbilityGroup {
    Melee,
    Ranged,
    Magic,
    Healing,
    Miscellaneous,
    Class,
}

impl MythicAbilityGroup {
    pub const ALL: [Self; 6] = [
        Self::Melee,
        Self::Ranged,
        Self::Magic,
        Self::Healing,
        Self::Miscellaneous,
        Self::Class,
    ];

    /// The heading as the guide prints it, without "Abilities".
    pub fn name(self) -> &'static str {
        match self {
            Self::Melee => "Melee",
            Self::Ranged => "Ranged",
            Self::Magic => "Magic",
            Self::Healing => "Healing",
            Self::Miscellaneous => "Miscellaneous",
            Self::Class => "Class",
        }
    }
}

/// The guide's group headings under "Mythic Feats".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MythicFeatGroup {
    Melee,
    Ranged,
    ArmourAndShield,
    Magic,
    Miscellaneous,
    CombatManoeuvre,
}

impl MythicFeatGroup {
    pub const ALL: [Self; 6] = [
        Self::Melee,
        Self::Ranged,
        Self::ArmourAndShield,
        Self::Magic,
        Self::Miscellaneous,
        Self::CombatManoeuvre,
    ];

    /// The heading as the guide prints it, without "Feats".
    pub fn name(self) -> &'static str {
        match self {
            Self::Melee => "Melee",
            Self::Ranged => "Ranged",
            Self::ArmourAndShield => "Armour and Shield",
            Self::Magic => "Magic",
            Self::Miscellaneous => "Miscellaneous",
            Self::CombatManoeuvre => "Combat Manoeuvre",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MythicAbility {
    pub key: MythicAbilityKey,
    pub name: String,
    /// The guide's group heading. Display only.
    pub group: MythicAbilityGroup,
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
    pub group: MythicFeatGroup,
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
    #[serde(default)]
    pub feat_lists: Vec<FeatList>,
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

/// What happens when a Build switches to this path at the switch rank.
///
/// A Mythic rank gives a general pick (a mythic ability or feat) and the old path's own grants (its rank row's
/// features and the picks in its slots). The switch keeps each kind up to its own rank, and can take named grants
/// of the old path away whatever their rank.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathSwitch {
    /// The paths a Build may switch from; empty: any (Devil: Aeon or Azata).
    #[serde(default)]
    pub from: BTreeSet<MythicPathKey>,
    /// The general picks of MR 1 up to this rank are kept (Legend: 2; Gold Dragon: 7).
    pub kept_through_rank: u8,
    /// The old path's own grants of MR 1 up to this rank are kept (Legend and Gold Dragon: 2; Devil: 7).
    pub path_kept_through_rank: u8,
    /// Old-path features lost whatever their rank (Devil: the Aeon's extra gaze, the Azata's dragon).
    #[serde(default)]
    pub dropped_features: BTreeSet<FeatureKey>,
    /// Old-path selections whose picks are lost whatever their rank (Devil: the Aeon's Enforcing Gaze).
    #[serde(default)]
    pub dropped_selections: BTreeSet<FeatureSelectionKey>,
    /// Features the switch itself grants, held from the switch on (Legend: +4 to every score).
    #[serde(default)]
    pub features: BTreeSet<FeatureKey>,
    /// Whether later Mythic ascensions still happen (Legend: no).
    pub further_ascensions: bool,
    /// The Character level cap after switching, when the switch raises the usual one (Legend: 40).
    #[serde(default)]
    pub level_cap: Option<u8>,
}

impl PathSwitch {
    /// The Mythic rank right after the switch: one above the general picks kept (Legend: 3).
    pub fn rank_after_switch(&self) -> u8 {
        self.kept_through_rank.saturating_add(1)
    }

    /// Whether a Build on `path` may take this switch.
    pub fn open_from(&self, path: Option<&MythicPathKey>) -> bool {
        self.from.is_empty() || path.is_some_and(|path| self.from.contains(path))
    }
}
