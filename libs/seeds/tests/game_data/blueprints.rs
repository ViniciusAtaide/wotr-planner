//! Ticket 237: every entry with a game blueprint behind it names its GUID. Each spot below cites the blueprint by its
//! name and GUID in the game's 2.7.0x data (BubblePrintsData `2.7.0x_0`), across the kinds, and includes some of the
//! research's mappings the hand review corrected. The entries left empty, and why, are in the ticket's Answer.

use std::collections::BTreeMap;

use rules::*;

use crate::support::*;

/// Every entry's blueprint, by where it is.
fn blueprints() -> BTreeMap<DataAt, Option<&'static str>> {
    DATA.blueprints()
        .into_iter()
        .map(|(at, guid)| (at, guid.map(BlueprintGuid::as_str)))
        .collect()
}

#[test]
fn entries_name_their_game_blueprint() {
    let spots = [
        // RogueClass
        (
            DataAt::Class(ClassKey::new("rogue")),
            "299aa766dee3cbf4790da4efb8c72484",
        ),
        // ArcaneTricksterClass: a prestige class is a class.
        (
            DataAt::Class(ClassKey::new("arcane-trickster")),
            "9c935a076d4fe4d4999fd48d853e3cf3",
        ),
        // SneakAttack
        (
            DataAt::Feature(FeatureKey::new("rogue/sneak-attack")),
            "9b9eac6709e1c084cb18c3a366e0ec87",
        ),
        // EldritchScoundrelArchetype
        (
            DataAt::Archetype(ArchetypeKey::new("rogue/eldritch-scoundrel")),
            "57f93dd8423c97c49989501281296c4a",
        ),
        // PowerAttackFeature
        (
            DataAt::Feat(FeatKey::new("power-attack")),
            "9972f33f977fc724c838e59641b2fca5",
        ),
        // TricksterMythicClass
        (
            DataAt::MythicPath(MythicPathKey::new("trickster")),
            "8df873a8c6e48294abdb78c45834aa0a",
        ),
        // AbundantKiPool
        (
            DataAt::MythicAbility(MythicAbilityKey::new("abundant-ki-pool")),
            "e8752f9126d986748b10d0bdac693264",
        ),
        // SneakAttackerMythicFeat
        (
            DataAt::MythicFeat(MythicFeatKey::new("mythic-sneak-attack")),
            "d0a53bf03b978634890e5ebab4a90ecb",
        ),
        // HumanRace
        (
            DataAt::Race(RaceKey::new("human")),
            "0a5d473ead98b0646b94495af250fdc4",
        ),
        // TieflingHeritageDemon
        (
            DataAt::Subrace(SubraceKey::new("tiefling/pitborn")),
            "c09ffb2657f6c2b41b5ed0ed607b362a",
        ),
        // BackgroundAcolyte
        (
            DataAt::Background(BackgroundKey::new("acolyte")),
            "7cc9014488caa5445a0c8b5e17b95466",
        ),
        // CaydenCaileanFeature
        (
            DataAt::Deity(DeityKey::new("cayden-cailean")),
            "300e212868bca984687c92bcb66d381b",
        ),
        // BeltOfConstitution2
        (
            DataAt::Gear(GearKey::new("belt-of-the-mighty-constitution-2")),
            "b9cc5d85d4a5032458ef4491f5fed251",
        ),
        // Fireball
        (
            DataAt::Spell(SpellKey::new("fireball")),
            "2d81362af43aeac4387a3d4fced489c3",
        ),
        // WizardSpellList
        (
            DataAt::SpellList(SpellListKey::new("wizard")),
            "ba0401fdeb4062f40a7aa95b6f07fe89",
        ),
        // Longsword (the weapon type named as its category)
        (
            DataAt::Weapon(WeaponKey::new("longsword")),
            "d56c44bc9eb10204c8b386a02c7eed21",
        ),
        // BiteType: the base type of category Bite, which the per-weapon feats name
        (
            DataAt::Weapon(WeaponKey::new("bite")),
            "952e30e6cb40b454789a9db6e5f6dd09",
        ),
        // SpecialistSchoolSelection
        (
            DataAt::FeatureSelection(FeatureSelectionKey::new("wizard/school")),
            "5f838049069f1ac4d804ce0862ab5110",
        ),
        // SpecialisationSchoolEvocationProgression: left unmapped by the research
        (
            DataAt::SelectionOption(SelectionOptionKey::new("wizard/school/invoker")),
            "f8019b7724d72a241a97157bc37f1c3b",
        ),
        // RangerStyleArcherySelection2, which NomadArchetype and StormwalkerArchetype add at 2
        (
            DataAt::FeatList(FeatListKey::new("ranger/bonus-feats-2b")),
            "3cf94df3f1cf9a443a880bc1ae9be3c6",
        ),
        // OracleRevelationSelection: a per-mystery pool is the game's one revelation selection, filtered
        (
            DataAt::FeatureSelection(FeatureSelectionKey::new("oracle/revelations/flame")),
            "60008a10ad7ad6543b1f63016741a5d2",
        ),
        // ShifterAspectBear: the research's elimination pairing had ShifterAspectSpider
        (
            DataAt::SelectionOption(SelectionOptionKey::new(
                "shifter/shifter-aspect/bear-aspect",
            )),
            "73ef2c330ff14e4bb2f5b7300622c552",
        ),
        // FireBlessingMajorFeature: the research had the blessing itself
        (
            DataAt::Feature(FeatureKey::new("warpriest/blessings/fire/major-blessing")),
            "f5b993b62c3ef064ba9f35d1f21611d8",
        ),
        // ElementalFocusAir, the kineticist's: the research had the selection
        (
            DataAt::SelectionOption(SelectionOptionKey::new("kineticist/element/air")),
            "2bd0d44953a536f489082534c48f8e31",
        ),
    ];
    let blueprints = blueprints();
    for (at, guid) in spots {
        assert_eq!(blueprints.get(&at), Some(&Some(guid)), "{at}");
    }
}

/// The entries left empty, by kind, as the ticket's Answer lists them: planner placeholders (the bloodline-spell
/// features), a feature the game keeps as one blueprint per colour or element, an ability rather than a feature, a
/// planner-made pool or list. A change here is a change to that list.
#[test]
fn the_entries_left_without_a_blueprint_are_the_answers() {
    let mut empty: BTreeMap<&str, usize> = BTreeMap::new();
    for (at, blueprint) in blueprints() {
        if blueprint.is_none() {
            *empty.entry(at.kind()).or_default() += 1;
        }
    }
    assert_eq!(
        empty,
        BTreeMap::from([
            ("feature", 129),
            ("gear item", 1),
            ("selection", 2),
            ("selection option", 49),
            ("spell", 1),
            ("spell list", 2),
            ("weapon", 1),
        ])
    );
}

/// The research left oracle/curse-2 to -4 on `OracleCurse` (403766e65e2d4f06b185443c4bb65700), which OracleClass
/// only shows as a signature ability; OracleProgression grants nothing at 5, 10 and 15, so they stay empty.
#[test]
fn the_curse_stages_name_no_blueprint() {
    let blueprints = blueprints();
    for stage in ["oracle/curse-2", "oracle/curse-3", "oracle/curse-4"] {
        let at = DataAt::Feature(FeatureKey::new(stage));
        assert_eq!(blueprints.get(&at), Some(&None), "{stage}");
    }
}
