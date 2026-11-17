//! Applying changes (ticket 293b): the write fixture's tree reads as game data, a change writes only its field, a
//! planner file is refused with nothing written, and a failed check puts every file back.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use game_check::write::facts::Facts;
use game_check::write::items::gear_stat;
use game_check::write::span::Value;
use game_check::write::{
    Change, WriteArea, WriteError, apply, apply_with, atomic_write, entry_file, read_back, run,
};
use rules::DataAt;

use crate::support::data_copy;

fn change(file: &str, field: &'static str, value: Option<Value>) -> Change {
    Change {
        at: DataAt::Gear(rules::GearKey::new("cloak-of-test")),
        file: PathBuf::from(file),
        field,
        kind: "gear",
        value,
        old: String::new(),
        new: String::new(),
    }
}

fn components(text: &str) -> Option<Value> {
    Some(Value::List(vec![text.to_owned()]))
}

fn read(dir: &Path, file: &str) -> String {
    std::fs::read_to_string(dir.join(file)).unwrap()
}

#[test]
fn the_write_fixture_reads_as_game_data() {
    let dir = data_copy();
    let data = seeds::read_game_data_files(dir.path()).expect("the tree reads");
    assert_eq!(data.classes.len(), 1);
    assert_eq!(data.prestige_classes.len(), 1);
    assert_eq!(data.archetypes.len(), 2);
    assert_eq!(data.gear.len(), 7, "six entries and the planner's trinket");
    let class = DataAt::Class(rules::ClassKey::new("warden"));
    assert_eq!(entry_file(&data, &class), Path::new("classes/warden.ron"));
    let prestige = DataAt::Class(rules::ClassKey::new("sentinel"));
    assert_eq!(
        entry_file(&data, &prestige),
        Path::new("prestige/sentinel.ron")
    );
    let archetype = DataAt::Archetype(rules::ArchetypeKey::new("warden/stoic"));
    assert_eq!(
        entry_file(&data, &archetype),
        Path::new("archetypes/warden/stoic.ron")
    );
    let gear = DataAt::Gear(rules::GearKey::new("cloak-of-test"));
    assert_eq!(
        entry_file(&data, &gear),
        Path::new("gear/cloak-of-test.ron")
    );
}

#[test]
fn applying_a_change_writes_only_its_field() {
    let dir = data_copy();
    let before = read(dir.path(), "gear/cloak-of-test.ron");
    let new = "GearStat(stat: AllSaves, value: 3, descriptor: Resistance)";
    let applied = apply(
        dir.path(),
        &[change(
            "gear/cloak-of-test.ron",
            "components",
            components(new),
        )],
        &[],
        |path| {
            seeds::read_game_data_files(path)
                .map(|_| ())
                .map_err(|e| e.to_string())
        },
    )
    .unwrap();
    assert_eq!(applied.files, [PathBuf::from("gear/cloak-of-test.ron")]);
    let after = read(dir.path(), "gear/cloak-of-test.ron");
    assert_eq!(
        after,
        before.replace(
            "GearStat(stat: AllSaves, value: 2, descriptor: Resistance)",
            new
        )
    );
    // The comment above and the other fields survive.
    assert!(after.starts_with("// Planted: +3 in the game.\n"));
}

#[test]
fn a_planner_file_is_refused_and_nothing_is_written() {
    let dir = data_copy();
    let cloak = read(dir.path(), "gear/cloak-of-test.ron");
    let trinket = read(dir.path(), "planner/gear/test-trinket.ron");
    let error = apply(
        dir.path(),
        &[
            change("gear/cloak-of-test.ron", "components", components("X")),
            change(
                "planner/gear/test-trinket.ron",
                "components",
                components("Y"),
            ),
        ],
        &[],
        |_| Ok(()),
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::Planner(_)), "{error}");
    assert_eq!(read(dir.path(), "gear/cloak-of-test.ron"), cloak);
    assert_eq!(read(dir.path(), "planner/gear/test-trinket.ron"), trinket);
    // A file that isn't there is refused too.
    let error = apply(
        dir.path(),
        &[change("gear/nothing.ron", "components", components("X"))],
        &[],
        |_| Ok(()),
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::Missing(_)), "{error}");
}

#[test]
fn a_failed_validation_restores_every_file() {
    let dir = data_copy();
    let files = ["gear/cloak-of-test.ron", "gear/headband-of-test.ron"];
    let before: Vec<String> = files.iter().map(|file| read(dir.path(), file)).collect();
    let error = apply(
        dir.path(),
        &[
            change(files[0], "components", components("A")),
            change(files[1], "components", components("B")),
        ],
        &[],
        |path| {
            // The files are written when the check runs.
            let text = std::fs::read_to_string(path.join(files[1])).unwrap();
            assert!(text.contains("components: [B]"), "{text}");
            Err("a planted failure".to_owned())
        },
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::Invalid(_)), "{error}");
    for (file, text) in files.iter().zip(before) {
        assert_eq!(read(dir.path(), file), text, "{file}");
    }
}

fn bab_change(value: &str) -> Change {
    Change {
        at: DataAt::Class(rules::ClassKey::new("warden")),
        file: PathBuf::from("classes/warden.ron"),
        field: "bab",
        kind: "bab",
        value: Some(Value::One(value.to_owned())),
        old: String::new(),
        new: String::new(),
    }
}

#[test]
fn read_back_tells_unit_variants_apart() {
    let before = "(key: \"w\", bab: Poor, fort: Good)";
    let change = bab_change("Full");
    let file = Path::new("classes/warden.ron");
    // As written.
    read_back(file, before, "(key: \"w\", bab: Full, fort: Good)", &change).unwrap();
    // A buggy writer's text: another variant of the field.
    let error =
        read_back(file, before, "(key: \"w\", bab: Half, fort: Good)", &change).unwrap_err();
    assert!(
        matches!(error, WriteError::ReadBack { field: "bab", .. }),
        "{error}"
    );
    // Another field changed instead of the one meant: the field written is still the old one.
    let error =
        read_back(file, before, "(key: \"w\", bab: Poor, fort: Full)", &change).unwrap_err();
    assert!(matches!(error, WriteError::ReadBack { .. }), "{error}");
    // A value that parses but sits in an unrelated field, the meant one gone.
    let error = read_back(
        file,
        before,
        "(key: \"w\", fort: Good, will: Full)",
        &change,
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::ReadBack { .. }), "{error}");
}

#[test]
fn read_back_keeps_the_bytes_between_the_fields() {
    let file = Path::new("classes/warden.ron");
    let change = bab_change("Full");
    let before = "(\n    key: \"w\",\n    // keep\n    bab: Poor,\n    fort: Good,\n)\n";
    let good = "(\n    key: \"w\",\n    // keep\n    bab: Full,\n    fort: Good,\n)\n";
    read_back(file, before, good, &change).unwrap();
    for planted in [
        "(\n    key: \"w\",\n    // kept\n    bab: Full,\n    fort: Good,\n)\n",
        "(\n    key: \"w\",\n    bab: Full,\n    fort: Good,\n)\n",
        "(\n    key: \"w\",\n    // keep\n    bab: Full\n    ,fort: Good,\n)\n",
        "(\n    key: \"w\",\n    // keep\n    bab: Full,\n      fort: Good,\n)\n",
    ] {
        let error = read_back(file, before, planted, &change).unwrap_err();
        assert!(matches!(error, WriteError::ReadBack { .. }), "{planted}");
    }
    // An insert and a removal with the writer's separators pass.
    let removal = Change {
        value: None,
        ..bab_change("Full")
    };
    read_back(
        file,
        before,
        "(\n    key: \"w\",\n    // keep\n    fort: Good,\n)\n",
        &removal,
    )
    .unwrap();
    read_back(
        file,
        "(\n    key: \"w\",\n    // keep\n    fort: Good,\n)\n",
        good,
        &change,
    )
    .unwrap();
}

fn all_files(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_owned()];
    while let Some(next) = stack.pop() {
        for entry in std::fs::read_dir(&next).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push((path.clone(), std::fs::read_to_string(&path).unwrap()));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn a_field_the_importer_does_not_own_is_refused_before_any_write() {
    let dir = data_copy();
    let before = all_files(dir.path());
    for field in ["key", "nonsense", "name"] {
        let error = apply(
            dir.path(),
            &[
                change("gear/cloak-of-test.ron", "components", components("A")),
                change("gear/headband-of-test.ron", field, components("B")),
            ],
            &[],
            |_| Ok(()),
        )
        .unwrap_err();
        assert!(
            matches!(error, WriteError::NotOwned { .. }),
            "{field}: {error}"
        );
    }
    // A folder with no list owns nothing yet.
    let error = apply(
        dir.path(),
        &[Change {
            file: PathBuf::from("progression.ron"),
            ..change("", "mythic_ascensions", components("A"))
        }],
        &[],
        |_| Ok(()),
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::NotOwned { .. }), "{error}");
    assert_eq!(all_files(dir.path()), before);
}

#[test]
fn a_write_that_lands_wrong_is_a_read_back_failure_and_every_file_is_restored() {
    let dir = data_copy();
    let before = all_files(dir.path());
    let error = apply_with(
        dir.path(),
        &[
            change("gear/cloak-of-test.ron", "components", components("A")),
            change("gear/headband-of-test.ron", "components", components("B")),
        ],
        &[],
        |_| Ok(()),
        |path, text| std::fs::write(path, text.replace("[B]", "[C]")),
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::ReadBack { .. }), "{error}");
    assert_eq!(all_files(dir.path()), before);
}

#[test]
fn a_write_that_fails_is_an_io_error_and_every_file_is_restored() {
    let dir = data_copy();
    let before = all_files(dir.path());
    let error = apply_with(
        dir.path(),
        &[
            change("gear/cloak-of-test.ron", "components", components("A")),
            change("gear/headband-of-test.ron", "components", components("B")),
        ],
        &[],
        |_| Ok(()),
        |path, text| {
            if path.ends_with("headband-of-test.ron") {
                Err(std::io::Error::other("a planted failure"))
            } else {
                std::fs::write(path, text)
            }
        },
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::Io { .. }), "{error}");
    assert_eq!(all_files(dir.path()), before);
}

#[cfg(unix)]
#[test]
fn a_restore_that_fails_names_the_files_left_written() {
    use std::os::unix::fs::PermissionsExt;
    let dir = data_copy();
    let error = apply(
        dir.path(),
        &[change(
            "gear/cloak-of-test.ron",
            "components",
            components("A"),
        )],
        &[],
        |path| {
            // The folder turns read-only, so the file can't be put back.
            std::fs::set_permissions(path.join("gear"), std::fs::Permissions::from_mode(0o555))
                .unwrap();
            Err("a planted failure".to_owned())
        },
    )
    .unwrap_err();
    std::fs::set_permissions(
        dir.path().join("gear"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let WriteError::RestoreFailed { cause, files } = error else {
        panic!("not a restore failure: {error}");
    };
    assert!(matches!(*cause, WriteError::Invalid(_)));
    assert_eq!(files, [PathBuf::from("gear/cloak-of-test.ron")]);
}

#[test]
fn two_changes_to_one_field_apply_in_order_and_the_later_wins() {
    let dir = data_copy();
    apply(
        dir.path(),
        &[
            change("gear/cloak-of-test.ron", "components", components("A")),
            change("gear/cloak-of-test.ron", "components", components("B")),
        ],
        &[],
        |_| Ok(()),
    )
    .unwrap();
    let text = read(dir.path(), "gear/cloak-of-test.ron");
    assert!(
        text.contains("    B,\n") && !text.contains("    A,\n"),
        "{text}"
    );
}

#[test]
fn a_write_that_parses_but_breaks_a_data_rule_is_rejected_by_the_real_check() {
    let dir = data_copy();
    let problems = |path: &Path| -> Result<Vec<String>, String> {
        let data = seeds::read_game_data_files(path).map_err(|e| e.to_string())?;
        Ok(data.check().iter().map(|p| format!("{p:?}")).collect())
    };
    let known = problems(dir.path()).unwrap();
    let before = all_files(dir.path());
    // The Stoic's Fortitude equal to its class's own is a data rule's refusal (an override that overrides nothing).
    let stoic = Change {
        at: DataAt::Archetype(rules::ArchetypeKey::new("warden/stoic")),
        file: PathBuf::from("archetypes/warden/stoic.ron"),
        field: "fort",
        kind: "fort",
        value: Some(Value::One("Good".into())),
        old: String::new(),
        new: String::new(),
    };
    let error = apply(dir.path(), &[stoic], &[], |path| {
        let now = problems(path)?;
        match now.iter().find(|problem| !known.contains(problem)) {
            Some(new) => Err(new.clone()),
            None => Ok(()),
        }
    })
    .unwrap_err();
    assert!(matches!(error, WriteError::Invalid(_)), "{error}");
    assert!(
        error.to_string().contains("SaveProgressionSameAsClass"),
        "{error}"
    );
    assert_eq!(all_files(dir.path()), before);
}

/// The check over a data folder, with the fixture's dump.
fn report_of(dir: &Path) -> game_check::Report {
    let (pins, dump) = crate::support::dump(&crate::support::binz(&crate::support::records(), 2));
    let data = seeds::read_game_data_files(dir).unwrap();
    game_check::check(&pins, &dump, &data, &game_check::Known::parse("").unwrap()).unwrap()
}

fn write_items(dir: &Path) -> game_check::write::Outcome {
    let (_, dump) = crate::support::dump(&crate::support::binz(&crate::support::records(), 2));
    run(
        &dump,
        dir,
        &[WriteArea::Items],
        None,
        false,
        &Facts::default(),
        |_| Ok(()),
    )
    .unwrap()
}

#[test]
fn writing_items_leaves_only_the_slot_finding() {
    let dir = data_copy();
    let circlet = read(dir.path(), "gear/circlet-of-test.ron");
    let tome = read(dir.path(), "gear/tome-of-test.ron");
    let belt = read(dir.path(), "gear/belt-of-test-strength.ron");
    let outcome = write_items(dir.path());
    assert!(outcome.unexpressed.is_empty(), "{:?}", outcome.unexpressed);
    let files: Vec<_> = outcome
        .applied
        .files
        .iter()
        .map(|file| file.display().to_string())
        .collect();
    assert_eq!(
        files,
        [
            "gear/cloak-of-test.ron",
            "gear/headband-of-test.ron",
            "gear/ring-of-nothing.ron"
        ]
    );
    // The items the game agrees with, and the tome, are the same bytes.
    assert_eq!(read(dir.path(), "gear/circlet-of-test.ron"), circlet);
    assert_eq!(read(dir.path(), "gear/tome-of-test.ron"), tome);
    assert_eq!(read(dir.path(), "gear/belt-of-test-strength.ron"), belt);
    let cloak = read(dir.path(), "gear/cloak-of-test.ron");
    assert!(cloak.contains("GearStat(stat: AllSaves, value: 3, descriptor: Resistance)"));
    assert!(cloak.starts_with("// Planted: +3 in the game.\n"));
    assert!(cloak.contains("always_on"), "the other fields stay");
    let headband = read(dir.path(), "gear/headband-of-test.ron");
    assert!(
        headband.contains("GearStat(stat: Skill(\"perception\"), value: 1, descriptor: Insight)")
    );
    assert!(headband.contains("GearStat(stat: Initiative, value: 1, descriptor: Morale)"));
    assert!(
        headband
            .contains("GearStat(stat: Ability(Intelligence), value: 2, descriptor: Enhancement)")
    );
    assert!(
        !headband.trim_end().contains('\n'),
        "a one-line file stays one line"
    );
    assert!(read(dir.path(), "gear/ring-of-nothing.ron").contains("components: []"));

    // What is left of the items' findings is the slot, which no bonus fixes.
    let report = report_of(dir.path());
    let text = report.text();
    let mut in_items = false;
    let mut lines = Vec::new();
    for line in text.lines() {
        if !line.starts_with("  ") {
            in_items = line.starts_with("items / ");
            if in_items {
                lines.push(line.to_owned());
            }
            continue;
        }
        if in_items {
            lines.push(line.to_owned());
        }
    }
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert!(lines[0].starts_with("items / slot differs"), "{lines:?}");
    assert!(
        lines[1].starts_with("  gear item headband-of-test |"),
        "{lines:?}"
    );
}

#[test]
fn a_second_items_run_writes_nothing() {
    let dir = data_copy();
    write_items(dir.path());
    let after = all_files(dir.path());
    let again = write_items(dir.path());
    assert!(again.changes.is_empty(), "{:?}", again.changes);
    assert!(again.applied.files.is_empty());
    assert_eq!(all_files(dir.path()), after);
}

fn guid(n: u32) -> String {
    format!("{:032x}", 0xfa00_0000u32 + n)
}

fn record(n: u32, name: &str, type_name: &str, raw: &str) -> crate::support::Record {
    crate::support::Record {
        guid: guid(n),
        name: name.to_owned(),
        type_name: type_name.to_owned(),
        raw: raw.to_owned(),
    }
}

/// A ring blueprint with one enchantment holding `components`, and the planner's file for it.
fn ring(
    dir: &Path,
    records: &mut Vec<crate::support::Record>,
    n: u32,
    key: &str,
    components: &str,
    feature: Option<&str>,
) {
    let enchantment = format!("!bp_{}", guid(n + 1));
    records.push(record(
        n,
        key,
        "Kingmaker.Blueprints.Items.Equipment.BlueprintItemEquipmentRing",
        &format!("{{\"m_Enchantments\": [\"{enchantment}\"]}}"),
    ));
    records.push(record(
        n + 1,
        &format!("{key}-enchantment"),
        "Kingmaker.Blueprints.Items.Ecnchantments.BlueprintEquipmentEnchantment",
        &format!("{{\"Components\": [{components}]}}"),
    ));
    if let Some(feature) = feature {
        records.push(record(
            n + 2,
            &format!("{key}-feature"),
            "Kingmaker.Blueprints.Classes.BlueprintFeature",
            &format!("{{\"$type\": \"0, BlueprintFeature\", \"Components\": [{feature}]}}"),
        ));
    }
    let file = format!(
        "(key: \"{key}\", blueprint: \"{}\", name: \"{key}\", slot: Ring, components: [GearStat(stat: Ability(Dexterity), value: 2, descriptor: Enhancement)])\n",
        guid(n)
    );
    std::fs::write(dir.join(format!("gear/{key}.ron")), file).unwrap();
}

#[test]
fn an_item_with_a_bonus_the_importer_cannot_read_or_name_stays_unwritten_and_listed() {
    let dir = data_copy();
    let mut records = crate::support::records();
    // A bonus whose value the game works out in play: the check skips it, so the target is partial.
    ring(
        dir.path(),
        &mut records,
        1,
        "ring-of-unreadable",
        &format!(
            "{{\"$type\": \"0, AddUnitFeatureEquipment\", \"m_Feature\": \"!bp_{}\"}}",
            guid(3)
        ),
        Some(
            "{\"$type\": \"0, AddContextStatBonus\", \"Descriptor\": \"Insight\", \"Stat\": \"SkillPerception\", \"Value\": {\"ValueType\": \"Rank\"}, \"Multiplier\": 1}",
        ),
    );
    // A type and a second one the planner has no word for: both are listed.
    ring(
        dir.path(),
        &mut records,
        10,
        "ring-of-unnamed",
        "{\"$type\": \"0, AddStatBonusEquipment\", \"Descriptor\": \"Fumbling\", \"Stat\": \"Strength\", \"Value\": 1}, {\"$type\": \"0, AddStatBonusEquipment\", \"Descriptor\": \"Mystery\", \"Stat\": \"Wisdom\", \"Value\": 1}",
        None,
    );
    let before = all_files(dir.path());
    let (_, dump) = crate::support::dump(&crate::support::binz(&records, 2));
    let outcome = run(
        &dump,
        dir.path(),
        &[WriteArea::Items],
        None,
        false,
        &Facts::default(),
        |_| Ok(()),
    )
    .unwrap();
    let left: Vec<_> = outcome
        .unexpressed
        .iter()
        .map(|item| (item.at.to_string(), item.what.clone()))
        .collect();
    assert_eq!(left.len(), 2, "{left:?}");
    assert_eq!(left[0].0, "gear item ring-of-unnamed");
    assert!(left[0].1.contains("Strength +1 Fumbling"), "{left:?}");
    assert!(
        left[0].1.contains("Wisdom +1 Mystery"),
        "every missing bonus is listed: {left:?}"
    );
    assert_eq!(left[1].0, "gear item ring-of-unreadable");
    assert!(left[1].1.contains("worked out in play"), "{left:?}");
    // Neither item's bytes changed; the others were written as before.
    for key in ["ring-of-unreadable", "ring-of-unnamed"] {
        let file = format!("gear/{key}.ron");
        let kept = before
            .iter()
            .find(|(path, _)| path.ends_with(&file))
            .unwrap();
        assert_eq!(read(dir.path(), &file), kept.1, "{key}");
    }
    assert!(
        outcome
            .applied
            .files
            .iter()
            .all(|file| !file.to_string_lossy().contains("ring-of-un"))
    );
}

#[test]
fn the_inverse_of_the_check_s_targets_names_each_stat_the_planner_counts() {
    use rules::{
        Ability, Component, Descriptor, GearTarget, Save, SkillKey, SpellSchool, SpellScope,
    };
    let stat = |name: &str, kind: &str| gear_stat(&(name.to_owned(), kind.to_owned(), 2));
    let gear = |stat, descriptor| {
        Some(Component::GearStat {
            stat,
            value: 2,
            descriptor,
        })
    };
    for (name, ability) in [
        ("Strength", Ability::Strength),
        ("Dexterity", Ability::Dexterity),
        ("Constitution", Ability::Constitution),
        ("Intelligence", Ability::Intelligence),
        ("Wisdom", Ability::Wisdom),
        ("Charisma", Ability::Charisma),
    ] {
        assert_eq!(
            stat(name, "Enhancement"),
            gear(GearTarget::Ability(ability), Descriptor::Enhancement),
            "{name}"
        );
    }
    for (name, target) in [
        ("AllSaves", GearTarget::AllSaves),
        ("SaveFortitude", GearTarget::Save(Save::Fortitude)),
        ("SaveReflex", GearTarget::Save(Save::Reflex)),
        ("SaveWill", GearTarget::Save(Save::Will)),
        ("Initiative", GearTarget::Initiative),
        ("SpellDC", GearTarget::SpellDc(SpellScope::All)),
        (
            "SpellDC Enchantment",
            GearTarget::SpellDc(SpellScope::School(SpellSchool::Enchantment)),
        ),
        ("CasterLevel", GearTarget::CasterLevel),
        ("SpellPenetration", GearTarget::SpellPenetration),
        ("Concentration", GearTarget::Concentration),
    ] {
        assert_eq!(
            stat(name, "Resistance"),
            gear(target, Descriptor::Resistance),
            "{name}"
        );
    }
    for (name, key) in [
        ("SkillAthletics", "athletics"),
        ("SkillKnowledgeArcana", "knowledge-arcana"),
        ("SkillKnowledgeWorld", "knowledge-world"),
        ("SkillLoreNature", "lore-nature"),
        ("SkillLoreReligion", "lore-religion"),
        ("SkillMobility", "mobility"),
        ("SkillPerception", "perception"),
        ("SkillPersuasion", "persuasion"),
        ("SkillStealth", "stealth"),
        ("SkillThievery", "trickery"),
        ("SkillUseMagicDevice", "use-magic-device"),
    ] {
        assert_eq!(
            stat(name, "Competence"),
            gear(
                GearTarget::Skill(SkillKey::new(key)),
                Descriptor::Competence
            ),
            "{name}"
        );
    }
    // The game's `None` is the planner's untyped; a name with no word stays unwritten.
    assert_eq!(
        stat("Initiative", "Untyped"),
        gear(GearTarget::Initiative, Descriptor::Untyped)
    );
    assert_eq!(stat("Strength", "Fumbling"), None);
    assert_eq!(stat("SkillNothing", "Insight"), None);
    assert_eq!(stat("SpellDC Nothing", "Insight"), None);
    assert_eq!(
        gear_stat(&("Strength".to_owned(), "Insight".to_owned(), 1000)),
        None
    );
}

/// What `--write items` does on a copy: the files after, and the log.
fn items_with_log(dir: &Path) -> (Vec<(PathBuf, String)>, String) {
    let before = report_of(dir);
    let outcome = write_items(dir);
    let after = report_of(dir);
    let log = game_check::write::log::tsv(&outcome.changes, &[], &outcome.facts, &before, &after);
    (all_files(dir), log)
}

#[test]
fn two_runs_on_two_copies_give_the_same_files_and_logs() {
    let (one, two) = (data_copy(), data_copy());
    let (files_one, log_one) = items_with_log(one.path());
    let (files_two, log_two) = items_with_log(two.path());
    // The copies are in different folders: compare the paths from the data folder.
    let relative = |files: Vec<(PathBuf, String)>, dir: &Path| -> Vec<(PathBuf, String)> {
        files
            .into_iter()
            .map(|(path, text)| (path.strip_prefix(dir).unwrap().to_owned(), text))
            .collect()
    };
    assert_eq!(
        relative(files_one, one.path()),
        relative(files_two, two.path())
    );
    assert_eq!(log_one, log_two);
    assert!(log_one.starts_with("entry\tfield\told\tnew\tfindings removed\tfact\n"));
}

#[test]
fn the_log_names_the_findings_each_change_removed() {
    let dir = data_copy();
    let (_, log) = items_with_log(dir.path());
    let row = |entry: &str| {
        log.lines()
            .find(|line| line.starts_with(&format!("{entry}\t")))
            .unwrap_or_else(|| panic!("no row for {entry} in:\n{log}"))
            .split('\t')
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let cloak = row("gear item cloak-of-test");
    assert_eq!(cloak[1], "components");
    assert_eq!(cloak[4], "items / amount differs");
    assert_eq!(cloak[5], "-");
    assert_eq!(
        row("gear item headband-of-test")[4],
        "items / bonus missing in planner; items / type differs"
    );
    assert_eq!(
        row("gear item ring-of-nothing")[4],
        "items / bonus not in game"
    );
    // Rows are in entry order.
    let entries: Vec<_> = log
        .lines()
        .skip(1)
        .map(|l| l.split('\t').next().unwrap())
        .collect();
    let mut sorted = entries.clone();
    sorted.sort_unstable();
    assert_eq!(entries, sorted);
}

#[test]
fn a_dry_run_writes_nothing() {
    let dir = data_copy();
    let before = all_files(dir.path());
    let (_, dump) = crate::support::dump(&crate::support::binz(&crate::support::records(), 2));
    let outcome = run(
        &dump,
        dir.path(),
        &[WriteArea::Items],
        None,
        true,
        &Facts::default(),
        |_| panic!("a dry run checks nothing written"),
    )
    .unwrap();
    assert_eq!(outcome.changes.len(), 3);
    assert_eq!(outcome.applied.files.len(), 3, "the files it would write");
    assert_eq!(all_files(dir.path()), before);
}

fn items_with_facts(
    dir: &Path,
    facts: &str,
) -> Result<game_check::write::Outcome, game_check::write::WriteError> {
    let (_, dump) = crate::support::dump(&crate::support::binz(&crate::support::records(), 2));
    let facts = Facts::parse(facts).unwrap();
    run(&dump, dir, &[WriteArea::Items], None, false, &facts, |_| {
        Ok(())
    })
}

#[test]
fn a_held_fact_is_not_written_and_goes_to_the_user() {
    let dir = data_copy();
    let cloak = read(dir.path(), "gear/cloak-of-test.ron");
    let outcome =
        items_with_facts(dir.path(), "items / * / gear item cloak-of-test -> held").unwrap();
    assert_eq!(read(dir.path(), "gear/cloak-of-test.ron"), cloak);
    let held: Vec<_> = outcome
        .to_the_user
        .iter()
        .map(|c| c.at.to_string())
        .collect();
    assert_eq!(held, ["gear item cloak-of-test"]);
    assert_eq!(outcome.to_the_user[0].field, "components");
    assert!(
        outcome.to_the_user[0]
            .new
            .contains("AllSaves +3 Resistance")
    );
    // The others are written as without the registry, and the held finding stays in the report.
    assert_eq!(outcome.changes.len(), 2);
    assert!(read(dir.path(), "gear/ring-of-nothing.ron").contains("components: []"));
    assert!(
        report_of(dir.path())
            .text()
            .contains("gear item cloak-of-test")
    );
}

#[test]
fn an_applied_fact_is_written_and_logged_with_its_ticket() {
    let dir = data_copy();
    let before = report_of(dir.path());
    let outcome = items_with_facts(
        dir.path(),
        "items / * -> held\nitems / * / gear item cloak-of-test -> applied 999",
    )
    .unwrap();
    // The area is held, the entry's own line applies: only the cloak is written.
    assert_eq!(outcome.changes.len(), 1);
    assert_eq!(outcome.to_the_user.len(), 2);
    assert!(read(dir.path(), "gear/cloak-of-test.ron").contains("value: 3"));
    let log = game_check::write::log::tsv(
        &outcome.changes,
        &[],
        &outcome.facts,
        &before,
        &report_of(dir.path()),
    );
    let row: Vec<_> = log.lines().nth(1).unwrap().split('\t').collect();
    assert_eq!(row[0], "gear item cloak-of-test");
    assert_eq!(row[5], "applied 999");
}

#[test]
fn a_fact_naming_no_entry_fails_the_run_and_writes_nothing() {
    let dir = data_copy();
    let before = all_files(dir.path());
    let error =
        items_with_facts(dir.path(), "items / * / gear item nothing-here -> held").unwrap_err();
    assert!(
        matches!(error, WriteError::StaleFacts(ref lines) if lines.len() == 1),
        "{error}"
    );
    assert!(error.to_string().contains("gear item nothing-here"));
    assert_eq!(all_files(dir.path()), before);
}

#[test]
fn the_registry_refuses_what_it_cannot_read() {
    for bad in [
        "items / * / gear item x",
        "items / * / gear item x -> maybe",
        "items / * / gear item x -> applied",
        "nowhere / * -> held",
        "items / * / gear item x / 3 -> held",
    ] {
        assert!(Facts::parse(bad).is_err(), "{bad}");
    }
}

#[test]
fn the_embedded_registry_names_only_entries_the_data_has() {
    let data = seeds::read_game_data_files(&seeds::game_data_dir()).unwrap();
    let facts = Facts::embedded().unwrap();
    assert!(!facts.is_empty());
    assert_eq!(facts.stale(&data), Vec::<String>::new());
}

#[test]
fn the_most_specific_line_wins() {
    use game_check::Area;
    use game_check::write::facts::Decision;
    // The real class-table line shapes: the area-wide kind held, one entry applied.
    let facts = Facts::parse(
        "class tables / class skills differ (the user's facts) -> held\n\
         class tables / class skills differ (the user's facts) / archetype alchemist/metamorph -> applied 262\n\
         items -> held\n\
         items / bonuses -> applied 7\n\
         items / * / gear item cloak-of-test -> held\n",
    )
    .unwrap();
    let skills = "class skills differ (the user's facts)";
    // Entry over kind.
    assert_eq!(
        facts.decide(Area::ClassTables, skills, "archetype alchemist/metamorph"),
        Decision::Applied("262".to_owned())
    );
    assert_eq!(
        facts.decide(Area::ClassTables, skills, "archetype alchemist/other"),
        Decision::Held
    );
    // Kind over area, and the entry over both.
    assert_eq!(
        facts.decide(Area::Items, "bonuses", "gear item ring-of-nothing"),
        Decision::Applied("7".to_owned())
    );
    assert_eq!(
        facts.decide(Area::Items, "slot", "gear item ring-of-nothing"),
        Decision::Held
    );
    assert_eq!(
        facts.decide(Area::Items, "bonuses", "gear item cloak-of-test"),
        Decision::Held
    );
    // A kind no line names in an area no line names is free.
    assert_eq!(
        facts.decide(Area::ClassTables, "bab differs", "class warden"),
        Decision::Free
    );
}

#[test]
fn two_lines_of_one_specificity_are_a_parse_error() {
    for bad in [
        "items / * / gear item x -> held\nitems / * / gear item x -> applied 1",
        "items -> held\nitems / * -> applied 1",
        "items / bonuses -> held\nitems / bonuses -> held",
    ] {
        let error = Facts::parse(bad).unwrap_err().to_string();
        assert!(
            error.contains("line 2: the same as line 1"),
            "{bad}: {error}"
        );
    }
}

/// The registry's two kind-level lines: every class's and archetype's skill points and class skills are held.
const USER_HELD: &str = "class tables / skill points differ (the user's facts) -> held\nclass tables / class skills differ (the user's facts) -> held\n";

fn write_class_tables(dir: &Path) -> game_check::write::Outcome {
    write_tables(dir, crate::support::records(), USER_HELD)
}

fn write_tables(
    dir: &Path,
    records: Vec<crate::support::Record>,
    facts: &str,
) -> game_check::write::Outcome {
    let (_, dump) = crate::support::dump(&crate::support::binz(&records, 2));
    let facts = Facts::parse(facts).unwrap();
    run(
        &dump,
        dir,
        &[WriteArea::ClassTables],
        None,
        false,
        &facts,
        |_| Ok(()),
    )
    .unwrap()
}

#[test]
fn writing_class_tables_brings_bab_and_saves_to_the_game() {
    let dir = data_copy();
    let sentinel = read(dir.path(), "prestige/sentinel.ron");
    let outcome = write_class_tables(dir.path());
    // The Warden's class skills name a check, which is no skill.
    assert_eq!(outcome.unexpressed.len(), 1, "{:?}", outcome.unexpressed);
    assert!(outcome.unexpressed[0].what.contains("CheckIntimidate"));
    let held: Vec<_> = outcome
        .to_the_user
        .iter()
        .map(|c| format!("{} {}", c.at, c.field))
        .collect();
    assert_eq!(
        held,
        ["class warden skill_points", "class warden class_skills"]
    );
    let fields: Vec<_> = outcome
        .changes
        .iter()
        .map(|c| format!("{} {}", c.at, c.field))
        .collect();
    assert_eq!(
        fields,
        [
            "class warden reflex",
            "archetype warden/stoic will",
            "archetype warden/stray bab"
        ]
    );
    // The game's Reflex table is the good one.
    let warden = read(dir.path(), "classes/warden.ron");
    assert!(warden.contains("    reflex: Good,\n"), "{warden}");
    assert!(warden.contains("    // Planted: good in the game.\n"));
    // The game keeps the class's Will for the Stoic: its override goes, the comment above it stays.
    let stoic = read(dir.path(), "archetypes/warden/stoic.ron");
    assert!(!stoic.contains("will:"), "{stoic}");
    assert!(stoic.contains("// Planted: the game keeps the class's poor Will."));
    // The Stray's own table is a half-BAB one.
    let stray = read(dir.path(), "archetypes/warden/stray.ron");
    assert!(
        stray.contains("guide: \"warden#stray\", bab: Half)"),
        "{stray}"
    );
    // What the game agrees with is the same bytes.
    assert_eq!(read(dir.path(), "prestige/sentinel.ron"), sentinel);
    let report = report_of(dir.path());
    let report = report.text();
    for kind in [
        "BAB differs",
        "Fortitude differs",
        "Reflex differs",
        "Will differs",
    ] {
        assert!(
            !report.contains(&format!("class tables / {kind}")),
            "{kind} is left:\n{report}"
        );
    }
    // The user's facts are not this target's.
    assert!(report.contains("class tables / skill points differ"));
}

#[test]
fn a_save_table_no_formula_gives_is_none() {
    use game_check::write::tables::{bab_progression, save_progression};
    use rules::{BabProgression, SaveProgression};
    assert_eq!(
        save_progression(&[0, 2, 3, 3], 3, false),
        Some(SaveProgression::Good)
    );
    assert_eq!(
        save_progression(&[0, 0, 0, 1], 3, false),
        Some(SaveProgression::Poor)
    );
    // The prestige formulas are not the base ones.
    assert_eq!(save_progression(&[0, 1, 1, 2], 3, false), None);
    assert_eq!(
        save_progression(&[0, 1, 1, 2], 3, true),
        Some(SaveProgression::Good)
    );
    // An odd table, a short one and one that matches only at some levels.
    assert_eq!(save_progression(&[0, 2, 3, 9], 3, false), None);
    assert_eq!(save_progression(&[0, 2, 3], 3, false), None);
    assert_eq!(bab_progression(&[0, 1, 1, 5], 3), None);
    assert_eq!(
        bab_progression(&[0, 1, 2, 3], 3),
        Some(BabProgression::Full)
    );
    assert_eq!(
        bab_progression(&[0, 0, 1, 1], 3),
        Some(BabProgression::Half)
    );
    assert_eq!(
        bab_progression(&[0, 0, 1, 2, 3], 4),
        Some(BabProgression::ThreeQuarters)
    );
}

#[test]
fn a_class_table_run_is_idempotent() {
    let dir = data_copy();
    write_class_tables(dir.path());
    let after_one = all_files(dir.path());
    let again = write_class_tables(dir.path());
    assert!(again.changes.is_empty(), "{:?}", again.changes);
    assert!(again.applied.files.is_empty());
    assert_eq!(all_files(dir.path()), after_one);
}

fn applied_facts() -> String {
    format!(
        "{USER_HELD}\
         class tables / skill points differ (the user's facts) / class warden -> applied 262\n\
         class tables / class skills differ (the user's facts) / class warden -> applied 262\n\
         class tables / skill points differ (the user's facts) / archetype warden/stoic -> applied 262\n"
    )
}

#[test]
fn the_users_class_tables_are_held_without_an_applied_line() {
    let dir = data_copy();
    let warden = read(dir.path(), "classes/warden.ron");
    let outcome = write_class_tables(dir.path());
    // The Stoic's 5 is the held class's 4 and its 1, so only the class is the user's to decide.
    assert_eq!(outcome.to_the_user.len(), 2);
    assert!(
        outcome
            .changes
            .iter()
            .all(|c| c.field != "skill_points" && c.field != "class_skills"),
        "{:?}",
        outcome.changes
    );
    let after = read(dir.path(), "classes/warden.ron");
    assert!(after.contains("    skill_points: 4,\n"), "{after}");
    assert!(after.contains("class_skills: [\"athletics\", \"perception\"]"));
    // Only the saves moved in the file.
    assert_eq!(after.replace("reflex: Good", "reflex: Poor"), warden);
}

#[test]
fn applied_class_tables_are_written() {
    let dir = data_copy();
    let outcome = write_tables(dir.path(), crate::support::records(), &applied_facts());
    assert!(outcome.to_the_user.is_empty(), "{:?}", outcome.to_the_user);
    let warden = read(dir.path(), "classes/warden.ron");
    assert!(warden.contains("    skill_points: 3,\n"), "{warden}");
    // Intimidate is left (a check, not a skill); the rest is written, in the data's key order.
    assert!(
        warden.contains("class_skills: [\"athletics\", \"stealth\"]"),
        "{warden}"
    );
    // 3 + 1 in the game: the Stoic's own count differs from the class's.
    let stoic = read(dir.path(), "archetypes/warden/stoic.ron");
    assert!(stoic.contains("    skill_points: 4,\n"), "{stoic}");
    let key = |entry: &str, field: &'static str| (entry.to_owned(), field);
    assert_eq!(
        outcome
            .facts
            .get(&key("class warden", "skill_points"))
            .map(String::as_str),
        Some("applied 262")
    );
    // The finding for Intimidate stays, and a second run has nothing to write.
    let report = report_of(dir.path());
    assert!(report.text().contains("class tables / class skills differ"));
    let again = write_tables(dir.path(), crate::support::records(), &applied_facts());
    assert!(again.changes.is_empty(), "{:?}", again.changes);
}

/// The fixture's dump with the Warden's Fortitude table one no formula gives (`2, 3, 9`), and its data with a
/// Fortitude and a Reflex exception.
fn odd_fortitude() -> (tempfile::TempDir, game_check::write::Outcome) {
    let mut records = crate::support::records();
    records.push(record(
        0x57,
        "TestSavesOdd",
        "Kingmaker.Blueprints.Classes.BlueprintStatProgression",
        r#"{"Bonuses":[0,2,3,9,4]}"#,
    ));
    let warden = records
        .iter_mut()
        .find(|r| r.name == "WardenClass")
        .unwrap();
    warden.raw = warden.raw.replace(
        "\"m_FortitudeSave\":\"!bp_00000000000000000000000000000052\"",
        "\"m_FortitudeSave\":\"!bp_00000000000000000000000000000057\"",
    );
    let dir = data_copy();
    let path = dir.path().join("classes/warden.ron");
    let text = std::fs::read_to_string(&path).unwrap().replace(
        "    will: Poor,\n",
        "    will: Poor,\n    save_exceptions: [(level: 1, save: Fortitude, value: 2), (level: 1, save: Reflex, value: 9)],\n",
    );
    std::fs::write(&path, text).unwrap();
    let outcome = write_tables(dir.path(), records, USER_HELD);
    (dir, outcome)
}

#[test]
fn save_exceptions_stay_for_a_save_no_formula_gives() {
    let (dir, outcome) = odd_fortitude();
    let warden = read(dir.path(), "classes/warden.ron");
    // Reflex has a formula, so its exception goes; Fortitude has none, so its exception stays.
    assert!(
        warden.contains("save_exceptions: [(level: 1, save: Fortitude, value: 2)]"),
        "{warden}"
    );
    assert!(!warden.contains("value: 9"), "{warden}");
    assert!(warden.contains("    reflex: Good,\n"));
    assert!(outcome.changes.iter().any(|c| c.field == "save_exceptions"));
}

#[test]
fn an_unexpressed_class_table_is_left_and_listed() {
    let (dir, outcome) = odd_fortitude();
    let warden = read(dir.path(), "classes/warden.ron");
    assert!(warden.contains("    fort: Good,\n"), "{warden}");
    assert!(outcome.changes.iter().all(|c| c.field != "fort"));
    assert!(
        outcome
            .unexpressed
            .iter()
            .any(|each| each.at.to_string() == "class warden" && each.what.contains("Fortitude")),
        "{:?}",
        outcome.unexpressed
    );
}

/// The Stray overrides Reflex with the value the game gives the Warden.
fn stray_reflex(facts: &str) -> (tempfile::TempDir, game_check::write::Outcome) {
    let dir = data_copy();
    let path = dir.path().join("archetypes/warden/stray.ron");
    let text = std::fs::read_to_string(&path).unwrap().replace(
        "guide: \"warden#stray\")",
        "guide: \"warden#stray\", reflex: Good)",
    );
    std::fs::write(&path, text).unwrap();
    let outcome = write_tables(dir.path(), crate::support::records(), facts);
    (dir, outcome)
}

#[test]
fn an_archetype_override_the_class_now_equals_is_removed() {
    let (dir, _) = stray_reflex(USER_HELD);
    assert!(read(dir.path(), "classes/warden.ron").contains("    reflex: Good,\n"));
    let stray = read(dir.path(), "archetypes/warden/stray.ron");
    assert!(!stray.contains("reflex:"), "{stray}");
}

#[test]
fn a_held_class_save_is_not_assumed_by_its_archetypes() {
    let facts = format!("{USER_HELD}class tables / Reflex differs / class warden -> held\n");
    let (dir, outcome) = stray_reflex(&facts);
    // The class keeps Poor, so the Stray's Good still differs from it and stays.
    assert!(read(dir.path(), "classes/warden.ron").contains("    reflex: Poor,\n"));
    let stray = read(dir.path(), "archetypes/warden/stray.ron");
    assert!(stray.contains("reflex: Good"), "{stray}");
    assert!(
        outcome
            .to_the_user
            .iter()
            .any(|c| c.at.to_string() == "class warden" && c.field == "reflex")
    );
}

#[test]
fn every_change_is_of_a_kind_its_area_names() {
    let dir = data_copy();
    let tables = write_class_tables(dir.path());
    assert!(!tables.changes.is_empty() && !tables.to_the_user.is_empty());
    for change in tables.changes.iter().chain(&tables.to_the_user) {
        assert!(
            WriteArea::ClassTables.kinds().contains(&change.kind),
            "{}",
            change.kind
        );
    }
    let items = write_items(data_copy().path());
    for change in &items.changes {
        assert!(WriteArea::Items.kinds().contains(&change.kind));
    }
}

#[test]
fn every_kind_in_the_registry_is_one_the_check_emits() {
    use game_check::Area;
    let sources = [
        (
            Area::ClassTables,
            include_str!("../../src/areas/class_tables.rs"),
        ),
        (Area::Items, include_str!("../../src/areas/items.rs")),
    ];
    let kinds = Facts::embedded().unwrap().kinds();
    assert!(!kinds.is_empty());
    for (area, kind) in kinds {
        let (_, source) = sources
            .iter()
            .find(|(each, _)| *each == area)
            .unwrap_or_else(|| panic!("no source for {area:?}"));
        assert!(
            source.contains(&format!("\"{kind}\"")),
            "`{kind}` is no kind the check emits for {area:?}"
        );
    }
}

// The selections target (ticket 306a). The fixture's dump holds `TestBonusFeatSelection` (two feats, its picks ignore
// prerequisites), `TestShortFeatSelection` (a feat the data has no entry for) and `TestPathAbilitySelection` (two
// mythic abilities); the data's three selections each differ from them.

fn write_lists(
    dir: &Path,
    only: Option<&[&str]>,
) -> Result<game_check::write::Outcome, game_check::write::WriteError> {
    let (_, dump) = crate::support::dump(&crate::support::binz(&crate::support::records(), 2));
    let only: Option<std::collections::BTreeSet<String>> =
        only.map(|keys| keys.iter().map(|key| (*key).to_owned()).collect());
    run(
        &dump,
        dir,
        &[WriteArea::OptionLists],
        only.as_ref(),
        false,
        &Facts::default(),
        |_| Ok(()),
    )
}

#[test]
fn writing_a_feat_list_brings_it_to_the_game() {
    let dir = data_copy();
    let outcome = write_lists(dir.path(), None).unwrap();
    // TestBonusFeatSelection lists TestBonusOneFeat 000000000000000000000000000000b11 and TestBonusTwoFeat ...b12.
    let bonus = read(dir.path(), "selections/bonus-feats.ron");
    assert!(
        bonus.contains("feats: [\"bonus-one\", \"bonus-two\"],"),
        "{bonus}"
    );
    // TestPathAbilitySelection lists ...b21 and ...b22.
    let path = read(dir.path(), "selections/path-abilities.ron");
    assert!(
        path.contains("mythic_abilities: [\"path-one\", \"path-two\"],"),
        "{path}"
    );
    let planned: Vec<_> = outcome
        .changes
        .iter()
        .filter(|c| !matches!(c.field, "options" | "shared_options"))
        .map(|c| (c.at.to_string(), c.field, c.old.as_str(), c.new.as_str()))
        .collect();
    assert_eq!(
        planned,
        [
            (
                "selection bonus-feats".to_owned(),
                "feats",
                "bonus-stray",
                "bonus-two"
            ),
            (
                "selection bonus-feats".to_owned(),
                "ignores_prerequisites",
                "false",
                "true"
            ),
            (
                "selection path-abilities".to_owned(),
                "mythic_abilities",
                "",
                "path-two"
            ),
        ]
    );
}

#[test]
fn ignores_prerequisites_is_written_where_it_differs() {
    let dir = data_copy();
    write_lists(dir.path(), None).unwrap();
    // TestBonusFeatSelection: IgnorePrerequisites true; the data's selection had it unset.
    let bonus = read(dir.path(), "selections/bonus-feats.ron");
    assert!(bonus.contains("ignores_prerequisites: true,"), "{bonus}");
    // Planted comment and the other fields stay.
    assert!(bonus.starts_with("// Planted: "), "{bonus}");
    assert!(bonus.contains("name: \"Bonus feat\","), "{bonus}");
    // A selection of mythic abilities has no such flag.
    let path = read(dir.path(), "selections/path-abilities.ron");
    assert!(!path.contains("ignores_prerequisites"), "{path}");
}

#[test]
fn a_member_with_no_feat_entry_leaves_the_selection_unwritten() {
    let dir = data_copy();
    let short = read(dir.path(), "selections/short-feats.ron");
    let outcome = write_lists(dir.path(), None).unwrap();
    // TestShortFeatSelection lists TestBonusLostFeat ...b13, which no feat names.
    assert_eq!(read(dir.path(), "selections/short-feats.ron"), short);
    assert!(
        outcome
            .changes
            .iter()
            .all(|c| c.at.to_string() != "selection short-feats")
    );
    let left = unexpressed_of(&outcome, "selection short-feats");
    assert_eq!(left.len(), 1, "{left:?}");
    assert!(
        left[0].contains("no feat entry: TestBonusLostFeat"),
        "{}",
        left[0]
    );
}

#[test]
fn only_writes_only_the_selections_named() {
    let dir = data_copy();
    let bonus = read(dir.path(), "selections/bonus-feats.ron");
    let outcome = write_lists(dir.path(), Some(&["path-abilities"])).unwrap();
    assert_eq!(read(dir.path(), "selections/bonus-feats.ron"), bonus);
    assert_eq!(outcome.changes.len(), 1);
    assert!(read(dir.path(), "selections/path-abilities.ron").contains("path-two"));
    // The unwritten selection outside the set is not listed either.
    assert!(outcome.unexpressed.is_empty(), "{:?}", outcome.unexpressed);
    // A key that is no selection is refused, with nothing written.
    let error = write_lists(dir.path(), Some(&["no-such-selection"])).unwrap_err();
    assert!(error.to_string().contains("no-such-selection"), "{error}");
}

#[test]
fn a_second_option_lists_run_writes_nothing() {
    let dir = data_copy();
    write_lists(dir.path(), None).unwrap();
    let written = all_files(dir.path());
    let again = write_lists(dir.path(), None).unwrap();
    assert!(again.changes.is_empty(), "{:?}", again.changes);
    assert_eq!(all_files(dir.path()), written);
}

fn unexpressed_of(outcome: &game_check::write::Outcome, entry: &str) -> Vec<String> {
    (outcome.unexpressed.iter())
        .filter(|item| item.at.to_string() == entry)
        .map(|item| item.what.clone())
        .collect()
}

fn lists_of(
    outcome: &game_check::write::Outcome,
    entry: &str,
) -> Vec<(&'static str, String, String)> {
    (outcome.changes.iter())
        .filter(|c| c.at.to_string() == entry)
        .map(|c| (c.field, c.old.clone(), c.new.clone()))
        .collect()
}

// Options and ownership (ticket 306b). `base-powers` and `copy-powers` are an Improved-Weapon-shaped pair: the game's
// list of the base (TestBasePowers ...b43) holds TestPowerA ...b41, that of the copy (TestCopyPowers ...b44) holds
// TestPowerA and TestPowerX ...b42, and the data's base defines Power X and Power Z.

#[test]
fn an_option_moves_to_the_selection_whose_game_list_holds_it() {
    let dir = data_copy();
    let outcome = write_lists(dir.path(), None).unwrap();
    let base = read(dir.path(), "selections/base-powers.ron");
    assert!(
        base.contains("options: [\"power-a\", \"power-z\"],"),
        "{base}"
    );
    let copy = read(dir.path(), "selections/copy-powers.ron");
    assert!(copy.contains("options: [\"power-x\"],"), "{copy}");
    // Power A stays the base's and the copy shares it.
    assert!(copy.contains("shared_options: [\"power-a\"],"), "{copy}");
    assert_eq!(
        lists_of(&outcome, "selection copy-powers"),
        [("options", String::new(), "power-x".to_owned())]
    );
}

#[test]
fn an_option_no_game_list_holds_stays_and_is_listed() {
    let dir = data_copy();
    let outcome = write_lists(dir.path(), None).unwrap();
    // Power Z names no blueprint and no game list holds it: it stays in the base's options.
    assert!(read(dir.path(), "selections/base-powers.ron").contains("\"power-z\""));
    let left = unexpressed_of(&outcome, "selection option power-z");
    assert_eq!(left.len(), 1, "{left:?}");
    assert!(left[0].contains("no game list holds it"), "{left:?}");
}

#[test]
fn a_guid_two_options_hold_resolves_to_the_offered_one() {
    let dir = data_copy();
    let twin = read(dir.path(), "selections/twin-list.ron");
    let outcome = write_lists(dir.path(), None).unwrap();
    // TestTwinList lists TestTwin ...b46, which Twin A and Twin B both name; the list offers Twin B.
    assert_eq!(read(dir.path(), "selections/twin-list.ron"), twin);
    assert!(lists_of(&outcome, "selection twin-list").is_empty());
    assert!(unexpressed_of(&outcome, "selection twin-list").is_empty());
}

#[test]
fn a_move_outside_only_is_not_made() {
    let dir = data_copy();
    let base = read(dir.path(), "selections/base-powers.ron");
    let outcome = write_lists(dir.path(), Some(&["base-powers"])).unwrap();
    // The copy is outside the set, so Power X stays the base's and is listed.
    assert_eq!(read(dir.path(), "selections/base-powers.ron"), base);
    assert!(outcome.changes.is_empty(), "{:?}", outcome.changes);
    let left = unexpressed_of(&outcome, "selection option power-x");
    assert_eq!(left.len(), 1, "{left:?}");
    assert!(left[0].contains("not made"), "{left:?}");
    // With the copy alone, it shares the option the base still defines.
    let outcome = write_lists(dir.path(), Some(&["copy-powers"])).unwrap();
    assert_eq!(
        lists_of(&outcome, "selection copy-powers"),
        [("shared_options", String::new(), "power-x".to_owned())]
    );
}

#[test]
fn the_written_lists_pass_the_data_check() {
    let dir = data_copy();
    let defined_once = |path: &Path| {
        let data = seeds::read_game_data_files(path).unwrap();
        (data.check().iter())
            .map(|p| format!("{p:?}"))
            .filter(|p| p.contains("OptionNotDefinedOnce"))
            .count()
    };
    assert_eq!(defined_once(dir.path()), 0);
    write_lists(dir.path(), None).unwrap();
    assert_eq!(defined_once(dir.path()), 0);
}

// 306a's review.

#[test]
fn a_selection_with_no_blueprint_or_none_in_the_dump_is_listed() {
    let dir = data_copy();
    let outcome = write_lists(dir.path(), None).unwrap();
    assert_eq!(
        unexpressed_of(&outcome, "selection twin-other"),
        ["no blueprint"]
    );
    assert_eq!(
        unexpressed_of(&outcome, "selection ghost-feats"),
        ["blueprint not in the dump"]
    );
}

#[test]
fn an_only_key_that_is_no_selection_is_a_usage_error() {
    let dir = data_copy();
    let error = write_lists(dir.path(), Some(&["no-such-selection"])).unwrap_err();
    assert!(matches!(error, WriteError::Usage(_)), "{error}");
    assert!(!error.to_string().contains("fails its check"), "{error}");
}

#[test]
fn a_selection_whose_blueprint_is_another_selections_too_is_listed_unless_both_are_in_only() {
    // TestEchoList ...b48 lists Power X; echo-a and echo-b both name it, and the data's lists would be widened to it.
    let dir = data_copy();
    let echo = read(dir.path(), "selections/echo-a.ron");
    let outcome = write_lists(dir.path(), Some(&["echo-a"])).unwrap();
    assert_eq!(read(dir.path(), "selections/echo-a.ron"), echo);
    assert_eq!(
        unexpressed_of(&outcome, "selection echo-a"),
        ["its blueprint is another selection's too (a copy: 316)"]
    );
    let outcome = write_lists(dir.path(), Some(&["echo-a", "echo-b"])).unwrap();
    assert!(unexpressed_of(&outcome, "selection echo-a").is_empty());
    assert!(read(dir.path(), "selections/echo-a.ron").contains("shared_options: [\"power-x\"]"));
}

/// The fixture's records with `from` replaced by `to` in the raw JSON of the blueprint named `name`.
fn records_with(name: &str, from: &str, to: &str) -> Vec<crate::support::Record> {
    let mut records = crate::support::records();
    let record = records.iter_mut().find(|r| r.name == name).unwrap();
    assert!(record.raw.contains(from), "{name} has no {from}");
    record.raw = record.raw.replace(from, to);
    records
}

fn the_skill_points_stay_and_are_listed(
    records: Vec<crate::support::Record>,
    at: &str,
    what: &str,
) {
    let dir = data_copy();
    let outcome = write_tables(dir.path(), records, &applied_facts());
    assert!(
        outcome
            .unexpressed
            .iter()
            .any(|each| each.at.to_string() == at && each.what.contains(what)),
        "{:?}",
        outcome.unexpressed
    );
    assert!(
        outcome
            .changes
            .iter()
            .chain(&outcome.to_the_user)
            .all(|c| c.at.to_string() != at || c.field != "skill_points"),
        "{:?}",
        outcome.changes
    );
    let (file, kept) = match at {
        "class warden" => ("classes/warden.ron", "    skill_points: 4,\n"),
        _ => ("archetypes/warden/stoic.ron", "    skill_points: 5,\n"),
    };
    assert!(read(dir.path(), file).contains(kept));
}

#[test]
fn a_class_whose_blueprint_has_no_skill_points_is_listed_not_planned_to_zero() {
    the_skill_points_stay_and_are_listed(
        records_with("WardenClass", "\"SkillPoints\":3,", ""),
        "class warden",
        "skill points",
    );
}

#[test]
fn a_skill_points_value_out_of_range_is_listed_not_skipped() {
    the_skill_points_stay_and_are_listed(
        records_with("WardenClass", "\"SkillPoints\":3", "\"SkillPoints\":300"),
        "class warden",
        "skill points",
    );
    the_skill_points_stay_and_are_listed(
        records_with(
            "StoicArchetype",
            "\"AddSkillPoints\":1",
            "\"AddSkillPoints\":300",
        ),
        "archetype warden/stoic",
        "skill points",
    );
}

fn feature_text(key: &str, blueprint: &str) -> String {
    format!(
        "(key: \"{key}\", blueprint: \"{blueprint}\", name: \"Planted {key}\", summary: \"\", guide: None)\n"
    )
}

fn new_file(file: &str, text: &str) -> game_check::write::NewFile {
    let key = Path::new(file).file_stem().unwrap().to_str().unwrap();
    game_check::write::NewFile {
        at: DataAt::Feature(rules::FeatureKey::new(key)),
        file: PathBuf::from(file),
        text: text.to_owned(),
        kind: "carrier",
    }
}

fn valid(path: &Path) -> Result<(), String> {
    seeds::read_game_data_files(path)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

const BLUEPRINT_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1";
const BLUEPRINT_B: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa2";

#[test]
fn a_new_file_is_written_whole_and_read_back() {
    let dir = data_copy();
    let text = feature_text("hidden-trick", BLUEPRINT_A);
    let file = new_file("features/hidden-trick.ron", &text);
    // A field change in the same run lands too, and the new file's folder doesn't exist yet.
    assert!(!dir.path().join("features").exists());
    let applied = apply(
        dir.path(),
        &[change(
            "gear/cloak-of-test.ron",
            "components",
            components("GearStat(stat: AllSaves, value: 3, descriptor: Resistance)"),
        )],
        std::slice::from_ref(&file),
        valid,
    )
    .unwrap();
    assert_eq!(
        applied.created,
        [PathBuf::from("features/hidden-trick.ron")]
    );
    assert_eq!(applied.files, [PathBuf::from("gear/cloak-of-test.ron")]);
    assert_eq!(read(dir.path(), "features/hidden-trick.ron"), text);
    let data = seeds::read_game_data_files(dir.path()).unwrap();
    assert!(
        data.features
            .iter()
            .any(|feature| feature.key.as_str() == "hidden-trick")
    );
    // The log has one row for it: `(new file)`, old `-`, new its key and blueprint.
    let (_, dump) = crate::support::dump(&crate::support::binz(&crate::support::records(), 2));
    let report = game_check::check(
        &crate::support::dump(&crate::support::binz(&crate::support::records(), 2)).0,
        &dump,
        &data,
        &game_check::Known::parse("").unwrap(),
    )
    .unwrap();
    let log = game_check::write::log::tsv(
        &[],
        std::slice::from_ref(&file),
        &BTreeMap::new(),
        &report,
        &report,
    );
    assert_eq!(
        log.lines().nth(1).unwrap(),
        format!("feature hidden-trick\t(new file)\t-\thidden-trick {BLUEPRINT_A}\t-\t-")
    );
}

#[test]
fn an_existing_file_is_never_replaced_by_a_new_one() {
    let dir = data_copy();
    std::fs::create_dir(dir.path().join("features")).unwrap();
    let held = "(key: \"held\", blueprint: \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa9\", name: \"Held\")\n";
    std::fs::write(dir.path().join("features/held.ron"), held).unwrap();
    let before = all_files(dir.path());
    let error = apply(
        dir.path(),
        &[],
        &[
            new_file("features/fresh.ron", &feature_text("fresh", BLUEPRINT_A)),
            new_file("features/held.ron", &feature_text("held", BLUEPRINT_B)),
        ],
        valid,
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::Exists(_)), "{error}");
    assert_eq!(all_files(dir.path()), before, "nothing written");
    // Two new files with one path are refused too.
    let twice = new_file("features/fresh.ron", &feature_text("fresh", BLUEPRINT_A));
    let error = apply(dir.path(), &[], &[twice.clone(), twice], valid).unwrap_err();
    assert!(matches!(error, WriteError::Exists(_)), "{error}");
    assert_eq!(all_files(dir.path()), before);
}

#[test]
fn a_new_file_under_planner_or_outside_features_is_refused() {
    let dir = data_copy();
    let before = all_files(dir.path());
    let fresh = new_file("features/fresh.ron", &feature_text("fresh", BLUEPRINT_A));
    let error = apply(
        dir.path(),
        &[],
        &[
            fresh.clone(),
            new_file("planner/features/x.ron", &feature_text("x", BLUEPRINT_B)),
        ],
        valid,
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::Planner(_)), "{error}");
    for outside in [
        "gear/x.ron",
        "feats/x.ron",
        "features/deep/x.ron",
        "features/x.txt",
    ] {
        let error = apply(
            dir.path(),
            &[],
            &[
                fresh.clone(),
                new_file(outside, &feature_text("x", BLUEPRINT_B)),
            ],
            valid,
        )
        .unwrap_err();
        assert!(
            matches!(
                error,
                WriteError::NotOwned {
                    field: "(new file)",
                    ..
                }
            ),
            "{outside}: {error}"
        );
    }
    assert_eq!(all_files(dir.path()), before, "nothing written");
    assert!(game_check::write::creatable(Path::new("features/x.ron")));
    assert!(!game_check::write::creatable(Path::new("gear/x.ron")));
}

#[test]
fn a_failed_validation_removes_every_new_file() {
    let dir = data_copy();
    let before = all_files(dir.path());
    let files = [
        new_file("features/one.ron", &feature_text("one", BLUEPRINT_A)),
        new_file("features/two.ron", &feature_text("two", BLUEPRINT_B)),
    ];
    let field = change(
        "gear/cloak-of-test.ron",
        "components",
        components("GearStat(stat: AllSaves, value: 3, descriptor: Resistance)"),
    );
    let error = apply(dir.path(), std::slice::from_ref(&field), &files, |path| {
        // The new files are on the disk when the check runs.
        assert!(path.join("features/two.ron").is_file());
        Err("a planted failure".to_owned())
    })
    .unwrap_err();
    assert!(matches!(error, WriteError::Invalid(_)), "{error}");
    assert_eq!(all_files(dir.path()), before, "files and folder gone");
    assert!(!dir.path().join("features").exists());

    // A file that does not parse as a feature, or whose key is not its stem, is a read-back failure and undone.
    let bad = [
        new_file("features/odd.ron", "(key: \"odd\", name: \"Odd\")\n"),
        new_file(
            "features/other.ron",
            &feature_text("not-other", BLUEPRINT_A),
        ),
    ];
    for file in bad {
        let error = apply(dir.path(), &[], std::slice::from_ref(&file), valid).unwrap_err();
        assert!(matches!(error, WriteError::ReadBack { .. }), "{error}");
        assert_eq!(all_files(dir.path()), before, "{}", file.file.display());
    }
    assert!(!dir.path().join("features").exists());
}

#[test]
fn a_dry_run_lists_the_new_files_and_writes_none() {
    let dir = data_copy();
    let before = all_files(dir.path());
    let files = [
        new_file("features/two.ron", &feature_text("two", BLUEPRINT_B)),
        new_file("features/one.ron", &feature_text("one", BLUEPRINT_A)),
    ];
    let planned = game_check::write::plan(dir.path(), &[], &files).unwrap();
    assert_eq!(
        planned.created,
        [
            PathBuf::from("features/one.ron"),
            PathBuf::from("features/two.ron")
        ]
    );
    assert!(planned.files.is_empty());
    assert_eq!(all_files(dir.path()), before);
    assert!(!dir.path().join("features").exists());
    // The guards run in a dry run too.
    let taken = [new_file("gear/x.ron", &feature_text("x", BLUEPRINT_A))];
    assert!(game_check::write::plan(dir.path(), &[], &taken).is_err());
}

#[test]
fn a_file_planted_after_the_plan_is_neither_replaced_nor_removed() {
    let dir = data_copy();
    let root = dir.path().to_owned();
    let planted =
        "(key: \"one\", blueprint: \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa9\", name: \"Planted\")\n";
    let files = [
        new_file("features/one.ron", &feature_text("one", BLUEPRINT_A)),
        new_file("features/two.ron", &feature_text("two", BLUEPRINT_B)),
    ];
    let field = change(
        "gear/cloak-of-test.ron",
        "components",
        components("GearStat(stat: AllSaves, value: 3, descriptor: Resistance)"),
    );
    let before_gear = read(&root, "gear/cloak-of-test.ron");
    // The field file's write is the step between the plan and the new files: it plants `features/one.ron`.
    let error = apply_with(
        &root,
        std::slice::from_ref(&field),
        &files,
        valid,
        |path, text| {
            std::fs::create_dir_all(root.join("features"))?;
            std::fs::write(root.join("features/one.ron"), planted)?;
            atomic_write(path, text)
        },
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::Exists(_)), "{error}");
    assert_eq!(
        read(&root, "features/one.ron"),
        planted,
        "not replaced or removed"
    );
    assert!(!root.join("features/two.ron").exists());
    assert_eq!(read(&root, "gear/cloak-of-test.ron"), before_gear);
}

#[test]
fn two_new_files_a_case_insensitive_disk_would_merge_are_refused() {
    let dir = data_copy();
    let error = apply(
        dir.path(),
        &[],
        &[
            new_file("features/Foo.ron", &feature_text("Foo", BLUEPRINT_A)),
            new_file("features/foo.ron", &feature_text("foo", BLUEPRINT_B)),
        ],
        valid,
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::Exists(_)), "{error}");
    assert!(!dir.path().join("features").exists());
}

// The carriers target (ticket 307b), on a world of its own: a feat whose prerequisites name hidden features, a
// chain of two up to a mapped holder, a selection's member, and keys that share a suffix-less form.

const FEATURE: &str = "Kingmaker.Blueprints.Classes.BlueprintFeature";
const PROGRESSION: &str = "Kingmaker.Blueprints.Classes.Selection.BlueprintProgression";
const SELECTION: &str = "Kingmaker.Blueprints.Classes.Selection.BlueprintFeatureSelection";

fn feature_record(n: u32, name: &str, type_name: &str, components: &str) -> crate::support::Record {
    record(
        n,
        name,
        type_name,
        &format!("{{\"$type\": \"0, {name}\", \"Components\": [{components}]}}"),
    )
}

fn requires(n: u32) -> String {
    format!(
        "{{\"$type\": \"0, PrerequisiteFeature\", \"Group\": \"All\", \"m_Feature\": \"!bp_{}\"}}",
        guid(n)
    )
}

fn adds(n: u32) -> String {
    format!(
        "{{\"$type\": \"0, AddFacts\", \"m_Facts\": [\"!bp_{}\"]}}",
        guid(n)
    )
}

/// The world's data folder and dump. The feat `steady-feat` (100) requires, directly or inside a condition's tree,
/// `HiddenTrickFeature` (101), `DeepRankUp2Feature` (102), `ListedMemberFeature` (103, a member of the selection 104, an entry), `UnmodelledPickFeature` (109, listed by the selection 108, no entry)
/// and four whose keys share a form: 110 `FooFeature`, 111 `FooProgression`, 112 `BarFeature`, 113 `BazFeature`
/// (the entry `baz` exists). 101 is added by `TrickWrapperFeature` (105), which `MappedHolderFeature` (106, an entry)
/// adds; 107 `AboveMappedFeature` adds 106.
fn carriers_world() -> (tempfile::TempDir, game_check::Dump) {
    let dir = data_copy();
    let put = |file: &str, text: String| std::fs::write(dir.path().join(file), text).unwrap();
    std::fs::create_dir(dir.path().join("features")).unwrap();
    put(
        "feats/steady-feat.ron",
        format!(
            "(key: \"steady-feat\", blueprint: \"{}\", name: \"Steady\", category: Weapon)\n",
            guid(100)
        ),
    );
    put(
        "features/mapped-holder.ron",
        format!(
            "(key: \"mapped-holder\", blueprint: \"{}\", name: \"Mapped holder\")\n",
            guid(106)
        ),
    );
    put(
        "features/baz.ron",
        format!(
            "(key: \"baz\", blueprint: \"{}\", name: \"Baz\")\n",
            guid(114)
        ),
    );
    put(
        "selections/listed.ron",
        format!(
            "(key: \"listed\", blueprint: \"{}\", name: \"Listed\", feats: [\"steady-feat\"])\n",
            guid(104)
        ),
    );
    let tree = format!(
        "{{\"$type\": \"0, PrerequisiteCondition\", \"Group\": \"All\", \"Condition\": {{\"Conditions\": [{{\"m_Feature\": \"!bp_{}\"}}, {{\"Inner\": {{\"m_Feature\": \"!bp_{}\"}}}}]}}}}",
        guid(102),
        guid(103)
    );
    let steady = [101, 109, 110, 111, 112, 113]
        .iter()
        .map(|n| requires(*n))
        .chain([tree])
        .collect::<Vec<_>>()
        .join(", ");
    let records = vec![
        feature_record(100, "SteadyFeat", FEATURE, &steady),
        feature_record(101, "HiddenTrickFeature", FEATURE, ""),
        feature_record(102, "DeepRankUp2Feature", FEATURE, ""),
        feature_record(103, "ListedMemberFeature", FEATURE, ""),
        record(
            104,
            "ListedSelection",
            SELECTION,
            &format!(
                "{{\"$type\": \"0, BlueprintFeatureSelection\", \"m_AllFeatures\": [\"!bp_{}\"]}}",
                guid(103)
            ),
        ),
        record(
            108,
            "UnmodelledSelection",
            SELECTION,
            &format!(
                "{{\"$type\": \"0, BlueprintFeatureSelection\", \"m_AllFeatures\": [\"!bp_{}\"]}}",
                guid(109)
            ),
        ),
        feature_record(109, "UnmodelledPickFeature", FEATURE, ""),
        feature_record(105, "TrickWrapperFeature", FEATURE, &adds(101)),
        feature_record(106, "MappedHolderFeature", FEATURE, &adds(105)),
        feature_record(107, "AboveMappedFeature", FEATURE, &adds(106)),
        feature_record(110, "FooFeature", FEATURE, ""),
        feature_record(111, "FooProgression", PROGRESSION, ""),
        feature_record(112, "BarFeature", FEATURE, ""),
        feature_record(113, "BazFeature", FEATURE, ""),
    ];
    let (_, dump) = crate::support::dump(&crate::support::binz(&records, 1));
    (dir, dump)
}

fn write_carriers(
    dir: &Path,
    dump: &game_check::Dump,
    only: Option<&[u32]>,
    dry_run: bool,
) -> Result<game_check::write::Outcome, WriteError> {
    let only: Option<std::collections::BTreeSet<String>> =
        only.map(|only| only.iter().map(|n| guid(*n)).collect());
    run(
        dump,
        dir,
        &[WriteArea::Carriers],
        only.as_ref(),
        dry_run,
        &Facts::default(),
        valid,
    )
}

fn made(outcome: &game_check::write::Outcome) -> Vec<String> {
    outcome
        .created
        .iter()
        .map(|new| new.file.display().to_string())
        .collect()
}

#[test]
fn a_prerequisite_naming_an_unmapped_feature_makes_a_carrier() {
    let (dir, dump) = carriers_world();
    let outcome = write_carriers(dir.path(), &dump, None, false).unwrap();
    assert_eq!(
        read(dir.path(), "features/hidden-trick.ron"),
        format!(
            "(key: \"hidden-trick\", blueprint: \"{}\", name: \"Hidden Trick\")\n",
            guid(101)
        )
    );
    // A reference deeper inside a condition's tree counts, and digits are a word of their own.
    assert_eq!(
        read(dir.path(), "features/deep-rank-up-2.ron"),
        format!(
            "(key: \"deep-rank-up-2\", blueprint: \"{}\", name: \"Deep Rank Up 2\")\n",
            guid(102)
        )
    );
    assert!(outcome.unexpressed.is_empty());
    assert!(outcome.changes.is_empty());
    // The files read back as game data (`run`'s own check ran on them).
    let data = seeds::read_game_data_files(dir.path()).unwrap();
    assert_eq!(data.features.len(), 2 + 8);
}

#[test]
fn a_chain_up_to_an_entry_makes_carriers() {
    let (dir, dump) = carriers_world();
    let outcome = write_carriers(dir.path(), &dump, None, false).unwrap();
    let files = made(&outcome);
    // 105 adds 101 and is held by the entry 106: a carrier; 107 adds the entry, above the chain: not one.
    assert!(
        files.contains(&"features/trick-wrapper.ron".to_owned()),
        "{files:?}"
    );
    assert!(
        !files.iter().any(|f| f.contains("above-mapped")),
        "{files:?}"
    );
    assert!(
        !files.iter().any(|f| f.contains("mapped-holder")),
        "{files:?}"
    );
    // Nothing else: the feat itself is mapped, the selection and its member are not carriers.
    assert_eq!(files.len(), 8, "{files:?}");
}

#[test]
fn a_selection_member_is_never_a_carrier() {
    let (dir, dump) = carriers_world();
    let outcome = write_carriers(dir.path(), &dump, None, false).unwrap();
    assert!(
        !made(&outcome).iter().any(|f| f.contains("listed-member")),
        "{:?}",
        made(&outcome)
    );
    assert!(!dir.path().join("features/listed-member.ron").exists());
}

#[test]
fn keys_drop_the_suffix_unless_two_would_share_one() {
    let (dir, dump) = carriers_world();
    let outcome = write_carriers(dir.path(), &dump, None, false).unwrap();
    let keys: Vec<(String, String)> = outcome
        .created
        .iter()
        .map(|new| {
            (new.at.key().unwrap().to_owned(), {
                let text = read(dir.path(), &new.file.display().to_string());
                text.split("name: \"")
                    .nth(1)
                    .unwrap()
                    .trim_end_matches("\")\n")
                    .to_owned()
            })
        })
        .collect();
    let keys: std::collections::BTreeMap<_, _> = keys.into_iter().collect();
    assert_eq!(
        keys.iter()
            .map(|(k, n)| format!("{k}={n}"))
            .collect::<Vec<_>>(),
        [
            "bar=Bar",
            // `baz` is an entry's key already: the carrier keeps its suffix.
            "baz-feature=Baz Feature",
            "deep-rank-up-2=Deep Rank Up 2",
            // Two assets share `foo`: each keeps its suffix.
            "foo-feature=Foo Feature",
            "foo-progression=Foo Progression",
            "hidden-trick=Hidden Trick",
            "trick-wrapper=Trick Wrapper",
            "unmodelled-pick=Unmodelled Pick",
        ]
    );
    // The keys don't depend on `--only`: the one asset keeps its suffix when the other isn't asked for.
    let (dir, dump) = carriers_world();
    let outcome = write_carriers(dir.path(), &dump, Some(&[110]), false).unwrap();
    assert_eq!(made(&outcome), ["features/foo-feature.ron"]);
}

#[test]
fn a_second_carriers_run_writes_nothing() {
    let (dir, dump) = carriers_world();
    write_carriers(dir.path(), &dump, None, false).unwrap();
    let after = all_files(dir.path());
    let again = write_carriers(dir.path(), &dump, None, false).unwrap();
    assert!(again.created.is_empty(), "{:?}", made(&again));
    assert!(again.applied.created.is_empty());
    assert_eq!(all_files(dir.path()), after);
}

#[test]
fn two_carriers_runs_on_two_copies_make_the_same_files() {
    let ((one, dump), (two, _)) = (carriers_world(), carriers_world());
    write_carriers(one.path(), &dump, None, false).unwrap();
    write_carriers(two.path(), &dump, None, false).unwrap();
    let relative = |dir: &Path| -> Vec<(PathBuf, String)> {
        all_files(dir)
            .into_iter()
            .map(|(path, text)| (path.strip_prefix(dir).unwrap().to_owned(), text))
            .collect()
    };
    assert_eq!(relative(one.path()), relative(two.path()));
}

#[test]
fn only_takes_guids_and_a_dry_run_writes_none() {
    let (dir, dump) = carriers_world();
    let before = all_files(dir.path());
    let dry = write_carriers(dir.path(), &dump, Some(&[101, 105]), true).unwrap();
    assert_eq!(
        made(&dry),
        ["features/hidden-trick.ron", "features/trick-wrapper.ron"]
    );
    assert_eq!(dry.applied.created.len(), 2);
    assert_eq!(all_files(dir.path()), before);
    // A word that is no GUID, and both areas at once, are usage errors.
    let bad: std::collections::BTreeSet<String> = ["steady-feat".to_owned()].into();
    let facts = Facts::default();
    let error = run(
        &dump,
        dir.path(),
        &[WriteArea::Carriers],
        Some(&bad),
        true,
        &facts,
        valid,
    )
    .unwrap_err();
    assert!(matches!(error, WriteError::Usage(_)), "{error}");
    let one: std::collections::BTreeSet<String> = [guid(101)].into();
    let areas = [WriteArea::Carriers, WriteArea::OptionLists];
    let error = run(&dump, dir.path(), &areas, Some(&one), true, &facts, valid).unwrap_err();
    assert!(matches!(error, WriteError::Usage(_)), "{error}");
}

#[test]
fn a_member_of_a_list_no_entry_stands_for_is_still_a_carrier() {
    let (dir, dump) = carriers_world();
    let outcome = write_carriers(dir.path(), &dump, None, false).unwrap();
    // 109 is listed by a selection (108) that has no entry and no row: nothing makes it a pick.
    assert!(made(&outcome).contains(&"features/unmodelled-pick.ron".to_owned()));
}

#[test]
fn only_creates_a_carrier_another_ticket_owns() {
    // 109 stands for a blueprint another ticket will own (a member of a list no entry stands for yet): nothing
    // stops `--only` naming it until 307c's registry exists, so a slice names only its own GUIDs.
    let (dir, dump) = carriers_world();
    let outcome = write_carriers(dir.path(), &dump, Some(&[109]), false).unwrap();
    assert_eq!(made(&outcome), ["features/unmodelled-pick.ron"]);
    assert!(dir.path().join("features/unmodelled-pick.ron").is_file());
}

/// A world shaped as the composite blasts are: the feat `blast-feat` (130) requires `BlueFlameBlastFeature` (131),
/// which the orphan `BlueFlameBlastProgression` (132) lists in its `LevelEntries`, as does `ListedProgression` (133),
/// which a selection (134, no entry) lists and `MappedHolderFeature` (135, an entry) adds.
fn blast_world() -> (tempfile::TempDir, game_check::Dump) {
    let dir = data_copy();
    let put = |file: &str, text: String| std::fs::write(dir.path().join(file), text).unwrap();
    std::fs::create_dir(dir.path().join("features")).unwrap();
    put(
        "feats/blast-feat.ron",
        format!(
            "(key: \"blast-feat\", blueprint: \"{}\", name: \"Blast\", category: Weapon)\n",
            guid(130)
        ),
    );
    put(
        "features/mapped-holder.ron",
        format!(
            "(key: \"mapped-holder\", blueprint: \"{}\", name: \"Mapped holder\")\n",
            guid(135)
        ),
    );
    let progression = |n: u32, name: &str| {
        record(
            n,
            name,
            PROGRESSION,
            &format!(
                "{{\"$type\": \"0, BlueprintProgression\", \"Components\": [], \"LevelEntries\": [{{\"Level\": 1, \"m_Features\": [\"!bp_{}\"]}}]}}",
                guid(131)
            ),
        )
    };
    let records = vec![
        feature_record(130, "BlastFeat", FEATURE, &requires(131)),
        feature_record(131, "BlueFlameBlastFeature", FEATURE, ""),
        progression(132, "BlueFlameBlastProgression"),
        progression(133, "ListedProgression"),
        record(
            134,
            "UnmodelledSelection",
            SELECTION,
            &format!(
                "{{\"$type\": \"0, BlueprintFeatureSelection\", \"m_AllFeatures\": [\"!bp_{}\"]}}",
                guid(133)
            ),
        ),
        feature_record(135, "MappedHolderFeature", FEATURE, &adds(133)),
    ];
    let (_, dump) = crate::support::dump(&crate::support::binz(&records, 1));
    (dir, dump)
}

#[test]
fn a_chain_walk_climbs_only_to_a_blueprint_that_is_held_and_no_pick() {
    let (dir, dump) = blast_world();
    let outcome = write_carriers(dir.path(), &dump, None, false).unwrap();
    // 132 nothing grants and 133 is a pick of a selection's list (held by the entry 135 though): neither is planned,
    // so the feature keeps its short key.
    assert_eq!(made(&outcome), ["features/blue-flame-blast.ron"]);
}

#[test]
fn a_run_in_pieces_keys_as_one_run_does() {
    let (whole, dump) = carriers_world();
    write_carriers(whole.path(), &dump, Some(&[110, 111]), false).unwrap();
    let (pieces, _) = carriers_world();
    let first = write_carriers(pieces.path(), &dump, Some(&[110]), false).unwrap();
    assert_eq!(made(&first), ["features/foo-feature.ron"]);
    // `FooFeature` is an entry now, and `FooProgression` still doesn't take `foo`.
    let second = write_carriers(pieces.path(), &dump, Some(&[111]), false).unwrap();
    assert_eq!(made(&second), ["features/foo-progression.ron"]);
    let relative = |dir: &Path| -> Vec<(PathBuf, String)> {
        all_files(dir)
            .into_iter()
            .map(|(path, text)| (path.strip_prefix(dir).unwrap().to_owned(), text))
            .collect()
    };
    assert_eq!(relative(whole.path()), relative(pieces.path()));
}

#[test]
fn only_refuses_a_value_that_is_no_guid() {
    let (dir, dump) = carriers_world();
    let facts = Facts::default();
    let good = guid(101);
    for bad in [
        // Not hex, at 32 characters.
        "z".repeat(32),
        // Hex, too short and too long.
        good[1..].to_owned(),
        format!("{good}0"),
        // Uppercase hex.
        good.to_uppercase(),
    ] {
        let only: std::collections::BTreeSet<String> = [bad.clone()].into();
        let error = run(
            &dump,
            dir.path(),
            &[WriteArea::Carriers],
            Some(&only),
            true,
            &facts,
            valid,
        )
        .unwrap_err();
        assert!(matches!(error, WriteError::Usage(_)), "{bad}: {error}");
    }
}

// The carrier-grants target (ticket 307m): the grants an existing entry's blueprint makes of a carrier. The world:
// `SteadyFeat` (100, the feat `steady-feat`) requires the hidden `HiddenTrickFeature` (101) and `DeepRankUp2Feature`
// (102); `TrickWrapperFeature` (105) adds 101 and is added by `MappedHolderFeature` (106, the entry `mapped-holder`),
// so 101, 102 and 105 are carriers. The option `rooted` (120) is the progression `RootedProgression`, granting 102 at
// option level 1 and 101 at 2, where its file already has a row. The option `plain` (121) holds 102 if it has the
// feat, and 101 if it has a blueprint no entry maps.

fn grants_world() -> (tempfile::TempDir, game_check::Dump) {
    let dir = data_copy();
    let put = |file: &str, text: String| std::fs::write(dir.path().join(file), text).unwrap();
    std::fs::create_dir(dir.path().join("features")).unwrap();
    put(
        "feats/steady-feat.ron",
        format!(
            "(key: \"steady-feat\", blueprint: \"{}\", name: \"Steady\", category: Weapon)\n",
            guid(100)
        ),
    );
    put(
        "features/mapped-holder.ron",
        format!(
            "(key: \"mapped-holder\", blueprint: \"{}\", name: \"Mapped holder\")\n",
            guid(106)
        ),
    );
    put(
        "options/rooted.ron",
        format!(
            "(key: \"rooted\", blueprint: \"{}\", name: \"Rooted\", levels: [(level: 2, features: [\"mapped-holder\"])])\n",
            guid(120)
        ),
    );
    put(
        "options/plain.ron",
        format!(
            "(key: \"plain\", blueprint: \"{}\", name: \"Plain\")\n",
            guid(121)
        ),
    );
    let if_has = |feature: u32, checked: u32| {
        format!(
            "{{\"$type\": \"0, AddFeatureIfHasFact\", \"m_CheckedFact\": \"!bp_{}\", \"m_Feature\": \"!bp_{}\", \"Not\": true}}",
            guid(checked),
            guid(feature)
        )
    };
    let rows = format!(
        "{{\"$type\": \"0, BlueprintProgression\", \"Components\": [], \"LevelEntries\": [{{\"Level\": 1, \"m_Features\": [\"!bp_{}\"]}}, {{\"Level\": 2, \"m_Features\": [\"!bp_{}\"]}}]}}",
        guid(102),
        guid(101)
    );
    let records = vec![
        feature_record(
            100,
            "SteadyFeat",
            FEATURE,
            &format!("{}, {}", requires(101), requires(102)),
        ),
        feature_record(101, "HiddenTrickFeature", FEATURE, ""),
        feature_record(102, "DeepRankUp2Feature", FEATURE, ""),
        feature_record(105, "TrickWrapperFeature", FEATURE, &adds(101)),
        feature_record(106, "MappedHolderFeature", FEATURE, &adds(105)),
        record(120, "RootedProgression", PROGRESSION, &rows),
        feature_record(
            121,
            "PlainFeature",
            FEATURE,
            &format!("{}, {}", if_has(102, 100), if_has(101, 999)),
        ),
    ];
    let (_, dump) = crate::support::dump(&crate::support::binz(&records, 1));
    (dir, dump)
}

/// The carriers, then the grants, as two runs.
fn write_grants(dir: &Path, dump: &game_check::Dump, dry_run: bool) -> game_check::write::Outcome {
    let facts = Facts::default();
    run(
        dump,
        dir,
        &[WriteArea::CarrierGrants],
        None,
        dry_run,
        &facts,
        valid,
    )
    .unwrap()
}

fn grants_after_carriers() -> (
    tempfile::TempDir,
    game_check::Dump,
    game_check::write::Outcome,
) {
    let (dir, dump) = grants_world();
    write_carriers(dir.path(), &dump, None, false).unwrap();
    let outcome = write_grants(dir.path(), &dump, false);
    (dir, dump, outcome)
}

fn feature_fact(key: &str) -> rules::Fact {
    rules::Fact::Feature(rules::FeatureKey::new(key))
}

#[test]
fn an_option_s_row_names_the_carrier_its_progression_grants() {
    let (dir, _, outcome) = grants_after_carriers();
    let data = seeds::read_game_data_files(dir.path()).unwrap();
    let option = data
        .options
        .iter()
        .find(|o| o.key.as_str() == "rooted")
        .unwrap();
    // Level 1 is a new row; level 2 keeps what it has and gains the carrier (the loader sorts a row's keys).
    let rows: Vec<(u8, Vec<&str>)> = option
        .levels
        .iter()
        .map(|row| (row.level, row.features.iter().map(|k| k.as_str()).collect()))
        .collect();
    assert_eq!(
        rows,
        [
            (1, vec!["deep-rank-up-2"]),
            (2, vec!["hidden-trick", "mapped-holder"])
        ]
    );
    assert!(
        outcome
            .changes
            .iter()
            .any(|c| c.field == "levels" && c.kind == "carrier grant"),
        "{:?}",
        outcome.changes
    );
}

#[test]
fn a_feature_grants_the_carrier_its_blueprint_adds() {
    let (dir, _, outcome) = grants_after_carriers();
    let data = seeds::read_game_data_files(dir.path()).unwrap();
    let holder = data
        .features
        .iter()
        .find(|f| f.key.as_str() == "mapped-holder")
        .unwrap();
    assert_eq!(
        holder.components,
        [rules::Component::Grant(
            [feature_fact("trick-wrapper")].into()
        )]
    );
    // The entry's other fields are as they were (the read-back checked the bytes between).
    assert_eq!(holder.name, "Mapped holder");
    // `plain` holds `deep-rank-up-2` if it has the feat, with the game's `Not`.
    let plain = data
        .options
        .iter()
        .find(|o| o.key.as_str() == "plain")
        .unwrap();
    assert_eq!(
        plain.components,
        [rules::Component::GrantIf {
            has: rules::Fact::Feat(rules::FeatKey::new("steady-feat")),
            not: true,
            fact: feature_fact("deep-rank-up-2"),
        }]
    );
    // The grant checked on a blueprint no entry maps is left out and listed.
    let left: Vec<&str> = outcome
        .unexpressed
        .iter()
        .map(|u| u.what.as_str())
        .collect();
    assert_eq!(left.len(), 1, "{left:?}");
    assert!(left[0].contains("HiddenTrickFeature"), "{left:?}");
    assert!(outcome.unexpressed[0].at.to_string().contains("plain"));
}

#[test]
fn a_grant_already_named_is_not_written_again() {
    let (dir, dump, _) = grants_after_carriers();
    let after = all_files(dir.path());
    let again = write_grants(dir.path(), &dump, false);
    assert!(again.changes.is_empty(), "{:?}", again.changes);
    assert!(again.applied.files.is_empty());
    assert_eq!(all_files(dir.path()), after);
}

#[test]
fn a_dry_run_of_the_grants_counts_the_carriers_it_would_follow() {
    // Before any carrier exists, a dry run reads the ones the carriers area would make; a real run writes none of
    // the grants naming a carrier no file holds.
    let (dir, dump) = grants_world();
    let planned = write_grants(dir.path(), &dump, true);
    let entries: std::collections::BTreeSet<_> = planned.changes.iter().map(|c| &c.file).collect();
    assert_eq!(entries.len(), 3, "{:?}", planned.changes);
    assert!(!dir.path().join("features/hidden-trick.ron").exists());
    let real = write_grants(dir.path(), &dump, false);
    assert!(
        real.changes
            .iter()
            .all(|c| c.field != "components" || c.at.to_string() != "feature mapped-holder"),
        "{:?}",
        real.changes
    );
}
