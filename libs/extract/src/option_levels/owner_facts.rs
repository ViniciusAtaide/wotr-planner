//! The classes that pick a sorcerer bloodline or a wizard's element, given where they're built: Dragon Disciple,
//! Eldritch Scion, Dragonheir Scion, Elemental Specialist and Disruptive's line.

use std::collections::BTreeSet;

use rules::{
    AbilityBonus, ClassKey, DataAt, FeatSlot, FeatureKey, FeatureSelection, FeatureSelectionKey,
    GuideRef, Prerequisite, SelectionOptionKey, SpellType,
};

use crate::names::slug;
use crate::picks::{FeatureFact, Pick, Picks, Rule};

use super::Family;
use super::rows::{option, selection};

/// The classes that pick a sorcerer bloodline or a wizard's element, given where they're built.
///
/// - Dragon Disciple: "Must have a Draconic bloodline or no known bloodline", "Able to cast 1st level arcane spells
///   spontaneously"; "If the precursor class is other than Sorcerer, you must select a Draconic bloodline. Otherwise,
///   the Dragon Disciple continues your bloodline progression uninterrupted. If you selected a bloodline when you
///   became a Dragon Disciple, your bloodline level is your class level." Its 2/5/8 "Bloodline Feat" is the
///   bloodline's; Strength, Constitution and Intelligence +2 are ability bonuses.
/// - Eldritch Scion: "Spells are cast spontaneously", "Must select a bloodline which provides additional spells and
///   abilities", "but not the feats that Sorcerers would receive"; Dragon Disciple levels "do not hinder bloodline
///   progression" (magus).
/// - Dragonheir Scion: "Gains a Draconic Bloodline" with its own rows (Draconic Defences, Strike, Presence, Wings,
///   Might of Wyrms), so it skips every Draconic row.
/// - Elemental Specialist: "At 1st level, an Elemental Specialist selects one element (from Acid, Cold, Electricity
///   and Fire)": the choice is a selection.
/// - Disruptive: "Fighter Level 6 or Arcane bloodline".
pub(super) fn provide_others(sorcerer: &Family, picks: &mut Picks) {
    let draconic: BTreeSet<SelectionOptionKey> = sorcerer
        .option_keys()
        .into_iter()
        .filter(|key| key.as_str().starts_with("sorcerer/bloodline/draconic-"))
        .collect();
    let every: BTreeSet<SelectionOptionKey> = sorcerer.option_keys().into_iter().collect();
    let sorcerer_bloodline = FeatureSelectionKey::new("sorcerer/bloodline");

    // Dragon Disciple.
    let disciple = "dragon-disciple";
    let blood = "dragon-disciple/bloodline";
    picks.selection(
        disciple,
        FeatureSelection {
            name: "Blood of Dragons".into(),
            guide: GuideRef::new("prestige-classes#blood-of-dragons"),
            shared_options: draconic.clone(),
            only_without_option: true,
            ..selection(blood, "", String::new(), &[disciple])
        },
    );
    picks.row_selection(disciple, 1, blood);
    picks.rule(Rule {
        feature: FeatureKey::new("dragon-disciple/bloodline-feat"),
        table: Some(disciple.into()),
        cell: None,
        levels: Some(vec![2, 5, 8]),
        pick: Pick::Slots(vec![FeatSlot::OptionList {
            selection: sorcerer_bloodline.clone(),
            at_level: None,
            label: None,
        }]),
    });
    for (feature, ability) in [
        ("dragon-disciple/strength-2", rules::Ability::Strength),
        (
            "dragon-disciple/constitution-2",
            rules::Ability::Constitution,
        ),
        (
            "dragon-disciple/intelligence-2",
            rules::Ability::Intelligence,
        ),
    ] {
        picks.feature_fact(
            feature,
            FeatureFact::AbilityBonus(AbilityBonus::Fixed { ability, amount: 2 }),
        );
    }
    let mut either: Vec<Prerequisite> = draconic
        .iter()
        .cloned()
        .map(Prerequisite::SelectionOption)
        .collect();
    either.push(Prerequisite::All(vec![
        Prerequisite::NoOptionFrom(sorcerer_bloodline.clone()),
        Prerequisite::NoOptionFrom(FeatureSelectionKey::new("bloodrager/bloodline")),
    ]));
    let at = DataAt::Class(ClassKey::new(disciple));
    picks.stated(
        at.clone(),
        Prerequisite::SpontaneousSpellLevel {
            spell_type: SpellType::Arcane,
            level: 1,
        },
    );
    picks.stated(at, Prerequisite::Any(either));

    // Eldritch Scion.
    let scion = "magus/eldritch-scion";
    let scion_bloodline = "magus/eldritch-scion/bloodline";
    picks.selection(
        scion,
        FeatureSelection {
            name: "Bloodline (Eldritch Scion)".into(),
            guide: GuideRef::new("magus#eldritch-scion"),
            shared_options: every,
            ..selection(scion_bloodline, "", String::new(), &["magus", disciple])
        },
    );
    picks.row_selection(scion, 1, scion_bloodline);

    // Dragonheir Scion.
    let heir = "fighter/dragonheir-scion";
    let heir_bloodline = FeatureSelectionKey::new("fighter/dragonheir-scion/bloodline");
    let rows: BTreeSet<u8> = draconic
        .iter()
        .flat_map(|key| {
            sorcerer
                .option(key.as_str())
                .levels
                .iter()
                .map(|row| row.level)
        })
        .collect();
    picks.selection(
        heir,
        FeatureSelection {
            name: "Draconic Bloodline".into(),
            guide: GuideRef::new("fighter#draconic-bloodline"),
            shared_options: draconic,
            skipped_levels: rows,
            ..selection(heir_bloodline.as_str(), "", String::new(), &["fighter"])
        },
    );
    picks.rule(Rule {
        feature: FeatureKey::new("fighter/dragonheir-scion/draconic-bloodline"),
        table: Some(heir.into()),
        cell: None,
        levels: Some(vec![1]),
        pick: Pick::Selection(heir_bloodline),
    });

    // Elemental Specialist.
    let focused = "wizard/elemental-specialist/focused-element";
    let mut choice = selection(
        focused,
        "Focused Element",
        "wizard#focused-element".into(),
        &[],
    );
    for element in ["Acid", "Cold", "Electricity", "Fire"] {
        choice.options.push(option(
            &format!("{focused}/{}", slug(element)),
            element,
            "wizard#focused-element".into(),
            Vec::new(),
        ));
    }
    picks.selection("wizard/elemental-specialist", choice);
    picks.unnamed(focused);
    picks.rule(Rule {
        feature: FeatureKey::new(focused),
        table: Some("wizard/elemental-specialist".into()),
        cell: None,
        levels: Some(vec![1]),
        pick: Pick::Selection(FeatureSelectionKey::new(focused)),
    });

    // Disruptive: the "or" line, whose "Arcane bloodline" names no heading.
    picks.prerequisite(
        DataAt::Feat(rules::FeatKey::new("disruptive")),
        "Fighter Level 6 or Arcane bloodline",
        Prerequisite::Any(vec![
            Prerequisite::ClassLevel {
                class: ClassKey::new("fighter"),
                level: 6,
            },
            Prerequisite::SelectionOption(SelectionOptionKey::new("sorcerer/bloodline/arcane")),
            Prerequisite::SelectionOption(SelectionOptionKey::new("bloodrager/bloodline/arcane")),
        ]),
    );
}
