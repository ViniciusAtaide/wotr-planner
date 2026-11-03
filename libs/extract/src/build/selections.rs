//! Feature selections from a heading and its option headings, and the feat an option grants.

use std::collections::BTreeSet;

use rules::{
    ArchetypeKey, ClassKey, DataAt, FeatKey, FeatureSelection, FeatureSelectionKey, GuideRef,
    Prerequisite, SelectionOption, SelectionOptionKey,
};

use crate::html::{Page, Section};
use crate::names::{norm, number_forms};
use crate::prose;
use crate::resolve::Context;

use super::Builder;

impl<'g> Builder<'g> {
    /// The feature selections of a class or archetype section.
    pub(super) fn selections(
        &mut self,
        page: &Page,
        section: &Section,
        prefix: &str,
        granted: &BTreeSet<String>,
        class: &ClassKey,
        _archetype: Option<&ArchetypeKey>,
    ) -> Vec<FeatureSelection> {
        let mut out = Vec::new();
        for candidate in section
            .children
            .iter()
            .filter(|s| s.level == 4 && !s.children.is_empty())
        {
            let forms = number_forms(&norm(&candidate.name));
            let named = forms.iter().any(|form| granted.contains(form));
            if prose::is_merged_list(prefix, &candidate.anchor) {
                continue;
            }
            let key = FeatureSelectionKey::new(format!("{prefix}/{}", candidate.anchor));
            if !named || mostly_granted(candidate, granted) || !self.picks.is_selection(&key) {
                continue;
            }
            out.push(self.selection(page, candidate, prefix, class, Vec::new()));
        }
        out
    }

    /// A Feature selection from a heading and its option headings.
    pub fn selection(
        &mut self,
        page: &Page,
        heading: &Section,
        prefix: &str,
        class: &ClassKey,
        extra: Vec<Prerequisite>,
    ) -> FeatureSelection {
        let key = FeatureSelectionKey::new(format!("{prefix}/{}", heading.anchor));
        let mut selection = FeatureSelection {
            key: key.clone(),
            name: heading.name.clone(),
            summary: String::new(),
            guide: GuideRef::new(page.guide(&heading.anchor)),
            options: Vec::new(),
            shared_options: BTreeSet::new(),
            option_levels: BTreeSet::new(),
            skipped_levels: BTreeSet::new(),
            feat_lists_only: false,
            only_without_option: false,
            pool_of: None,
            levels_behind: 0,
            level_from: None,
            once_per_option: false,
            level_also_from: None,
        };
        self.add_options(page, heading, &mut selection, Some(class), extra);
        if let Some(classes) = self.picks.option_levels_of(&key) {
            selection.option_levels = classes;
        }
        selection
    }

    /// Adds `heading`'s children as options of `selection`, each with `extra` ANDed into its prerequisites.
    pub fn add_options(
        &mut self,
        page: &Page,
        heading: &Section,
        selection: &mut FeatureSelection,
        class: Option<&ClassKey>,
        extra: Vec<Prerequisite>,
    ) {
        let mut all_options = BTreeSet::new();
        for child in &heading.children {
            let key = SelectionOptionKey::new(format!("{}/{}", selection.key, child.anchor));
            self.catalog
                .options
                .entry(norm(&child.name))
                .or_default()
                .insert(key.clone());
            all_options.insert(key.clone());
            let feat = self.option_feat(child);
            let mut extra = extra.clone();
            extra.extend(self.picks.option_conditions_of(&key));
            self.prerequisites_of(
                page,
                child,
                &DataAt::SelectionOption(key.clone()),
                Context {
                    own_class: class.cloned(),
                },
                extra,
            );
            let parameter = self.picks.option_parameter_of(&key);
            let repeat = self.picks.option_repeat_of(&key);
            let levels = self.picks.option_rows_of(&key);
            selection.options.push(SelectionOption {
                key,
                name: child.name.clone(),
                summary: String::new(),
                guide: GuideRef::new(page.guide(&child.anchor)),
                feats: feat.into_iter().collect(),
                chosen_feat: None,
                repeat,
                prerequisite: None,
                levels,
                pool: None,
                replaces_row_of: None,
                group: None,
                arcana: None,
                parameter,
                values: Default::default(),
            });
        }
        // The options a provider gives the selection beside its heading's children, their prerequisites queued.
        let provided: Vec<SelectionOption> =
            self.picks.options_of(&selection.key).cloned().collect();
        for option in provided {
            self.catalog
                .options
                .entry(norm(&option.name))
                .or_default()
                .insert(option.key.clone());
            all_options.insert(option.key.clone());
            self.add_prerequisite(
                &DataAt::SelectionOption(option.key.clone()),
                option.guide.0.clone(),
                option.prerequisite.clone().into_iter().collect(),
            );
            selection.options.push(option);
        }
        self.catalog
            .selections
            .entry(norm(&selection.name))
            .or_default()
            .extend(all_options);
    }

    /// The feat an option grants: "the feat of the same name", or "Gain X as a bonus feat".
    fn option_feat(&self, option: &Section) -> Option<FeatKey> {
        let text = option.text();
        let lower = text.to_ascii_lowercase();
        if lower.contains("feat of the same name") || lower.contains("identical to the feat") {
            return self.feat_named(&option.name);
        }
        let start = lower.find("gain ")? + "gain ".len();
        let end = lower[start..].find(" as a bonus feat")? + start;
        self.feat_named(&text[start..end])
    }
}

/// Whether a table grants most of a heading's children by name: then they're features listed under a heading
/// (Ki Pool's Ki Strikes), not a list to pick from.
pub(super) fn mostly_granted(heading: &Section, granted: &BTreeSet<String>) -> bool {
    let named = heading
        .children
        .iter()
        .filter(|child| granted.contains(&norm(&child.name)))
        .count();
    named * 2 >= heading.children.len()
}
