//! The URL codec: `/#b=<code>` is a version byte, then the deflated postcard Build, in base64url.

use std::collections::BTreeMap;

use proptest::collection::{btree_map, btree_set, vec};
use proptest::option;
use proptest::prelude::*;
use rules::{
    Ability, AbilityScores, Alignment, ArchetypeKey, ArmourType, BackgroundKey, Build, BuildStart,
    ClassChoice, ClassKey, DecodeError, Energy, FeatSource, LevelUp, MythicPathKey, Parameter,
    Pick, PointBuy, RaceKey, SkillKey, SlotId, SpellSchool, SubraceKey, decode_build, encode_build,
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

fn start() -> impl Strategy<Value = BuildStart> {
    (
        option::of(key().prop_map(RaceKey)),
        option::of(key().prop_map(SubraceKey)),
        option::of(ability()),
        option::of(alignment()),
        option::of(key().prop_map(BackgroundKey)),
        point_buy_strategy(),
    )
        .prop_map(
            |(race, subrace, racial_ability, alignment, background, ability_scores)| BuildStart {
                race,
                subrace,
                racial_ability,
                alignment,
                background,
                ability_scores,
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
        key().prop_map(|k| Pick::MythicAbility(rules::MythicAbilityKey(k))),
        (key(), option::of(parameter())).prop_map(|(k, parameter)| Pick::MythicFeat {
            feat: rules::MythicFeatKey(k),
            parameter
        }),
    ]
}

fn any_build() -> impl Strategy<Value = Build> {
    (
        option::of(".{0,40}"),
        start(),
        vec(level_up(), 0..40),
        btree_map(key().prop_map(ClassKey), class_choice(), 0..4),
        option::of(key().prop_map(MythicPathKey)),
        option::of(key().prop_map(MythicPathKey)),
        btree_map(slot_id(), pick(), 0..40),
    )
        .prop_map(
            |(name, start, levels, class_choices, mythic_path, path_switch, picks)| Build {
                name,
                start,
                levels: possible(levels),
                class_choices,
                mythic_path,
                path_switch,
                picks,
            },
        )
}

proptest! {
    #[test]
    fn every_build_round_trips_through_its_link(build in any_build()) {
        let code = encode_build(&build);
        prop_assert!(
            code.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
            "not base64url: {code}"
        );
        prop_assert_eq!(decode_build(&code), Ok(build));
    }
}

/// The Build frozen in [`V1_LINK`]. Never change either: old links must keep decoding.
fn v1_build() -> Build {
    let mut build = build("human", &["fighter", "fighter", "rogue"]);
    build.name = Some("Cleaver".into());
    build.start.racial_ability = Some(Ability::Strength);
    build.start.background = Some(BackgroundKey::new("shackles-corsair"));
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
        SlotId::Mythic { rank: 2 },
        Pick::mythic_feat_for(
            rules::MythicFeatKey::new("school-mastery"),
            Parameter::SpellSchool(rules::SpellSchool::Evocation),
        ),
    );
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
/// 2026-11-01 when a mythic feat pick gained its parameter (ticket 32: School Mastery (Evocation)), and on
/// 2026-11-01 when the Build start gained its background (ticket 34: Shackles Corsair).
const V1_LINK: &str = "AXVQS27CQAy1wzcVIJbtrhcY5RA9iXFMZsRkjDyTRty-Q5C6Y-P_-8i4-4lCv2K48dNICRBwjefsiW9RsmO1TMHOp8PxY7_aXcPgixhgS8VHKYEz_g9hYzpMAoCvAj-X1EnsLRT2LrNOqTeJFV8s8C0_YW3VBDjcdRZzVEpVrhR1tuXFWm2aup-F7prcVXnKiG3UNORZrccV4PdLaImuUJRUcsc6Xqi4RQgabI7jo_jAjj3ZIE2zOmX2qtGN9PTxwGa9ecd1rSeuXlEsHrb1R19ZjcXEuktU7WNI0pExJYHdHqAtOg0-Sc7wBw";

#[test]
fn a_version_1_link_still_decodes() {
    assert_eq!(decode_build(V1_LINK), Ok(v1_build()));
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

/// Decoding never looks at the game data; opening the link does (`stale_links.rs`).
#[test]
fn a_link_with_keys_the_game_data_lacks_still_decodes() {
    let mut build = v1_build();
    build
        .picks
        .insert(general_slot(3), pick_feat("cut-in-a-later-patch"));
    let decoded = decode_build(&encode_build(&build)).expect("unknown keys still decode");
    assert_eq!(decoded, build);
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

    let decoded = decode_build(&encode_build(&build)).unwrap();

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
    assert_eq!(decode_build(""), Err(DecodeError::Empty));
    assert_eq!(decode_build("not base64!"), Err(DecodeError::Base64));
    // Version 9 doesn't exist: 0x09 is "CQ" in base64url.
    assert_eq!(decode_build("CQ"), Err(DecodeError::UnsupportedVersion(9)));
    // Version 1 followed by bytes that aren't deflate data.
    assert_eq!(decode_build("Af____8"), Err(DecodeError::Decompress));

    // Valid deflate of a truncated Build.
    let code = encode_build(&v1_build());
    let mut shortened = code.clone();
    shortened.truncate(code.len() / 2);
    assert!(decode_build(&shortened).is_err());
}

#[test]
fn a_hand_edited_link_with_ranks_above_the_level_or_a_misplaced_increase_does_not_decode() {
    // Two ranks at level 1.
    let mut ranks = v1_build();
    ranks.levels[0].skill_ranks.insert(skill("athletics"), 2);
    assert_eq!(
        decode_build(&encode_build(&ranks)),
        Err(DecodeError::Format)
    );
    // One at level 1 and two more at level 2: three at level 2.
    let mut ranks = v1_build();
    ranks.levels[1].skill_ranks.insert(skill("athletics"), 2);
    assert_eq!(
        decode_build(&encode_build(&ranks)),
        Err(DecodeError::Format)
    );
    // One more at level 2 fits.
    ranks.levels[1].skill_ranks.insert(skill("athletics"), 1);
    assert_eq!(decode_build(&encode_build(&ranks)), Ok(ranks));

    // An ability increase at level 3, which has none.
    let mut increase = v1_build();
    increase.levels[2].ability_increase = Some(Ability::Strength);
    assert_eq!(
        decode_build(&encode_build(&increase)),
        Err(DecodeError::Format)
    );
    // At level 4 it decodes.
    increase.levels[2].ability_increase = None;
    increase.levels.push(LevelUp {
        ability_increase: Some(Ability::Strength),
        ..LevelUp::new(class("rogue"))
    });
    assert_eq!(decode_build(&encode_build(&increase)), Ok(increase));
}

#[test]
fn a_full_40_level_build_makes_a_link_of_a_few_hundred_characters() {
    let mut build = build(
        "human",
        &[times("fighter", 20), times("rogue", 20)].concat(),
    );
    build.mythic_path = Some(path("trickster"));
    build.path_switch = Some(path("legend"));
    let feats: Vec<_> = engine()
        .data()
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
    assert!(code.len() < 800, "{} characters", code.len());
    let _: BTreeMap<SlotId, Pick> = build.picks;
}
