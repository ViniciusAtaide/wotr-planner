//! The game data the planner works from: races, skills, feats, classes, mythic paths and the progression facts.
//!
//! These types are the single shape of the data. The `libs/game-data/*.ron` files deserialise straight into them,
//! the database is loaded from and read back into them, and the browser receives them as JSON.
//!
//! Order carries no meaning anywhere in the data: catalogs are ordered by key, level and rank rows by their
//! number, and set-like children either are sets or get sorted. [`GameData::canonicalize`] puts a value in that
//! one order, so two `GameData`s with the same content compare equal however their files were written.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::bonus::{ItemBonus, ItemInteractions};
use crate::gear::GearItem;
use crate::keys::{
    ArchetypeKey, BackgroundKey, ClassKey, DeityKey, FeatKey, FeatListKey, FeatureKey,
    FeatureSelectionKey, MythicAbilityKey, MythicFeatKey, MythicPathKey, RaceKey,
    SelectionOptionKey, SkillKey, SubraceKey, WeaponKey,
};
use crate::parameter::{Parameter, ParameterKind, Repeat};
use crate::prerequisite::Prerequisite;
use crate::spells::{Spell, SpellList};

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
    /// The flat bonuses items give while held (`bonuses.ron`, tickets 119 and 120): spell DCs, caster level against
    /// spell resistance, concentration, initiative, saves and skills.
    #[serde(default)]
    pub bonuses: Vec<ItemBonus>,
    /// What the guide says about how some items combine with others (`interactions.ron`, ticket 124), at most one
    /// note per item.
    #[serde(default)]
    pub interactions: Vec<ItemInteractions>,
    /// The named items a character wears or wields, with their always-on bonuses (`gear.ron`, ticket 165). A
    /// Build holds them in its gear ([`Build::gear`](crate::Build::gear), ticket 166).
    #[serde(default)]
    pub gear: Vec<GearItem>,
    pub mythic_paths: Vec<MythicPath>,
    /// The backgrounds chosen at Build start (`backgrounds.ron`, the user's list).
    pub backgrounds: Vec<Background>,
    /// The deities a Build start may follow (`deities.ron`, the guide's table).
    pub deities: Vec<Deity>,
    /// The spell catalogue (`spells/<page>/level-<n>.ron`, ticket 121): reference data no Build choice names.
    /// Left out of the JSON the browser gets, which has no use for it (see [`GameData::spell_lists`]).
    #[serde(skip_serializing, default)]
    pub spells: Vec<Spell>,
    /// The spell lists (`spells/lists.ron`): which spells each class, archetype and Mythic path casts, at which
    /// level. Like [`spells`](Self::spells), never serialised: the browser's rules don't read them, and the MCP
    /// server reads the data files.
    #[serde(skip_serializing, default)]
    pub spell_lists: Vec<SpellList>,
}

impl GameData {
    /// Every class: the base classes, then the prestige classes.
    pub fn all_classes(&self) -> impl Iterator<Item = &Class> {
        self.classes.iter().chain(&self.prestige_classes)
    }

    /// [`all_classes`](Self::all_classes), to edit in place.
    pub fn all_classes_mut(&mut self) -> impl Iterator<Item = &mut Class> {
        self.classes.iter_mut().chain(&mut self.prestige_classes)
    }

    /// A class by key, base or prestige. A walk of the lists: the engine finds classes through its index instead.
    pub fn class(&self, key: &ClassKey) -> Option<&Class> {
        self.all_classes().find(|class| class.key == *key)
    }

    /// Every class's Feature selections: each class's own, then its archetypes', for every class in
    /// [`all_classes`](Self::all_classes) order.
    pub fn class_selections(&self) -> impl Iterator<Item = &FeatureSelection> {
        self.all_classes().flat_map(|class| {
            class.selections.iter().chain(
                class
                    .archetypes
                    .iter()
                    .flat_map(|archetype| &archetype.selections),
            )
        })
    }

    /// Every Feature selection: the [`class_selections`](Self::class_selections), then each path's.
    pub fn selections(&self) -> impl Iterator<Item = &FeatureSelection> {
        self.class_selections()
            .chain(self.mythic_paths.iter().flat_map(|path| &path.selections))
    }

    /// Every class's features: each class's own, then its archetypes', for every class in
    /// [`all_classes`](Self::all_classes) order.
    pub fn class_features(&self) -> impl Iterator<Item = &Feature> {
        self.all_classes().flat_map(|class| {
            class.features.iter().chain(
                class
                    .archetypes
                    .iter()
                    .flat_map(|archetype| &archetype.features),
            )
        })
    }

    /// A Selection option by key, from any selection's own options (the first, in
    /// [`selections`](Self::selections) order). A walk of the lists: the engine finds options through its index
    /// instead.
    pub fn selection_option(&self, key: &SelectionOptionKey) -> Option<&SelectionOption> {
        self.selections()
            .flat_map(|selection| &selection.options)
            .find(|option| option.key == *key)
    }

    /// Every archetype of every base class.
    pub fn archetypes(&self) -> impl Iterator<Item = &Archetype> {
        self.classes.iter().flat_map(|class| &class.archetypes)
    }

    /// Every feat list: each race's, each class's (base, then prestige), each of its archetypes', then each path's.
    pub fn feat_lists(&self) -> impl Iterator<Item = &FeatList> {
        let races = self.races.iter().flat_map(|race| &race.feat_lists);
        let classes = self.all_classes().flat_map(|class| {
            class.feat_lists.iter().chain(
                class
                    .archetypes
                    .iter()
                    .flat_map(|archetype| &archetype.feat_lists),
            )
        });
        races
            .chain(classes)
            .chain(self.mythic_paths.iter().flat_map(|path| &path.feat_lists))
    }

    /// How much of each kind of thing the data holds, labelled, as a quick check that seeding worked.
    pub fn counts(&self) -> [(&'static str, usize); 15] {
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
            ("Backgrounds", self.backgrounds.len()),
            ("Deities", self.deities.len()),
            ("Spells", self.spells.len()),
            ("Spell lists", self.spell_lists.len()),
            ("Gear", self.gear.len()),
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
            sort_by_key(&mut race.feat_lists, |l| l.key.clone());
            sort_by_key(&mut race.subraces, |s| s.key.clone());
            for subrace in &mut race.subraces {
                subrace.ability_modifiers.sort();
                subrace.feat_slots.sort();
            }
        }
        sort_by_key(&mut self.skills, |s| s.key.clone());
        sort_by_key(&mut self.weapons, |w| w.key.clone());
        sort_by_key(&mut self.feats, |f| f.key.clone());
        for feat in &mut self.feats {
            canonicalize_prerequisite(&mut feat.prerequisite);
        }
        for class in self.all_classes_mut() {
            class.canonicalize();
        }
        sort_by_key(&mut self.classes, |c| c.key.clone());
        sort_by_key(&mut self.prestige_classes, |c| c.key.clone());
        sort_by_key(&mut self.mythic_abilities, |a| a.key.clone());
        for ability in &mut self.mythic_abilities {
            canonicalize_prerequisite(&mut ability.prerequisite);
            ability.features.sort();
            ability.selections.sort();
        }
        sort_by_key(&mut self.mythic_feats, |f| f.key.clone());
        for feat in &mut self.mythic_feats {
            canonicalize_prerequisite(&mut feat.prerequisite);
        }
        for bonus in &mut self.bonuses {
            canonicalize_prerequisite(&mut bonus.requires);
        }
        sort_by_key(&mut self.bonuses, |b| {
            (
                b.item.clone(),
                b.target.clone(),
                b.amount,
                b.requires.clone(),
            )
        });
        sort_by_key(&mut self.interactions, |note| note.item.clone());
        sort_by_key(&mut self.gear, |item| item.key.clone());
        for item in &mut self.gear {
            item.bonuses.sort();
            item.other_effects.sort();
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
        sort_by_key(&mut self.backgrounds, |b| b.key.clone());
        sort_by_key(&mut self.deities, |d| d.key.clone());
        sort_by_key(&mut self.spells, |s| s.key.clone());
        sort_by_key(&mut self.spell_lists, |l| l.key.clone());
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
            option.levels.sort_by_key(|row| row.level);
            for row in &mut option.levels {
                row.features.sort();
                row.feat_slots.sort();
                row.selections.sort();
            }
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
/// rest of how a character progresses is rules in code ([`FIRST_ASCENSION_RANK`](crate::FIRST_ASCENSION_RANK),
/// [`PATH_CHOICE_RANK`](crate::PATH_CHOICE_RANK),
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
    /// The race's own feat lists, which its slots may name (a half-elf's Adaptability: Skill Focus as a bonus
    /// feat).
    #[serde(default)]
    pub feat_lists: Vec<FeatList>,
    /// Skill points it adds at every level, on top of the class's and Intelligence's (a human's Skilled: 1).
    #[serde(default)]
    pub skill_points: u8,
    #[serde(default)]
    pub subraces: Vec<Subrace>,
}

/// A subrace. The guide lists each subrace's own racial traits, so its ability modifiers and feat slots replace the
/// race's.
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
    /// Its feat slots at character level 1, in place of the race's (a half-elf's Dual Heritage and Kindred Raised
    /// list no Adaptability, so they have none). They may name the race's lists.
    #[serde(default)]
    pub feat_slots: Vec<FeatSlot>,
    /// Skill points it adds at every level, in place of the race's (a Human-Raised half-orc's: 1).
    #[serde(default)]
    pub skill_points: u8,
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

/// A background, chosen at Build start (the user's list: the saved guide pages don't list backgrounds, so it has
/// no guide link). Its skills become class skills and its feats are granted feats, held from the start with no
/// slot and no pick, as a class level row's are. Everything else it gives is its note.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Background {
    pub key: BackgroundKey,
    pub name: String,
    /// The group the game lists it under; `None` for the background "None".
    #[serde(default)]
    pub group: Option<BackgroundGroup>,
    /// What it gives that the planner doesn't model (a weapon's proficiency, +1 Fortitude), shown with the Build.
    #[serde(default)]
    pub note: String,
    /// Skills that become class skills.
    #[serde(default)]
    pub class_skills: BTreeSet<SkillKey>,
    /// Granted feats (the Martial Disciple's Improved Unarmed Strike). A parameterised one is held for every value.
    #[serde(default)]
    pub feats: BTreeSet<FeatKey>,
    /// Parameterised feats granted for one value (the Aldori Swordsman's Exotic Weapon Proficiency (Dueling
    /// Sword)).
    #[serde(default)]
    pub feats_for: BTreeSet<FeatValue>,
}

/// The groups the game lists backgrounds under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BackgroundGroup {
    Craftsman,
    Noble,
    Oblate,
    Regional,
    Scholar,
    StreetUrchin,
    Wanderer,
    Warrior,
}

impl BackgroundGroup {
    pub const ALL: [Self; 8] = [
        Self::Craftsman,
        Self::Noble,
        Self::Oblate,
        Self::Regional,
        Self::Scholar,
        Self::StreetUrchin,
        Self::Wanderer,
        Self::Warrior,
    ];

    /// The group as the game names it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Craftsman => "Craftsman",
            Self::Noble => "Noble",
            Self::Oblate => "Oblate",
            Self::Regional => "Regional",
            Self::Scholar => "Scholar",
            Self::StreetUrchin => "Street Urchin",
            Self::Wanderer => "Wanderer",
            Self::Warrior => "Warrior",
        }
    }
}

/// The Feature selection whose options are the domains: what a deity's domains and a blessing's domain name.
pub const DOMAIN_SELECTION: &str = "cleric/domain";
/// The feat a follower holds for an exotic favoured weapon, taken for that weapon (Achaekek: Sawtooth Saber).
pub const EXOTIC_WEAPON_PROFICIENCY: &str = "exotic-weapon-proficiency";
/// The feat a follower holds for the unarmed strike as favoured weapon (Irori).
pub const IMPROVED_UNARMED_STRIKE: &str = "improved-unarmed-strike";
/// The weapon that is the unarmed strike.
pub const UNARMED_STRIKE: &str = "unarmed-strike";

/// A deity of the guide's table (`character-creation#deities`). A Build start may follow one, or choose atheism
/// ([`Faith`](crate::Faith)); following one limits the character's alignment to the deity's, and the domains a
/// deity-limited selection offers to its domains ([`FeatureSelection::deity_domains`]). The deity's own alignment
/// isn't stored: the table doesn't give it, and a rule that needs it reads its alignment domains (a good deity offers
/// the Good domain).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Deity {
    pub key: DeityKey,
    pub name: String,
    pub guide: GuideRef,
    /// The alignments a follower may have: the table's ticks, within one step of the deity's (Godclaw: LG, LN, LE).
    pub alignments: BTreeSet<Alignment>,
    /// Its favoured weapons (Lamashtu: falchion and kukri). A feature with
    /// [`favoured_weapon_proficiency`](Feature::favoured_weapon_proficiency) makes a follower proficient: an exotic
    /// one is held as [`EXOTIC_WEAPON_PROFICIENCY`] for it, the unarmed strike as [`IMPROVED_UNARMED_STRIKE`], and a
    /// simple or martial one needs no feat.
    pub favoured_weapons: BTreeSet<WeaponKey>,
    /// The domains it offers: options of [`DOMAIN_SELECTION`].
    pub domains: BTreeSet<SelectionOptionKey>,
}

/// How the Build's deity limits what a Feature selection offers ([`FeatureSelection::deity_domains`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DeityDomains {
    /// Only options whose domain the deity offers (a cleric's domains, a warpriest's blessings).
    Offered,
    /// Only options whose domain the deity doesn't offer (the Separatist's second domain).
    NotOffered,
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
    /// What it deals as a weapon in hand, for the attack line ([`Engine::attack_line`](crate::Engine::attack_line));
    /// `None` for what has no dice of its own: what deals a spell's or a class feature's damage (rays, touch
    /// attacks, Kinetic Blast, Bomb) and the natural attacks, whose dice depend on what grants them.
    #[serde(default)]
    pub stats: Option<WeaponStats>,
}

/// A weapon's base stats, as a Medium character wields it: no enhancement, material or size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponStats {
    /// Its damage dice (a double weapon's: the first head's).
    pub damage: Dice,
    /// The lowest d20 roll that threatens a critical hit: 19 for 19–20, 20 for a weapon that threatens on a 20 only.
    pub threat: u8,
    /// The critical multiplier: 2 for ×2.
    pub multiplier: u8,
    pub handedness: Handedness,
    /// `Some` for a ranged weapon: thrown, or one that shoots.
    #[serde(default)]
    pub ranged: Option<Ranged>,
    /// Whether Weapon Finesse applies to it: the guide's list under Weapon Finesse.
    #[serde(default)]
    pub finesse: bool,
}

/// Dice to roll: `count` dice of `sides` sides (2d6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dice {
    pub count: u8,
    pub sides: u8,
}

impl std::fmt::Display for Dice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}d{}", self.count, self.sides)
    }
}

/// How many hands a weapon takes: a two-handed melee weapon adds 1½ times the Strength bonus to damage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Handedness {
    Light,
    OneHanded,
    TwoHanded,
}

/// What kind of ranged weapon it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Ranged {
    /// Thrown (a javelin): the Strength modifier adds to its damage.
    Thrown,
    /// It shoots (a bow, a crossbow): no Strength to damage.
    Projectile,
    /// It shoots, but the game's Strength trait adds the Strength modifier to its damage (the sling staff).
    ProjectileWithStrength,
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
    /// The Feature selections a pick of it gives one more slot of, each time it's taken, at the level it's taken
    /// ("You gain one additional hex": a witch's or a shaman's). The slot is of the one of these the character had a
    /// slot of first; a character that had a slot of none gets none. Empty: it gives no slot (Extra Rage, Extra
    /// Channel).
    #[serde(default)]
    pub extra_pick_of: BTreeSet<FeatureSelectionKey>,
    /// `None`: the guide lists no prerequisites, so there are none.
    #[serde(default)]
    pub prerequisite: Option<Prerequisite>,
}

impl Feat {
    /// The key prefix of the metamagic feats: the guide names each "Metamagic (…)".
    pub const METAMAGIC_PREFIX: &'static str = "metamagic-";

    /// Whether it's a metamagic feat, a value of a `Metamagic` pick (Favorite Metamagic): its key starts
    /// [`METAMAGIC_PREFIX`](Self::METAMAGIC_PREFIX). A rule on the key, not a column (no derived values).
    pub fn is_metamagic(&self) -> bool {
        self.key.as_str().starts_with(Self::METAMAGIC_PREFIX)
    }
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
    /// A bonus slot whose list comes from a held option: the list in force of each option held that `selection`
    /// offers (a sorcerer's bloodline feat; a Crossblooded sorcerer's slot offers both bloodlines' lists).
    /// Checked after the level's Feature selections, since it depends on them.
    OptionList {
        selection: FeatureSelectionKey,
        /// The option level whose list it offers, whatever level the option has reached (a slayer's combat style
        /// feat at 2 or 4: the style's 2nd-level list, even when a ranger's levels have taken the style further).
        /// `None`: the list in force at the option's level, and left out when written (the data files write a
        /// slot on one line, `OptionList(selection: …)`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        at_level: Option<u8>,
        /// What the sheet calls the slot, when not its selection's name and "feat" (the kineticist's bonus-feat
        /// wild talent's: "Wild talent feat", though it offers the Elemental Focus's lists). `None`: the
        /// selection's name and "feat" (a sorcerer's "Bloodline feat"), and left out when written.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
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
    /// Whether its spells are cast spontaneously rather than prepared (sorcerer, bard). Dragon Disciple's entry
    /// checks it ([`Prerequisite::SpontaneousSpellLevel`]).
    #[serde(default)]
    pub spontaneous: bool,
    /// The ability its spells' save DCs, concentration and bonus spells use (a sorcerer's Charisma); `None` for a
    /// class that casts no spells of its own (a fighter, every prestige class: they advance another's).
    #[serde(default)]
    pub casting_ability: Option<Ability>,
    /// Prestige classes: how many classes the rows' spellcasting advancement serves (Mystic Theurge: one of each
    /// spell type; Loremaster: one, arcane or divine).
    #[serde(default)]
    pub advances: Advancement,
    /// Other classes this class's levels count as, and toward what (Eldritch Knight: fighter levels for feat
    /// prerequisites).
    #[serde(default)]
    pub counts_as: BTreeSet<CountsAs>,
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

/// How many classes a prestige class's spellcasting advancement serves. Its rows say which spell types a level
/// advances (`advances_arcane`, `advances_divine`); the Build names the classes on entry
/// ([`ClassChoice::advances`](crate::ClassChoice::advances)).
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Advancement {
    /// One class for each spell type the rows advance (Mystic Theurge: an arcane class and a divine class).
    #[default]
    Each,
    /// One class, of either spell type the rows advance ("advances one class, arcane or divine": Hellknight
    /// Signifer, Loremaster, Winter Witch).
    Either,
}

/// One "counts as" fact: a class's levels count as levels of `class`, toward one thing only.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CountsAs {
    pub class: ClassKey,
    pub toward: CountsToward,
}

/// What a "counts as" fact's levels count toward.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CountsToward {
    /// The class level a feat's prerequisites see (Diverse Training: Eldritch Knight levels as fighter levels).
    FeatPrerequisites,
    /// The class's rows that grant this feature are reached at the two classes' levels together (Frost Power:
    /// Winter Witch levels stack with witch levels for access to major and grand hexes). Only while the class is
    /// held.
    Feature(FeatureKey),
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
    /// `None`: casts like the base class. `Some`: spontaneously or not (Eldritch Scion: a spontaneous magus).
    #[serde(default)]
    pub spontaneous: Option<bool>,
    /// `None`: casts with the base class's ability. `Some`: replaces it (Sage Sorcerer: Intelligence).
    #[serde(default)]
    pub casting_ability: Option<Ability>,
    /// `None`: the base class's Fortitude progression. `Some`: replaces it (Arcane Enforcer: poor), and with it
    /// the class's [`save_exceptions`](Class::save_exceptions) for that save, which are cells of the class's own.
    #[serde(default)]
    pub fort: Option<SaveProgression>,
    /// `None`: the base class's Reflex progression. `Some`: replaces it, as [`fort`](Self::fort) does.
    #[serde(default)]
    pub reflex: Option<SaveProgression>,
    /// `None`: the base class's Will progression. `Some`: replaces it, as [`fort`](Self::fort) does (Arcane
    /// Enforcer: good).
    #[serde(default)]
    pub will: Option<SaveProgression>,
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
    /// Feats granted for one value: see [`ClassLevelRow::feats_for`].
    #[serde(default)]
    pub feats_for: BTreeSet<FeatValue>,
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

    /// One save at a class level, taken with `archetype` (one of the class's, or none): the archetype's
    /// progression when it replaces the class's for that save, else the guide's cell when it's an exception, else
    /// the class's progression's formula (`prestige` picks the prestige formula).
    pub fn save_bonus(
        &self,
        save: Save,
        class_level: u8,
        prestige: bool,
        archetype: Option<&Archetype>,
    ) -> u8 {
        if let Some(progression) = archetype.and_then(|archetype| archetype.save_progression(save))
        {
            return progression.save(class_level, prestige);
        }
        self.save_exceptions
            .iter()
            .find(|exception| exception.level == class_level && exception.save == save)
            .map_or_else(
                || self.save_progression(save).save(class_level, prestige),
                |exception| exception.value,
            )
    }
}

impl Archetype {
    /// The progression that replaces its class's for one save, if any.
    pub fn save_progression(&self, save: Save) -> Option<SaveProgression> {
        match save {
            Save::Fortitude => self.fort,
            Save::Reflex => self.reflex,
            Save::Will => self.will,
        }
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
    /// Parameterised feats granted for one value, as `feats` grants them otherwise (Mantis Zealot: Weapon Focus
    /// (Sawtooth Saber)).
    #[serde(default)]
    pub feats_for: BTreeSet<FeatValue>,
    #[serde(default)]
    pub feat_slots: Vec<FeatSlot>,
    /// One Feature selection slot per entry.
    #[serde(default)]
    pub selections: Vec<FeatureSelectionKey>,
}

/// A parameterised feat for one value (Weapon Focus (Sawtooth Saber)).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatValue {
    pub feat: FeatKey,
    pub value: Parameter,
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
    /// They count toward ability-score prerequisites like point buy and level increases do, after the gear's
    /// bonuses (ticket 166). A feature granted more than once adds them once per grant (Strength of the Abyss: +2 at 9, 13 and 17).
    #[serde(default)]
    pub ability_bonuses: Vec<AbilityBonus>,
    /// The row at one option level of the option held in a selection, which the feature grants while it's held
    /// (Bloodline Ascendance: the bloodline's 20th-level row).
    #[serde(default)]
    pub option_row: Option<OptionRow>,
    /// Set when the game asks for this choice again every day (Mixed Bloodline, a shaman's Wandering Spirit, an
    /// Abjurer's Resistance): it isn't part of a Build, so the sheet shows a note instead of a pick.
    #[serde(default)]
    pub daily_choice: bool,
    /// For prerequisites, the base attack bonus is at least this while the feature is held (Perfect Mind: 15).
    #[serde(default)]
    pub bab_floor: Option<u8>,
    /// For prerequisites, the ranks in every skill are at least the Mythic rank while the feature is held (Perfect
    /// Mind).
    #[serde(default)]
    pub skill_ranks_at_least_mythic_rank: bool,
    /// While the feature is held, the character is proficient with the favoured weapons of the Build's deity (Cleric
    /// Proficiencies): see [`Deity::favoured_weapons`].
    #[serde(default)]
    pub favoured_weapon_proficiency: bool,
}

/// One option level's row of whatever option a selection holds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionRow {
    pub selection: FeatureSelectionKey,
    pub level: u8,
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
    /// The classes whose levels add up to the option level of an option held through this selection (a
    /// sorcerer's bloodline: sorcerer and Dragon Disciple levels). An option's rows apply as its level reaches
    /// them, and an option taken late gets every row up to its level at once.
    #[serde(default)]
    pub option_levels: BTreeSet<ClassKey>,
    /// Option levels whose rows an option held through this selection doesn't get (Seeker: 3, 9 and 15); the
    /// archetype's own rows replace them.
    #[serde(default)]
    pub skipped_levels: BTreeSet<u8>,
    /// Set when an option held through this selection gives only its feat lists and its arcana (Crossblooded's
    /// second bloodline).
    #[serde(default)]
    pub feat_lists_only: bool,
    /// Set when a level grants this selection only to a character holding none of the options it offers (Dragon
    /// Disciple: a sorcerer's Draconic bloodline continues, anyone else picks one).
    #[serde(default)]
    pub only_without_option: bool,
    /// Set on a selection that offers the pools of the options held that this selection offers (an oracle's
    /// revelations come from the mystery held), beside any options of its own or shared (the Dual-Cursed Oracle's
    /// Misfortune and Fortune).
    #[serde(default)]
    pub pool_of: Option<FeatureSelectionKey>,
    /// How many option levels an option held through it lags behind the levels of its classes (the Separatist's
    /// second domain: "Higher level abilities are delayed by two class levels").
    #[serde(default)]
    pub levels_behind: u8,
    /// Where an option held through it gets its option level when the selection names no classes.
    #[serde(default)]
    pub level_from: Option<LevelFrom>,
    /// With `level_from: OtherOptions`, an option held elsewhere whose option level counts as one of the other
    /// options' (Impossible Domain: a Living Deity's way, so its extra domain runs on arcanist levels from the pick,
    /// before the way's own domains come at 20).
    #[serde(default)]
    pub level_also_from: Option<SelectionOptionKey>,
    /// Set when an option it has given once can't be picked through it again, however often the option itself
    /// repeats (Expanded Element: the 15th-level element can't be the 7th-level one).
    #[serde(default)]
    pub once_per_option: bool,
    /// How the Build's deity limits the options it offers, by each option's domain ([`SelectionOption::domain`]):
    /// only the deity's domains (a cleric's), or only the others (the Separatist's second). `None`: the deity
    /// doesn't matter (a druid's domains, Impossible Domain).
    #[serde(default)]
    pub deity_domains: Option<DeityDomains>,
}

/// Where an option held through a selection naming no classes gets its option level.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LevelFrom {
    /// The highest option level of the other options held that the selection offers, so it grows with whichever
    /// class levels those (Impossible Domain: the character's other domains; Second Mystery: the first mystery).
    OtherOptions,
    /// The option level of the option whose row granted the slot (Maneuver Mastery's maneuver: its revelation's,
    /// by oracle levels or a Living Deity's arcanist levels).
    GrantingOption,
    /// The Mythic ranks reached since the option was picked at a rank, that rank counting as 1, for a path's
    /// selection (Life-Bonding Friendship: "a teamwork feat this rank and the next rank", its rows 1 and 2). Its
    /// rows' slots belong to the rank that reaches them, after the path's own slots there.
    MythicRanks,
}

/// A family of options of which a character may hold only one, which [`Prerequisite::NoOtherOptionOfGroup`]
/// checks ("You can only take a single draconic bloodline").
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OptionGroup {
    /// The sorcerer's ten Draconic bloodlines.
    Draconic,
    /// The sorcerer's four Elemental bloodlines.
    Elemental,
    /// The bloodrager's ten Draconic bloodlines: one at most, as the sorcerer's, but apart from them.
    BloodragerDraconic,
    /// The bloodrager's four Elemental bloodlines.
    BloodragerElemental,
}

impl OptionGroup {
    pub const ALL: [Self; 4] = [
        Self::Draconic,
        Self::Elemental,
        Self::BloodragerDraconic,
        Self::BloodragerElemental,
    ];

    /// The group as the guide names it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Draconic => "Draconic",
            Self::Elemental => "Elemental",
            Self::BloodragerDraconic => "Draconic (Bloodrager)",
            Self::BloodragerElemental => "Elemental (Bloodrager)",
        }
    }
}

impl FeatureSelection {
    /// Whether a pick of `option` may fill this selection's slot: one of its own options, or one it shares. (A
    /// pool selection's offer depends on what's held: see `Engine`.)
    pub fn offers(&self, option: &SelectionOptionKey) -> bool {
        self.shared_options.contains(option) || self.options.iter().any(|own| &own.key == option)
    }

    /// Every option it offers, own and shared, in key order.
    pub fn offered(&self) -> BTreeSet<&SelectionOptionKey> {
        self.options
            .iter()
            .map(|option| &option.key)
            .chain(&self.shared_options)
            .collect()
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
    /// What the pick names when the option has no chosen feat (the shaman's Battle Master: a weapon, for the feats
    /// its rows grant). With a chosen feat, the pick names a value of that feat's kind instead.
    #[serde(default)]
    pub parameter: Option<ParameterKind>,
    /// The values the pick may name; empty: any of its kind (the Zen Archer's Way of the Bow: a longbow or a
    /// shortbow).
    #[serde(default)]
    pub values: BTreeSet<Parameter>,
    /// How many times it can be taken (Increased Damage Reduction: three times; Combat Trick: any number).
    /// `PerValue` only with a chosen feat: once for each value of it (the Skill Focus talents: once per skill).
    #[serde(default)]
    pub repeat: Repeat,
    #[serde(default)]
    pub prerequisite: Option<Prerequisite>,
    /// What the option gives by option level (a bloodline's powers at 1, 3, 9, 15 and 20), each level at most
    /// once. The selection it's held through says which class levels make up its level.
    #[serde(default)]
    pub levels: Vec<OptionLevelRow>,
    /// The selection whose options this option opens to a pool selection (a mystery's revelations).
    #[serde(default)]
    pub pool: Option<FeatureSelectionKey>,
    /// Picking this option skips the row, at the level reached, of the option held in that selection (a
    /// Primalist's two rage powers instead of the bloodline power).
    #[serde(default)]
    pub replaces_row_of: Option<FeatureSelectionKey>,
    /// The family it belongs to, of which a character holds only one when its prerequisite says so (the ten
    /// Draconic bloodlines).
    #[serde(default)]
    pub group: Option<OptionGroup>,
    /// Its arcana: one of its rows' features, which a holding that gives only feat lists still gives
    /// (Crossblooded's second bloodline: "Gains the arcana and spells from both bloodlines").
    #[serde(default)]
    pub arcana: Option<FeatureKey>,
    /// The domain it stands for, an option of [`DOMAIN_SELECTION`], when it isn't one itself (a warpriest's Air
    /// blessing: the Air domain). A deity-limited selection checks it against the deity's domains.
    #[serde(default)]
    pub domain: Option<SelectionOptionKey>,
}

impl SelectionOption {
    /// The domain a deity-limited selection checks it by: its `domain`, or itself.
    pub fn deity_domain(&self) -> &SelectionOptionKey {
        self.domain.as_ref().unwrap_or(&self.key)
    }
}

/// What an option gives at one option level.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionLevelRow {
    pub level: u8,
    #[serde(default)]
    pub features: Vec<FeatureKey>,
    /// Granted feats, as on a class level row (a bloodline's Wings at 15).
    #[serde(default)]
    pub feats: BTreeSet<FeatKey>,
    /// Parameterised feats granted for the value the option's pick named, as its
    /// [`chosen_feat`](SelectionOption::chosen_feat) is (Weapon Mastery: Improved Critical at 8 in the weapon its
    /// Weapon Focus was taken for).
    #[serde(default)]
    pub chosen_feats: BTreeSet<FeatKey>,
    /// Feat slots, as on a class level row (the War domain's bonus Combat feat at 8).
    #[serde(default)]
    pub feat_slots: Vec<FeatSlot>,
    /// One Feature selection slot per entry (the Arcane bloodline's school at 15).
    #[serde(default)]
    pub selections: Vec<FeatureSelectionKey>,
    /// The option's feat list from this level on, for [`FeatSlot::OptionList`] slots (a bloodline's feats; a
    /// ranger style's list per stage).
    #[serde(default)]
    pub feat_list: Option<FeatListKey>,
    /// Skills that are class skills while the row is held (an oracle's Flame mystery: Athletics and Mobility).
    #[serde(default)]
    pub class_skills: BTreeSet<SkillKey>,
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
    /// Features the ability grants while it's held (Bloodline Ascendance).
    #[serde(default)]
    pub features: Vec<FeatureKey>,
    /// Feature selection slots the ability grants when it's picked (Second Bloodline: another bloodline).
    #[serde(default)]
    pub selections: Vec<FeatureSelectionKey>,
    /// What the ability is taken for, when the guide says to choose one (Favorite Metamagic: a metamagic). Each pick
    /// of it then names a value of this kind, or none yet ([`ParameterKind::may_stay_open`]). An ability is held
    /// once, whatever its value.
    #[serde(default)]
    pub parameter: Option<ParameterKind>,
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
    /// What the mythic feat is taken for, as on a feat (Improved Critical (Mythic): a weapon).
    #[serde(default)]
    pub parameter: Option<ParameterKind>,
    /// How many times it can be taken, as on a feat.
    #[serde(default)]
    pub repeat: Repeat,
    /// The race's ability penalties no longer apply to the character's own scores while it's held (Destiny Beyond
    /// Birth).
    #[serde(default)]
    pub removes_racial_penalty: bool,
    /// The one more pick a pick of it gives, at the Mythic rank it's taken (Extra Feat: "You can select any normal
    /// feat"). `None`: it gives none.
    #[serde(default)]
    pub extra_pick: Option<MythicExtraPick>,
    #[serde(default)]
    pub prerequisite: Option<Prerequisite>,
}

/// What the slot a mythic feat gives offers ([`MythicFeat::extra_pick`], [`SlotId::MythicExtra`](crate::SlotId)).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MythicExtraPick {
    /// A feat slot that offers what the general slot offers: any feat whose prerequisites are met (Extra Feat).
    Feat,
    /// A mythic ability slot that offers what a rank's general pick does (Extra Mythic Ability).
    MythicAbility,
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

impl MythicPath {
    /// The path's row at a Mythic rank; `None` at a rank its table skips.
    pub fn row(&self, rank: u8) -> Option<&MythicRankRow> {
        self.ranks.iter().find(|row| row.rank == rank)
    }
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

    /// The Mythic rank from which a switch without further ascensions (Legend) holds no more ranks: the switch rank,
    /// so the ranks from it on are never reached. `None` when the ascensions go on.
    pub fn ends_ascensions_at(&self) -> Option<u8> {
        (!self.further_ascensions).then_some(crate::PATH_SWITCH_RANK)
    }

    /// Whether the switch keeps a grant of a rank before it by its rank alone: a general pick (`from_path: false`)
    /// up to [`kept_through_rank`](Self::kept_through_rank), the old path's own grant up to
    /// [`path_kept_through_rank`](Self::path_kept_through_rank). A grant it drops by name goes anyway
    /// ([`drops`](Self::drops)).
    pub fn keeps(&self, rank: u8, from_path: bool) -> bool {
        let kept_through = if from_path {
            self.path_kept_through_rank
        } else {
            self.kept_through_rank
        };
        rank <= kept_through
    }

    /// Whether the switch takes a grant of a rank before it away: a general pick past the general cut-off, the old
    /// path's grant (`from_path`) past the path cut-off ([`keeps`](Self::keeps)), or the old path's `feature` or a
    /// pick in its `selection` that the switch drops by name whatever its rank.
    pub fn drops(
        &self,
        rank: u8,
        from_path: bool,
        feature: Option<&FeatureKey>,
        selection: Option<&FeatureSelectionKey>,
    ) -> bool {
        !self.keeps(rank, from_path)
            || (from_path
                && (feature.is_some_and(|key| self.dropped_features.contains(key))
                    || selection.is_some_and(|key| self.dropped_selections.contains(key))))
    }

    /// Whether a Build on `path` may take this switch.
    pub fn open_from(&self, path: Option<&MythicPathKey>) -> bool {
        self.from.is_empty() || path.is_some_and(|path| self.from.contains(path))
    }
}
