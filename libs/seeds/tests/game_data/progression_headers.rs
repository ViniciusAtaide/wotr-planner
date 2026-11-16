//! The progression header (ticket 304): an option's level is made as its blueprint's `BlueprintProgression` says
//! (`m_Classes`, `m_Archetypes`, `ForAllOtherClasses` with `AlternateProgressionType`), as `CalcLevel` adds it
//! (Blueprints/Classes/BlueprintProgression.cs, WrathDecompiled 6ce4a6b). The five option levels that moved toward the
//! game (decisions Q1), each from its blueprint.

use std::collections::BTreeMap;

use rules::{
    ArchetypeKey, Build, ClassChoice, ClassKey, LevelUp, ProgressionHeader, SelectionOptionKey,
};

use crate::support::*;

/// An option's header, from the data.
fn header(option: &str) -> ProgressionHeader {
    DATA.selection_option(&SelectionOptionKey::new(option))
        .unwrap_or_else(|| panic!("no option {option}"))
        .header
        .clone()
        .unwrap_or_else(|| panic!("{option} has no header"))
}

fn classes(of: &[(&str, i8)]) -> BTreeMap<ClassKey, i8> {
    of.iter()
        .map(|&(class, offset)| (ClassKey::new(class), offset))
        .collect()
}

fn archetypes(of: &[(&str, i8)]) -> BTreeMap<ArchetypeKey, i8> {
    of.iter()
        .map(|&(archetype, offset)| (ArchetypeKey::new(archetype), offset))
        .collect()
}

/// `build` then `levels` levels of `class`, taken with `archetype` when given.
fn then(mut build: Build, class: &str, archetype: Option<&str>, levels: usize) -> Build {
    build.class_choices.insert(
        ClassKey::new(class),
        ClassChoice {
            archetype: archetype.map(ArchetypeKey::new),
            ..ClassChoice::default()
        },
    );
    for _ in 0..levels {
        build.levels.push(LevelUp::new(ClassKey::new(class)));
    }
    faithful(build)
}

/// The option level of `option` at the Build's last level.
fn level_of(build: &Build, option: &str) -> u8 {
    let last = u8::try_from(build.levels.len()).unwrap();
    at(build, last).option_level(&SelectionOptionKey::new(option))
}

/// A sorcerer's non-Draconic bloodline counts sorcerer levels and an Eldritch Scion magus's, not a plain magus's nor
/// a Dragon Disciple's (the planner counted the Dragon Disciple's before ticket 304).
#[test]
fn a_non_draconic_bloodline_counts_an_eldritch_scions_magus_levels_and_no_dragon_disciples() {
    // Game: BloodlineAbyssalProgression d3a4cb7be97a6694290f0dcfbd147113: `m_Classes` SorcererClass
    // b3a505fb61437dc4097f43c3f8f9a4cf and MagusClass 45a4607686d96a1498891b3286121780, `m_Archetypes`
    // EldritchScionArchetype d078b2ef073f2814c9e338a789d97b73, each +0; `GiveFeaturesForPreviousLevels` true.
    assert_eq!(
        header(ABYSSAL),
        ProgressionHeader {
            classes: classes(&[("magus", 0), ("sorcerer", 0)]),
            archetypes: archetypes(&[("magus/eldritch-scion", 0)]),
            half_other_classes: false,
            previous_levels: true,
        }
    );
    let sorcerer = with_option(build("sorcerer", 3), 1, 0, ABYSSAL);
    // Sorcerer 3 and Eldritch Scion 3: 6. A plain magus has an archetype the header names, so its levels count only
    // taken with it (`CalcLevel`): 3.
    let scion = then(sorcerer.clone(), "magus", Some("magus/eldritch-scion"), 3);
    assert_eq!(level_of(&scion, ABYSSAL), 6);
    let magus = then(sorcerer, "magus", None, 3);
    assert_eq!(level_of(&magus, ABYSSAL), 3);
}

/// A Draconic bloodline counts sorcerer, Dragon Disciple and Eldritch Scion magus levels, whichever selection holds
/// it (the sorcerer's counted no magus levels before, the Dragon Disciple's no sorcerer or magus levels).
#[test]
fn a_draconic_bloodline_counts_sorcerer_dragon_disciple_and_eldritch_scion_levels() {
    // Game: BloodlineDraconicRedProgression 8c6e5b3cf12f71e43949f52c41ae70a8: `m_Classes` SorcererClass
    // b3a505fb61437dc4097f43c3f8f9a4cf, MagusClass 45a4607686d96a1498891b3286121780 and DragonDiscipleClass
    // 72051275b1dbb2d42ba9118237794f7c, `m_Archetypes` EldritchScionArchetype d078b2ef073f2814c9e338a789d97b73, each
    // +0.
    let red = "sorcerer/bloodline/draconic-red";
    assert_eq!(
        header(red),
        ProgressionHeader {
            classes: classes(&[("dragon-disciple", 0), ("magus", 0), ("sorcerer", 0)]),
            archetypes: archetypes(&[("magus/eldritch-scion", 0)]),
            half_other_classes: false,
            previous_levels: true,
        }
    );
    // Sorcerer 3 and Eldritch Scion 2: 5.
    let sorcerer = with_option(build("sorcerer", 3), 1, 0, red);
    let scion = then(sorcerer, "magus", Some("magus/eldritch-scion"), 2);
    assert_eq!(level_of(&scion, red), 5);
}

/// A bloodrager's Draconic bloodline counts Dragon Disciple levels.
#[test]
fn a_bloodragers_draconic_bloodline_counts_dragon_disciple_levels() {
    // Game: BloodragerDragonRedBloodline 34209f220733fc444a039df1b1076b0b: `m_Classes` BloodragerClass
    // d77e67a814d686842802c9cfd8ef8499 and DragonDiscipleClass 72051275b1dbb2d42ba9118237794f7c, each +0.
    assert_eq!(
        header("bloodrager/bloodline/draconic-red"),
        ProgressionHeader {
            classes: classes(&[("bloodrager", 0), ("dragon-disciple", 0)]),
            archetypes: BTreeMap::new(),
            half_other_classes: false,
            previous_levels: true,
        }
    );
}

/// The shaman's Frost spirit counts Winter Witch levels.
#[test]
fn the_frost_spirit_counts_winter_witch_levels() {
    // Game: ShamanFrostSpiritProgression 44c130a12d49ebf4a95945feffe5aae7: `m_Classes` ShamanClass
    // 145f1d3d360a7ad48bd95d392c81b38e and WinterWitchClass eb24ca44debf6714aabe1af1fd905a07, each +0.
    assert_eq!(
        header("shaman/spirit/frost"),
        ProgressionHeader {
            classes: classes(&[("shaman", 0), ("winter-witch", 0)]),
            archetypes: BTreeMap::new(),
            half_other_classes: false,
            previous_levels: true,
        }
    );
    // Another spirit counts shaman levels only.
    // Game: ShamanBattleSpiritProgression 26e81d83d13dbf6448b10019c802612d: `m_Classes` ShamanClass alone.
    assert_eq!(
        header("shaman/spirit/battle").classes,
        classes(&[("shaman", 0)])
    );
}

/// Weapon Mastery counts oracle levels and a Magic Deceiver's arcanist levels, wherever it's held (the oracle's
/// revelations let its header go first; the revelations' own oracle levels give the others').
#[test]
fn weapon_mastery_counts_oracle_and_magic_deceiver_levels() {
    // Game: OracleRevelationWeaponMastery 0a4c3556355747241b2d3dcc0a88ec10: `m_Classes` OracleClass
    // 20ce9bf8af32bee4c8557a045ab499b1 and ArcanistClass 52dbfd8505e22f84fad8d702611f60b7, `m_Archetypes`
    // MagicDeceiver 5c77110cd0414e7eb4c2e485659c9a46, each +0.
    let mastery = "oracle/revelations/battle/weapon-mastery";
    assert_eq!(
        header(mastery),
        ProgressionHeader {
            classes: classes(&[("arcanist", 0), ("oracle", 0)]),
            archetypes: archetypes(&[("arcanist/magic-deceiver", 0)]),
            half_other_classes: false,
            previous_levels: true,
        }
    );
    for revelations in [
        "oracle/revelations",
        "oracle/dual-cursed-oracle/revelation",
        "arcanist/magic-deceiver/revelation",
    ] {
        let selection = any_selection(revelations);
        assert!(
            selection.offers(&SelectionOptionKey::new(mastery)),
            "{revelations}"
        );
        let option = DATA
            .selection_option(&SelectionOptionKey::new(mastery))
            .unwrap();
        assert!(selection.takes_header_of(option), "{revelations}");
    }
}

/// A wizard's school and a kineticist's element give only the row of the level they reach
/// (`GiveFeaturesForPreviousLevels` false), and every row as a level-up reaches it.
#[test]
fn a_school_and_an_element_get_no_rows_of_the_levels_before() {
    // Game: SpecialisationSchoolAbjurationProgression c451fde0aec46454091b70384ea91989 and ElementalFocusAir
    // 2bd0d44953a536f489082534c48f8e31: `GiveFeaturesForPreviousLevels` false
    // (`LevelUpHelper.UpdateProgression`, WrathDecompiled 6ce4a6b, then adds only the reached level's entry).
    for option in ["wizard/school/abjurer", "kineticist/element/air"] {
        assert!(!header(option).previous_levels, "{option}");
    }
    // The bloodlines' do (BloodlineAbyssalProgression d3a4cb7be97a6694290f0dcfbd147113).
    assert!(header(ABYSSAL).previous_levels);
}
