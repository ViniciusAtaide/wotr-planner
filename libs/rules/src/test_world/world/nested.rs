//! The test world's entries written nested, each owner holding what it grants, as the data was before ticket 244 laid
//! it out flat: the invented game reads best that way. [`flatten`] turns them into the flat catalogs of
//! [`GameData`], each subrace naming its race and each archetype its class.
//!
//! Each type has the fields of the flat one of its name, plus the lists it holds.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Ability, AbilityModifier, Advancement, ArchetypeKey, ArchetypeLevelRow, BabProgression, Band,
    BlueprintGuid, ClassKey, ClassLevelRow, CountsAs, DeityDomains, FeatList, FeatSlot, Feature,
    FeatureKey, FeatureSelectionKey, GameData, GuideRef, LevelFrom, MythicPathKey, MythicRankRow,
    PathSwitch, Prerequisite, RaceKey, SaveException, SaveProgression, SelectionOption,
    SelectionOptionKey, SkillKey, SubraceKey, WeaponKey,
};

pub struct Race {
    pub key: RaceKey,
    pub blueprint: Option<BlueprintGuid>,
    pub name: String,
    pub summary: String,
    pub guide: Option<GuideRef>,
    pub ability_modifiers: Vec<AbilityModifier>,
    pub feat_slots: Vec<FeatSlot>,
    pub feat_lists: Vec<FeatList>,
    pub skill_points: u8,
    pub weapons: BTreeSet<WeaponKey>,
    pub weapons_as_martial: BTreeSet<WeaponKey>,
    pub subraces: Vec<Subrace>,
}

pub struct Subrace {
    pub key: SubraceKey,
    pub blueprint: Option<BlueprintGuid>,
    pub name: String,
    pub summary: String,
    pub guide: Option<GuideRef>,
    pub ability_modifiers: Vec<AbilityModifier>,
    pub feat_slots: Vec<FeatSlot>,
    pub skill_points: u8,
    pub weapons: BTreeSet<WeaponKey>,
    pub weapons_as_martial: BTreeSet<WeaponKey>,
}

pub struct Class {
    pub key: ClassKey,
    pub blueprint: Option<BlueprintGuid>,
    pub name: String,
    pub summary: String,
    pub guide: Option<GuideRef>,
    pub skill_points: u8,
    pub bab: BabProgression,
    pub fort: SaveProgression,
    pub reflex: SaveProgression,
    pub will: SaveProgression,
    pub save_exceptions: Vec<SaveException>,
    pub class_skills: BTreeSet<SkillKey>,
    pub prerequisite: Option<Prerequisite>,
    pub spontaneous: bool,
    pub casting_ability: Option<Ability>,
    pub advances: Advancement,
    pub counts_as: BTreeSet<CountsAs>,
    pub features: Vec<Feature>,
    pub selections: Vec<FeatureSelection>,
    pub feat_lists: Vec<FeatList>,
    pub levels: Vec<ClassLevelRow>,
    pub archetypes: Vec<Archetype>,
}

pub struct Archetype {
    pub key: ArchetypeKey,
    pub blueprint: Option<BlueprintGuid>,
    pub name: String,
    pub summary: String,
    pub guide: Option<GuideRef>,
    pub prerequisite: Option<Prerequisite>,
    pub skill_points: Option<u8>,
    pub class_skills: Option<BTreeSet<SkillKey>>,
    pub spontaneous: Option<bool>,
    pub casting_ability: Option<Ability>,
    pub fort: Option<SaveProgression>,
    pub reflex: Option<SaveProgression>,
    pub will: Option<SaveProgression>,
    pub features: Vec<Feature>,
    pub selections: Vec<FeatureSelection>,
    pub feat_lists: Vec<FeatList>,
    pub levels: Vec<ArchetypeLevelRow>,
}

pub struct FeatureSelection {
    pub key: FeatureSelectionKey,
    pub blueprint: Option<BlueprintGuid>,
    pub name: String,
    pub summary: String,
    pub guide: Option<GuideRef>,
    pub options: Vec<SelectionOption>,
    pub shared_options: BTreeSet<SelectionOptionKey>,
    pub option_levels: BTreeSet<ClassKey>,
    pub option_level_archetypes: BTreeMap<ArchetypeKey, i8>,
    pub half_other_classes: bool,
    pub skipped_levels: BTreeSet<u8>,
    pub replaced_by: BTreeMap<u8, FeatureKey>,
    pub feat_lists_only: bool,
    pub only_without_option: bool,
    pub filtered_by: Option<FeatureSelectionKey>,
    pub later_rows_behind: u8,
    pub level_from: Option<LevelFrom>,
    pub level_also_from: Option<SelectionOptionKey>,
    pub once_per_option: bool,
    pub deity_domains: Option<DeityDomains>,
    pub band: Band,
}

pub struct MythicPath {
    pub key: MythicPathKey,
    pub blueprint: Option<BlueprintGuid>,
    pub name: String,
    pub summary: String,
    pub guide: Option<GuideRef>,
    pub features: Vec<Feature>,
    pub selections: Vec<FeatureSelection>,
    pub feat_lists: Vec<FeatList>,
    pub ranks: Vec<MythicRankRow>,
    pub switch: Option<PathSwitch>,
}

/// The nested parts of the test world, to flatten into a [`GameData`] that has the flat ones.
pub struct Nested {
    pub races: Vec<Race>,
    pub classes: Vec<Class>,
    pub prestige_classes: Vec<Class>,
    pub mythic_paths: Vec<MythicPath>,
}

/// Moves every nested entry into `data`'s flat catalogs.
pub fn flatten(nested: Nested, data: &mut GameData) {
    for race in nested.races {
        data.feat_lists.extend(race.feat_lists);
        for subrace in race.subraces {
            data.subraces.push(crate::Subrace {
                components: Vec::new(),
                key: subrace.key,
                race: race.key.clone(),
                blueprint: subrace.blueprint,
                name: subrace.name,
                summary: subrace.summary,
                guide: subrace.guide,
                ability_modifiers: subrace.ability_modifiers,
                feat_slots: subrace.feat_slots,
                skill_points: subrace.skill_points,
                weapons: subrace.weapons,
                weapons_as_martial: subrace.weapons_as_martial,
            });
        }
        data.races.push(crate::Race {
            components: Vec::new(),
            key: race.key,
            blueprint: race.blueprint,
            name: race.name,
            summary: race.summary,
            guide: race.guide,
            ability_modifiers: race.ability_modifiers,
            feat_slots: race.feat_slots,
            skill_points: race.skill_points,
            weapons: race.weapons,
            weapons_as_martial: race.weapons_as_martial,
        });
    }
    for (classes, prestige) in [(nested.classes, false), (nested.prestige_classes, true)] {
        for class in classes {
            let flat = class_into(class, data);
            if prestige {
                data.prestige_classes.push(flat);
            } else {
                data.classes.push(flat);
            }
        }
    }
    for path in nested.mythic_paths {
        owned_into(path.features, path.selections, path.feat_lists, data);
        data.mythic_paths.push(crate::MythicPath {
            key: path.key,
            blueprint: path.blueprint,
            name: path.name,
            summary: path.summary,
            guide: path.guide,
            ranks: path.ranks,
            switch: path.switch,
        });
    }
}

fn class_into(class: Class, data: &mut GameData) -> crate::Class {
    owned_into(class.features, class.selections, class.feat_lists, data);
    for archetype in class.archetypes {
        owned_into(
            archetype.features,
            archetype.selections,
            archetype.feat_lists,
            data,
        );
        data.archetypes.push(crate::Archetype {
            components: Vec::new(),
            key: archetype.key,
            class: class.key.clone(),
            blueprint: archetype.blueprint,
            name: archetype.name,
            summary: archetype.summary,
            guide: archetype.guide,
            prerequisite: archetype.prerequisite,
            skill_points: archetype.skill_points,
            class_skills: archetype.class_skills,
            spontaneous: archetype.spontaneous,
            casting_ability: archetype.casting_ability,
            fort: archetype.fort,
            reflex: archetype.reflex,
            will: archetype.will,
            levels: archetype.levels,
        });
    }
    crate::Class {
        components: Vec::new(),
        key: class.key,
        blueprint: class.blueprint,
        name: class.name,
        summary: class.summary,
        guide: class.guide,
        skill_points: class.skill_points,
        bab: class.bab,
        fort: class.fort,
        reflex: class.reflex,
        will: class.will,
        save_exceptions: class.save_exceptions,
        class_skills: class.class_skills,
        prerequisite: class.prerequisite,
        spontaneous: class.spontaneous,
        casting_ability: class.casting_ability,
        advances: class.advances,
        counts_as: class.counts_as,
        levels: class.levels,
    }
}

fn owned_into(
    features: Vec<Feature>,
    selections: Vec<FeatureSelection>,
    feat_lists: Vec<FeatList>,
    data: &mut GameData,
) {
    data.features.extend(features);
    data.feat_lists.extend(feat_lists);
    selections_into(selections, data);
}

fn selections_into(selections: Vec<FeatureSelection>, data: &mut GameData) {
    for selection in selections {
        data.selections.push(crate::FeatureSelection {
            components: Vec::new(),
            key: selection.key,
            blueprint: selection.blueprint,
            name: selection.name,
            summary: selection.summary,
            guide: selection.guide,
            options: selection.options.iter().map(|o| o.key.clone()).collect(),
            shared_options: selection.shared_options,
            option_levels: selection.option_levels,
            option_level_archetypes: selection.option_level_archetypes,
            half_other_classes: selection.half_other_classes,
            skipped_levels: selection.skipped_levels,
            replaced_by: selection.replaced_by,
            feat_lists_only: selection.feat_lists_only,
            only_without_option: selection.only_without_option,
            filtered_by: selection.filtered_by,
            later_rows_behind: selection.later_rows_behind,
            level_from: selection.level_from,
            level_also_from: selection.level_also_from,
            once_per_option: selection.once_per_option,
            deity_domains: selection.deity_domains,
            band: selection.band,
        });
        data.options.extend(selection.options);
    }
}
