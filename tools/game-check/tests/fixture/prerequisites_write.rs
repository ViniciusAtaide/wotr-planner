//! The prerequisites target (ticket 308a), on a world of its own: the write fixture's data with entries whose
//! blueprints carry `Prerequisite*` components, and a dump of those blueprints.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use game_check::write::facts::Facts;
use game_check::write::{Outcome, WriteArea, WriteError, run};
use rules::{
    Ability, Alignment, ArchetypeKey, ClassKey, Component, Condition, Fact, FeatKey, GameData,
    Group, Prerequisite, SelectionOptionKey, StatRef,
};

use crate::support::{Record, assert_agrees, binz, data_copy, dump};

const FEATURE: &str = "Kingmaker.Blueprints.Classes.BlueprintFeature";
const SELECTION: &str = "Kingmaker.Blueprints.Classes.Selection.BlueprintFeatureSelection";

/// The write fixture's own entries a prerequisite names: two feats, the class and its two archetypes.
const BONUS_ONE: &str = "000000000000000000000000000000b11";
const BONUS_TWO: &str = "000000000000000000000000000000b12";
const WARDEN: &str = "000000000000000000000000000000c1";
const STOIC: &str = "000000000000000000000000000000c3";
const STRAY: &str = "000000000000000000000000000000c4";

fn guid(n: u32) -> String {
    format!("{:032x}", 0xfb00_0000u32 + n)
}

fn bp(guid: &str) -> String {
    format!("\"!bp_{guid}\"")
}

/// One component of a group, with its other fields (`, "name": value…`).
fn part(kind: &str, group: &str, fields: &str) -> String {
    format!("{{\"$type\": \"0, {kind}\", \"Group\": \"{group}\"{fields}}}")
}

fn needs(group: &str, feature: &str) -> String {
    part(
        "PrerequisiteFeature",
        group,
        &format!(", \"m_Feature\": {}", bp(feature)),
    )
}

fn no_feature(feature: &str) -> String {
    part(
        "PrerequisiteNoFeature",
        "All",
        &format!(", \"m_Feature\": {}", bp(feature)),
    )
}

/// A selection's cap on its own rank: `Not` of `ContextConditionBuffRank` on itself at `rank`.
fn cap(own: &str, rank: u8) -> String {
    let rank = format!(
        "{{\"$type\": \"0, ContextConditionBuffRank\", \"Not\": true, \"UseFactInsteadBuff\": true, \"Fact\": {}, \"RankValue\": {{\"ValueType\": \"Simple\", \"Value\": {rank}}}}}",
        bp(own)
    );
    part(
        "PrerequisiteCondition",
        "All",
        &format!(
            ", \"Condition\": {{\"$type\": \"0, OrAndLogic\", \"Not\": false, \"ConditionsChecker\": {{\"Operation\": \"And\", \"Conditions\": [{rank}]}}}}"
        ),
    )
}

fn blueprint(n: u32, name: &str, type_name: &str, ranks: u8, parts: &[String]) -> Record {
    let kind = type_name.rsplit('.').next().unwrap();
    Record {
        guid: guid(n),
        name: name.to_owned(),
        type_name: type_name.to_owned(),
        raw: format!(
            "{{\"$type\": \"0, {kind}\", \"Ranks\": {ranks}, \"Components\": [{}]}}",
            parts.join(", ")
        ),
    }
}

/// The world: its data folder, its dump and the dump's pins.
///
/// - `keen-feat` (200): a feature, a class level, archetype levels 1 and 3, a stat (and one no Build fails), an
///   alignment and an `Any` group.
/// - `lost-feat` (201): a feature no entry maps (`GhostlyFeature`, 250).
/// - `needs-twin` (202): the blueprint 260, which the feat `twin-feat` and the option `twin-option` share; 260 refuses
///   itself.
/// - `once-feat` (203): a feature of one rank that refuses itself; `solo-pick` (204, an option): a selection that does.
/// - `extra-thing` (205, ranks 3) and `extra-odd` (206, ranks 2): a selection capped at its own rank 3.
/// - `hand-feat` (207): a hand prerequisite. `same-feat` (208): the game's two features, in the other order, with a
///   comment inside its components.
/// - the mythic ability `deep-path` (210): a Mythic rank and a feature.
fn world() -> (tempfile::TempDir, game_check::Pins, game_check::Dump) {
    let dir = data_copy();
    let put = |file: &str, text: String| std::fs::write(dir.path().join(file), text).unwrap();
    let feat = |key: &str, n: u32, more: &str| {
        put(
            &format!("feats/{key}.ron"),
            format!(
                "(key: \"{key}\", blueprint: \"{}\", name: \"Planted {key}\", category: Weapon{more})\n",
                guid(n)
            ),
        );
    };
    feat("keen-feat", 200, "");
    feat("lost-feat", 201, "");
    feat("needs-twin", 202, "");
    feat("twin-feat", 260, "");
    feat("once-feat", 203, "");
    feat("extra-thing", 205, ", ranks: 3");
    feat("extra-odd", 206, ", ranks: 2");
    feat("hand-feat", 207, ", prerequisite: Feat(\"bonus-one\")");
    put(
        "feats/same-feat.ron",
        format!(
            "(\n    key: \"same-feat\",\n    blueprint: \"{}\",\n    name: \"Same\",\n    category: Weapon,\n    components: [\n        // Planted: the game's order is Bonus One, then Bonus Two.\n        Requires(condition: Has(Feat(\"bonus-two\"))),\n        Requires(condition: Has(Feat(\"bonus-one\"))),\n    ],\n)\n",
            guid(208)
        ),
    );
    put(
        "options/twin-option.ron",
        format!(
            "(key: \"twin-option\", blueprint: \"{}\", name: \"Twin option\")\n",
            guid(260)
        ),
    );
    put(
        "options/solo-pick.ron",
        format!(
            "(key: \"solo-pick\", blueprint: \"{}\", name: \"Solo pick\")\n",
            guid(204)
        ),
    );
    put(
        "mythic/abilities/deep-path.ron",
        format!(
            "(key: \"deep-path\", blueprint: \"{}\", name: \"Deep Path\", group: Miscellaneous)\n",
            guid(210)
        ),
    );
    let class_level = |group: &str, level: u8| {
        part(
            "PrerequisiteClassLevel",
            group,
            &format!(
                ", \"Level\": {level}, \"Not\": false, \"m_CharacterClass\": {}",
                bp(WARDEN)
            ),
        )
    };
    let archetype_level = |group: &str, archetype: &str, level: u8| {
        part(
            "PrerequisiteArchetypeLevel",
            group,
            &format!(
                ", \"Level\": {level}, \"m_Archetype\": {}, \"m_CharacterClass\": {}",
                bp(archetype),
                bp(WARDEN)
            ),
        )
    };
    let stat = |name: &str, value: u8| {
        part(
            "PrerequisiteStatValue",
            "All",
            &format!(", \"Stat\": \"{name}\", \"Value\": {value}"),
        )
    };
    let keen = [
        needs("All", BONUS_ONE),
        class_level("All", 3),
        archetype_level("All", STOIC, 1),
        stat("Strength", 13),
        stat("Intelligence", 3),
        part("PrerequisiteAlignment", "All", ", \"Alignment\": \"Good\""),
        needs("Any", BONUS_TWO),
        archetype_level("Any", STRAY, 3),
        "{\"$type\": \"0, AddFacts\", \"m_Facts\": []}".to_owned(),
    ];
    let mythic_rank = part("PrerequisiteMythicLevel", "All", ", \"Level\": 3");
    let records = vec![
        blueprint(200, "KeenFeat", FEATURE, 1, &keen),
        blueprint(
            201,
            "LostFeat",
            FEATURE,
            1,
            &[needs("All", BONUS_ONE), needs("All", &guid(250))],
        ),
        blueprint(250, "GhostlyFeature", FEATURE, 1, &[]),
        blueprint(
            202,
            "NeedsTwinFeat",
            FEATURE,
            1,
            &[needs("All", &guid(260))],
        ),
        blueprint(260, "TwinFeature", FEATURE, 1, &[no_feature(&guid(260))]),
        blueprint(
            203,
            "OnceFeat",
            FEATURE,
            1,
            &[no_feature(&guid(203)), needs("All", BONUS_ONE)],
        ),
        blueprint(
            204,
            "SoloPickSelection",
            SELECTION,
            1,
            &[no_feature(&guid(204))],
        ),
        blueprint(
            205,
            "ExtraThingSelection",
            SELECTION,
            3,
            &[needs("Any", BONUS_ONE), cap(&guid(205), 3)],
        ),
        blueprint(
            206,
            "ExtraOddSelection",
            SELECTION,
            3,
            &[needs("Any", BONUS_ONE), cap(&guid(206), 3)],
        ),
        blueprint(207, "HandFeat", FEATURE, 1, &[needs("All", BONUS_TWO)]),
        blueprint(
            208,
            "SameFeat",
            FEATURE,
            1,
            &[needs("All", BONUS_ONE), needs("All", BONUS_TWO)],
        ),
        blueprint(
            210,
            "DeepPathAbility",
            FEATURE,
            1,
            &[mythic_rank, needs("All", BONUS_ONE)],
        ),
    ];
    let (pins, dump) = dump(&binz(&records, 1));
    (dir, pins, dump)
}

fn valid(path: &Path) -> Result<(), String> {
    seeds::read_game_data_files(path)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn write_with(
    dir: &Path,
    dump: &game_check::Dump,
    areas: &[WriteArea],
    only: Option<&[&str]>,
    facts: &str,
) -> Result<Outcome, WriteError> {
    let only: Option<BTreeSet<String>> =
        only.map(|names| names.iter().map(|name| (*name).to_owned()).collect());
    run(
        dump,
        dir,
        areas,
        only.as_ref(),
        false,
        &Facts::parse(facts).unwrap(),
        valid,
    )
}

fn write(dir: &Path, dump: &game_check::Dump) -> Outcome {
    write_with(dir, dump, &[WriteArea::Prerequisites], None, "").unwrap()
}

fn read(dir: &Path, file: &str) -> String {
    std::fs::read_to_string(dir.join(file)).unwrap()
}

fn parsed<T: serde::de::DeserializeOwned>(dir: &Path, file: &str) -> T {
    ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(&read(dir, file))
        .unwrap()
}

fn feat_components(dir: &Path, key: &str) -> Vec<Component> {
    parsed::<rules::Feat>(dir, &format!("feats/{key}.ron")).components
}

fn requires(condition: Condition, group: Group) -> Component {
    Component::Requires { condition, group }
}

fn feat(key: &str) -> Fact {
    Fact::Feat(FeatKey::new(key))
}

fn option(key: &str) -> Fact {
    Fact::Option(SelectionOptionKey::new(key))
}

fn changed(outcome: &Outcome) -> Vec<String> {
    outcome
        .changes
        .iter()
        .map(|change| change.at.to_string())
        .collect()
}

fn unexpressed(outcome: &Outcome, entry: &str) -> Vec<String> {
    outcome
        .unexpressed
        .iter()
        .filter(|item| item.at.to_string() == entry)
        .map(|item| item.what.clone())
        .collect()
}

/// The data as the check reads it: each entry's `Requires` compiled into its prerequisite, as
/// `GameData::checked` does (the fixture's data has planted problems, so it isn't checked whole).
fn compiled(mut data: GameData) -> GameData {
    let classes: BTreeMap<ArchetypeKey, ClassKey> = data
        .archetypes
        .iter()
        .map(|archetype| (archetype.key.clone(), archetype.class.clone()))
        .collect();
    let compile = |prerequisite: &mut Option<Prerequisite>, components: &mut Vec<Component>| {
        if let Some(mut tree) = rules::compile_requires(components, |a| classes.get(a)) {
            tree.canonicalize();
            *prerequisite = Some(tree);
            components.retain(|part| !part.is_requires());
        }
    };
    for each in &mut data.feats {
        compile(&mut each.prerequisite, &mut each.components);
    }
    for each in &mut data.options {
        compile(&mut each.prerequisite, &mut each.components);
    }
    for each in &mut data.mythic_abilities {
        compile(&mut each.prerequisite, &mut each.components);
    }
    for each in &mut data.mythic_feats {
        compile(&mut each.prerequisite, &mut each.components);
    }
    for each in data.classes.iter_mut().chain(&mut data.prestige_classes) {
        compile(&mut each.prerequisite, &mut each.components);
    }
    for each in &mut data.archetypes {
        compile(&mut each.prerequisite, &mut each.components);
    }
    data
}

fn report(dir: &Path, pins: &game_check::Pins, dump: &game_check::Dump) -> game_check::Report {
    let data = compiled(seeds::read_game_data_files(dir).unwrap());
    game_check::check(pins, dump, &data, &game_check::Known::parse("").unwrap()).unwrap()
}

#[test]
fn the_prerequisites_target_writes_each_component_as_a_requires() {
    let (dir, _, dump) = world();
    let outcome = write(dir.path(), &dump);
    let keen = outcome
        .changes
        .iter()
        .find(|change| change.at.to_string() == "feat keen-feat")
        .expect("keen-feat is written");
    assert_eq!((keen.field, keen.kind), ("components", "requires"));
    // In the blueprint's order; Intelligence 3, which no Build fails, and the `AddFacts` are left out.
    assert_eq!(
        feat_components(dir.path(), "keen-feat"),
        [
            requires(Condition::Has(feat("bonus-one")), Group::All),
            requires(
                Condition::ClassLevel(ClassKey::new("warden"), 3),
                Group::All
            ),
            requires(
                Condition::Has(Fact::Archetype(ArchetypeKey::new("warden/stoic"))),
                Group::All
            ),
            requires(
                Condition::Stat {
                    stat: StatRef::Ability(Ability::Strength),
                    value: 13,
                    full: false
                },
                Group::All
            ),
            requires(
                Condition::Alignment(BTreeSet::from([
                    Alignment::LawfulGood,
                    Alignment::NeutralGood,
                    Alignment::ChaoticGood
                ])),
                Group::All
            ),
            requires(Condition::Has(feat("bonus-two")), Group::Any),
            requires(
                Condition::ArchetypeLevel(ArchetypeKey::new("warden/stray"), 3),
                Group::Any
            ),
        ]
    );
}

#[test]
fn an_entry_naming_an_unmapped_blueprint_is_left_whole_and_listed() {
    let (dir, _, dump) = world();
    let before = read(dir.path(), "feats/lost-feat.ron");
    let outcome = write(dir.path(), &dump);
    assert_eq!(read(dir.path(), "feats/lost-feat.ron"), before);
    assert!(!changed(&outcome).contains(&"feat lost-feat".to_owned()));
    assert_eq!(
        unexpressed(&outcome, "feat lost-feat"),
        [format!(
            "GhostlyFeature {}, which no entry maps -> 307",
            guid(250)
        )]
    );
}

#[test]
fn a_shared_blueprint_becomes_an_any_of_its_entries() {
    let (dir, _, dump) = world();
    write(dir.path(), &dump);
    let [Component::Requires { condition, group }] = &feat_components(dir.path(), "needs-twin")[..]
    else {
        panic!("one Requires");
    };
    assert_eq!(*group, Group::All);
    let Condition::Any(members) = condition else {
        panic!("an Any, not {condition:?}");
    };
    let members: BTreeSet<&Condition> = members.iter().collect();
    assert_eq!(
        members,
        BTreeSet::from([
            &Condition::Has(feat("twin-feat")),
            &Condition::Has(option("twin-option"))
        ])
    );
}

#[test]
fn an_own_no_feature_is_omitted_unless_the_blueprint_is_shared() {
    let (dir, _, dump) = world();
    write(dir.path(), &dump);
    // A feature of one rank refusing itself says nothing `CanSelect` doesn't.
    assert_eq!(
        feat_components(dir.path(), "once-feat"),
        [requires(Condition::Has(feat("bonus-one")), Group::All)]
    );
    // Shared: each entry refuses the other, the blueprint held once.
    assert_eq!(
        feat_components(dir.path(), "twin-feat"),
        [requires(
            Condition::HasNot(option("twin-option")),
            Group::All
        )]
    );
    assert_eq!(
        parsed::<rules::SelectionOption>(dir.path(), "options/twin-option.ron").components,
        [requires(Condition::HasNot(feat("twin-feat")), Group::All)]
    );
    // A selection refusing itself is once-only (303).
    assert_eq!(
        parsed::<rules::SelectionOption>(dir.path(), "options/solo-pick.ron").components,
        [requires(Condition::HasNot(option("solo-pick")), Group::All)]
    );
}

#[test]
fn an_extra_feats_own_cap_is_the_headers_ranks() {
    let (dir, _, dump) = world();
    let odd = read(dir.path(), "feats/extra-odd.ron");
    let outcome = write(dir.path(), &dump);
    assert_eq!(
        feat_components(dir.path(), "extra-thing"),
        [
            requires(Condition::Has(feat("bonus-one")), Group::Any),
            requires(
                Condition::Not(Box::new(Condition::FactRank(feat("extra-thing"), 3))),
                Group::All
            ),
        ]
    );
    // A cap that isn't the header's ranks is no form the writer has.
    assert_eq!(read(dir.path(), "feats/extra-odd.ron"), odd);
    assert_eq!(
        unexpressed(&outcome, "feat extra-odd"),
        ["an own cap of 3, not the header's ranks 2 -> 308"]
    );
}

#[test]
fn a_hand_prerequisite_is_never_written() {
    let (dir, _, dump) = world();
    let before = read(dir.path(), "feats/hand-feat.ron");
    let outcome = write(dir.path(), &dump);
    assert_eq!(read(dir.path(), "feats/hand-feat.ron"), before);
    assert_eq!(
        unexpressed(&outcome, "feat hand-feat"),
        ["a hand prerequisite, which the writer never writes"]
    );
}

#[test]
fn an_equal_compiled_tree_is_not_rewritten() {
    let (dir, _, dump) = world();
    let before = read(dir.path(), "feats/same-feat.ron");
    // Its components hold a comment, which a write would refuse: the run succeeds because nothing is planned there.
    let outcome = write(dir.path(), &dump);
    assert_eq!(read(dir.path(), "feats/same-feat.ron"), before);
    assert!(!changed(&outcome).contains(&"feat same-feat".to_owned()));
}

#[test]
fn a_second_prerequisites_run_writes_nothing() {
    let (dir, _, dump) = world();
    let first = write(dir.path(), &dump);
    assert!(!first.changes.is_empty());
    let again = write(dir.path(), &dump);
    assert!(again.changes.is_empty(), "{:?}", changed(&again));
    assert!(again.applied.files.is_empty());
}

#[test]
fn the_check_reports_no_prerequisite_finding_for_a_written_entry() {
    let (dir, pins, dump) = world();
    let before = report(dir.path(), &pins, &dump);
    assert!(
        before.text().contains("  feat keen-feat |"),
        "{}",
        before.text()
    );
    let outcome = write(dir.path(), &dump);
    let after = report(dir.path(), &pins, &dump);
    assert!(changed(&outcome).len() >= 8, "{:?}", changed(&outcome));
    for entry in changed(&outcome) {
        assert_agrees(&after, "prerequisites", &entry);
    }
}

#[test]
fn only_names_entries_and_a_prefix() {
    let (dir, _, dump) = world();
    let one = write_with(
        dir.path(),
        &dump,
        &[WriteArea::Prerequisites],
        Some(&["feat keen-feat"]),
        "",
    )
    .unwrap();
    assert_eq!(changed(&one), ["feat keen-feat"]);
    let prefix = write_with(
        dir.path(),
        &dump,
        &[WriteArea::Prerequisites],
        Some(&["selection option *", "feat twin-*"]),
        "",
    )
    .unwrap();
    assert_eq!(
        changed(&prefix),
        [
            "feat twin-feat",
            "selection option solo-pick",
            "selection option twin-option"
        ]
    );
    let none = write_with(
        dir.path(),
        &dump,
        &[WriteArea::Prerequisites],
        Some(&["feat keen-feat", "feat no-such-feat"]),
        "",
    )
    .unwrap_err();
    assert!(
        matches!(&none, WriteError::Usage(why) if why.contains("feat no-such-feat")),
        "{none}"
    );
    let mixed = write_with(
        dir.path(),
        &dump,
        &[WriteArea::Prerequisites, WriteArea::Items],
        Some(&["feat keen-feat"]),
        "",
    )
    .unwrap_err();
    assert!(matches!(mixed, WriteError::Usage(_)), "{mixed}");
}

#[test]
fn a_held_prerequisite_fact_goes_to_the_user() {
    let (dir, _, dump) = world();
    let before = read(dir.path(), "feats/keen-feat.ron");
    let outcome = write_with(
        dir.path(),
        &dump,
        &[WriteArea::Prerequisites],
        None,
        "prerequisites / * / feat keen-feat -> held",
    )
    .unwrap();
    assert_eq!(read(dir.path(), "feats/keen-feat.ron"), before);
    let held: Vec<String> = outcome
        .to_the_user
        .iter()
        .map(|change| change.at.to_string())
        .collect();
    assert_eq!(held, ["feat keen-feat"]);
    assert!(changed(&outcome).contains(&"feat once-feat".to_owned()));
}

#[test]
fn a_mythic_abilitys_components_are_owned() {
    let (dir, _, dump) = world();
    let outcome = write(dir.path(), &dump);
    assert!(changed(&outcome).contains(&"mythic ability deep-path".to_owned()));
    let ability: rules::MythicAbility = parsed(dir.path(), "mythic/abilities/deep-path.ron");
    assert_eq!(
        ability.components,
        [
            requires(Condition::MythicLevel(3), Group::All),
            requires(Condition::Has(feat("bonus-one")), Group::All),
        ]
    );
}
