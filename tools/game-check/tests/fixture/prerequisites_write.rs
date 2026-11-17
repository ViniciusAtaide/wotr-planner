//! The prerequisites target (ticket 308a), on a world of its own: the write fixture's data with entries whose
//! blueprints carry `Prerequisite*` components, and a dump of those blueprints.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use game_check::write::facts::Facts;
use game_check::write::prerequisites::{self, Narrowings};
use game_check::write::{Outcome, WriteArea, WriteError, run, run_with};
use rules::{
    Ability, Alignment, ArchetypeKey, ClassKey, Component, Condition, Fact, FeatKey, GameData,
    Group, Prerequisite, SelectionOptionKey, StatRef,
};

use crate::support::{Record, assert_agrees, binz, data_copy, dump, report_with};

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

/// The game's `AddFacts` of one fixture blueprint.
fn add_facts(n: u32) -> String {
    format!(
        "{{\"$type\": \"0, AddFacts\", \"m_Facts\": [{}]}}",
        bp(&guid(n))
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
/// - the narrowings: `wide-feat` (211) accepts `keen-feat` or `once-feat` and the game names `once-feat` alone;
///   `deny-feat` (212) gains the game's refusal of `once-feat`; `both-feat` (213) does both, dropping `keen-feat` and
///   refusing `lost-feat`; `shun-twin` (214) refuses the shared blueprint 260.
/// - `kept-feat` (215): a `WeaponFocus` before its one `Requires`, and the game names a second feature.
/// - the narrowings' shapes and owners (308g): `narrow-domain` (217) accepts `domain-feat` (216, a blueprint named as a
///   domain progression, which 294 owns) or `once-feat`; `nested-not` (218) accepts `keen-feat` under two `Not`s;
///   `all-any` (219) needs `bonus-one` and `keen-feat` or `once-feat`. The game names `once-feat` alone in each.
/// - the carriers (308h): `sworn-feat` (221) and `held-sworn` (226, whose data already says so) need `carried-feat`
///   (220), which `PlantedGranterFeature` (230, the feature `granter`) grants through `AddFacts`; `leveled-feat` (223)
///   needs `ranked-feat` (222), which `PlantedLevelGranterFeature` (231, `level-granter`) grants through
///   `AddFeatureOnClassLevel`. Neither planner feature grants it. `thankful-feat` (225) needs `given-feat` (224), which
///   `PlantedGiverFeature` (232, `giver`) grants, and `PlantedFarGiverFeature` (234, `far-giver`) through the unmapped
///   `PlantedMiddleFeature` (233): both planner features grant `given-feat`. `either-feat` (227) needs `carried-feat`
///   or the feature `granter` itself. `kept-cover` (229) needs `covered-fact` (228), which `PlantedProficienciesFeature`
///   (235, `proficiencies`) grants and the planner's class `keeper` gives beside it, as a class row gives a class's
///   proficiencies; `fist-feat` (238) needs `improved-unarmed-strike` (237), which `PlantedGodFeature` (236, the deity
///   `keeper-god`, whose favoured weapon is the unarmed strike) grants.
fn world() -> (tempfile::TempDir, game_check::Pins, game_check::Dump) {
    let dir = data_copy();
    std::fs::create_dir_all(dir.path().join("features")).unwrap();
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
    let either = ", components: [Requires(condition: Any([Has(Feat(\"keen-feat\")), Has(Feat(\"once-feat\"))]))]";
    feat("wide-feat", 211, either);
    feat("deny-feat", 212, "");
    feat("both-feat", 213, either);
    feat("shun-twin", 214, "");
    feat(
        "kept-feat",
        215,
        ", components: [WeaponFocus, Requires(condition: Has(Feat(\"bonus-one\")))]",
    );
    feat("domain-feat", 216, "");
    feat(
        "narrow-domain",
        217,
        ", components: [Requires(condition: Any([Has(Feat(\"domain-feat\")), Has(Feat(\"once-feat\"))]))]",
    );
    feat(
        "nested-not",
        218,
        ", components: [Requires(condition: Not(Not(Has(Feat(\"keen-feat\")))))]",
    );
    feat(
        "all-any",
        219,
        ", components: [Requires(condition: Has(Feat(\"bonus-one\"))), Requires(condition: Any([Has(Feat(\"keen-feat\")), Has(Feat(\"once-feat\"))]))]",
    );
    feat("carried-feat", 220, "");
    feat("sworn-feat", 221, "");
    feat("ranked-feat", 222, "");
    feat("leveled-feat", 223, "");
    feat("given-feat", 224, "");
    feat("thankful-feat", 225, "");
    feat(
        "held-sworn",
        226,
        ", components: [Requires(condition: Has(Feat(\"carried-feat\")))]",
    );
    feat("either-feat", 227, "");
    let feature = |key: &str, n: u32, more: &str| {
        put(
            &format!("features/{key}.ron"),
            format!(
                "(key: \"{key}\", blueprint: \"{}\", name: \"Planted {key}\"{more})\n",
                guid(n)
            ),
        );
    };
    feature("granter", 230, "");
    feature("level-granter", 231, "");
    let gives = ", components: [Grant([Feat(\"given-feat\")])]";
    feature("giver", 232, gives);
    feature("far-giver", 234, gives);
    feat("covered-fact", 228, "");
    feat("kept-cover", 229, "");
    feature("proficiencies", 235, "");
    put(
        "classes/keeper.ron",
        "(key: \"keeper\", name: \"Keeper\", skill_points: 2, bab: Full, fort: Good, reflex: Poor, will: Poor, levels: [(level: 1, features: [\"proficiencies\"], feats: [\"covered-fact\"])])\n".to_owned(),
    );
    std::fs::create_dir_all(dir.path().join("deities")).unwrap();
    put(
        "deities/keeper-god.ron",
        format!(
            "(key: \"keeper-god\", blueprint: \"{}\", name: \"Keeper God\", alignments: [LawfulGood], favoured_weapons: [\"unarmed-strike\"], domains: [])\n",
            guid(236)
        ),
    );
    feat("improved-unarmed-strike", 237, "");
    feat("fist-feat", 238, "");
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
        blueprint(211, "WideFeat", FEATURE, 1, &[needs("All", &guid(203))]),
        blueprint(212, "DenyFeat", FEATURE, 1, &[no_feature(&guid(203))]),
        blueprint(
            213,
            "BothFeat",
            FEATURE,
            1,
            &[needs("All", &guid(203)), no_feature(&guid(201))],
        ),
        blueprint(214, "ShunTwinFeat", FEATURE, 1, &[no_feature(&guid(260))]),
        blueprint(
            215,
            "KeptFeat",
            FEATURE,
            1,
            &[needs("All", BONUS_ONE), needs("All", BONUS_TWO)],
        ),
        blueprint(216, "PlantedDomainProgression", FEATURE, 1, &[]),
        blueprint(
            217,
            "NarrowDomainFeat",
            FEATURE,
            1,
            &[needs("All", &guid(203))],
        ),
        blueprint(
            218,
            "NestedNotFeat",
            FEATURE,
            1,
            &[needs("All", &guid(203))],
        ),
        blueprint(
            219,
            "AllAnyFeat",
            FEATURE,
            1,
            &[needs("All", BONUS_ONE), needs("All", &guid(203))],
        ),
        blueprint(220, "CarriedFeat", FEATURE, 1, &[]),
        blueprint(221, "SwornFeat", FEATURE, 1, &[needs("All", &guid(220))]),
        blueprint(222, "RankedFeat", FEATURE, 1, &[]),
        blueprint(223, "LeveledFeat", FEATURE, 1, &[needs("All", &guid(222))]),
        blueprint(224, "GivenFeat", FEATURE, 1, &[]),
        blueprint(225, "ThankfulFeat", FEATURE, 1, &[needs("All", &guid(224))]),
        blueprint(
            226,
            "HeldSwornFeat",
            FEATURE,
            1,
            &[needs("All", &guid(220))],
        ),
        blueprint(
            227,
            "EitherFeat",
            FEATURE,
            1,
            &[needs("Any", &guid(220)), needs("Any", &guid(230))],
        ),
        blueprint(230, "PlantedGranterFeature", FEATURE, 1, &[add_facts(220)]),
        blueprint(
            231,
            "PlantedLevelGranterFeature",
            FEATURE,
            1,
            &[format!(
                "{{\"$type\": \"0, AddFeatureOnClassLevel\", \"m_Feature\": {}, \"m_Class\": {}, \"Level\": 1}}",
                bp(&guid(222)),
                bp(WARDEN)
            )],
        ),
        blueprint(232, "PlantedGiverFeature", FEATURE, 1, &[add_facts(224)]),
        blueprint(233, "PlantedMiddleFeature", FEATURE, 1, &[add_facts(224)]),
        blueprint(234, "PlantedFarGiverFeature", FEATURE, 1, &[add_facts(233)]),
        blueprint(228, "CoveredFeat", FEATURE, 1, &[]),
        blueprint(
            229,
            "KeptCoverFeat",
            FEATURE,
            1,
            &[needs("All", &guid(228))],
        ),
        blueprint(
            235,
            "PlantedProficienciesFeature",
            FEATURE,
            1,
            &[add_facts(228)],
        ),
        blueprint(
            236,
            "PlantedGodFeature",
            FEATURE,
            1,
            &[format!(
                "{{\"$type\": \"0, AddFeatureOnClassLevel\", \"m_Feature\": {}, \"m_Class\": {}, \"Level\": 1}}",
                bp(&guid(237)),
                bp(WARDEN)
            )],
        ),
        blueprint(237, "ImprovedUnarmedStrike", FEATURE, 1, &[]),
        blueprint(238, "FistFeat", FEATURE, 1, &[needs("All", &guid(237))]),
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

/// A run with these narrowings allowed (`prerequisite-narrowing.txt`'s grammar).
fn write_allowing(dir: &Path, dump: &game_check::Dump, allowed: &str) -> Outcome {
    run_with(
        dump,
        dir,
        &[WriteArea::Prerequisites],
        None,
        false,
        &Facts::default(),
        &Narrowings::parse(allowed).unwrap(),
        valid,
    )
    .unwrap()
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
fn a_rewrite_that_drops_an_old_alternative_is_left_whole_and_listed() {
    let (dir, _, dump) = world();
    let before = read(dir.path(), "feats/wide-feat.ron");
    let outcome = write(dir.path(), &dump);
    assert_eq!(read(dir.path(), "feats/wide-feat.ron"), before);
    assert!(!changed(&outcome).contains(&"feat wide-feat".to_owned()));
    assert_eq!(
        unexpressed(&outcome, "feat wide-feat"),
        [format!(
            "narrows: drops feat keen-feat (KeenFeat {}) -> 308",
            guid(200)
        )]
    );
}

#[test]
fn a_rewrite_that_adds_a_negation_is_left_whole_and_listed() {
    let (dir, _, dump) = world();
    let before = read(dir.path(), "feats/deny-feat.ron");
    let outcome = write(dir.path(), &dump);
    assert_eq!(read(dir.path(), "feats/deny-feat.ron"), before);
    assert!(!changed(&outcome).contains(&"feat deny-feat".to_owned()));
    assert_eq!(
        unexpressed(&outcome, "feat deny-feat"),
        [format!(
            "narrows: adds Not feat once-feat (OnceFeat {}) -> 308",
            guid(203)
        )]
    );
}

#[test]
fn an_allowlisted_narrowing_is_written() {
    let (dir, _, dump) = world();
    let both = read(dir.path(), "feats/both-feat.ron");
    let outcome = write_allowing(
        dir.path(),
        &dump,
        "# Planted.\n\
         feat wide-feat -> planted: the game's WideFeat names OnceFeat alone\n\
         feat wide-feat / drops feat keen-feat\n\
         feat deny-feat / adds Not feat once-feat -> planted: the game's DenyFeat refuses OnceFeat\n\
         feat both-feat / drops feat keen-feat -> planted: one of its two narrowings\n",
    );
    assert_eq!(
        feat_components(dir.path(), "wide-feat"),
        [requires(Condition::Has(feat("once-feat")), Group::All)]
    );
    assert_eq!(
        feat_components(dir.path(), "deny-feat"),
        [requires(Condition::HasNot(feat("once-feat")), Group::All)]
    );
    // All or nothing: one narrowing of two allowed leaves the entry whole, listing the other.
    assert_eq!(read(dir.path(), "feats/both-feat.ron"), both);
    assert_eq!(
        unexpressed(&outcome, "feat both-feat"),
        [format!(
            "narrows: adds Not feat lost-feat (LostFeat {}) -> 308",
            guid(201)
        )]
    );
}

#[test]
fn a_blocked_narrowing_names_its_facts_owner() {
    let (dir, _, dump) = world();
    let outcome = write(dir.path(), &dump);
    // A domain progression is 294's (`owner`, research/308/slices.md §1.2), as it is for a gap.
    assert_eq!(
        unexpressed(&outcome, "feat narrow-domain"),
        [format!(
            "narrows: drops feat domain-feat (PlantedDomainProgression {}) -> 294",
            guid(216)
        )]
    );
}

#[test]
fn a_drop_under_two_nots_is_left_whole_and_listed() {
    let (dir, _, dump) = world();
    let before = read(dir.path(), "feats/nested-not.ron");
    let outcome = write(dir.path(), &dump);
    assert_eq!(read(dir.path(), "feats/nested-not.ron"), before);
    assert_eq!(
        unexpressed(&outcome, "feat nested-not"),
        [format!(
            "narrows: drops feat keen-feat (KeenFeat {}) -> 308",
            guid(200)
        )]
    );
}

#[test]
fn a_drop_from_an_any_inside_an_all_is_left_whole_and_listed() {
    let (dir, _, dump) = world();
    let before = read(dir.path(), "feats/all-any.ron");
    let outcome = write(dir.path(), &dump);
    assert_eq!(read(dir.path(), "feats/all-any.ron"), before);
    assert_eq!(
        unexpressed(&outcome, "feat all-any"),
        [format!(
            "narrows: drops feat keen-feat (KeenFeat {}) -> 308",
            guid(200)
        )]
    );
}

/// Refusing another entry of its own blueprint is the game's `PrerequisiteNoFeature` exactly, so it needs no line.
#[test]
fn a_shared_own_refusal_is_written_with_no_allowlist_line() {
    let (dir, _, dump) = world();
    let outcome = write(dir.path(), &dump);
    for entry in ["feat twin-feat", "selection option twin-option"] {
        assert!(changed(&outcome).contains(&entry.to_owned()), "{entry}");
        assert!(unexpressed(&outcome, entry).is_empty(), "{entry}");
    }
}

#[test]
fn the_allowlist_parser_refuses_a_malformed_line() {
    let refusals = [
        ("feat a / drops feat b ->\n", "line 1: an empty reason"),
        (
            "# A comment.\nfeat a\n",
            "line 2: neither a narrowing nor a reason",
        ),
        (
            "feat a / widens feat b -> r\n",
            "line 1: a narrowing is `drops <fact>` or `adds Not <fact>`",
        ),
        (
            "feat a / drops feat b -> r\nfeat a / drops feat b -> s\n",
            "line 2: a narrowing said twice",
        ),
        (
            "feat a / drops feat b\n",
            "line 1: a narrowing with no reason, its own or its entry's",
        ),
        (
            "feat a -> r\nfeat b -> s\nfeat b / drops feat c\n",
            "line 1: a reason with no narrowing of its entry",
        ),
    ];
    for (text, why) in refusals {
        let error = Narrowings::parse(text).unwrap_err().to_string();
        assert!(error.contains(why), "{text:?}: {error}");
    }
}

#[test]
fn the_embedded_allowlist_parses() {
    Narrowings::embedded().expect("prerequisite-narrowing.txt parses");
}

/// A line whose narrowing no rewrite makes (its entry written, or the data changed) would allow that narrowing again
/// unseen, so the check lists it and fails, as it does a stale `known.txt` line.
#[test]
fn an_allowlist_line_that_matches_no_narrowing_fails_the_check() {
    let (dir, _, dump) = world();
    let narrowings = Narrowings::parse(
        "feat wide-feat -> planted: the game's WideFeat names OnceFeat alone\n\
         feat wide-feat / drops feat keen-feat\n\
         feat wide-feat / drops feat bonus-one\n\
         feat same-feat / adds Not feat keen-feat -> planted: same-feat's rewrite narrows nothing\n",
    )
    .unwrap();
    let data = seeds::read_game_data_files(dir.path()).unwrap();
    let stale = prerequisites::stale(&dump, &data, &narrowings);
    assert_eq!(
        stale,
        [
            "line 3: feat wide-feat / drops feat bonus-one",
            "line 4: feat same-feat / adds Not feat keen-feat -> planted: same-feat's rewrite narrows nothing",
        ]
    );
    let every_area = "mapping -> a\nprerequisites -> a\nclass tables -> a\nprogression rows -> a\noption lists -> a\n\
                      mythic ranks -> a\nitems -> a\n";
    let passing = report_with(every_area);
    assert!(!passing.failed(), "{}", passing.text());
    assert!(!report_with(every_area).with_stale_narrowings(&[]).failed());
    let failing = passing.with_stale_narrowings(&stale);
    assert!(failing.failed());
    assert!(
        failing.text().contains(
            "\nprerequisite-narrowing.txt lines that match no narrowing:\n  line 3: feat wide-feat / drops feat bonus-one\n"
        ),
        "{}",
        failing.text()
    );
}

#[test]
fn an_entrys_non_requires_components_are_kept() {
    let (dir, _, dump) = world();
    write(dir.path(), &dump);
    assert_eq!(
        feat_components(dir.path(), "kept-feat"),
        [
            requires(Condition::Has(feat("bonus-one")), Group::All),
            requires(Condition::Has(feat("bonus-two")), Group::All),
            Component::WeaponFocus,
        ]
    );
}

/// The game's `PrerequisiteNoFeature` refuses a character holding the blueprint
/// (`Blueprints/Classes/Prerequisites/PrerequisiteNoFeature.cs` `CheckInternal`, WrathDecompiled 6ce4a6b:
/// `!unit.HasFact(Feature)`), and a blueprint several entries share is held through any of
/// them: so it refuses each, `Not` of the `Any`.
#[test]
fn a_no_feature_on_a_shared_guid_becomes_a_not_of_each_entry() {
    let (dir, _, dump) = world();
    write_allowing(
        dir.path(),
        &dump,
        "feat shun-twin / adds Not feat twin-feat -> planted: the game's TwinFeature\n\
         feat shun-twin / adds Not selection option twin-option -> planted: the game's TwinFeature\n",
    );
    assert_eq!(
        feat_components(dir.path(), "shun-twin"),
        [requires(
            Condition::Not(Box::new(Condition::Any(vec![
                Condition::Has(feat("twin-feat")),
                Condition::Has(option("twin-option")),
            ]))),
            Group::All
        )]
    );
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

/// 308h: the game's `PrerequisiteFeature` passes on `HasFact`
/// (`Blueprints/Classes/Prerequisites/PrerequisiteFeature.cs` `CheckInternal`, WrathDecompiled 6ce4a6b), and `AddFacts`
/// makes a fact held while its source is (`UnitLogic/FactLogic/AddFacts.cs` `OnActivate`). A planner entry of the
/// granting blueprint that doesn't grant the fact would refuse its holder, so the entry is left whole.
#[test]
fn a_fact_an_add_facts_carrier_grants_and_its_planner_entry_doesnt_leaves_the_entry_whole() {
    let (dir, _, dump) = world();
    let before = read(dir.path(), "feats/sworn-feat.ron");
    let outcome = write(dir.path(), &dump);
    assert_eq!(read(dir.path(), "feats/sworn-feat.ron"), before);
    assert_eq!(
        unexpressed(&outcome, "feat sworn-feat"),
        [format!(
            "feat carried-feat granted by PlantedGranterFeature {}, which feature granter doesn't grant -> 307",
            guid(230)
        )]
    );
}

/// 308h: `AddFeatureOnClassLevel` (`Designers/Mechanics/Facts/AddFeatureOnClassLevel.cs` `Apply`) is a carrier too.
#[test]
fn a_fact_an_add_feature_on_class_level_carrier_grants_and_its_planner_entry_doesnt_leaves_the_entry_whole()
 {
    let (dir, _, dump) = world();
    let before = read(dir.path(), "feats/leveled-feat.ron");
    let outcome = write(dir.path(), &dump);
    assert_eq!(read(dir.path(), "feats/leveled-feat.ron"), before);
    assert_eq!(
        unexpressed(&outcome, "feat leveled-feat"),
        [format!(
            "feat ranked-feat granted by PlantedLevelGranterFeature {}, which feature level-granter doesn't grant -> 307",
            guid(231)
        )]
    );
}

/// 308h: a grantor whose planner entry grants the fact under the fact's own key refuses no one, directly or through a
/// blueprint no entry maps.
#[test]
fn a_fact_each_planner_grantor_grants_is_written() {
    let (dir, _, dump) = world();
    let outcome = write(dir.path(), &dump);
    assert!(unexpressed(&outcome, "feat thankful-feat").is_empty());
    assert_eq!(
        feat_components(dir.path(), "thankful-feat"),
        [requires(Condition::Has(feat("given-feat")), Group::All)]
    );
}

/// 308h: a holder of the grantor meets the `Any` through the grantor itself, so it is no gap (Defensive Study's
/// base shape).
#[test]
fn a_grantor_the_same_any_accepts_is_no_gap() {
    let (dir, _, dump) = world();
    let outcome = write(dir.path(), &dump);
    assert!(unexpressed(&outcome, "feat either-feat").is_empty());
    assert!(changed(&outcome).contains(&"feat either-feat".to_owned()));
}

/// 308h: the audit runs the guard over every entry's tree as the data holds it (`holds`) and over the game's tree
/// the writer would write (`left whole`).
#[test]
fn the_carrier_audit_lists_what_the_data_holds_and_what_the_writer_leaves_whole() {
    let (dir, _, dump) = world();
    let data = seeds::read_game_data_files(dir.path()).unwrap();
    let audit: BTreeSet<String> = prerequisites::audit(&dump, &data).into_iter().collect();
    let granter = format!(
        "feat carried-feat granted by PlantedGranterFeature {}, which feature granter doesn't grant -> 307",
        guid(230)
    );
    assert_eq!(
        audit,
        BTreeSet::from([
            format!("feat held-sworn | holds: {granter}"),
            format!("feat sworn-feat | left whole: {granter}"),
            format!(
                "feat leveled-feat | left whole: feat ranked-feat granted by PlantedLevelGranterFeature {}, which feature level-granter doesn't grant -> 307",
                guid(231)
            ),
        ])
    );
}

/// 308h: a grantor the planner gives only in a row that gives the fact too refuses no one (the rogue's class row gives
/// `rogue/rogue-proficiencies` and Armor Proficiency (Light Armor) at 1, where the game's `RogueProficiencies`
/// 33e2a7e4ad9daa54eaf808e1483bb43c gives the second through `AddFacts`).
#[test]
fn a_grantor_every_planner_row_gives_beside_the_fact_is_no_gap() {
    let (dir, _, dump) = world();
    let outcome = write(dir.path(), &dump);
    assert!(unexpressed(&outcome, "feat kept-cover").is_empty());
    assert_eq!(
        feat_components(dir.path(), "kept-cover"),
        [requires(Condition::Has(feat("covered-fact")), Group::All)]
    );
}

/// 308h: a deity gives its favoured weapon's feat as the engine holds it (`engine/catalog.rs` `favoured_weapon_feats`): the game's
/// `IroriFeature` 23a77a5985de08349820429ce1b5a234 grants Improved Unarmed Strike through `AddFeatureOnClassLevel`.
#[test]
fn a_deitys_favoured_unarmed_strike_gives_improved_unarmed_strike() {
    let (dir, _, dump) = world();
    let outcome = write(dir.path(), &dump);
    assert!(unexpressed(&outcome, "feat fist-feat").is_empty());
    assert_eq!(
        feat_components(dir.path(), "fist-feat"),
        [requires(
            Condition::Has(feat("improved-unarmed-strike")),
            Group::All
        )]
    );
}
