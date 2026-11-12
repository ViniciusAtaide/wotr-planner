//! The URL codec: the Build in the link fragment, `/#b=<code>`.
//!
//! `code` is base64url (no padding) over one format-version byte followed by the version's body. The version byte
//! stays outside the compression so a later version can change anything after it.
//!
//! # Version 1
//!
//! The body is compressed with raw DEFLATE (any stream inflates; the planner writes miniz_oxide's level 9).
//! Inflated, it lists the hashes of the Build's keys, then gives the Build's structure:
//!
//! ```text
//! body = varint count, count × hash, structure
//! ```
//!
//! **Keys.** Every game-data key in the Build is written as its hash, never its text: [FNV-1a] 32-bit over the
//! key's UTF-8 bytes (offset basis `0x811c9dc5`, prime `0x01000193`), cut to its low 24 bits ([`key_hash`]) and
//! written as 3 bytes, least significant first. The list holds one hash per key the structure names, in the order it
//! names them; a key named twice is listed twice (DEFLATE shrinks the repeats).
//!
//! The structure gives each key's kind, and a hash is looked up only among the current game data's keys of that
//! kind ([`LinkKeys`]): races, subraces, backgrounds, deities, skills, weapons, classes (base and prestige together),
//! archetypes, mythic paths, feats, selection options, mythic abilities, mythic feats and gear items ([`KeyKind`]). Keys of two
//! kinds may share a hash; two keys of one kind may not. `seeds`'s real-data test
//! `no_two_keys_of_one_kind_share_a_link_hash` fails if a game-data change ever makes two collide; then the hash
//! gets wider or another offset basis, which is a new format. A hash no key of its kind has (a key the game data no
//! longer has), or one that two keys of its kind share, is [`DecodeError::UnknownKey`]. So a link survives every
//! game-data change that keeps the keys it names.
//!
//! **Structure.** Bytes in this order. `varint` is unsigned LEB128 in the fewest bytes (one byte below 128).
//! `opt(x)` is `0` for none, or `1` then `x`. `key(kind)` writes nothing here: it takes the next hash from the list,
//! a key of that kind.
//!
//! ```text
//! structure     = name, start, levels, class choices, path, path [the switch], picks, first ascension, gear,
//!                 rank levels
//! name          = opt(varint length in bytes, UTF-8 bytes)
//! start         = opt(key(race)), opt(key(subrace)), ability [racial], alignment, opt(key(background)), faith,
//!                 6 × u8 score [point buy: Str, Dex, Con, Int, Wis, Cha]
//! levels        = varint count, count × (key(class), ability [increase], varint count,
//!                 count × (key(skill), u8 ranks))
//! class choices = varint count, count × (key(class), opt(key(archetype)), varint count,
//!                 count × key(class) [advances])
//! path          = opt(key(mythic path))
//! picks         = varint count, count × group
//! first ascension = nothing [none] | 1, key(mythic path)
//! gear          = nothing [none] | 2, varint count (at least 1), count × (u8 place, key(gear item))
//! rank levels   = nothing [none] | 3, varint count (at least 1), count × (u8 rank, u8 level)
//! group         = u8 slot kind, varint count (at least 1), count × (u8 step, slot fields, pick)
//! pick          = 0, key(feat), parameter | 1, key(selection option), parameter | 2, key(mythic ability)
//!               | 3, key(mythic feat), parameter | 4, key(mythic ability), parameter [not none]
//! parameter     = 0 [none] | 1, key(skill) | 2, key(weapon) | 3, u8 school | 4, u8 energy | 5, u8 armour type
//!               | 6, key(feat) [a metamagic] | 7, u8 stat | 8, u8 element
//! ability       = 0 [none] | 1 + its index in Str, Dex, Con, Int, Wis, Cha
//! alignment     = 0 [none] | 1 + its index in LG, NG, CG, LN, TN, CN, LE, NE, CE
//! faith         = 0 [none] | 1 [atheism] | 2, key(deity)
//! ```
//!
//! A group holds the picks of one kind of slot: [`SlotId`]'s variants in order, `0` Feat, `1` Selection, `2`
//! Mythic, `3` PathFeat, `4` PathSelection, `5` OptionSelection, `6` MythicSelection, `7` OptionFeat, `8`
//! FeatSelection, `9` MythicExtra. Groups come in that order and picks in [`SlotId`]'s. A pick's step is its slot's
//! level (or rank) less the previous pick's in the group, the first's less 0. The slot fields are a Feat slot's
//! source (`0` Race, `1` Class, `2` General) and index, nothing for a Mythic or MythicExtra slot, and the index for
//! the others. A school, energy,
//! armour type, stat or element is its index in [`SpellSchool::ALL`], [`Energy::ALL`], [`ArmourType::ALL`],
//! [`Stat::ALL`] or [`Element::ALL`]. Skill ranks, class choices and advances come in their keys' text order, as the
//! [`Build`] holds them.
//!
//! Mythic abilities' values (pick `4`) and the metamagic, stat and element values (parameters `6` to `8`) came
//! after links had been made (ticket 125), as new tags: a mythic ability with no value is still tag `2`, so every
//! link made before them reads as it did, and a Build without them writes the link it always did. A link naming
//! one of them doesn't open in a planner older than them.
//!
//! The first ascension ([`Build::first_ascension`], ticket 91) came after links had been made, so it goes last and
//! writes nothing when there is none: every link made before it reads as a Build without one, and a Build without
//! one writes the link it always did. Its key, when there is one, is the last hash listed.
//!
//! The gear ([`Build::gear`], ticket 166) came later still, so it goes after the first ascension, behind a tag of
//! its own (`2`, which a first ascension never starts with), and writes nothing when the Build holds none: every
//! link made before it reads as a Build without gear, and a Build without gear writes the link it always did. Each
//! item is its place and its key: a worn item's place is its index in [`GearPlace::ALL`] (`0` Belt to `11` Weapon,
//! with `8` and `9` the two rings and `10` the body), each place once and in that order; then the usable items
//! used, each with place `12`, in their keys' text order. Their keys are the last hashes listed.
//!
//! The rank levels ([`Build::rank_levels`], ticket 171) came after the gear, so they go after it, behind the next
//! tag (`3`), and write nothing when no rank is taken later than the rank table's level: every link made before
//! them reads as a Build that moves no rank, and such a Build writes the link it always did. Each is a Mythic rank
//! and the character level it is taken at, in rank order, each rank once; they name no key. The three trailing
//! sections are each there or not on their own, always in the order of their tags (`1`, `2`, `3`).
//!
//! **Exact.** A Build reads back unchanged from its link, given the keys it names, and has only one body: decoding
//! refuses ([`DecodeError::Format`]) anything the encoder never writes: an unknown tag; groups or slots out of order,
//! a slot named twice, a group with no picks; a map or set out of key order or naming a key twice; a varint longer
//! than needed; a hash listed but not named; bytes left over; a gear section with no item, a place out of order or
//! named twice, a used item out of key order; a rank levels section with no rank, a rank out of order or named twice,
//! a rank or a level of 0; a trailing section before one with a lower tag. It also refuses a Build no planner could make,
//! whatever the game data: a point buy that doesn't fit, ranks above the character level, an ability increase on a
//! level without one.
//!
//! The sheet opens a link with [`Engine::open_link`](crate::Engine::open_link): a link that doesn't decode, or whose
//! Build isn't possible with the current game data, is a [`StaleLink`], one generic error with no reason.
//!
//! Changing this layout or the hash changes version 1's format. Once links have shipped, don't: add a version 2 and
//! keep decoding version 1. The frozen v1 test vector guards this.
//!
//! [FNV-1a]: http://www.isthe.com/chongo/tech/comp/fnv/

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

use crate::build::{
    AbilityScores, Build, BuildStart, ClassChoice, Faith, FeatSource, Gear, GearPlace, LevelUp,
    Pick, PointBuy, SlotId,
};
use crate::game_data::{Ability, Alignment, GameData};
use crate::keys::{
    ArchetypeKey, BackgroundKey, ClassKey, DeityKey, FeatKey, GearKey, MythicAbilityKey,
    MythicFeatKey, MythicPathKey, RaceKey, SelectionOptionKey, SkillKey, SubraceKey, WeaponKey,
};
use crate::parameter::{ArmourType, Element, Energy, Parameter, SpellSchool, Stat};

/// The format version new links are written in.
pub const CODEC_VERSION: u8 = 1;

/// The most a link may inflate to. A 40-level Build is well under a kilobyte.
const MAX_INFLATED: usize = 256 * 1024;

/// How many bits of a key's FNV-1a hash a link keeps.
pub const KEY_HASH_BITS: u32 = 24;
/// The bytes a key's hash takes in a link.
const KEY_HASH_BYTES: usize = 3;
const FNV_OFFSET_BASIS: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;

/// A key's hash in a link: FNV-1a 32-bit over its UTF-8 bytes, cut to its low [`KEY_HASH_BITS`] bits.
pub fn key_hash(key: &str) -> u32 {
    let hash = key.bytes().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(FNV_PRIME)
    });
    hash & ((1 << KEY_HASH_BITS) - 1)
}

/// Why a link couldn't be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    Empty,
    /// Not base64url.
    Base64,
    /// A format version this planner doesn't know (made by a newer one?).
    UnsupportedVersion(u8),
    /// The body isn't valid DEFLATE data, or inflates past the size limit.
    Decompress,
    /// The inflated body isn't a Build of this version, or is one no planner makes (a hand-edited link: see the
    /// module docs).
    Format,
    /// The link names a key the game data has no key of its kind for, or one that two keys of its kind share: the
    /// game data changed since the link was made.
    UnknownKey,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Empty => f.write_str("the link has no Build in it"),
            DecodeError::Base64 => f.write_str("the link isn't valid base64url"),
            DecodeError::UnsupportedVersion(version) => {
                write!(
                    f,
                    "the link uses Build format version {version}, which this planner doesn't know"
                )
            }
            DecodeError::Decompress => f.write_str("the link's Build data is damaged"),
            DecodeError::Format => {
                f.write_str("the link's Build data doesn't describe a Build the planner can make")
            }
            DecodeError::UnknownKey => {
                f.write_str("the link names something the game data doesn't have")
            }
        }
    }
}

impl std::error::Error for DecodeError {}

/// A link that opens nothing: it doesn't decode, or its Build isn't possible with the current game data (a key
/// gone, a pick that no longer fits, a rule broken). The sheet shows this one message and a blank Build; it never
/// says why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StaleLink;

impl fmt::Display for StaleLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("This link no longer works with the current game data")
    }
}

impl std::error::Error for StaleLink {}

/// The kinds of key a link names. Each is its own namespace: a hash is looked up only among the keys of the kind
/// the link's structure says it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KeyKind {
    Race,
    Subrace,
    Background,
    Deity,
    Skill,
    Weapon,
    /// Base and prestige classes.
    Class,
    Archetype,
    MythicPath,
    Feat,
    SelectionOption,
    MythicAbility,
    MythicFeat,
    /// A gear item a Build holds (ticket 166).
    Gear,
}

/// Two keys of one kind with the same hash: a link naming either can't say which.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyCollision {
    pub kind: KeyKind,
    /// In text order.
    pub keys: [String; 2],
}

/// The keys a link can name, by kind and hash, to read a link's hashes back into keys.
#[derive(Clone, Debug, Default)]
pub struct LinkKeys {
    keys: HashMap<(KeyKind, u32), HashedKey>,
    collisions: Vec<KeyCollision>,
}

/// The first key of a kind with a hash, and whether another key of the kind has it too.
#[derive(Clone, Debug)]
struct HashedKey {
    key: String,
    shared: bool,
}

impl LinkKeys {
    /// The game data's keys: what the sheet reads links with.
    pub fn new(data: &GameData) -> Self {
        let mut keys = Self::default();
        for race in &data.races {
            keys.insert(KeyKind::Race, race.key.as_str());
            for subrace in &race.subraces {
                keys.insert(KeyKind::Subrace, subrace.key.as_str());
            }
        }
        for background in &data.backgrounds {
            keys.insert(KeyKind::Background, background.key.as_str());
        }
        for deity in &data.deities {
            keys.insert(KeyKind::Deity, deity.key.as_str());
        }
        for skill in &data.skills {
            keys.insert(KeyKind::Skill, skill.key.as_str());
        }
        for weapon in &data.weapons {
            keys.insert(KeyKind::Weapon, weapon.key.as_str());
        }
        for class in data.all_classes() {
            keys.insert(KeyKind::Class, class.key.as_str());
        }
        for archetype in data.archetypes() {
            keys.insert(KeyKind::Archetype, archetype.key.as_str());
        }
        for path in &data.mythic_paths {
            keys.insert(KeyKind::MythicPath, path.key.as_str());
        }
        for feat in &data.feats {
            keys.insert(KeyKind::Feat, feat.key.as_str());
        }
        for option in data.selections().flat_map(|selection| &selection.options) {
            keys.insert(KeyKind::SelectionOption, option.key.as_str());
        }
        for ability in &data.mythic_abilities {
            keys.insert(KeyKind::MythicAbility, ability.key.as_str());
        }
        for feat in &data.mythic_feats {
            keys.insert(KeyKind::MythicFeat, feat.key.as_str());
        }
        for item in &data.gear {
            keys.insert(KeyKind::Gear, item.key.as_str());
        }
        keys
    }

    /// The keys one Build names: enough to read its own link back without game data (a round trip). Tests only.
    #[cfg(any(test, feature = "testing"))]
    pub fn of_build(build: &Build) -> Self {
        let mut keys = Self::default();
        for (kind, key) in Writer::new(build).keys {
            keys.insert(kind, key);
        }
        keys
    }

    /// Every pair of keys of one kind that share a hash. Links can't name them.
    pub fn collisions(&self) -> &[KeyCollision] {
        &self.collisions
    }

    fn insert(&mut self, kind: KeyKind, key: &str) {
        let held = self
            .keys
            .entry((kind, key_hash(key)))
            .or_insert_with(|| HashedKey {
                key: key.to_owned(),
                shared: false,
            });
        if held.key != key {
            held.shared = true;
            let mut keys = [held.key.clone(), key.to_owned()];
            keys.sort();
            let collision = KeyCollision { kind, keys };
            if !self.collisions.contains(&collision) {
                self.collisions.push(collision);
            }
        }
    }

    /// The key of a kind with a hash, unless no key or more than one has it.
    fn get(&self, kind: KeyKind, hash: u32) -> Option<&str> {
        let held = self.keys.get(&(kind, hash))?;
        (!held.shared).then_some(held.key.as_str())
    }
}

/// The link code for a Build, in the current format version.
pub fn encode_build(build: &Build) -> String {
    let mut bytes = vec![CODEC_VERSION];
    bytes.extend(miniz_oxide::deflate::compress_to_vec(&body(build), 9));
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Reads a link code made by any format version so far, turning its hashes back into the keys of `keys`.
pub fn decode_build(code: &str, keys: &LinkKeys) -> Result<Build, DecodeError> {
    if code.is_empty() {
        return Err(DecodeError::Empty);
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(code)
        .map_err(|_| DecodeError::Base64)?;
    let (&version, body) = bytes.split_first().ok_or(DecodeError::Empty)?;
    match version {
        1 => decode_v1(body, keys),
        other => Err(DecodeError::UnsupportedVersion(other)),
    }
}

/// Version 1's body, before compression.
fn body(build: &Build) -> Vec<u8> {
    let writer = Writer::new(build);
    let mut body = Vec::new();
    write_varint(&mut body, writer.keys.len());
    for (_, key) in &writer.keys {
        body.extend(&key_hash(key).to_le_bytes()[..KEY_HASH_BYTES]);
    }
    body.extend(writer.bytes);
    body
}

fn decode_v1(body: &[u8], keys: &LinkKeys) -> Result<Build, DecodeError> {
    let inflated = miniz_oxide::inflate::decompress_to_vec_with_limit(body, MAX_INFLATED)
        .map_err(|_| DecodeError::Decompress)?;
    let build = Reader::new(&inflated, keys)?.build()?;
    if build.keeps_fixed_rules() {
        Ok(build)
    } else {
        Err(DecodeError::Format)
    }
}

fn write_varint(bytes: &mut Vec<u8>, mut value: usize) {
    while value >= 0x80 {
        bytes.push((value & 0x7f) as u8 | 0x80);
        value >>= 7;
    }
    bytes.push(value as u8);
}

fn index_of<T: PartialEq>(all: &[T], value: &T) -> u8 {
    all.iter()
        .position(|item| item == value)
        .expect("every value is listed") as u8
}

/// The slot kind of a group: [`SlotId`]'s variant, in order.
fn slot_kind(slot: &SlotId) -> u8 {
    match slot {
        SlotId::Feat { .. } => 0,
        SlotId::Selection { .. } => 1,
        SlotId::Mythic { .. } => 2,
        SlotId::PathFeat { .. } => 3,
        SlotId::PathSelection { .. } => 4,
        SlotId::OptionSelection { .. } => 5,
        SlotId::MythicSelection { .. } => 6,
        SlotId::OptionFeat { .. } => 7,
        SlotId::FeatSelection { .. } => 8,
        SlotId::MythicExtra { .. } => 9,
    }
}

/// A slot's level or rank: what picks of one kind of slot are ordered by first, and what their steps add up to.
fn level_or_rank(slot: &SlotId) -> u8 {
    slot.level()
        .or(slot.rank())
        .expect("every slot has a level or a rank")
}

/// The tag before the first ascension, when there is one.
const FIRST_ASCENSION_TAG: u8 = 1;
/// The tag before the gear, when the Build holds any.
const GEAR_TAG: u8 = 2;
/// The place of a usable item used, after every place an item is worn in ([`GearPlace::ALL`]).
const USED_PLACE: u8 = GearPlace::ALL.len() as u8;
/// The tag before the rank levels, when a rank is taken later than the rank table's level.
const RANK_LEVELS_TAG: u8 = 3;

/// Writes a version 1 structure, collecting the keys it names in order.
struct Writer<'a> {
    keys: Vec<(KeyKind, &'a str)>,
    bytes: Vec<u8>,
}

impl<'a> Writer<'a> {
    fn new(build: &'a Build) -> Self {
        let mut writer = Self {
            keys: Vec::new(),
            bytes: Vec::new(),
        };
        writer.build(build);
        writer
    }

    fn byte(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    fn varint(&mut self, value: usize) {
        write_varint(&mut self.bytes, value);
    }

    fn key(&mut self, kind: KeyKind, key: &'a str) {
        self.keys.push((kind, key));
    }

    fn opt_key(&mut self, kind: KeyKind, key: Option<&'a str>) {
        match key {
            None => self.byte(0),
            Some(key) => {
                self.byte(1);
                self.key(kind, key);
            }
        }
    }

    fn ability(&mut self, ability: Option<Ability>) {
        self.byte(ability.map_or(0, |ability| {
            1 + index_of(&AbilityScores::ABILITIES, &ability)
        }));
    }

    fn build(&mut self, build: &'a Build) {
        // Destructured so that a new field can't be left out of the link.
        let Build {
            name,
            start,
            levels,
            class_choices,
            first_ascension,
            mythic_path,
            path_switch,
            picks,
            gear,
            rank_levels,
        } = build;
        match name {
            None => self.byte(0),
            Some(name) => {
                self.byte(1);
                self.varint(name.len());
                self.bytes.extend(name.as_bytes());
            }
        }
        let BuildStart {
            race,
            subrace,
            racial_ability,
            alignment,
            background,
            deity,
            ability_scores,
        } = start;
        self.opt_key(KeyKind::Race, race.as_ref().map(RaceKey::as_str));
        self.opt_key(KeyKind::Subrace, subrace.as_ref().map(SubraceKey::as_str));
        self.ability(*racial_ability);
        self.byte(alignment.map_or(0, |alignment| 1 + index_of(&Alignment::ALL, &alignment)));
        self.opt_key(
            KeyKind::Background,
            background.as_ref().map(BackgroundKey::as_str),
        );
        match deity {
            None => self.byte(0),
            Some(Faith::Atheism) => self.byte(1),
            Some(Faith::Deity(deity)) => {
                self.byte(2);
                self.key(KeyKind::Deity, deity.as_str());
            }
        }
        for ability in AbilityScores::ABILITIES {
            self.byte(ability_scores.get(ability));
        }
        self.varint(levels.len());
        for level in levels {
            let LevelUp {
                class,
                ability_increase,
                skill_ranks,
            } = level;
            self.key(KeyKind::Class, class.as_str());
            self.ability(*ability_increase);
            self.varint(skill_ranks.len());
            for (skill, ranks) in skill_ranks {
                self.key(KeyKind::Skill, skill.as_str());
                self.byte(*ranks);
            }
        }
        self.varint(class_choices.len());
        for (class, choice) in class_choices {
            let ClassChoice {
                archetype,
                advances,
            } = choice;
            self.key(KeyKind::Class, class.as_str());
            self.opt_key(
                KeyKind::Archetype,
                archetype.as_ref().map(ArchetypeKey::as_str),
            );
            self.varint(advances.len());
            for advanced in advances {
                self.key(KeyKind::Class, advanced.as_str());
            }
        }
        self.opt_key(
            KeyKind::MythicPath,
            mythic_path.as_ref().map(MythicPathKey::as_str),
        );
        self.opt_key(
            KeyKind::MythicPath,
            path_switch.as_ref().map(MythicPathKey::as_str),
        );
        self.picks(picks);
        // Nothing when there's none, so the links made before it stay the same (module docs).
        if let Some(path) = first_ascension {
            self.byte(1);
            self.key(KeyKind::MythicPath, path.as_str());
        }
        // Nothing either without gear: a Build without it writes the link it always did.
        let Gear { worn, used } = gear;
        if !gear.is_empty() {
            self.byte(GEAR_TAG);
            self.varint(worn.len() + used.len());
            for (place, item) in worn {
                self.byte(index_of(&GearPlace::ALL, place));
                self.key(KeyKind::Gear, item.as_str());
            }
            for item in used {
                self.byte(USED_PLACE);
                self.key(KeyKind::Gear, item.as_str());
            }
        }
        // Nothing either when no rank is taken later: such a Build writes the link it always did.
        if !rank_levels.is_empty() {
            self.byte(RANK_LEVELS_TAG);
            self.varint(rank_levels.len());
            for (rank, level) in rank_levels {
                self.byte(*rank);
                self.byte(*level);
            }
        }
    }

    fn picks(&mut self, picks: &'a BTreeMap<SlotId, Pick>) {
        let mut groups: Vec<Vec<(&SlotId, &Pick)>> = Vec::new();
        for (slot, pick) in picks {
            match groups.last_mut() {
                Some(group) if slot_kind(group[0].0) == slot_kind(slot) => group.push((slot, pick)),
                _ => groups.push(vec![(slot, pick)]),
            }
        }
        self.varint(groups.len());
        for group in groups {
            self.byte(slot_kind(group[0].0));
            self.varint(group.len());
            let mut previous = 0;
            for (slot, pick) in group {
                let base = level_or_rank(slot);
                self.byte(base - previous);
                previous = base;
                match *slot {
                    SlotId::Feat { source, index, .. } => {
                        self.byte(match source {
                            FeatSource::Race => 0,
                            FeatSource::Class => 1,
                            FeatSource::General => 2,
                        });
                        self.byte(index);
                    }
                    SlotId::Mythic { .. } | SlotId::MythicExtra { .. } => {}
                    SlotId::Selection { index, .. }
                    | SlotId::PathFeat { index, .. }
                    | SlotId::PathSelection { index, .. }
                    | SlotId::OptionSelection { index, .. }
                    | SlotId::MythicSelection { index, .. }
                    | SlotId::OptionFeat { index, .. }
                    | SlotId::FeatSelection { index, .. } => self.byte(index),
                }
                self.pick(pick);
            }
        }
    }

    fn pick(&mut self, pick: &'a Pick) {
        match pick {
            Pick::Feat { feat, parameter } => {
                self.byte(0);
                self.key(KeyKind::Feat, feat.as_str());
                self.parameter(parameter.as_ref());
            }
            Pick::SelectionOption { option, parameter } => {
                self.byte(1);
                self.key(KeyKind::SelectionOption, option.as_str());
                self.parameter(parameter.as_ref());
            }
            // Tag 2 keeps every link made before mythic abilities took a value (ticket 125); one with a value is 4.
            Pick::MythicAbility {
                ability,
                parameter: None,
            } => {
                self.byte(2);
                self.key(KeyKind::MythicAbility, ability.as_str());
            }
            Pick::MythicAbility {
                ability,
                parameter: Some(parameter),
            } => {
                self.byte(4);
                self.key(KeyKind::MythicAbility, ability.as_str());
                self.parameter(Some(parameter));
            }
            Pick::MythicFeat { feat, parameter } => {
                self.byte(3);
                self.key(KeyKind::MythicFeat, feat.as_str());
                self.parameter(parameter.as_ref());
            }
        }
    }

    fn parameter(&mut self, parameter: Option<&'a Parameter>) {
        match parameter {
            None => self.byte(0),
            Some(Parameter::Skill(skill)) => {
                self.byte(1);
                self.key(KeyKind::Skill, skill.as_str());
            }
            Some(Parameter::Weapon(weapon)) => {
                self.byte(2);
                self.key(KeyKind::Weapon, weapon.as_str());
            }
            Some(Parameter::SpellSchool(school)) => {
                self.byte(3);
                self.byte(index_of(&SpellSchool::ALL, school));
            }
            Some(Parameter::Energy(energy)) => {
                self.byte(4);
                self.byte(index_of(&Energy::ALL, energy));
            }
            Some(Parameter::ArmourType(armour)) => {
                self.byte(5);
                self.byte(index_of(&ArmourType::ALL, armour));
            }
            Some(Parameter::Metamagic(feat)) => {
                self.byte(6);
                self.key(KeyKind::Feat, feat.as_str());
            }
            Some(Parameter::Stat(stat)) => {
                self.byte(7);
                self.byte(index_of(&Stat::ALL, stat));
            }
            Some(Parameter::Element(element)) => {
                self.byte(8);
                self.byte(index_of(&Element::ALL, element));
            }
        }
    }
}

/// Reads a version 1 body: the structure from the front, the keys from the hash list.
struct Reader<'a> {
    hashes: std::slice::Iter<'a, [u8; KEY_HASH_BYTES]>,
    bytes: &'a [u8],
    keys: &'a LinkKeys,
}

impl<'a> Reader<'a> {
    fn new(body: &'a [u8], keys: &'a LinkKeys) -> Result<Self, DecodeError> {
        let mut reader = Self {
            hashes: [].iter(),
            bytes: body,
            keys,
        };
        let count = reader.varint()?;
        let length = count
            .checked_mul(KEY_HASH_BYTES)
            .filter(|&length| length <= reader.bytes.len())
            .ok_or(DecodeError::Format)?;
        let (hashes, structure) = reader.bytes.split_at(length);
        reader.hashes = hashes.as_chunks().0.iter();
        reader.bytes = structure;
        Ok(reader)
    }

    fn byte(&mut self) -> Result<u8, DecodeError> {
        let (&byte, rest) = self.bytes.split_first().ok_or(DecodeError::Format)?;
        self.bytes = rest;
        Ok(byte)
    }

    /// An unsigned LEB128 in the fewest bytes.
    fn varint(&mut self) -> Result<usize, DecodeError> {
        let mut value = 0usize;
        for shift in (0..usize::BITS).step_by(7) {
            let byte = self.byte()?;
            let bits = usize::from(byte & 0x7f);
            if shift > 0 && byte == 0 {
                return Err(DecodeError::Format);
            }
            let shifted = bits << shift;
            if shifted >> shift != bits {
                return Err(DecodeError::Format);
            }
            value |= shifted;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err(DecodeError::Format)
    }

    /// A count of things that each take at least a byte or a hash, so a hand-made count can't ask for more than
    /// the body holds.
    fn count(&mut self) -> Result<usize, DecodeError> {
        let count = self.varint()?;
        if count > self.bytes.len() + self.hashes.len() {
            return Err(DecodeError::Format);
        }
        Ok(count)
    }

    fn flag(&mut self) -> Result<bool, DecodeError> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::Format),
        }
    }

    fn key(&mut self, kind: KeyKind) -> Result<String, DecodeError> {
        let &[low, middle, high] = self.hashes.next().ok_or(DecodeError::Format)?;
        let hash = u32::from_le_bytes([low, middle, high, 0]);
        self.keys
            .get(kind, hash)
            .map(str::to_owned)
            .ok_or(DecodeError::UnknownKey)
    }

    fn opt_key(&mut self, kind: KeyKind) -> Result<Option<String>, DecodeError> {
        if self.flag()? {
            self.key(kind).map(Some)
        } else {
            Ok(None)
        }
    }

    /// `0` for none, or 1 + an index into `all`.
    fn opt_index<T: Copy>(&mut self, all: &[T]) -> Result<Option<T>, DecodeError> {
        match self.byte()? {
            0 => Ok(None),
            index => all
                .get(usize::from(index) - 1)
                .copied()
                .map(Some)
                .ok_or(DecodeError::Format),
        }
    }

    fn index<T: Copy>(&mut self, all: &[T]) -> Result<T, DecodeError> {
        let index = self.byte()?;
        all.get(usize::from(index))
            .copied()
            .ok_or(DecodeError::Format)
    }

    fn build(mut self) -> Result<Build, DecodeError> {
        let name = if self.flag()? {
            let length = self.count()?;
            if length > self.bytes.len() {
                return Err(DecodeError::Format);
            }
            let (name, rest) = self.bytes.split_at(length);
            self.bytes = rest;
            Some(String::from_utf8(name.to_vec()).map_err(|_| DecodeError::Format)?)
        } else {
            None
        };
        let race = self.opt_key(KeyKind::Race)?.map(RaceKey);
        let subrace = self.opt_key(KeyKind::Subrace)?.map(SubraceKey);
        let racial_ability = self.opt_index(&AbilityScores::ABILITIES)?;
        let alignment = self.opt_index(&Alignment::ALL)?;
        let background = self.opt_key(KeyKind::Background)?.map(BackgroundKey);
        let deity = match self.byte()? {
            0 => None,
            1 => Some(Faith::Atheism),
            2 => Some(Faith::Deity(DeityKey(self.key(KeyKind::Deity)?))),
            _ => return Err(DecodeError::Format),
        };
        let mut scores = AbilityScores::default();
        for ability in AbilityScores::ABILITIES {
            *scores.get_mut(ability) = self.byte()?;
        }
        let ability_scores = PointBuy::try_from(scores).map_err(|_| DecodeError::Format)?;
        let start = BuildStart {
            race,
            subrace,
            racial_ability,
            alignment,
            background,
            deity,
            ability_scores,
        };

        let mut levels = Vec::new();
        for _ in 0..self.count()? {
            let class = ClassKey(self.key(KeyKind::Class)?);
            let ability_increase = self.opt_index(&AbilityScores::ABILITIES)?;
            let mut skill_ranks = BTreeMap::new();
            for _ in 0..self.count()? {
                let skill = SkillKey(self.key(KeyKind::Skill)?);
                let ranks = self.byte()?;
                insert_in_order(&mut skill_ranks, skill, ranks)?;
            }
            levels.push(LevelUp {
                class,
                ability_increase,
                skill_ranks,
            });
        }

        let mut class_choices = BTreeMap::new();
        for _ in 0..self.count()? {
            let class = ClassKey(self.key(KeyKind::Class)?);
            let archetype = self.opt_key(KeyKind::Archetype)?.map(ArchetypeKey);
            let mut advances = BTreeSet::new();
            for _ in 0..self.count()? {
                let advanced = ClassKey(self.key(KeyKind::Class)?);
                if advances.last().is_some_and(|last| *last >= advanced) {
                    return Err(DecodeError::Format);
                }
                advances.insert(advanced);
            }
            let choice = ClassChoice {
                archetype,
                advances,
            };
            insert_in_order(&mut class_choices, class, choice)?;
        }

        let mythic_path = self.opt_key(KeyKind::MythicPath)?.map(MythicPathKey);
        let path_switch = self.opt_key(KeyKind::MythicPath)?.map(MythicPathKey);
        let picks = self.picks()?;
        // Each absent, or its tag and what follows, in the order of their tags (the first ascension, the gear, the
        // rank levels): the encoder never writes a `0` here.
        let mut tag = self.tag()?;
        let first_ascension = if tag == Some(FIRST_ASCENSION_TAG) {
            let path = MythicPathKey(self.key(KeyKind::MythicPath)?);
            tag = self.tag()?;
            Some(path)
        } else {
            None
        };
        let gear = if tag == Some(GEAR_TAG) {
            let gear = self.gear()?;
            tag = self.tag()?;
            gear
        } else {
            Gear::default()
        };
        let rank_levels = match tag {
            None => BTreeMap::new(),
            Some(RANK_LEVELS_TAG) => self.rank_levels()?,
            Some(_) => return Err(DecodeError::Format),
        };

        if !self.bytes.is_empty() || self.hashes.len() > 0 {
            return Err(DecodeError::Format);
        }
        Ok(Build {
            name,
            start,
            levels,
            class_choices,
            first_ascension,
            mythic_path,
            path_switch,
            picks,
            gear,
            rank_levels,
        })
    }

    /// The next section's tag; `None` at the end of the body.
    fn tag(&mut self) -> Result<Option<u8>, DecodeError> {
        if self.bytes.is_empty() {
            Ok(None)
        } else {
            self.byte().map(Some)
        }
    }

    /// The gear after its tag: at least one item, the worn ones in place order and the used ones after them in key
    /// order, each place and each used item once.
    fn gear(&mut self) -> Result<Gear, DecodeError> {
        let count = self.count()?;
        if count == 0 {
            return Err(DecodeError::Format);
        }
        let mut gear = Gear::default();
        for _ in 0..count {
            let place = self.byte()?;
            let item = GearKey(self.key(KeyKind::Gear)?);
            if place == USED_PLACE {
                if gear.used.last().is_some_and(|last| *last >= item) {
                    return Err(DecodeError::Format);
                }
                gear.used.insert(item);
            } else {
                let place = *GearPlace::ALL
                    .get(usize::from(place))
                    .ok_or(DecodeError::Format)?;
                // A worn item never follows a used one.
                if !gear.used.is_empty() {
                    return Err(DecodeError::Format);
                }
                insert_in_order(&mut gear.worn, place, item)?;
            }
        }
        Ok(gear)
    }

    /// The rank levels after their tag: at least one, in rank order, each rank once, no rank or level of 0.
    fn rank_levels(&mut self) -> Result<BTreeMap<u8, u8>, DecodeError> {
        let count = self.count()?;
        if count == 0 {
            return Err(DecodeError::Format);
        }
        let mut rank_levels = BTreeMap::new();
        for _ in 0..count {
            let (rank, level) = (self.byte()?, self.byte()?);
            if rank == 0 || level == 0 {
                return Err(DecodeError::Format);
            }
            insert_in_order(&mut rank_levels, rank, level)?;
        }
        Ok(rank_levels)
    }

    fn picks(&mut self) -> Result<BTreeMap<SlotId, Pick>, DecodeError> {
        const SOURCES: [FeatSource; 3] = [FeatSource::Race, FeatSource::Class, FeatSource::General];
        let mut picks = BTreeMap::new();
        let mut previous_kind = None;
        for _ in 0..self.count()? {
            let kind = self.byte()?;
            if kind > 9 || previous_kind.is_some_and(|previous| previous >= kind) {
                return Err(DecodeError::Format);
            }
            previous_kind = Some(kind);
            let count = self.count()?;
            if count == 0 {
                return Err(DecodeError::Format);
            }
            let mut base = 0u8;
            for _ in 0..count {
                base = base.checked_add(self.byte()?).ok_or(DecodeError::Format)?;
                let slot = match kind {
                    0 => SlotId::Feat {
                        level: base,
                        source: self.index(&SOURCES)?,
                        index: self.byte()?,
                    },
                    1 => SlotId::Selection {
                        level: base,
                        index: self.byte()?,
                    },
                    2 => SlotId::Mythic { rank: base },
                    3 => SlotId::PathFeat {
                        rank: base,
                        index: self.byte()?,
                    },
                    4 => SlotId::PathSelection {
                        rank: base,
                        index: self.byte()?,
                    },
                    5 => SlotId::OptionSelection {
                        level: base,
                        index: self.byte()?,
                    },
                    6 => SlotId::MythicSelection {
                        rank: base,
                        index: self.byte()?,
                    },
                    7 => SlotId::OptionFeat {
                        level: base,
                        index: self.byte()?,
                    },
                    8 => SlotId::FeatSelection {
                        level: base,
                        index: self.byte()?,
                    },
                    _ => SlotId::MythicExtra { rank: base },
                };
                let pick = self.pick()?;
                insert_in_order(&mut picks, slot, pick)?;
            }
        }
        Ok(picks)
    }

    fn pick(&mut self) -> Result<Pick, DecodeError> {
        Ok(match self.byte()? {
            0 => Pick::Feat {
                feat: FeatKey(self.key(KeyKind::Feat)?),
                parameter: self.parameter()?,
            },
            1 => Pick::SelectionOption {
                option: SelectionOptionKey(self.key(KeyKind::SelectionOption)?),
                parameter: self.parameter()?,
            },
            2 => Pick::mythic_ability(MythicAbilityKey(self.key(KeyKind::MythicAbility)?)),
            3 => Pick::MythicFeat {
                feat: MythicFeatKey(self.key(KeyKind::MythicFeat)?),
                parameter: self.parameter()?,
            },
            // A mythic ability with a value: never none, which tag 2 writes.
            4 => Pick::MythicAbility {
                ability: MythicAbilityKey(self.key(KeyKind::MythicAbility)?),
                parameter: Some(self.parameter()?.ok_or(DecodeError::Format)?),
            },
            _ => return Err(DecodeError::Format),
        })
    }

    fn parameter(&mut self) -> Result<Option<Parameter>, DecodeError> {
        Ok(Some(match self.byte()? {
            0 => return Ok(None),
            1 => Parameter::Skill(SkillKey(self.key(KeyKind::Skill)?)),
            2 => Parameter::Weapon(WeaponKey(self.key(KeyKind::Weapon)?)),
            3 => Parameter::SpellSchool(self.index(&SpellSchool::ALL)?),
            4 => Parameter::Energy(self.index(&Energy::ALL)?),
            5 => Parameter::ArmourType(self.index(&ArmourType::ALL)?),
            6 => Parameter::Metamagic(FeatKey(self.key(KeyKind::Feat)?)),
            7 => Parameter::Stat(self.index(&Stat::ALL)?),
            8 => Parameter::Element(self.index(&Element::ALL)?),
            _ => return Err(DecodeError::Format),
        }))
    }
}

/// Inserts after every key already in the map, refusing a key out of order or named twice.
fn insert_in_order<K: Ord, V>(
    map: &mut BTreeMap<K, V>,
    key: K,
    value: V,
) -> Result<(), DecodeError> {
    if map.last_key_value().is_some_and(|(last, _)| *last >= key) {
        return Err(DecodeError::Format);
    }
    map.insert(key, value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{ClassChoice, LevelUp, Pick, SlotId};
    use crate::keys::{ClassKey, FeatKey};

    /// Version 1's binary layout, before compression, for a small Build. The compressed bytes may differ between
    /// DEFLATE implementations; this may not.
    #[test]
    fn version_1_body_layout_is_frozen() {
        let mut build = Build {
            name: Some("A".into()),
            ..Build::default()
        };
        build.levels.push(LevelUp::new(ClassKey::new("x")));
        build
            .class_choices
            .insert(ClassKey::new("x"), ClassChoice::default());
        build
            .picks
            .insert(SlotId::Mythic { rank: 1 }, Pick::feat(FeatKey::new("y")));
        assert_eq!(
            body(&build),
            [
                3, // three keys, in the order the structure names them
                0x87, 0x50, 0x0c, // "x" (the level's class): FNV-1a 0x..0c5087
                0x87, 0x50, 0x0c, // "x" (the class choice's class)
                0xf4, 0x4e, 0x0c, // "y" (the pick's feat): 0x..0c4ef4
                1, 1, b'A', // name: Some("A")
                0, 0, 0, 0, 0,
                0, // race, subrace, racial ability, alignment, background, faith: all None
                10, 10, 10, 10, 10, 10, // point-buy scores
                1,  // one level
                0, 0, // its class (key 1), no ability increase, no skill ranks
                1, // one class choice
                0, 0, // its class (key 2), archetype None, no advances
                0, 0, // mythic path, path switch: None
                1, // one group of picks
                2, 1, // of SlotId::Mythic, one pick
                1, // rank 1, a step of 1 from 0
                0, 0, // Pick::Feat (key 3), parameter None
            ]
        );
    }

    /// A link made by hand around a version 1 body.
    fn link(body: &[u8]) -> String {
        let mut bytes = vec![CODEC_VERSION];
        bytes.extend(miniz_oxide::deflate::compress_to_vec(body, 9));
        URL_SAFE_NO_PAD.encode(bytes)
    }

    fn decode(body: &[u8]) -> Result<Build, DecodeError> {
        decode_build(&link(body), &LinkKeys::default())
    }

    /// A blank Build's body: no keys, then name, race, subrace, racial ability, alignment, background, faith (all
    /// None), the six scores, no levels, no class choices, no path or switch, no groups of picks.
    const BLANK: [u8; 19] = [
        0, 0, 0, 0, 0, 0, 0, 0, 10, 10, 10, 10, 10, 10, 0, 0, 0, 0, 0,
    ];

    /// A point buy no planner makes (a score of 19, or 34 points) can't be read into a Build.
    #[test]
    fn a_hand_edited_point_buy_does_not_decode() {
        assert_eq!(body(&Build::default()), BLANK);
        assert_eq!(decode(&BLANK), Ok(Build::default()));

        let mut nineteen = BLANK;
        nineteen[8] = 19;
        assert_eq!(decode(&nineteen), Err(DecodeError::Format));

        let mut over_budget = BLANK;
        over_budget[8..10].copy_from_slice(&[18, 18]);
        assert_eq!(decode(&over_budget), Err(DecodeError::Format));

        let mut fits = BLANK;
        fits[8..14].copy_from_slice(&[18, 14, 14, 7, 10, 7]);
        assert!(decode(&fits).is_ok());
    }

    /// The faith is 0, 1 or 2 and a deity: any other tag is no Build's, and a deity's hash must be listed.
    #[test]
    fn a_faith_tag_the_encoder_never_writes_does_not_decode() {
        let mut atheist = BLANK;
        atheist[7] = 1;
        assert!(decode(&atheist).is_ok_and(|build| build.start.deity == Some(Faith::Atheism)));
        let mut unknown = BLANK;
        unknown[7] = 3;
        assert_eq!(decode(&unknown), Err(DecodeError::Format));
        let mut no_hash = BLANK;
        no_hash[7] = 2;
        assert_eq!(decode(&no_hash), Err(DecodeError::Format));
    }

    /// A blank Build with these groups of picks, each pick a mythic ability whose key is the next of `keys`.
    fn with_picks(keys: &[&str], groups: &[u8]) -> (Vec<u8>, LinkKeys) {
        let mut body = vec![u8::try_from(keys.len()).unwrap()];
        let mut build = Build::default();
        for (rank, key) in (1..).zip(keys) {
            body.extend(&key_hash(key).to_le_bytes()[..3]);
            build.picks.insert(
                SlotId::Mythic { rank },
                Pick::mythic_ability(crate::keys::MythicAbilityKey::new(*key)),
            );
        }
        body.extend(&BLANK[1..BLANK.len() - 1]);
        body.extend(groups);
        (body, LinkKeys::of_build(&build))
    }

    /// The first ascension goes after the picks, `1` and its key (listed last), and only when there is one; a `0`
    /// there is no Build's.
    #[test]
    fn the_first_ascension_follows_the_picks_only_when_there_is_one() {
        let build = Build {
            first_ascension: Some(MythicPathKey::new("p")),
            ..Build::default()
        };
        let hash = key_hash("p").to_le_bytes();
        let mut expected = vec![1, hash[0], hash[1], hash[2]];
        expected.extend(&BLANK[1..]);
        expected.push(1);
        assert_eq!(body(&build), expected);
        let keys = LinkKeys::of_build(&build);
        assert_eq!(decode_build(&link(&expected), &keys), Ok(build));

        let mut zero = BLANK.to_vec();
        zero.push(0);
        assert_eq!(decode(&zero), Err(DecodeError::Format));
        let mut no_hash = BLANK.to_vec();
        no_hash.push(1);
        assert_eq!(decode(&no_hash), Err(DecodeError::Format));
    }

    /// The gear goes last, after the first ascension when there is one: `2`, how many items, then each item's place
    /// and key (listed last), the worn ones in place order and the used ones (place `12`) in key order. Only when
    /// the Build holds gear; never an empty section, a place out of order or twice, or a used item out of order.
    #[test]
    fn the_gear_follows_the_first_ascension_only_when_there_is_some() {
        use crate::build::{Gear, GearPlace};
        use crate::keys::GearKey;

        let gear = Gear {
            worn: BTreeMap::from([
                (GearPlace::Belt, GearKey::new("b")),
                (GearPlace::SecondRing, GearKey::new("r")),
                (GearPlace::Body, GearKey::new("a")),
            ]),
            used: BTreeSet::from([GearKey::new("t"), GearKey::new("m")]),
        };
        let build = Build {
            gear: gear.clone(),
            ..Build::default()
        };
        let hashes = |keys: &[&str]| -> Vec<u8> {
            let mut bytes = vec![u8::try_from(keys.len()).unwrap()];
            for key in keys {
                bytes.extend(&key_hash(key).to_le_bytes()[..3]);
            }
            bytes
        };
        // Belt (0), the second ring (9), the body (10), then the used ones in key order.
        let section = [2, 5, 0, 9, 10, 12, 12];
        let mut expected = hashes(&["b", "r", "a", "m", "t"]);
        expected.extend(&BLANK[1..]);
        expected.extend(section);
        assert_eq!(body(&build), expected);
        let keys = LinkKeys::of_build(&build);
        assert_eq!(decode_build(&link(&expected), &keys), Ok(build));

        // After a first ascension: its tag and key first.
        let both = Build {
            first_ascension: Some(MythicPathKey::new("p")),
            gear,
            ..Build::default()
        };
        let mut expected = hashes(&["p", "b", "r", "a", "m", "t"]);
        expected.extend(&BLANK[1..]);
        expected.push(1);
        expected.extend(section);
        assert_eq!(body(&both), expected);
        let keys = LinkKeys::of_build(&both);
        assert_eq!(decode_build(&link(&expected), &keys), Ok(both));

        // What the encoder never writes.
        let refused = |keys: &[&str], section: &[u8]| {
            let mut body = hashes(keys);
            body.extend(&BLANK[1..]);
            body.extend(section);
            let mut names = LinkKeys::default();
            for key in keys {
                names.insert(KeyKind::Gear, key);
            }
            assert_eq!(
                decode_build(&link(&body), &names),
                Err(DecodeError::Format),
                "{section:?}"
            );
        };
        refused(&[], &[2, 0]); // no item
        refused(&["a", "b"], &[2, 2, 9, 0]); // places out of order
        refused(&["a", "b"], &[2, 2, 0, 0]); // a place twice
        refused(&["a"], &[2, 1, 13]); // no such place
        refused(&["b", "a"], &[2, 2, 12, 12]); // used items out of key order
        refused(&["a", "a"], &[2, 2, 12, 12]); // a used item twice
        refused(&["a", "b"], &[2, 2, 12, 0]); // a worn item after a used one
        refused(&["a"], &[2, 1, 0, 1]); // the first ascension after the gear
        refused(&["a"], &[4, 1, 0]); // an unknown tag
    }

    /// Ticket 125's tags, after links had shipped: a mythic ability with a value is pick `4` (with none it's `2`, as
    /// before), and a metamagic, a stat and an element are parameters `6` (a feat's key), `7` and `8` (an index). Pick
    /// `4` with no value is no Build's, nor is a stat or element index past the last.
    #[test]
    fn pick_values_after_links_shipped_are_new_tags() {
        use crate::keys::MythicAbilityKey;
        use crate::parameter::{Element, Stat};

        let mut build = Build::default();
        build.picks.insert(
            SlotId::Mythic { rank: 1 },
            Pick::mythic_ability_for(
                MythicAbilityKey::new("a"),
                Parameter::Metamagic(FeatKey::new("m")),
            ),
        );
        build.picks.insert(
            SlotId::Mythic { rank: 2 },
            Pick::mythic_ability_for(
                MythicAbilityKey::new("b"),
                Parameter::Element(Element::NegativeEnergy),
            ),
        );
        build.picks.insert(
            SlotId::Mythic { rank: 3 },
            Pick::feat_for(FeatKey::new("s"), Parameter::Stat(Stat::Speed)),
        );
        build.picks.insert(
            SlotId::Mythic { rank: 4 },
            Pick::mythic_ability(MythicAbilityKey::new("c")),
        );
        let hash = |key: &str| key_hash(key).to_le_bytes();
        let mut expected = vec![5];
        for key in ["a", "m", "b", "s", "c"] {
            expected.extend(&hash(key)[..3]);
        }
        expected.extend(&BLANK[1..BLANK.len() - 1]);
        expected.extend([
            1, // one group of picks
            2, 4, // of SlotId::Mythic, four picks
            1, 4, 6, // rank 1: a mythic ability with a value ("a"), a metamagic ("m")
            1, 4, 8,
            5, // rank 2: a mythic ability with a value ("b"), negative energy (index 5)
            1, 0, 7, 11, // rank 3: a feat ("s"), Speed (index 11)
            1, 2, // rank 4: a mythic ability with no value ("c"), as before ticket 125
        ]);
        assert_eq!(body(&build), expected);
        let keys = LinkKeys::of_build(&build);
        assert_eq!(decode_build(&link(&expected), &keys), Ok(build));

        // Pick 4 with no value; an element index past the last.
        let (mut no_value, keys) = with_picks(&["a"], &[1, 2, 1, 1]);
        no_value.extend([4, 0]);
        assert_eq!(
            decode_build(&link(&no_value), &keys),
            Err(DecodeError::Format)
        );
        let (mut past, keys) = with_picks(&["a"], &[1, 2, 1, 1]);
        past.extend([4, 8, 6]);
        assert_eq!(decode_build(&link(&past), &keys), Err(DecodeError::Format));
    }

    /// Each Build has one body: picks come in slot order, each slot once, and every group holds a pick.
    #[test]
    fn a_body_naming_a_slot_twice_or_out_of_order_does_not_decode() {
        // One group of two Mythic picks at ranks 1 and 2: steps 1, 1.
        let (fits, keys) = with_picks(&["a", "b"], &[1, 2, 2, 1, 2, 1, 2]);
        let decoded = decode_build(&link(&fits), &keys).unwrap();
        assert_eq!(decoded.picks.len(), 2);

        // Rank 1 twice: a step of 0.
        let (twice, keys) = with_picks(&["a", "b"], &[1, 2, 2, 1, 2, 0, 2]);
        assert_eq!(decode_build(&link(&twice), &keys), Err(DecodeError::Format));
        // Two groups of the same kind of slot.
        let (split, keys) = with_picks(&["a", "b"], &[2, 2, 1, 1, 2, 2, 1, 2, 2]);
        assert_eq!(decode_build(&link(&split), &keys), Err(DecodeError::Format));
        // A group with no picks.
        let (empty, keys) = with_picks(&[], &[1, 2, 0]);
        assert_eq!(decode_build(&link(&empty), &keys), Err(DecodeError::Format));
    }

    /// A body is read to its last byte, and every hash it lists is named once.
    #[test]
    fn a_body_with_bytes_or_hashes_left_over_does_not_decode() {
        let mut trailing = BLANK.to_vec();
        trailing.push(0);
        assert_eq!(decode(&trailing), Err(DecodeError::Format));

        // One hash listed, none named.
        let (unused, keys) = with_picks(&["a"], &[0]);
        assert_eq!(
            decode_build(&link(&unused), &keys),
            Err(DecodeError::Format)
        );
        // A pick naming a hash the list doesn't have.
        let mut missing = BLANK.to_vec();
        missing.pop();
        missing.extend([1, 2, 1, 1, 2]);
        assert_eq!(decode(&missing), Err(DecodeError::Format));
    }
}
