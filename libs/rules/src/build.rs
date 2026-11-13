//! A Build: the choices a player makes, and nothing derived from them.
//!
//! Everything is stored as game-data text keys, so a Build survives edits to the data. Its link holds a hash of each key,
//! which the game data's keys turn back into the key (see the codec). A link may name things that no longer exist; it
//! then opens nothing ([`Engine::open_link`](crate::Engine::open_link)).
//!
//! The rules no game data changes are kept by the types and edits rather than checked by the engine: a
//! [`PointBuy`] holds only scores that fit, and [`Build::apply`] never puts ranks in a skill above the character
//! level or an ability increase on a level without one ([`Build::keeps_fixed_rules`]). A link that breaks them
//! doesn't decode.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::choice::Choice;
use crate::game_data::{Ability, Alignment};
use crate::gear::GearSlot;
use crate::keys::{
    ArchetypeKey, BackgroundKey, ClassKey, DeityKey, FeatKey, GearKey, MythicAbilityKey,
    MythicFeatKey, MythicPathKey, RaceKey, SelectionOptionKey, SkillKey, SubraceKey,
};
use crate::parameter::Parameter;

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
    /// The first ascension (MR 1): the path whose path-linked ability the character takes at MR 1 (its MR 1 row),
    /// with that row's follow-ups at MR 2 (its MR 2 row), whatever path it chooses at MR 3. `None` reads as the path
    /// chosen at MR 3, as every link from before the choice existed does.
    #[serde(default)]
    pub first_ascension: Option<MythicPathKey>,
    /// The mythic path chosen at the path choice rank (MR 3).
    pub mythic_path: Option<MythicPathKey>,
    /// The path switched to at the switch rank (MR 8), if any.
    pub path_switch: Option<MythicPathKey>,
    /// Feats, Feature selections, mythic abilities and mythic feats, keyed by the slot they fill. A slot's id says
    /// which Level-up or Mythic ascension it belongs to.
    pub picks: BTreeMap<SlotId, Pick>,
    /// The gear (ticket 166): none unless the player adds it. It belongs to no level: an item counts at every one.
    #[serde(default)]
    pub gear: Gear,
    /// The character level of each Mythic rank taken later than the rank table's level (ticket 171), by rank. The
    /// table's level is the earliest a rank comes at, and a rank without an entry comes there, or with the rank
    /// below it when that one comes later: ranks keep their order
    /// ([`Engine::rank_level`](crate::Engine::rank_level)). Empty for every Build that moves no rank.
    #[serde(default)]
    pub rank_levels: BTreeMap<u8, u8>,
}

/// The items a Build holds: one worn in each place at most, and any number of usable ones (the manuals and tomes,
/// an elixir, a potion), each once.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gear {
    /// The item worn in each place. A place takes the items of its slots ([`GearPlace::takes`]).
    pub worn: BTreeMap<GearPlace, GearKey>,
    /// The usable items used ([`GearSlot::Usable`]): they share no place, so a Build holds as many as it likes.
    pub used: BTreeSet<GearKey>,
}

impl Gear {
    pub fn is_empty(&self) -> bool {
        self.worn.is_empty() && self.used.is_empty()
    }

    /// Every item held with where it is, in the order the walk counts them and the link writes them: the worn ones
    /// in place order, then the used ones in key order.
    pub fn items(&self) -> impl Iterator<Item = (GearSpot, &GearKey)> {
        let worn = self
            .worn
            .iter()
            .map(|(place, item)| (GearSpot::Worn(*place), item));
        let used = self
            .used
            .iter()
            .map(|item| (GearSpot::Used(item.clone()), item));
        worn.chain(used)
    }

    /// The item held at a spot, if any.
    pub fn at(&self, spot: &GearSpot) -> Option<&GearKey> {
        match spot {
            GearSpot::Worn(place) => self.worn.get(place),
            GearSpot::Used(item) => self.used.get(item),
        }
    }
}

/// A place a Build wears one item in. The guide's slots map onto them ([`GearPlace::takes`]): a slot is a place,
/// except that rings have two and a robe and an armour share the body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GearPlace {
    Belt,
    Boots,
    Bracers,
    Cloak,
    Glasses,
    Gloves,
    Headwear,
    Necklace,
    FirstRing,
    SecondRing,
    /// A robe or an armour: only one is worn.
    Body,
    Weapon,
}

impl GearPlace {
    pub const ALL: [Self; 12] = [
        Self::Belt,
        Self::Boots,
        Self::Bracers,
        Self::Cloak,
        Self::Glasses,
        Self::Gloves,
        Self::Headwear,
        Self::Necklace,
        Self::FirstRing,
        Self::SecondRing,
        Self::Body,
        Self::Weapon,
    ];

    /// The place as the Gear tab names its row: "Belt", "Ring 1", "Body".
    pub fn name(self) -> &'static str {
        match self {
            Self::Belt => "Belt",
            Self::Boots => "Boots",
            Self::Bracers => "Bracers",
            Self::Cloak => "Cloak",
            Self::Glasses => "Glasses",
            Self::Gloves => "Gloves",
            Self::Headwear => "Headwear",
            Self::Necklace => "Necklace",
            Self::FirstRing => "Ring 1",
            Self::SecondRing => "Ring 2",
            Self::Body => "Body",
            Self::Weapon => "Weapon",
        }
    }

    /// The guide's slots whose items the place takes: its own, both rings the ring slot's, the body a robe or an
    /// armour. No place takes a usable item ([`Gear::used`]).
    pub fn slots(self) -> &'static [GearSlot] {
        match self {
            Self::Belt => &[GearSlot::Belt],
            Self::Boots => &[GearSlot::Boots],
            Self::Bracers => &[GearSlot::Bracers],
            Self::Cloak => &[GearSlot::Cloak],
            Self::Glasses => &[GearSlot::Glasses],
            Self::Gloves => &[GearSlot::Gloves],
            Self::Headwear => &[GearSlot::Headwear],
            Self::Necklace => &[GearSlot::Necklace],
            Self::FirstRing | Self::SecondRing => &[GearSlot::Ring],
            Self::Body => &[GearSlot::Robe, GearSlot::Armour],
            Self::Weapon => &[GearSlot::Weapon],
        }
    }

    /// Whether the place takes an item of a slot.
    pub fn takes(self, slot: GearSlot) -> bool {
        self.slots().contains(&slot)
    }

    /// The other ring place, for a ring place: an item is worn once, so one ring isn't on both.
    pub fn other_ring(self) -> Option<Self> {
        match self {
            Self::FirstRing => Some(Self::SecondRing),
            Self::SecondRing => Some(Self::FirstRing),
            _ => None,
        }
    }
}

/// Where a Build holds one item: worn in a place, or used (a usable item, which is its own spot, since it shares
/// none).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GearSpot {
    Worn(GearPlace),
    Used(GearKey),
}

/// The choices made before level 1.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildStart {
    pub race: Option<RaceKey>,
    pub subrace: Option<SubraceKey>,
    /// The ability a "+2 to one ability score" racial modifier goes to.
    pub racial_ability: Option<Ability>,
    pub alignment: Option<Alignment>,
    /// `None` until one is chosen; the background "None" is a key like any other.
    pub background: Option<BackgroundKey>,
    /// The deity followed, or atheism; `None` until one is chosen.
    #[serde(default)]
    pub deity: Option<Faith>,
    /// The point-buy scores, before racial modifiers.
    pub ability_scores: PointBuy,
}

/// What a character believes in, chosen at Build start: a deity of the game data, or none at all. Atheism is a
/// choice of its own, not the absence of one: it allows every alignment, the Living Deity needs it, and a class that
/// serves a deity refuses it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Faith {
    Deity(DeityKey),
    Atheism,
}

impl Faith {
    /// The deity followed, if any.
    pub fn deity(&self) -> Option<&DeityKey> {
        match self {
            Faith::Deity(key) => Some(key),
            Faith::Atheism => None,
        }
    }
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

/// The point-buy budget at Build start.
pub const POINT_BUY_BUDGET: u8 = 25;

/// The point-buy scores: each from 7 to 18, costing at most [`POINT_BUY_BUDGET`] points (Pathfinder's table; 7 to 9
/// refund points). Only scores that fit make one, so a Build never holds a point buy the planner can't make. In a
/// link it's the six scores, as [`AbilityScores`] are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "AbilityScores", into = "AbilityScores")]
pub struct PointBuy(AbilityScores);

impl Default for PointBuy {
    /// All 10s: nothing spent.
    fn default() -> Self {
        Self(AbilityScores::default())
    }
}

impl PointBuy {
    pub fn scores(&self) -> AbilityScores {
        self.0
    }

    pub fn get(&self, ability: Ability) -> u8 {
        self.0.get(ability)
    }

    /// The points spent (less those 7 to 9 refund), at most the budget.
    pub fn cost(&self) -> i8 {
        AbilityScores::ABILITIES
            .into_iter()
            .map(|ability| point_buy_cost(self.get(ability)).unwrap_or(0))
            .sum()
    }

    /// The point buy with one score changed, if it still fits.
    pub fn with(&self, ability: Ability, score: u8) -> Option<Self> {
        let mut scores = self.0;
        *scores.get_mut(ability) = score;
        Self::try_from(scores).ok()
    }
}

impl TryFrom<AbilityScores> for PointBuy {
    type Error = NotAPointBuy;

    fn try_from(scores: AbilityScores) -> Result<Self, NotAPointBuy> {
        let mut cost = 0i16;
        for ability in AbilityScores::ABILITIES {
            cost += i16::from(point_buy_cost(scores.get(ability)).ok_or(NotAPointBuy)?);
        }
        if cost > i16::from(POINT_BUY_BUDGET) {
            return Err(NotAPointBuy);
        }
        Ok(Self(scores))
    }
}

impl From<PointBuy> for AbilityScores {
    fn from(point_buy: PointBuy) -> Self {
        point_buy.0
    }
}

/// Scores that make no point buy: one outside 7 to 18, or more than the budget in all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NotAPointBuy;

impl fmt::Display for NotAPointBuy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "a point buy has every score from 7 to 18 and costs at most {POINT_BUY_BUDGET} points"
        )
    }
}

/// The cost of one point-buy score (Pathfinder's table; 7 to 9 refund points), or `None` outside 7..=18.
fn point_buy_cost(score: u8) -> Option<i8> {
    const COSTS: [i8; 12] = [-4, -2, -1, 0, 1, 2, 3, 5, 7, 10, 13, 17];
    COSTS.get(usize::from(score.checked_sub(7)?)).copied()
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
    /// A Feat slot from the mythic path's rank table, then those the rank reaches in the rows of options levelled
    /// by Mythic rank (Life-Bonding Friendship's teamwork feat), numbered on after the table's.
    PathFeat { rank: u8, index: u8 },
    /// A Feature selection slot from the mythic path's rank table, then, as for [`SlotId::PathFeat`], those of
    /// options levelled by Mythic rank.
    PathSelection { rank: u8, index: u8 },
    /// A Feature selection slot from the rows of options held, reached at a character level (the Arcane
    /// bloodline's school at option level 15), in the order the level reaches them.
    OptionSelection { level: u8, index: u8 },
    /// A Feature selection slot granted by the general mythic pick of a Mythic rank (Second Bloodline).
    MythicSelection { rank: u8, index: u8 },
    /// A Feat slot from the rows of options held, reached at a character level (the War domain's bonus Combat
    /// feat at option level 8), in the order the level reaches them.
    OptionFeat { level: u8, index: u8 },
    /// A Feature selection slot that a feat picked at a character level gives (Extra Rage Power: one more rage
    /// power), in the order the level holds those feats, after the slots of the options' rows. A feat picked at a
    /// Mythic rank gives it at the rank's character level.
    FeatSelection { level: u8, index: u8 },
    /// The slot that the mythic feat of a Mythic rank's general pick gives
    /// ([`MythicFeat::extra_pick`](crate::MythicFeat::extra_pick)): a feat slot (Extra Feat) or a mythic ability slot
    /// (Extra Mythic Ability), after the general pick. A rank has one general pick, so one such slot at most.
    MythicExtra { rank: u8 },
}

impl SlotId {
    /// The Mythic rank, for slots that belong to a Mythic ascension.
    pub fn rank(&self) -> Option<u8> {
        match *self {
            SlotId::Feat { .. }
            | SlotId::Selection { .. }
            | SlotId::OptionSelection { .. }
            | SlotId::OptionFeat { .. }
            | SlotId::FeatSelection { .. } => None,
            SlotId::Mythic { rank }
            | SlotId::PathFeat { rank, .. }
            | SlotId::PathSelection { rank, .. }
            | SlotId::MythicSelection { rank, .. }
            | SlotId::MythicExtra { rank } => Some(rank),
        }
    }

    /// The character level, for slots that belong to a Level-up (an option row's slot belongs to the level that
    /// reaches the row).
    pub fn level(&self) -> Option<u8> {
        match *self {
            SlotId::Feat { level, .. }
            | SlotId::Selection { level, .. }
            | SlotId::OptionSelection { level, .. }
            | SlotId::OptionFeat { level, .. }
            | SlotId::FeatSelection { level, .. } => Some(level),
            _ => None,
        }
    }
}

/// What fills a slot. A pick of a parameterised feat, mythic feat or mythic ability names what it's taken for (Weapon
/// Focus (Longsword); Favorite Metamagic (Persistent Spell)), and so does a pick of an option that takes a value (the
/// rogue talent Weapon Focus).
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
    /// A mythic ability, with its value when it takes one (Favorite Metamagic (Persistent Spell)); `None` too while
    /// a value that may stay open isn't chosen yet ([`ParameterKind::may_stay_open`](crate::ParameterKind::may_stay_open)).
    MythicAbility {
        ability: MythicAbilityKey,
        parameter: Option<Parameter>,
    },
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

    /// A mythic ability taken for nothing in particular.
    pub fn mythic_ability(ability: MythicAbilityKey) -> Self {
        Pick::MythicAbility {
            ability,
            parameter: None,
        }
    }

    /// A parameterised mythic ability taken for `value`.
    pub fn mythic_ability_for(ability: MythicAbilityKey, value: Parameter) -> Self {
        Pick::MythicAbility {
            ability,
            parameter: Some(value),
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
            Pick::MythicAbility { ability, .. } => ability.as_str(),
            Pick::MythicFeat { feat, .. } => feat.as_str(),
        }
    }

    /// What the pick is taken for, if anything.
    pub fn parameter(&self) -> Option<&Parameter> {
        match self {
            Pick::Feat { parameter, .. }
            | Pick::SelectionOption { parameter, .. }
            | Pick::MythicAbility { parameter, .. }
            | Pick::MythicFeat { parameter, .. } => parameter.as_ref(),
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
    /// Set the first ascension's path (MR 1); `None` goes back to the MR 3 path's.
    FirstAscension(Option<MythicPathKey>),
    MythicPath(Option<MythicPathKey>),
    /// Set the Path switch ([`Engine::apply`](crate::Engine::apply) also drops the picks of the ranks a switch
    /// without further ascensions ends: Legend's MR 8 to 10).
    PathSwitch(Option<MythicPathKey>),
    /// Remove the last Level-up, with the picks of its slots ([`Engine::apply`](crate::Engine::apply) also drops
    /// the picks of the Mythic ranks it reached, and the path or the switch at a rank it reached).
    RemoveLevel,
    SetName(Option<String>),
    /// Wear an item in a place, instead of the one there; `None` takes the place's item off. A ring worn in the
    /// other ring place moves here.
    Wear {
        place: GearPlace,
        item: Option<GearKey>,
    },
    /// Use a usable item (`used`), or stop using it.
    Use {
        item: GearKey,
        used: bool,
    },
    /// Take a Mythic rank at a character level later than the rank table's; `None` takes it as early as it comes
    /// again (the table's level, or the rank below's when that one is later). [`Engine::apply`](crate::Engine::apply)
    /// keeps only a level later than the table's and no later than the last level.
    RankLevel {
        rank: u8,
        level: Option<u8>,
    },
}

impl Edit {
    /// The choices this edit sets directly. Whether they fit is the picker's business; an edit is allowed when
    /// it breaks no other choice ([`Engine::allows`](crate::Engine::allows)).
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
            Edit::Start(_) => [
                Choice::Race,
                Choice::Subrace,
                Choice::Alignment,
                Choice::Background,
                Choice::Deity,
            ]
            .into_iter()
            .chain(AbilityScores::ABILITIES.map(|ability| Choice::AbilityScore { ability }))
            .collect(),
            Edit::FirstAscension(_) => vec![Choice::FirstAscension],
            Edit::MythicPath(_) => vec![Choice::MythicPath],
            Edit::PathSwitch(_) => vec![Choice::PathSwitch],
            // Nothing after the last level can depend on it, and a name breaks nothing.
            Edit::RemoveLevel | Edit::SetName(_) => vec![],
            // A ring may move here from the other ring place.
            Edit::Wear { place, .. } => std::iter::once(*place)
                .chain(place.other_ring())
                .map(|place| Choice::Gear(GearSpot::Worn(place)))
                .collect(),
            Edit::Use { item, .. } => vec![Choice::Gear(GearSpot::Used(item.clone()))],
            Edit::RankLevel { rank, .. } => vec![Choice::RankLevel { rank: *rank }],
        }
    }
}

impl Build {
    /// Applies an edit. Edits naming a level the Build doesn't have (other than the next one, for a class; a Mythic
    /// rank taken at such a level included) do nothing, and so do edits that would break a fixed rule ([`keeps_fixed_rules`](Self::keeps_fixed_rules)):
    /// an ability increase on a level without one, or ranks that take a skill above the character level. The sheet
    /// applies edits through [`Engine::apply`](crate::Engine::apply), which also drops what the sheet no longer shows:
    /// the path, switch and picks of Mythic ranks the Build no longer reaches (that needs the game data's ascension
    /// table).
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
                if ability.is_some() && !has_ability_increase(*level) {
                    return;
                }
                if let Some(level_up) = self.level_mut(*level) {
                    level_up.ability_increase = *ability;
                }
            }
            Edit::SkillRanks {
                level,
                skill,
                ranks,
            } => {
                let Some(level_up) = self.level_mut(*level) else {
                    return;
                };
                let before = if *ranks == 0 {
                    level_up.skill_ranks.remove(skill)
                } else {
                    level_up.skill_ranks.insert(skill.clone(), *ranks)
                };
                if !self.keeps_fixed_rules()
                    && let Some(level_up) = self.level_mut(*level)
                {
                    match before {
                        Some(before) => level_up.skill_ranks.insert(skill.clone(), before),
                        None => level_up.skill_ranks.remove(skill),
                    };
                }
            }
            Edit::Start(start) => self.start = start.clone(),
            Edit::FirstAscension(path) => self.first_ascension = path.clone(),
            Edit::MythicPath(path) => self.mythic_path = path.clone(),
            Edit::PathSwitch(path) => self.path_switch = path.clone(),
            Edit::RemoveLevel => {
                if self.levels.pop().is_some() {
                    let removed = u8::try_from(self.levels.len() + 1).unwrap_or(u8::MAX);
                    self.picks.retain(|slot, _| slot.level() != Some(removed));
                    self.drop_unused_class_choices();
                }
            }
            Edit::SetName(name) => self.name = name.clone(),
            Edit::Wear { place, item } => match item {
                Some(item) => {
                    if let Some(other) = place.other_ring()
                        && self.gear.worn.get(&other) == Some(item)
                    {
                        self.gear.worn.remove(&other);
                    }
                    self.gear.worn.insert(*place, item.clone());
                }
                None => {
                    self.gear.worn.remove(place);
                }
            },
            Edit::Use { item, used } => {
                if *used {
                    self.gear.used.insert(item.clone());
                } else {
                    self.gear.used.remove(item);
                }
            }
            Edit::RankLevel { rank, level } => {
                match level {
                    // A level the Build doesn't have is no move: the rank stays where it is.
                    Some(level) if usize::from(*level) > self.levels.len() => None,
                    Some(level) => self.rank_levels.insert(*rank, *level),
                    None => self.rank_levels.remove(rank),
                };
            }
        }
    }

    fn level_mut(&mut self, level: u8) -> Option<&mut LevelUp> {
        self.levels.get_mut(usize::from(level).checked_sub(1)?)
    }

    /// Whether the Build keeps the rules no game data changes, which the planner's edits never break: ranks in a
    /// skill never above the character level, and an ability increase only at every 4th level. (A [`PointBuy`]
    /// can't break its own.) A link whose Build breaks them doesn't decode, so the engine's walk doesn't check them.
    pub fn keeps_fixed_rules(&self) -> bool {
        let mut totals: BTreeMap<&SkillKey, usize> = BTreeMap::new();
        for (index, level_up) in self.levels.iter().enumerate() {
            let level = index + 1;
            if level_up.ability_increase.is_some()
                && !u8::try_from(level).is_ok_and(has_ability_increase)
            {
                return false;
            }
            for (skill, ranks) in &level_up.skill_ranks {
                let total = totals.entry(skill).or_default();
                *total += usize::from(*ranks);
                if *total > level {
                    return false;
                }
            }
        }
        true
    }

    /// Forgets the choices of classes no level takes any more, so a link carries no dead choices.
    fn drop_unused_class_choices(&mut self) {
        let levels = &self.levels;
        self.class_choices
            .retain(|class, _| levels.iter().any(|level| &level.class == class));
    }
}

/// Whether a character level has the +1 ability increase: every 4th level.
pub fn has_ability_increase(level: u8) -> bool {
    level % 4 == 0
}
