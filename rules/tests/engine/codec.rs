//! The URL codec: `/#b=<code>` is a version byte, then the deflated postcard Build, in base64url.

use std::collections::BTreeMap;

use proptest::collection::{btree_map, btree_set, vec};
use proptest::option;
use proptest::prelude::*;
use rules::{
    Ability, AbilityScores, Alignment, ArchetypeKey, Build, BuildStart, ClassChoice, ClassKey,
    DecodeError, FeatSource, LevelUp, MythicPathKey, Pick, RaceKey, SkillKey, SlotId, SubraceKey,
    decode_build, encode_build,
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
        prop::array::uniform6(any::<u8>()),
    )
        .prop_map(|(race, subrace, racial_ability, alignment, s)| BuildStart {
            race,
            subrace,
            racial_ability,
            alignment,
            ability_scores: scores(s),
        })
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
    ]
}

fn pick() -> impl Strategy<Value = Pick> {
    prop_oneof![
        key().prop_map(|k| Pick::Feat(rules::FeatKey(k))),
        key().prop_map(|k| Pick::SelectionOption(rules::SelectionOptionKey(k))),
        key().prop_map(|k| Pick::MythicAbility(rules::MythicAbilityKey(k))),
        key().prop_map(|k| Pick::MythicFeat(rules::MythicFeatKey(k))),
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
                levels,
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
    build.levels[0].skill_ranks.insert(skill("athletics"), 1);
    build.picks.insert(race_slot(0), pick_feat("power-attack"));
    build.picks.insert(class_slot(1, 0), pick_feat("cleave"));
    build.picks.insert(general_slot(1), pick_feat("toughness"));
    build.picks.insert(
        selection_slot(3, 0),
        pick_option("rogue/rogue-talents/combat-trick"),
    );
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        pick_mythic_ability("mythic-charge"),
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
/// before any link shipped: when archetype and advancement moved into `Build::class_choices`, and on 2026-11-01 when
/// keys lost their type prefix (`feat:power-attack` became `power-attack`).
const V1_LINK: &str = "AT1OWw6CMBDcBRExYPz00ws0HMKT1GWlDYWadqvx9mJJ_JmZTDIPrG-O9YsDVibNegEE3J1PbXc8lPXDjkY4ADZajGOxFPFvQhX8mBgAN4GXTD27IVghoyL5tAyB3ZqXYGmKv1i1LgC0T__moLSIpglwtfaUfwAWAI34NJqFY8QS8Lr1ZlSiHS8Se_LzXYvKvQUW3fwRY0mR0WHkLw";

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

#[test]
fn a_link_with_keys_the_game_data_lacks_decodes_and_reports_them() {
    let mut build = v1_build();
    build
        .picks
        .insert(general_slot(3), pick_feat("cut-in-a-later-patch"));
    let decoded = decode_build(&encode_build(&build)).expect("unknown keys still decode");
    assert_eq!(decoded, build);

    let unknown: Vec<_> = engine()
        .validate(&decoded)
        .into_iter()
        .filter(rules::Problem::is_unknown_key)
        .map(|p| p.at)
        .collect();
    assert_eq!(unknown, [rules::Choice::Slot(general_slot(3))]);
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
                .insert(general_slot(level), Pick::Feat(feat.clone()));
        }
        if level <= 20 && (level == 1 || level % 2 == 0) {
            build.picks.insert(class_slot(level, 0), Pick::Feat(feat));
        }
        build.levels[usize::from(level) - 1]
            .skill_ranks
            .insert(skill("mobility"), 1);
    }
    let code = encode_build(&build);
    assert!(code.len() < 800, "{} characters", code.len());
    let _: BTreeMap<SlotId, Pick> = build.picks;
}
