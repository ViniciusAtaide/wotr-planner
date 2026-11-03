//! The feats by category, the Trickster's bonus feats, and the queuing of an entry's `Prerequisites:` lines.

use std::collections::BTreeSet;

use rules::{DataAt, Feat, FeatCategory, FeatKey, GuideRef, Prerequisite, Repeat};

use crate::grammar;
use crate::html::{Page, Section};
use crate::names::norm;
use crate::prose;
use crate::report::Kind;
use crate::resolve::Context;

use super::{Builder, Pending};

/// The feats page's category headings.
const FEAT_CATEGORIES: [(&str, FeatCategory); 12] = [
    ("weapon-feats", FeatCategory::Weapon),
    ("ranged-feats", FeatCategory::Ranged),
    ("unarmed-combat-feats", FeatCategory::UnarmedCombat),
    ("armour-and-shield-feats", FeatCategory::ArmourAndShield),
    ("magic-feats", FeatCategory::Magic),
    ("miscellaneous-feats", FeatCategory::Miscellaneous),
    ("mounted-combat-feats", FeatCategory::MountedCombat),
    ("combat-manoeuvres", FeatCategory::CombatManoeuvres),
    ("teamwork-feats", FeatCategory::Teamwork),
    ("skill-feats", FeatCategory::Skill),
    ("class-feats", FeatCategory::Class),
    ("racial-feats", FeatCategory::Racial),
];

impl<'g> Builder<'g> {
    pub(super) fn feats(&mut self) -> Vec<Feat> {
        let page = self.guide.page("feats");
        let mut feats = Vec::new();
        for section in page.root.walk() {
            let Some((_, category)) = FEAT_CATEGORIES
                .iter()
                .find(|(anchor, _)| *anchor == section.anchor)
            else {
                continue;
            };
            for entry in section.children.iter().filter(|s| s.level == 4) {
                feats.push(self.feat(page, entry, *category, Vec::new()));
            }
        }
        feats
    }

    fn feat(
        &mut self,
        page: &Page,
        entry: &Section,
        category: FeatCategory,
        extra: Vec<Prerequisite>,
    ) -> Feat {
        let key = FeatKey::new(entry.anchor.clone());
        self.catalog.feats.insert(norm(&entry.name), key.clone());
        self.feat_names.push((entry.name.clone(), key.clone()));
        self.prerequisites_of(
            page,
            entry,
            &DataAt::Feat(key.clone()),
            Context::default(),
            extra,
        );
        Feat {
            key,
            name: entry.name.clone(),
            category,
            summary: String::new(),
            guide: GuideRef::new(page.guide(&entry.anchor)),
            parameter: None,
            repeat: Repeat::Once,
            extra_pick_of: BTreeSet::new(),
            prerequisite: None,
        }
    }

    /// The Trickster's bonus feats: open to a Trickster from Perception II.
    pub(super) fn trickster_feats(&mut self) -> Vec<Feat> {
        let page = self.guide.page("trickster");
        let Some(section) = page.root.find("trickster-bonus-feats") else {
            return Vec::new();
        };
        let gate = prose::trickster_feat_gate();
        let mut feats = Vec::new();
        for entry in section.children.iter().filter(|s| s.level == 4) {
            let category = prose::trickster_feat_category(&entry.name);
            feats.push(self.feat(page, entry, category, vec![gate.clone()]));
        }
        feats
    }

    /// Queues the prerequisites in `entry`'s paragraphs (and kineticist-style `Prerequisites` table rows).
    pub(super) fn prerequisites_of(
        &mut self,
        page: &Page,
        entry: &Section,
        owner: &DataAt,
        context: Context,
        extra: Vec<Prerequisite>,
    ) {
        let mut lines = Vec::new();
        for paragraph in entry.paragraphs() {
            if let Some(found) = grammar::prerequisite_lines(paragraph) {
                lines.extend(found);
                break;
            }
        }
        for rows in entry.tables() {
            for row in rows {
                if row.first().map(String::as_str) == Some("Prerequisites")
                    && let Some(line) = row.get(1)
                {
                    lines.push(line.clone());
                }
            }
        }
        lines.extend(self.picks.lines_for(owner));
        if lines.is_empty() && extra.is_empty() {
            return;
        }
        self.pending.push(Pending {
            owner: owner.clone(),
            at: page.guide(&entry.anchor),
            conds: lines
                .into_iter()
                .map(|line| {
                    let cond = grammar::parse_line(&line);
                    (line, cond)
                })
                .collect(),
            context,
            kind: Kind::Prerequisite,
            extra,
        });
    }
}
