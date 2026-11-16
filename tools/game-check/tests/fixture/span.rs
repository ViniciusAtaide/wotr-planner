//! The span writer (ticket 293a): one top-level field of an entry's text is replaced, inserted or removed, and every
//! other byte stays. The texts are shaped like the data's own files (`classes/paladin.ron`,
//! `archetypes/druid/feyspeaker.ron`, `features/oracle/seeker/seekers-trapfinding.ron`, `gear/dirge-for-sarkoris.ron`).

use game_check::write::span::{
    ARCHETYPE_FIELDS, CLASS_FIELDS, GEAR_FIELDS, SpanError, Value, field_span, ron_text,
    write_field,
};

const PALADIN: &str = r#"Class(
    key: "paladin",
    name: "Paladin",
    // The saves are the guide's.
    skill_points: 2,
    bab: Full,
    fort: Good,
    reflex: Poor, // a comment after
    will: Good,
    class_skills: ["knowledge-arcana", "persuasion"],
    levels: [
        (level: 1, features: ["smite-evil"]),
    ],
)
"#;

const FEYSPEAKER: &str = r#"(
    key: "druid/feyspeaker",
    class: "druid",
    name: "Feyspeaker",
    summary: "skill_points: 3",
    skill_points: 4,
    class_skills: ["athletics", "perception"],
    levels: [
        (level: 1, features: ["druid/nature-sense"]),
        (level: 4, features: ["druid/feyspeaker/fey-magic"]),
    ],
)
"#;

const ONE_LINE: &str = r#"(key: "oracle/seeker/seekers-trapfinding", name: "Seeker's Trapfinding", summary: "Trickery.", guide: "oracle#seekers-trapfinding")"#;

const DIRGE: &str = r#"// Weapon.
(
    key: "dirge-for-sarkoris",
    name: "Dirge for Sarkoris",
    slot: Weapon,
    always_on: [
        "A +2 keen club",
    ],
components: [
    GearStat(stat: Ability(Strength), value: 2),
],
)
"#;

fn list(items: &[&str]) -> Value {
    Value::List(items.iter().map(|item| (*item).to_owned()).collect())
}

#[test]
fn replacing_a_field_keeps_every_other_byte() {
    let written = write_field(
        PALADIN,
        "fort",
        Some(&Value::One("Poor".into())),
        CLASS_FIELDS,
    )
    .unwrap();
    assert_eq!(written, PALADIN.replace("fort: Good,", "fort: Poor,"));
    let span = field_span(PALADIN, "reflex").unwrap();
    assert_eq!(&PALADIN[span], "reflex: Poor");
    let written = write_field(
        PALADIN,
        "reflex",
        Some(&Value::One("Good".into())),
        CLASS_FIELDS,
    )
    .unwrap();
    assert_eq!(
        written,
        PALADIN.replace(
            "reflex: Poor, // a comment after",
            "reflex: Good, // a comment after"
        )
    );
    assert!(written.contains("// The saves are the guide's."));
}

#[test]
fn a_value_that_spans_lines_is_written_one_element_a_line() {
    assert_eq!(ron_text(&vec![1u8, 2]), "[1, 2]");
    assert_eq!(ron_text(&Some(3u8)), "3");
    let value = list(&["GearStat(stat: Ability(Dexterity), value: 4)"]);
    let written = write_field(DIRGE, "components", Some(&value), GEAR_FIELDS).unwrap();
    assert_eq!(
        written,
        DIRGE.replace(
            "    GearStat(stat: Ability(Strength), value: 2),",
            "    GearStat(stat: Ability(Dexterity), value: 4),"
        )
    );
    // An indented field's list closes at the field's indent.
    let written = write_field(
        PALADIN,
        "class_skills",
        Some(&list(&["a", "b"])),
        CLASS_FIELDS,
    )
    .unwrap();
    assert!(written.contains("class_skills: [a, b],"), "{written}");
    let written = write_field(
        PALADIN,
        "levels",
        Some(&list(&["(level: 1)", "(level: 2)"])),
        CLASS_FIELDS,
    )
    .unwrap();
    assert!(
        written.contains("levels: [\n        (level: 1),\n        (level: 2),\n    ],\n)"),
        "{written}"
    );
}

#[test]
fn a_one_line_file_stays_one_line() {
    let written = write_field(
        ONE_LINE,
        "guide",
        Some(&Value::One("\"oracle#x\"".into())),
        GEAR_FIELDS,
    )
    .unwrap();
    assert_eq!(
        written,
        ONE_LINE.replace("\"oracle#seekers-trapfinding\"", "\"oracle#x\"")
    );
    let written = write_field(
        ONE_LINE,
        "components",
        Some(&list(&["A", "B"])),
        GEAR_FIELDS,
    )
    .unwrap();
    assert!(!written.contains('\n'), "{written}");
    assert!(written.ends_with(", components: [A, B])"), "{written}");
}

#[test]
fn a_missing_field_goes_after_the_last_field_before_it() {
    let written = write_field(
        FEYSPEAKER,
        "bab",
        Some(&Value::One("Half".into())),
        ARCHETYPE_FIELDS,
    )
    .unwrap();
    let expected = FEYSPEAKER.replace("perception\"],\n", "perception\"],\n    bab: Half,\n");
    assert_eq!(written, expected);
    let bab = written.find("bab:").unwrap();
    assert!(written.find("class_skills:").unwrap() < bab && bab < written.find("levels:").unwrap());
    // A field with nothing before it in the text has no anchor.
    assert!(matches!(
        write_field(
            "(key: \"x\")",
            "bab",
            Some(&Value::One("Half".into())),
            &["bab", "key"]
        ),
        Err(SpanError::NoAnchor { .. })
    ));
}

#[test]
fn removing_a_field_takes_its_line() {
    let written = write_field(FEYSPEAKER, "skill_points", None, ARCHETYPE_FIELDS).unwrap();
    assert_eq!(written, FEYSPEAKER.replace("    skill_points: 4,\n", ""));
    let written = write_field(ONE_LINE, "summary", None, GEAR_FIELDS).unwrap();
    assert_eq!(written, ONE_LINE.replace(" summary: \"Trickery.\",", ""));
    let written = write_field(ONE_LINE, "guide", None, GEAR_FIELDS).unwrap();
    assert!(written.ends_with("summary: \"Trickery.\")"), "{written}");
    // A field that isn't there is left alone.
    let same = write_field(ONE_LINE, "bab", None, ARCHETYPE_FIELDS).unwrap();
    assert_eq!(same, ONE_LINE);
}

#[test]
fn a_comment_inside_the_field_is_refused() {
    let text =
        "(\n    key: \"x\",\n    levels: [\n        // a note\n        (level: 1),\n    ],\n)\n";
    let error =
        write_field(text, "levels", Some(&list(&["(level: 2)"])), CLASS_FIELDS).unwrap_err();
    assert!(
        matches!(error, SpanError::CommentInside { field: "levels" }),
        "{error:?}"
    );
    let error = write_field(text, "levels", None, CLASS_FIELDS).unwrap_err();
    assert!(matches!(error, SpanError::CommentInside { .. }));
    // `//` in a string is not a comment.
    let text = "(key: \"x\", guide: \"a//b\")";
    write_field(
        text,
        "guide",
        Some(&Value::One("\"c\"".into())),
        GEAR_FIELDS,
    )
    .unwrap();
}

#[test]
fn a_name_in_a_string_or_a_nested_struct_is_not_the_field() {
    let span = field_span(FEYSPEAKER, "skill_points").unwrap();
    assert_eq!(&FEYSPEAKER[span.clone()], "skill_points: 4");
    assert!(FEYSPEAKER[..span.start].contains("summary: \"skill_points: 3\""));
    // A row's `features:` is nested, so it is no top-level field.
    assert_eq!(field_span(FEYSPEAKER, "features"), None);
    assert_eq!(
        field_span("(summary: \"skill_points: 3\")", "skill_points"),
        None
    );
    // A longer name that starts with it isn't it.
    assert_eq!(field_span("(fort_x: 1)", "fort"), None);
}
