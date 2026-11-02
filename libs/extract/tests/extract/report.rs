//! The extraction report: what couldn't be turned into data, grouped by the category a later fix pass takes.

use extract::report::{Kind, Report};
use rules::{DataAt, FeatKey, MythicPathKey, SelectionOptionKey, SkillKey};

fn section<'a>(markdown: &'a str, heading: &str) -> &'a str {
    let start = markdown
        .find(&format!("\n## {heading}"))
        .unwrap_or_else(|| panic!("no {heading:?} section in:\n{markdown}"));
    let rest = &markdown[start + 1..];
    let end = rest[3..].find("\n## ").map_or(rest.len(), |at| at + 3);
    &rest[..end]
}

#[test]
fn items_are_grouped_by_the_category_of_their_page() {
    let mut report = Report::default();
    report.add(
        Kind::Prerequisite,
        "feats#armor-focus",
        "Armor Proficiency (Odd)",
        "unknown name",
    );
    report.add(
        Kind::BonusSlot,
        "sorcerer#crossblooded",
        "Bonus Feat",
        "no list",
    );
    report.add(
        Kind::BonusSlot,
        "character-creation#bloodlines",
        "Bloodline Feat",
        "one list",
    );
    report.add(
        Kind::Requirement,
        "prestige-classes#loremaster",
        "Required feats: X",
        "unknown name",
    );
    report.add(Kind::ProseRule, "gold-dragon", "", "keeps whole ranks");
    report.add(
        Kind::Prerequisite,
        "mythic-paths#some-ability",
        "Something",
        "unknown name",
    );
    report.add(
        Kind::Cell,
        "character-creation#elf",
        "Ability Modifiers",
        "not understood",
    );

    let markdown = report.markdown("2026-10-31");

    assert!(section(&markdown, "Feats").contains("feats#armor-focus"));
    let classes = section(&markdown, "Base classes");
    assert!(classes.contains("sorcerer#crossblooded"));
    assert!(
        classes.contains("character-creation#bloodlines"),
        "bloodlines are class content"
    );
    assert!(section(&markdown, "Prestige classes").contains("prestige-classes#loremaster"));
    let mythic = section(&markdown, "Mythic");
    assert!(mythic.contains("`gold-dragon`"));
    assert!(mythic.contains("mythic-paths#some-ability"));
    let races = section(&markdown, "Races and skills");
    assert!(races.contains("character-creation#elf"));
    assert!(!races.contains("bloodlines"));
}

/// Keys carry no type prefix, so an item about a data entity (a data-check problem, prerequisites for something
/// missing) names the entity's kind, and the kind picks the section.
#[test]
fn an_item_about_a_data_entity_is_filed_by_its_kind() {
    let mut report = Report::default();
    let dodge = DataAt::Feat(FeatKey::new("dodge"));
    let legend = DataAt::MythicPath(MythicPathKey::new("legend"));
    let mobility = DataAt::Skill(SkillKey::new("mobility"));
    let talent = DataAt::SelectionOption(SelectionOptionKey::new("rogue/rogue-talents/x"));
    report.add_about(Kind::Cell, &dodge, "", "fails the data check");
    report.add_about(Kind::Cell, &legend, "", "fails the data check");
    report.add_about(Kind::Cell, &mobility, "", "fails the data check");
    report.add_about(Kind::Prerequisite, &talent, "", "for something missing");

    let markdown = report.markdown("2026-10-31");

    assert!(section(&markdown, "Feats").contains("- `feat dodge`:"));
    assert!(section(&markdown, "Mythic").contains("- `mythic path legend`:"));
    assert!(section(&markdown, "Races and skills").contains("- `skill mobility`:"));
    assert!(
        section(&markdown, "Base classes").contains("- `selection option rogue/rogue-talents/x`:")
    );
}

#[test]
fn every_category_heading_is_present_even_when_empty() {
    let markdown = Report::default().markdown("2026-10-31");

    for heading in [
        "Feats",
        "Base classes",
        "Prestige classes",
        "Mythic",
        "Races and skills",
    ] {
        assert!(
            section(&markdown, heading).contains("(0)"),
            "{heading}:\n{markdown}"
        );
    }
}

#[test]
fn a_repeated_item_is_listed_and_counted_once() {
    let mut report = Report::default();
    for _ in 0..3 {
        report.add(
            Kind::BonusSlot,
            "sorcerer#crossblooded",
            "Bonus Feat",
            "no list",
        );
    }

    assert_eq!(report.len(), 1);
    let markdown = report.markdown("2026-10-31");
    assert_eq!(markdown.matches("sorcerer#crossblooded").count(), 1);
    assert!(section(&markdown, "Base classes").contains("(1)"));
}
