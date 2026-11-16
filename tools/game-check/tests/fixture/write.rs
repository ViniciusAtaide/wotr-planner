//! Applying changes (ticket 293b): the write fixture's tree reads as game data, a change writes only its field, a
//! planner file is refused with nothing written, and a failed check puts every file back.

use std::path::{Path, PathBuf};

use game_check::write::facts::Facts;
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
    run(
        &dump,
        dir,
        &[WriteArea::Items],
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
    let log = game_check::write::log::tsv(&outcome.changes, &outcome.facts, &before, &after);
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
    run(&dump, dir, &[WriteArea::Items], false, &facts, |_| Ok(()))
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
    run(&dump, dir, &[WriteArea::ClassTables], false, &facts, |_| {
        Ok(())
    })
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
