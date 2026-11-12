//! The URL codec: `/#b=<code>` is a version byte, then the deflated Build with every key as a 24-bit hash, in
//! base64url. Reading a link back needs the keys it may name ([`LinkKeys`]): the game data's, or a Build's own.

use std::collections::{BTreeMap, BTreeSet};

use proptest::collection::{btree_map, btree_set, vec};
use proptest::option;
use proptest::prelude::*;
use rules::{
    Ability, AbilityScores, Alignment, ArchetypeKey, ArmourType, BackgroundKey, Build, BuildStart,
    ClassChoice, ClassKey, DecodeError, DeityKey, Element, Energy, Faith, FeatSource, Gear,
    GearKey, GearPlace, KeyCollision, KeyKind, LevelUp, LinkKeys, MythicPathKey, Parameter, Pick,
    PointBuy, RaceKey, SkillKey, SlotId, SpellSchool, Stat, SubraceKey, decode_build, encode_build,
    key_hash,
};

use crate::support::*;

/// A text key of any kind: `power-attack`, `rogue/eldritch-scoundrel`.
fn key() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9/-]{0,24}"
}

fn ability() -> impl Strategy<Value = Ability> {
    prop::sample::select(AbilityScores::ABILITIES.to_vec())
}

fn alignment() -> impl Strategy<Value = Alignment> {
    use Alignment::*;
    prop::sample::select(vec![
        LawfulGood,
        NeutralGood,
        ChaoticGood,
        LawfulNeutral,
        TrueNeutral,
        ChaoticNeutral,
        LawfulEvil,
        NeutralEvil,
        ChaoticEvil,
    ])
}

/// No faith yet, atheism, or a deity.
fn faith() -> impl Strategy<Value = Option<Faith>> {
    prop_oneof![
        Just(None),
        Just(Some(Faith::Atheism)),
        key().prop_map(|key| Some(Faith::Deity(DeityKey(key)))),
    ]
}

fn start() -> impl Strategy<Value = BuildStart> {
    (
        option::of(key().prop_map(RaceKey)),
        option::of(key().prop_map(SubraceKey)),
        option::of(ability()),
        option::of(alignment()),
        option::of(key().prop_map(BackgroundKey)),
        faith(),
        point_buy_strategy(),
    )
        .prop_map(
            |(race, subrace, racial_ability, alignment, background, deity, ability_scores)| {
                BuildStart {
                    race,
                    subrace,
                    racial_ability,
                    alignment,
                    background,
                    deity,
                    ability_scores,
                }
            },
        )
}

/// Any point buy: scores from 7 to 18 within the budget.
fn point_buy_strategy() -> impl Strategy<Value = PointBuy> {
    prop::array::uniform6(7u8..=18)
        .prop_filter_map("over the budget", |s| PointBuy::try_from(scores(s)).ok())
}

/// Levels as the planner makes them: an ability increase only at every 4th level, and a skill's ranks never above
/// the character level.
fn possible(mut levels: Vec<LevelUp>) -> Vec<LevelUp> {
    let mut totals: BTreeMap<SkillKey, u8> = BTreeMap::new();
    for (index, level) in levels.iter_mut().enumerate() {
        let number = u8::try_from(index + 1).unwrap();
        if number % 4 != 0 {
            level.ability_increase = None;
        }
        for (skill, ranks) in &mut level.skill_ranks {
            let total = totals.entry(skill.clone()).or_default();
            *ranks = (*ranks).min(number - *total);
            *total += *ranks;
        }
    }
    levels
}

fn level_up() -> impl Strategy<Value = LevelUp> {
    (
        key().prop_map(ClassKey),
        option::of(ability()),
        btree_map(key().prop_map(SkillKey), any::<u8>(), 0..4),
    )
        .prop_map(|(class, ability_increase, skill_ranks)| LevelUp {
            class,
            ability_increase,
            skill_ranks,
        })
}

fn class_choice() -> impl Strategy<Value = ClassChoice> {
    (
        option::of(key().prop_map(ArchetypeKey)),
        btree_set(key().prop_map(ClassKey), 0..3),
    )
        .prop_map(|(archetype, advances)| ClassChoice {
            archetype,
            advances,
        })
}

fn slot_id() -> impl Strategy<Value = SlotId> {
    let source = prop::sample::select(vec![
        FeatSource::Race,
        FeatSource::Class,
        FeatSource::General,
    ]);
    prop_oneof![
        (any::<u8>(), source, any::<u8>()).prop_map(|(level, source, index)| SlotId::Feat {
            level,
            source,
            index
        }),
        (any::<u8>(), any::<u8>()).prop_map(|(level, index)| SlotId::Selection { level, index }),
        any::<u8>().prop_map(|rank| SlotId::Mythic { rank }),
        (any::<u8>(), any::<u8>()).prop_map(|(rank, index)| SlotId::PathFeat { rank, index }),
        (any::<u8>(), any::<u8>()).prop_map(|(rank, index)| SlotId::PathSelection { rank, index }),
        (any::<u8>(), any::<u8>())
            .prop_map(|(level, index)| SlotId::OptionSelection { level, index }),
        (any::<u8>(), any::<u8>())
            .prop_map(|(rank, index)| SlotId::MythicSelection { rank, index }),
        (any::<u8>(), any::<u8>()).prop_map(|(level, index)| SlotId::OptionFeat { level, index }),
        (any::<u8>(), any::<u8>())
            .prop_map(|(level, index)| SlotId::FeatSelection { level, index }),
        any::<u8>().prop_map(|rank| SlotId::MythicExtra { rank }),
    ]
}

fn spell_school() -> impl Strategy<Value = SpellSchool> {
    prop::sample::select(SpellSchool::ALL.to_vec())
}

fn energy() -> impl Strategy<Value = Energy> {
    prop::sample::select(Energy::ALL.to_vec())
}

fn armour_type() -> impl Strategy<Value = ArmourType> {
    prop::sample::select(ArmourType::ALL.to_vec())
}

fn parameter() -> impl Strategy<Value = Parameter> {
    prop_oneof![
        key().prop_map(|k| Parameter::Skill(SkillKey(k))),
        key().prop_map(|k| Parameter::Weapon(rules::WeaponKey(k))),
        spell_school().prop_map(Parameter::SpellSchool),
        energy().prop_map(Parameter::Energy),
        armour_type().prop_map(Parameter::ArmourType),
        key().prop_map(|k| Parameter::Metamagic(rules::FeatKey(k))),
        prop::sample::select(Stat::ALL.to_vec()).prop_map(Parameter::Stat),
        prop::sample::select(Element::ALL.to_vec()).prop_map(Parameter::Element),
    ]
}

fn pick() -> impl Strategy<Value = Pick> {
    prop_oneof![
        (key(), option::of(parameter())).prop_map(|(k, parameter)| Pick::Feat {
            feat: rules::FeatKey(k),
            parameter
        }),
        (key(), option::of(parameter())).prop_map(|(k, parameter)| Pick::SelectionOption {
            option: rules::SelectionOptionKey(k),
            parameter
        }),
        (key(), option::of(parameter())).prop_map(|(k, parameter)| Pick::MythicAbility {
            ability: rules::MythicAbilityKey(k),
            parameter
        }),
        (key(), option::of(parameter())).prop_map(|(k, parameter)| Pick::MythicFeat {
            feat: rules::MythicFeatKey(k),
            parameter
        }),
    ]
}

/// Any gear, often none: an item in any of the places, and any usable items.
fn gear() -> impl Strategy<Value = Gear> {
    let place = prop::sample::select(GearPlace::ALL.to_vec());
    (
        btree_map(place, key().prop_map(GearKey), 0..6),
        btree_set(key().prop_map(GearKey), 0..4),
    )
        .prop_map(|(worn, used)| Gear { worn, used })
}

fn any_build() -> impl Strategy<Value = Build> {
    (
        option::of(".{0,40}"),
        start(),
        vec(level_up(), 0..40),
        btree_map(key().prop_map(ClassKey), class_choice(), 0..4),
        option::of(key().prop_map(MythicPathKey)),
        option::of(key().prop_map(MythicPathKey)),
        option::of(key().prop_map(MythicPathKey)),
        btree_map(slot_id(), pick(), 0..40),
        gear(),
        // Any rank levels, often none: the link holds whatever the Build does, ranks and levels from 1.
        btree_map(1..=20u8, 1..=u8::MAX, 0..4),
    )
        .prop_map(
            |(
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
            )| {
                Build {
                    name,
                    start,
                    levels: possible(levels),
                    class_choices,
                    first_ascension,
                    mythic_path,
                    path_switch,
                    picks,
                    gear,
                    rank_levels,
                }
            },
        )
}

proptest! {
    /// Read back with the keys the Build itself names, the game data aside. Random keys of one kind may share a hash
    /// (about once in 3,000 Builds); such a Build can't be told apart from another, so it's skipped.
    #[test]
    fn every_build_round_trips_through_its_link(build in any_build()) {
        let keys = LinkKeys::of_build(&build);
        prop_assume!(keys.collisions().is_empty());
        let code = encode_build(&build);
        prop_assert!(
            code.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
            "not base64url: {code}"
        );
        prop_assert_eq!(decode_build(&code, &keys), Ok(build));
    }
}

/// The keys of the test world, as the sheet reads links with the game data's.
fn world_keys() -> LinkKeys {
    LinkKeys::new(&world().canonical())
}

fn decode(code: &str) -> Result<Build, DecodeError> {
    decode_build(code, &world_keys())
}

/// The Build frozen in [`V1_LINK`]. Never change either: old links must keep decoding.
fn v1_build() -> Build {
    let mut build = build("human", &["fighter", "fighter", "rogue"]);
    build.name = Some("Cleaver".into());
    build.start.racial_ability = Some(Ability::Strength);
    build.start.background = Some(BackgroundKey::new("shackles-corsair"));
    build.start.deity = Some(Faith::Deity(DeityKey::new("balance")));
    build.levels[0].skill_ranks.insert(skill("athletics"), 1);
    build.picks.insert(race_slot(0), pick_feat("power-attack"));
    build.picks.insert(class_slot(1, 0), pick_feat("cleave"));
    build.picks.insert(
        general_slot(1),
        pick_feat_for("weapon-focus", weapon("longsword")),
    );
    build.picks.insert(
        selection_slot(3, 0),
        pick_option("rogue/rogue-talents/combat-trick"),
    );
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("mythic-charge"),
    );
    build.picks.insert(
        SlotId::OptionSelection { level: 3, index: 0 },
        pick_option("rogue/rogue-talents/fast-stealth"),
    );
    build.picks.insert(
        SlotId::MythicSelection { rank: 1, index: 0 },
        pick_option("sorcerer/bloodline/arcane"),
    );
    build.picks.insert(
        SlotId::OptionFeat { level: 8, index: 0 },
        pick_feat("toughness"),
    );
    build.picks.insert(
        SlotId::FeatSelection { level: 3, index: 0 },
        pick_option("rogue/rogue-talents/resiliency"),
    );
    build.picks.insert(
        SlotId::Mythic { rank: 2 },
        Pick::mythic_feat_for(
            rules::MythicFeatKey::new("school-mastery"),
            Parameter::SpellSchool(rules::SpellSchool::Evocation),
        ),
    );
    build
        .picks
        .insert(SlotId::MythicExtra { rank: 2 }, pick_feat("dodge"));
    build.class_choices.insert(
        class("rogue"),
        ClassChoice {
            archetype: Some(ArchetypeKey::new("rogue/eldritch-scoundrel")),
            ..ClassChoice::default()
        },
    );
    build.mythic_path = Some(path("trickster"));
    build
}

/// `encode_build(&v1_build())` as format version 1 produced it (2026-10-31, miniz_oxide 0.9.1, level 9). Regenerated
/// before any link shipped: when archetype and advancement moved into `Build::class_choices`, on 2026-11-01 when
/// keys lost their type prefix (`feat:power-attack` became `power-attack`), on 2026-11-01 again when a feat or
/// option pick gained its parameter (Weapon Focus (Longsword)), on 2026-11-01 once more for the slots of option
/// rows and of a mythic ability's selections, on 2026-11-01 for an option row's feat slot (ticket 31), on
/// 2026-11-01 when a mythic feat pick gained its parameter (ticket 32: School Mastery (Evocation)), on 2026-11-01
/// when the Build start gained its background (ticket 34: Shackles Corsair), and on 2026-11-02 when version 1's body
/// became shorter (ticket 46): every key a 24-bit hash instead of its text, the hashes listed before the structure,
/// and the picks grouped by kind of slot with levels as steps, in place of the Build in postcard (314 characters
/// before, 196 after), and on 2026-11-02 for the slot an Extra feat gives (ticket 49: a `FeatSelection` group, the
/// rogue talent Resiliency at 3), and on 2026-11-02 for the slot a mythic feat gives (ticket 55: a `MythicExtra` group,
/// Dodge at MR 2), and on 2026-11-06 when the Build start gained its faith (ticket 86: the Balance, after the
/// background).
const V1_LINK: &str = "AR2KPQrCQBSE3-was4b41yQeQrCwFSHaCGJlaRUkCKIgKSwURSwCKURJKegBhNzBdjsb7yB4AFsfOwwfzPB5m8fot4u_2fVYDXvPG3P5Cbjvmb8Snk5zPe-47eElndSDQTzN9bbbShqH7Ozfk_3JeY2bsPuLKFxHMQgWRK3ilh0lCSAOE6RImkGGgguwQEJAQMqCZVbRuDYUO8o8JbD7Bw";

#[test]
fn a_version_1_link_still_decodes() {
    assert_eq!(decode(V1_LINK), Ok(v1_build()));
}

/// Gear came after links had shipped (ticket 166): an old link reads as a Build without gear, and a Build without
/// gear writes the link it always did, byte for byte.
#[test]
fn a_build_without_gear_reads_and_writes_the_link_it_always_did() {
    let decoded = decode(V1_LINK).unwrap();
    assert!(decoded.gear.is_empty());
    assert_eq!(encode_build(&v1_build()), V1_LINK);
    assert_eq!(encode_build(&decoded), V1_LINK);
}

/// The Build frozen in [`V1_GEAR_LINK`]: [`v1_build`] with a belt, a ring in the second ring place, a robe on the
/// body and two usable items. Never change either.
fn v1_gear_build() -> Build {
    let mut build = v1_build();
    build = wearing(build, GearPlace::Belt, "belt-of-dexterity-2");
    build = wearing(build, GearPlace::SecondRing, "ring-of-luck");
    build = wearing(build, GearPlace::Body, "robe-of-the-mage");
    build = using(build, "tome-of-perfection-4");
    using(build, "manual-of-dexterity-2")
}

/// `encode_build(&v1_gear_build())` as ticket 166 first wrote it (2026-11-11, miniz_oxide level 9): version 1 with
/// its gear section after the picks.
const V1_GEAR_LINK: &str = "AR2KrwrCUByF7-9e5-7m9V9xgiazYLAOYbOIYjKahgxBNpAFg6LIwmBBlEVFH0DYOxgsaxbxFQSTySR4vYfDB-fwVebn_mfpvaK9n7fMy5HTfRq897E2xaUkjJOJzpq9XTgsGh1vFCeLViMor6OtdgpWG_U2qDtvPeMeurXrt2o-NB_ktmNbM9sDBBLgQo5lVUoQAOLhBEQREQMJYl4ALiCMAQMhKUmstHBloNyh4lHg70pIURn7AQ";

#[test]
fn a_link_with_gear_is_pinned_and_round_trips() {
    assert_eq!(decode(V1_GEAR_LINK), Ok(v1_gear_build()));
    assert_eq!(encode_build(&v1_gear_build()), V1_GEAR_LINK);
    // The gear is all the two links differ by.
    let mut without = decode(V1_GEAR_LINK).unwrap();
    without.gear = Gear::default();
    assert_eq!(encode_build(&without), V1_LINK);
}

/// The Build frozen in [`V1_RANK_LEVELS_LINK`]: [`v1_gear_build`] with MR 1 taken at level 3 and MR 10 at level 20.
/// Never change either.
fn v1_rank_levels_build() -> Build {
    let mut build = v1_gear_build();
    build.rank_levels = BTreeMap::from([(1, 3), (10, 20)]);
    build
}

/// `encode_build(&v1_rank_levels_build())` as ticket 171 first wrote it (2026-11-12, miniz_oxide level 9): version 1
/// with its rank levels section after the gear.
const V1_RANK_LEVELS_LINK: &str = "AR2KrwrCUByF7-9ep9v1-g_ECZrMgsE6hM0iisloGjIE2UAWDIoiC4MFURYVfQBh72CwrFnEVxBMJpPg9R4OH5zDV5mf-5-l-wr3Xs40LkdO56nz3sfqFJfiIIonGmv2dsGwoHfcURQvWg2_vA636slfbehtULffWto5dGvXb9V4qB6k2rZlziwXEEiA81mWoTJBAIiHE5CMiBhIEPMCcAFhDBgISUhiJYWbApk7sngU-LsSUihjhJu0-AM";

#[test]
fn a_link_with_a_rank_taken_later_is_pinned_and_round_trips() {
    assert_eq!(encode_build(&v1_rank_levels_build()), V1_RANK_LEVELS_LINK);
    assert_eq!(decode(V1_RANK_LEVELS_LINK), Ok(v1_rank_levels_build()));
    // The rank levels are all the two links differ by, and without the gear too it is the first link.
    let mut without = decode(V1_RANK_LEVELS_LINK).unwrap();
    without.rank_levels.clear();
    assert_eq!(encode_build(&without), V1_GEAR_LINK);
    without.gear = Gear::default();
    assert_eq!(encode_build(&without), V1_LINK);
}

/// Rank levels came after links had shipped (ticket 171): an old link reads as a Build that moves no rank.
#[test]
fn a_link_made_before_rank_levels_reads_as_a_build_that_moves_no_rank() {
    assert!(decode(V1_LINK).unwrap().rank_levels.is_empty());
    assert!(decode(V1_GEAR_LINK).unwrap().rank_levels.is_empty());
}

/// A Build with all three trailing sections: [`v1_rank_levels_build`] with a first ascension.
fn with_every_trailing_section() -> Build {
    let mut build = v1_rank_levels_build();
    build.first_ascension = Some(path("trickster"));
    build
}

/// The three trailing sections (the first ascension, the gear, the rank levels) are each there or not on their own:
/// all eight combinations read back, and each writes only its own sections.
#[test]
fn every_combination_of_the_trailing_sections_round_trips() {
    let full = with_every_trailing_section();
    let mut codes = BTreeSet::new();
    for mask in 0..8u8 {
        let mut build = full.clone();
        if mask & 1 == 0 {
            build.first_ascension = None;
        }
        if mask & 2 == 0 {
            build.gear = Gear::default();
        }
        if mask & 4 == 0 {
            build.rank_levels.clear();
        }
        let code = encode_build(&build);
        assert_eq!(decode(&code), Ok(build), "sections {mask:03b}");
        codes.insert(code);
    }
    assert_eq!(codes.len(), 8);
}

/// The body of a link, and a link around a body: the version byte, then the body deflated.
fn body_of(code: &str) -> Vec<u8> {
    use base64::Engine as _;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(code)
        .unwrap();
    miniz_oxide::inflate::decompress_to_vec(&bytes[1..]).unwrap()
}

fn link_of(body: &[u8]) -> String {
    use base64::Engine as _;
    let mut bytes = vec![1];
    bytes.extend(miniz_oxide::deflate::compress_to_vec(body, 9));
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Ticket 166's review fuzz, over all eight combinations of the trailing sections: a body cut short anywhere, or
/// with any byte after it, never decodes to another Build than its own and never panics. A cut body may only read
/// as the Build without the sections cut off whole.
#[test]
fn a_cut_or_padded_body_with_trailing_sections_is_refused() {
    let full = with_every_trailing_section();
    let (mut cuts, mut pads) = (0, 0);
    for mask in 0..8u8 {
        let mut build = full.clone();
        if mask & 1 == 0 {
            build.first_ascension = None;
        }
        if mask & 2 == 0 {
            build.gear = Gear::default();
        }
        if mask & 4 == 0 {
            build.rank_levels.clear();
        }
        let body = body_of(&encode_build(&build));
        for cut in 0..body.len() {
            cuts += 1;
            if let Ok(read) = decode(&link_of(&body[..cut])) {
                // Only whole sections came off the end, and the link it then writes is that body.
                assert_ne!(read, build, "sections {mask:03b}, cut at {cut}");
                assert_eq!(body_of(&encode_build(&read)), &body[..cut]);
            }
        }
        for byte in 0..=u8::MAX {
            pads += 1;
            let mut padded = body.clone();
            padded.push(byte);
            assert!(
                decode(&link_of(&padded)).is_err(),
                "sections {mask:03b}, byte {byte}"
            );
        }
    }
    assert!(cuts > 500 && pads == 8 * 256, "{cuts} cuts, {pads} pads");
}

/// What the encoder never writes in a rank levels section: none, a rank out of order or twice, a rank or level of
/// 0, the section before the gear or the first ascension, bytes after it.
#[test]
fn a_rank_levels_section_the_encoder_never_writes_does_not_decode() {
    let plain = body_of(&encode_build(&v1_build()));
    let with = |section: &[u8]| {
        let mut body = plain.clone();
        body.extend(section);
        decode(&link_of(&body))
    };
    assert_eq!(
        with(&[3, 2, 1, 3, 10, 20]).map(|build| build.rank_levels),
        Ok(BTreeMap::from([(1, 3), (10, 20)]))
    );
    for section in [
        &[3][..],
        &[3, 0],
        &[3, 1, 1],
        &[3, 2, 1, 3],
        &[3, 2, 10, 20, 1, 3],
        &[3, 2, 4, 12, 4, 13],
        &[3, 1, 0, 3],
        &[3, 1, 1, 0],
        &[3, 1, 1, 3, 0],
        &[3, 1, 1, 3, 3, 1, 2, 4],
        &[3, 1, 1, 3, 2],
        &[3, 1, 1, 3, 1],
        &[4, 1, 1, 3],
        &[0],
    ] {
        assert_eq!(with(section), Err(DecodeError::Format), "{section:?}");
    }
}

#[test]
fn links_start_with_the_format_version() {
    use base64_check::first_byte;
    assert_eq!(first_byte(&encode_build(&v1_build())), 1);
    assert_eq!(first_byte(V1_LINK), 1);
}

/// Reads the version byte back out of a link without the crate's help.
mod base64_check {
    pub fn first_byte(code: &str) -> u8 {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let sextet = |c: u8| ALPHABET.iter().position(|&a| a == c).unwrap() as u8;
        let bytes = code.as_bytes();
        (sextet(bytes[0]) << 2) | (sextet(bytes[1]) >> 4)
    }
}

/// A link holds each key's hash, which only the game data turns back into the key: a hash no key of its kind has
/// is a key the game data no longer has, and the link is stale (`stale_links.rs`).
#[test]
fn a_link_naming_a_key_the_game_data_lacks_does_not_decode() {
    let gone: [fn(&mut Build); 7] = [
        |b| {
            b.picks
                .insert(general_slot(3), pick_feat("cut-in-a-later-patch"));
        },
        |b| b.levels[2].class = class("cut-in-a-later-patch"),
        |b| {
            b.levels[0]
                .skill_ranks
                .insert(skill("cut-in-a-later-patch"), 1);
        },
        |b| b.start.background = Some(BackgroundKey::new("cut-in-a-later-patch")),
        |b| b.start.deity = Some(Faith::Deity(DeityKey::new("cut-in-a-later-patch"))),
        |b| {
            b.picks.insert(
                general_slot(3),
                pick_feat_for("weapon-focus", weapon("cut-in-a-later-patch")),
            );
        },
        |b| b.path_switch = Some(path("cut-in-a-later-patch")),
    ];
    for (index, change) in gone.into_iter().enumerate() {
        let mut build = v1_build();
        change(&mut build);
        let code = encode_build(&build);
        assert_eq!(
            decode(&code),
            Err(DecodeError::UnknownKey),
            "change {index}"
        );
        assert_eq!(
            decode_build(&code, &LinkKeys::of_build(&build)),
            Ok(build),
            "change {index}"
        );
    }
}

/// The faith is one byte after the background: none, atheism, or a deity whose key is the next hash.
#[test]
fn every_faith_round_trips_through_a_link() {
    for faith in [
        None,
        Some(Faith::Atheism),
        Some(Faith::Deity(DeityKey::new("fist"))),
    ] {
        let mut build = v1_build();
        build.start.deity = faith;
        assert_eq!(decode(&encode_build(&build)), Ok(build));
    }
    let mut atheist = Build::default();
    atheist.start.deity = Some(Faith::Atheism);
    let mut follower = Build::default();
    follower.start.deity = Some(Faith::Deity(DeityKey::new("fist")));
    // A deity costs its hash on top of the faith's byte.
    assert!(encode_build(&follower).len() > encode_build(&atheist).len());
}

/// A key is looked up only among the keys of its own kind: the class `rogue` in a link is never the feat
/// `rogue`.
#[test]
fn a_key_of_another_kind_does_not_stand_in_for_a_missing_one() {
    let mut build = v1_build();
    // `trickster` is a mythic path in the world, not a feat.
    build.picks.insert(general_slot(3), pick_feat("trickster"));
    assert_eq!(decode(&encode_build(&build)), Err(DecodeError::UnknownKey));
}

/// The link holds a fixed-width hash of each key, never its text, so a long key costs no more than a short one.
#[test]
fn a_keys_length_does_not_change_the_links_length() {
    let with = |key: &str| {
        let mut build = Build::default();
        build.picks.insert(general_slot(1), pick_feat(key));
        encode_build(&build).len()
    };
    assert_eq!(
        with("a"),
        with("a-feat-whose-key-is-much-longer-than-any-the-guide-has")
    );
}

/// The hash is FNV-1a 32-bit over the key's UTF-8 bytes, cut to its low 24 bits (the module docs' spec). The
/// first three are FNV's published test vectors.
#[test]
fn a_keys_hash_is_fnv_1a_cut_to_24_bits() {
    assert_eq!(key_hash(""), 0x811c9dc5 & 0xff_ffff);
    assert_eq!(key_hash("a"), 0xe40c292c & 0xff_ffff);
    assert_eq!(key_hash("foobar"), 0xbf9cf968 & 0xff_ffff);
    assert_eq!(key_hash("power-attack"), 0x00ad89c9);
}

/// `collides-58769` and `collides-60836` share their 24-bit hash (`0x284dd8`). Two keys of one kind with one hash
/// are reported, and a link naming either can't say which it means, so it doesn't decode. Keys of different kinds
/// may share a hash: each kind is looked up on its own.
#[test]
fn two_keys_of_one_kind_with_one_hash_are_a_collision() {
    assert_eq!(key_hash("collides-58769"), key_hash("collides-60836"));

    let mut build = v1_build();
    build
        .picks
        .insert(general_slot(3), pick_feat("collides-58769"));
    build
        .picks
        .insert(general_slot(5), pick_feat("collides-60836"));
    let keys = LinkKeys::of_build(&build);
    assert_eq!(
        keys.collisions(),
        [KeyCollision {
            kind: KeyKind::Feat,
            keys: ["collides-58769".into(), "collides-60836".into()],
        }]
    );
    assert_eq!(
        decode_build(&encode_build(&build), &keys),
        Err(DecodeError::UnknownKey)
    );

    let mut apart = v1_build();
    apart
        .picks
        .insert(general_slot(3), pick_feat("collides-58769"));
    apart.picks.insert(
        SlotId::Mythic { rank: 4 },
        pick_mythic_feat("collides-60836"),
    );
    let keys = LinkKeys::of_build(&apart);
    assert!(keys.collisions().is_empty());
    assert_eq!(decode_build(&encode_build(&apart), &keys), Ok(apart));
}

/// Keys carry no type prefix, so a feat, a mythic feat and a selection option may share one: the [`Pick`] variant
/// tells them apart.
#[test]
fn picks_of_different_kinds_with_the_same_key_stay_apart_in_a_link() {
    let mut build = v1_build();
    build
        .picks
        .insert(general_slot(3), pick_feat("power-attack"));
    build
        .picks
        .insert(SlotId::Mythic { rank: 2 }, pick_mythic_feat("power-attack"));
    build
        .picks
        .insert(selection_slot(5, 0), pick_option("power-attack"));

    let decoded = decode_build(&encode_build(&build), &LinkKeys::of_build(&build)).unwrap();

    assert_eq!(decoded.picks[&general_slot(3)], pick_feat("power-attack"));
    assert_eq!(
        decoded.picks[&SlotId::Mythic { rank: 2 }],
        pick_mythic_feat("power-attack")
    );
    assert_eq!(
        decoded.picks[&selection_slot(5, 0)],
        pick_option("power-attack")
    );
}

#[test]
fn bad_links_are_errors_not_panics() {
    assert_eq!(decode(""), Err(DecodeError::Empty));
    assert_eq!(decode("not base64!"), Err(DecodeError::Base64));
    // Version 9 doesn't exist: 0x09 is "CQ" in base64url.
    assert_eq!(decode("CQ"), Err(DecodeError::UnsupportedVersion(9)));
    // Version 1 followed by bytes that aren't deflate data.
    assert_eq!(decode("Af____8"), Err(DecodeError::Decompress));

    // Valid deflate of a truncated Build.
    let code = encode_build(&v1_build());
    let mut shortened = code.clone();
    shortened.truncate(code.len() / 2);
    assert!(decode(&shortened).is_err());
}

#[test]
fn a_hand_edited_link_with_ranks_above_the_level_or_a_misplaced_increase_does_not_decode() {
    // Two ranks at level 1.
    let mut ranks = v1_build();
    ranks.levels[0].skill_ranks.insert(skill("athletics"), 2);
    assert_eq!(decode(&encode_build(&ranks)), Err(DecodeError::Format));
    // One at level 1 and two more at level 2: three at level 2.
    let mut ranks = v1_build();
    ranks.levels[1].skill_ranks.insert(skill("athletics"), 2);
    assert_eq!(decode(&encode_build(&ranks)), Err(DecodeError::Format));
    // One more at level 2 fits.
    ranks.levels[1].skill_ranks.insert(skill("athletics"), 1);
    assert_eq!(decode(&encode_build(&ranks)), Ok(ranks));

    // An ability increase at level 3, which has none.
    let mut increase = v1_build();
    increase.levels[2].ability_increase = Some(Ability::Strength);
    assert_eq!(decode(&encode_build(&increase)), Err(DecodeError::Format));
    // At level 4 it decodes.
    increase.levels[2].ability_increase = None;
    increase.levels.push(LevelUp {
        ability_increase: Some(Ability::Strength),
        ..LevelUp::new(class("rogue"))
    });
    assert_eq!(decode(&encode_build(&increase)), Ok(increase));
}

/// Before ticket 46 this Build's link was 604 characters, its keys' text making up most of it.
#[test]
fn a_full_40_level_build_makes_a_link_of_a_few_hundred_characters() {
    let mut build = build(
        "human",
        &[times("fighter", 20), times("rogue", 20)].concat(),
    );
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("legend"));
    let feats: Vec<_> = world()
        .canonical()
        .feats
        .iter()
        .map(|f| f.key.clone())
        .collect();
    for level in 1..=40u8 {
        let feat = feats[usize::from(level) % feats.len()].clone();
        if level % 2 == 1 {
            build
                .picks
                .insert(general_slot(level), Pick::feat(feat.clone()));
        }
        if level <= 20 && (level == 1 || level % 2 == 0) {
            build.picks.insert(class_slot(level, 0), Pick::feat(feat));
        }
        build.levels[usize::from(level) - 1]
            .skill_ranks
            .insert(skill("mobility"), 1);
    }
    let code = encode_build(&build);
    assert!(code.len() <= 604 / 2, "{} characters", code.len());
    assert_eq!(decode_build(&code, &LinkKeys::of_build(&build)), Ok(build));
}
