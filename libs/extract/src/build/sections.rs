//! Reading the guide's sections: which hold progression tables, the headings a table can name, the class's tables,
//! and a section's text for the prose rules.

use std::collections::BTreeSet;

use rules::{FeatureSelection, FeatureSelectionKey};

use crate::guide::Guide;
use crate::html::{Block, Page, Section};
use crate::names::{norm, number_forms};

use super::level_tables::is_level_table;
use super::{FeatureOwner, Heading, Scope};

pub(super) fn has_progression_table(section: &Section) -> bool {
    section.tables().any(|rows| {
        is_level_table(rows)
            && rows[0]
                .iter()
                .any(|cell| cell.starts_with("Special") || cell == "Feat" || cell == "Feats")
    })
}

/// The tables of the base class `class`, as [`Builder::base_class`](super::Builder::base_class) reads them: the class's own, then each
/// archetype's (a section one level down with a progression table), by key.
pub(crate) fn class_tables(guide: &Guide, class: &str) -> Vec<String> {
    let root = guide
        .page(class)
        .root
        .children
        .first()
        .expect("a class page has an h2");
    std::iter::once(class.to_owned())
        .chain(
            root.children
                .iter()
                .filter(|s| s.level == 3 && has_progression_table(s))
                .map(|s| format!("{class}/{}", s.anchor)),
        )
        .collect()
}

/// The headings in `section` that a table can name: its own and its descendants', except archetypes (`h3`)
/// below a class's `h2`.
pub(super) fn headings<'g>(
    page: &'g Page,
    section: &'g Section,
    prefix: &str,
    owner: &FeatureOwner,
) -> Vec<Heading<'g>> {
    let mut out = Vec::new();
    collect_headings(page, section, prefix, owner, &mut out, true);
    out
}

fn collect_headings<'g>(
    page: &'g Page,
    section: &'g Section,
    prefix: &str,
    owner: &FeatureOwner,
    out: &mut Vec<Heading<'g>>,
    top: bool,
) {
    if !top && !section.anchor.is_empty() {
        out.push(Heading {
            section,
            page,
            prefix: prefix.to_owned(),
            owner: owner.clone(),
        });
    }
    for child in &section.children {
        if section.level == 2 && child.level == 3 {
            continue;
        }
        collect_headings(page, child, prefix, owner, out, false);
    }
}

pub(super) fn selection_names(
    selections: &[FeatureSelection],
) -> Vec<(Vec<String>, FeatureSelectionKey)> {
    selections
        .iter()
        .map(|selection| (number_forms(&norm(&selection.name)), selection.key.clone()))
        .collect()
}

/// The headings without the options of `selections`: a cell naming an option isn't granting it.
pub(super) fn without_options<'g>(
    headings: Vec<Heading<'g>>,
    selections: &[FeatureSelection],
) -> Vec<Heading<'g>> {
    let options: BTreeSet<String> = selections
        .iter()
        .flat_map(|selection| &selection.options)
        .map(|option| option.guide.0.clone())
        .collect();
    headings
        .into_iter()
        .filter(|heading| !options.contains(&heading.page.guide(&heading.section.anchor)))
        .collect()
}

/// The heading a cell names: by name or anchor, in its plural or singular form, disambiguated with the archetype,
/// as a summon, or without a trailing bonus ("Arcane Armor +2") or detail ("Fast Movement: +10'").
pub(super) fn find_heading<'a, 'g>(scope: &'a Scope<'g>, name: &str) -> Option<&'a Heading<'g>> {
    let mut forms = Vec::new();
    let mut bases = vec![name.to_owned()];
    let trimmed = name
        .split(": ")
        .next()
        .unwrap_or(name)
        .trim_end_matches(|c: char| c.is_ascii_digit() || c == '+' || c == ' ')
        .to_owned();
    if trimmed != name && !trimmed.is_empty() {
        bases.push(trimmed);
    }
    for base in &bases {
        let base_slug = norm(base);
        if let Some(anchor) = &scope.archetype_anchor {
            forms.push(format!("{base_slug}-{anchor}"));
        }
        forms.extend(number_forms(&base_slug));
        forms.push(format!("summon-{base_slug}"));
    }
    for form in &forms {
        if let Some(heading) = scope
            .headings
            .iter()
            .find(|heading| heading.section.anchor == *form || norm(&heading.section.name) == *form)
        {
            return Some(heading);
        }
    }
    None
}

/// Every block of a section of a given kind, for the prose rules.
pub fn section_text(section: &Section) -> String {
    let mut text = section.text();
    for block in &section.blocks {
        if let Block::Table(rows) = block {
            for row in rows {
                text.push(' ');
                text.push_str(&row.join(" "));
            }
        }
    }
    text
}
