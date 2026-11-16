//! Applying changes (ticket 293b): the write fixture's tree reads as game data, a change writes only its field, a
//! planner file is refused with nothing written, and a failed check puts every file back.

use std::path::{Path, PathBuf};

use game_check::write::items::gear_stat;
use game_check::write::span::Value;
use game_check::write::{
    Change, WriteArea, WriteError, apply, apply_with, entry_file, read_back, run,
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
    let error = apply(dir.path(), &[stoic], |path| {
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
    run(&dump, dir, &[WriteArea::Items], false, |_| Ok(())).unwrap()
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
    let outcome = run(&dump, dir.path(), &[WriteArea::Items], false, |_| Ok(())).unwrap();
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
    let log = game_check::write::log::tsv(&outcome.changes, &before, &after);
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
    let outcome = run(&dump, dir.path(), &[WriteArea::Items], true, |_| {
        panic!("a dry run checks nothing written")
    })
    .unwrap();
    assert_eq!(outcome.changes.len(), 3);
    assert_eq!(outcome.applied.files.len(), 3, "the files it would write");
    assert_eq!(all_files(dir.path()), before);
}
