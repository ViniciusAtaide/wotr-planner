//! What a table cell grants: a feature, a selection, bonus feat slots, feats or a pick; the providers' additions;
//! defining features; and finding feats by name or in prose.

use std::collections::BTreeSet;

use rules::{
    DataAt, FeatCategory, FeatKey, FeatSlot, Feature, FeatureKey, FeatureSelection, GuideRef,
};

use crate::html::Page;
use crate::names::{norm, number_forms, slug};
use crate::picks::Pick;
use crate::prose;
use crate::report::Kind;

use super::sections::find_heading;
use super::{Builder, FeatureOwner, Granted, Scope};

impl<'g> Builder<'g> {
    /// What a table cell grants.
    pub(super) fn grant(
        &mut self,
        page: &Page,
        scope: &Scope<'g>,
        name: &str,
        level: u8,
    ) -> Granted {
        // A cell the class's option family plans: a placeholder, the choice or a fixed option's row's.
        let planned = scope
            .owner
            .as_ref()
            .and_then(|owner| self.plans.get(owner.key()))
            .and_then(|plan| plan.planned(level, name));
        if let Some(planned) = planned {
            if planned == (crate::option_levels::Planned::Marker { kept: true })
                && let (Some((key, owner, name, guide)), _) = self.cell_of(scope, name)
            {
                self.define_feature(&key, owner, &name, guide);
            }
            return Granted::Nothing;
        }
        let cell_norm = norm(name);
        if cell_norm.ends_with("-feat") || cell_norm.ends_with("-feats") {
            // A slot cell a provider says picks otherwise (Dragon Disciple's bloodline feats).
            let key = FeatureKey::new(format!("{}/{}", scope.prefix, slug(name)));
            if let Some(picked) = self.picked(scope, name, &key, level) {
                return picked;
            }
            return Granted::Slots(self.bonus_slots(scope, name, level));
        }
        for form in number_forms(&cell_norm) {
            if let Some((_, key)) = scope
                .selections
                .iter()
                .find(|(names, _)| names.contains(&form))
            {
                let key = key.clone();
                if let Some(picked) =
                    self.picked(scope, name, &FeatureKey::new(key.as_str()), level)
                {
                    return picked;
                }
                return Granted::Selection(key);
            }
        }
        if let Some(heading) = find_heading(scope, name) {
            let key = FeatureKey::new(format!("{}/{}", heading.prefix, heading.section.anchor));
            let defined = |builder: &mut Self| {
                builder.define_feature(
                    &key,
                    heading.owner.clone(),
                    &heading.section.name,
                    heading.page.guide(&heading.section.anchor),
                );
            };
            return match self.picked(scope, name, &key, level) {
                Some(Granted::SelectionKeepingFeature(selection)) => {
                    defined(self);
                    Granted::Selection(selection)
                }
                Some(Granted::FeatureAndFeats(feats)) => {
                    defined(self);
                    Granted::Feats(feats, Some(key))
                }
                Some(picked) => picked,
                None => {
                    defined(self);
                    Granted::Feature(key)
                }
            };
        }
        if !scope.path
            && let Some(feat) = self.feat_named(name)
        {
            // A feat the class grants outright, whatever its prerequisites.
            return Granted::Feat(feat);
        }
        let key = FeatureKey::new(format!("{}/{}", scope.prefix, slug(name)));
        let granted = match self.picked(scope, name, &key, level) {
            Some(Granted::SelectionKeepingFeature(selection)) => Granted::Selection(selection),
            Some(Granted::FeatureAndFeats(feats)) => Granted::Feats(feats, Some(key.clone())),
            Some(picked) => return picked,
            None => Granted::Feature(key.clone()),
        };
        let owner = scope
            .owner
            .clone()
            .expect("a scope with grants has an owner");
        let _ = page;
        self.define_feature(&key, owner, name, scope.guide.clone());
        granted
    }

    /// What a cell naming the feature `key` is when it's a pick ([`Picks`](crate::picks::Picks)), in place of that feature.
    fn picked(
        &mut self,
        scope: &Scope<'g>,
        cell: &str,
        key: &FeatureKey,
        level: u8,
    ) -> Option<Granted> {
        let owner = scope.owner.as_ref()?;
        let pick = self.picks.pick(owner.key(), cell, key, level)?;
        Some(match pick.clone() {
            Pick::Selection(selection) => Granted::Selection(selection),
            Pick::Slots(slots) => Granted::Slots(slots),
            Pick::OptionRow | Pick::Nothing => Granted::Nothing,
            Pick::SelectionKeepingFeature(selection) => Granted::SelectionKeepingFeature(selection),
            Pick::Feats(feats) => Granted::Feats(feats, None),
            Pick::FeatureAndFeats(feats) => Granted::FeatureAndFeats(feats),
            Pick::FeatsFor { feats, feats_for } => Granted::FeatsFor(feats, feats_for),
            Pick::Feature(feature) => {
                let key = feature.key.clone();
                self.features
                    .entry(key.clone())
                    .or_insert_with(|| (owner.clone(), feature));
                Granted::Feature(key)
            }
        })
    }

    /// What the providers give a class or archetype besides its tables' cells: selections (their options'
    /// prerequisites queued, as the guide's lines are), features and feat lists.
    pub(super) fn provided(
        &mut self,
        owner: &FeatureOwner,
    ) -> (Vec<FeatureSelection>, Vec<rules::FeatList>) {
        let key = owner.key().to_owned();
        let mut selections: Vec<FeatureSelection> =
            self.picks.selections_of(&key).cloned().collect();
        // The classes whose levels level a provided selection, where another provider says (ticket 47's Combat Trick
        // rows need them).
        for selection in &mut selections {
            if let Some(classes) = self.picks.option_levels_of(&selection.key) {
                assert!(
                    selection.option_levels.is_empty() || selection.option_levels == classes,
                    "{}: levelled otherwise",
                    selection.key
                );
                selection.option_levels = classes;
            }
        }
        for selection in selections
            .iter()
            .filter(|s| !s.options.is_empty() && self.picks.is_named(&s.key))
        {
            // Naming the selection means having one of its options, as for the selections the tables define.
            self.catalog
                .selections
                .entry(norm(&selection.name))
                .or_default()
                .extend(selection.options.iter().map(|o| o.key.clone()));
        }
        for option in selections.iter().flat_map(|s| &s.options) {
            if let Some(prerequisite) = &option.prerequisite {
                self.add_prerequisite(
                    &DataAt::SelectionOption(option.key.clone()),
                    option.guide.0.clone(),
                    vec![prerequisite.clone()],
                );
            }
        }
        let features: Vec<Feature> = self.picks.features_of(&key).cloned().collect();
        for feature in features {
            let previous = self
                .features
                .insert(feature.key.clone(), (owner.clone(), feature));
            assert!(previous.is_none(), "{key}: a feature twice");
        }
        (
            selections,
            self.picks.feat_lists_of(&key).cloned().collect(),
        )
    }

    pub fn define_feature(
        &mut self,
        key: &FeatureKey,
        owner: FeatureOwner,
        name: &str,
        guide: String,
    ) {
        if self.features.contains_key(key) {
            return;
        }
        let mut feature = Feature {
            key: key.clone(),
            name: name.to_owned(),
            summary: String::new(),
            guide: GuideRef::new(guide),
            ability_bonuses: Vec::new(),
            option_row: None,
            daily_choice: false,
            bab_floor: None,
            skill_ranks_at_least_mythic_rank: false,
        };
        self.picks.shape(&mut feature);
        let mut names = vec![norm(name)];
        // "Sneak Attack (Eldritch Scoundrel)" is also a "Sneak Attack".
        if let Some((base, _)) = name.split_once(" (") {
            names.push(norm(base));
        }
        for form in names {
            self.catalog
                .features
                .entry(form)
                .or_default()
                .insert(key.clone());
        }
        self.features.insert(key.clone(), (owner, feature));
    }

    pub fn feat_named(&self, name: &str) -> Option<FeatKey> {
        let wanted = norm(name);
        self.feat_names
            .iter()
            .find(|(feat, _)| norm(feat) == wanted)
            .map(|(_, key)| key.clone())
    }

    /// The bonus feat slots a "… Feat" cell grants.
    fn bonus_slots(&mut self, scope: &Scope<'g>, label: &str, level: u8) -> Vec<FeatSlot> {
        let label_norm = norm(label);
        match label_norm.as_str() {
            "bonus-combat-feat" | "combat-feat" => return vec![FeatSlot::CombatFeat],
            "teamwork-feat" | "bonus-teamwork-feat" => {
                return vec![FeatSlot::Category(FeatCategory::Teamwork)];
            }
            "mounted-combat-feat" => return vec![FeatSlot::Category(FeatCategory::MountedCombat)],
            _ => {}
        }
        if let Some(slots) = prose::special_bonus_slots(self, scope, label, level) {
            return slots;
        }
        // The list: a heading naming the label or "bonus feat(s)", nearest first.
        let label_forms = number_forms(&label_norm);
        let heading = scope
            .headings
            .iter()
            .find(|heading| label_forms.contains(&norm(&heading.section.name)))
            .or_else(|| {
                scope.headings.iter().find(|heading| {
                    let name = norm(&heading.section.name);
                    label_forms.iter().any(|form| name.contains(form.as_str()))
                })
            });
        let heading = heading.or_else(|| {
            scope.headings.iter().find(|heading| {
                let name = norm(&heading.section.name);
                name.contains("bonus-feat")
            })
        });
        let Some(heading) = heading else {
            self.report.add(
                Kind::BonusSlot,
                scope.guide.clone(),
                label,
                "no list found for this slot; stored as a general slot",
            );
            return vec![FeatSlot::General];
        };
        let at = heading.page.guide(&heading.section.anchor);
        let mut text = heading.section.text();
        for child in &heading.section.children {
            text.push(' ');
            text.push_str(&child.text());
        }
        let lower = text.to_ascii_lowercase();
        if lower.contains("bonus magic feat") {
            return vec![FeatSlot::Category(FeatCategory::Magic)];
        }
        if lower.contains("bonus combat feat") {
            return vec![FeatSlot::CombatFeat];
        }
        if lower.contains("any feat for which") {
            return vec![FeatSlot::General];
        }
        let tiers = self.feats_in(&text);
        if tiers.is_empty() {
            self.report.add(
                Kind::BonusSlot,
                at,
                label,
                "the list names no feats; stored as a general slot",
            );
            return vec![FeatSlot::General];
        }
        let feats: BTreeSet<FeatKey> = tiers
            .into_iter()
            .filter(|(tier, _)| *tier <= level)
            .map(|(_, feat)| feat)
            .collect();
        let ignores = if lower.contains("not need to meet")
            || lower.contains("without having to meet")
            || lower.contains("do not need to meet")
        {
            true
        } else if lower.contains("must meet")
            || lower.contains("must satisfy")
            || lower.contains("meets the prerequisites")
        {
            false
        } else {
            // D1: a class's own fixed bonus list ignores prerequisites unless the guide says otherwise.
            true
        };
        vec![self.lists.slot(feats, ignores)]
    }

    /// Feats named in a prose list, each with the Class level its sentence opens at ("At 6th level, …").
    /// "(Improved) Great Fortitude" names both feats.
    pub fn feats_in(&self, text: &str) -> Vec<(u8, FeatKey)> {
        let mut found = Vec::new();
        let mut tier = 0u8;
        for sentence in text.split(". ") {
            if let Some(level) = sentence_level(sentence) {
                tier = level;
            }
            let expanded = expand_improved(sentence);
            let lower = expanded.to_ascii_lowercase();
            let mut names: Vec<&(String, FeatKey)> = self.feat_names.iter().collect();
            names.sort_by_key(|(name, _)| std::cmp::Reverse(name.len()));
            let mut taken: Vec<(usize, usize)> = Vec::new();
            for (name, key) in names {
                let needle = name.to_ascii_lowercase();
                let mut from = 0;
                while let Some(at) = lower[from..].find(&needle) {
                    let start = from + at;
                    let end = start + needle.len();
                    from = end;
                    let bounded = !lower[..start].ends_with(|c: char| c.is_alphanumeric())
                        && !lower[end..].starts_with(|c: char| c.is_alphanumeric());
                    if !bounded || taken.iter().any(|&(s, e)| start < e && s < end) {
                        continue;
                    }
                    taken.push((start, end));
                    found.push((tier, key.clone()));
                }
            }
        }
        found
    }
}

/// "At 6th level, …" or "From 10th level, …" opens a tier.
fn sentence_level(sentence: &str) -> Option<u8> {
    let lower = sentence.trim().to_ascii_lowercase();
    let rest = lower
        .strip_prefix("at ")
        .or_else(|| lower.strip_prefix("from "))?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let after = &rest[digits.len()..];
    if !(after.starts_with("th level")
        || after.starts_with("st level")
        || after.starts_with("nd level")
        || after.starts_with("rd level"))
    {
        return None;
    }
    digits.parse().ok()
}

/// "(Improved) Great Fortitude" names "Great Fortitude" and "Improved Great Fortitude".
fn expand_improved(sentence: &str) -> String {
    let mut out = String::new();
    let mut rest = sentence;
    while let Some(at) = rest.find("(Improved) ") {
        out.push_str(&rest[..at]);
        rest = &rest[at + "(Improved) ".len()..];
        let end = rest.find([',', '.']).unwrap_or(rest.len());
        let name = &rest[..end];
        out.push_str(&format!("{name}, Improved {name}"));
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}
