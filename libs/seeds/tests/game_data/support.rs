//! The helpers the real-data tests share: the data and an engine over it, Builds, slots and picks, and lookups.

use std::collections::{BTreeSet, HashMap};
use std::sync::LazyLock;

use rules::{
    ArchetypeKey, Build, CharacterState, CheckedGameData, ClassChoice, ClassKey, Engine, FeatKey,
    FeatSource, FeatureKey, GameData, Issue, LevelUp, MythicPathKey, Parameter, Pick, Prerequisite,
    SelectionOptionKey, SkillKey, SlotId, SlotKind, SlotOption, Unmet, WeaponKey,
};

pub static DATA: LazyLock<GameData> = LazyLock::new(|| {
    seeds::read_game_data_files(&seeds::game_data_dir()).expect("game-data reads")
});

/// [`DATA`], checked once for every engine the tests build.
static CHECKED: LazyLock<CheckedGameData> =
    LazyLock::new(|| DATA.clone().checked().expect("game-data checks"));

/// Every failing single condition in an `Unmet`, as a flat list.
pub fn conditions(unmet: &Unmet) -> Vec<(&Prerequisite, Option<u8>)> {
    match unmet {
        Unmet::Condition { needs, has } => vec![(needs, *has)],
        Unmet::All(children) | Unmet::Any(children) => {
            children.iter().flat_map(conditions).collect()
        }
        Unmet::AtLeast { failed, .. } => failed.iter().flat_map(conditions).collect(),
        Unmet::MetLater { unmet, .. } => conditions(unmet),
    }
}

pub fn engine() -> Engine {
    Engine::new(CHECKED.clone())
}

/// Why the pick in `slot` can't be taken there, for a Build that only that pick makes impossible (it's possible
/// without it): what the slot's picker says of it, or `None` when the picker doesn't offer it at all.
pub fn why_not(build: &Build, slot: &SlotId) -> Option<Issue> {
    let engine = engine();
    assert!(!engine.is_possible(build), "the Build is possible");
    let mut rest = build.clone();
    let pick = rest.picks.remove(slot).expect("a pick in the slot");
    assert!(engine.is_possible(&rest), "impossible without {slot:?} too");
    let status = engine
        .options(&rest, slot)?
        .into_iter()
        .find(|option| option.pick == pick)?
        .status;
    Some(status.expect_err("the picker offers it"))
}

/// The Build with a faith it's possible with (ticket 86): its own when it is, else the first of atheism and the
/// deities, in key order, it is possible with; unchanged when none is (a Build a test makes impossible on purpose). What the tests written before deities give a
/// cleric, inquisitor, warpriest, Divine Hunter or Deliverer, which needs a deity whose domains fit its picks.
pub fn faithful(build: Build) -> Build {
    let engine = engine();
    if engine.is_possible(&build) {
        return build;
    }
    let faiths = std::iter::once(rules::Faith::Atheism).chain(
        DATA.deities
            .iter()
            .map(|deity| rules::Faith::Deity(deity.key.clone())),
    );
    for faith in faiths {
        let mut candidate = build.clone();
        candidate.start.deity = Some(faith);
        if engine.is_possible(&candidate) {
            return candidate;
        }
    }
    build
}

/// The Build at `alignment`, with a faith that fits it ([`faithful`]).
pub fn realigned(mut build: Build, alignment: rules::Alignment) -> Build {
    build.start.alignment = Some(alignment);
    build.start.deity = None;
    faithful(build)
}

/// Sets the Build's alignment, as an atheist unless it already has a faith: the alignment needs one (ticket 94).
pub fn align(build: &mut Build, alignment: rules::Alignment) {
    build.start.alignment = Some(alignment);
    build.start.deity.get_or_insert(rules::Faith::Atheism);
}

/// The Build without the pick in `slot`.
pub fn without(build: &Build, slot: &SlotId) -> Build {
    let mut build = build.clone();
    build.picks.remove(slot);
    build
}

/// A Build of `levels` levels of one class, with no picks.
pub fn build(class: &str, levels: usize) -> Build {
    let mut build = Build::default();
    build
        .class_choices
        .insert(ClassKey::new(class), ClassChoice::default());
    for _ in 0..levels {
        build.levels.push(LevelUp::new(ClassKey::new(class)));
    }
    faithful(build)
}

/// A Build of one class taken with an archetype, `levels` levels, with no picks.
pub fn archetype_build(class: &str, archetype: &str, levels: usize) -> Build {
    let mut build = build(class, levels);
    build.class_choices.insert(
        ClassKey::new(class),
        ClassChoice {
            archetype: Some(ArchetypeKey::new(archetype)),
            ..ClassChoice::default()
        },
    );
    faithful(build)
}

pub fn selection_slot(level: u8) -> SlotId {
    SlotId::Selection { level, index: 0 }
}

pub fn pick(key: &str) -> Pick {
    Pick::option(SelectionOptionKey::new(key))
}

/// The options of a selection slot, by key, with their status.
pub fn offered(build: &Build, level: u8) -> HashMap<String, Result<(), Issue>> {
    engine()
        .options(build, &selection_slot(level))
        .unwrap_or_else(|| panic!("no selection slot at level {level}"))
        .into_iter()
        .map(|SlotOption { pick, status, .. }| (pick.key().to_owned(), status))
        .collect()
}

/// `levels` fighter levels on `path` from MR 3, with no picks.
pub fn mythic(path: &str, levels: usize) -> Build {
    let mut build = build("fighter", levels);
    build.mythic_path = Some(MythicPathKey::new(path));
    build
}

pub fn path_selection(rank: u8) -> SlotId {
    SlotId::PathSelection { rank, index: 0 }
}

pub fn feat(key: &str) -> &'static rules::Feat {
    DATA.feats
        .iter()
        .find(|feat| feat.key.as_str() == key)
        .unwrap_or_else(|| panic!("no feat {key}"))
}

pub fn weapon(key: &str) -> Parameter {
    Parameter::Weapon(WeaponKey::new(key))
}

pub fn skill(key: &str) -> Parameter {
    Parameter::Skill(SkillKey::new(key))
}

pub fn general_slot(level: u8) -> SlotId {
    SlotId::Feat {
        level,
        source: FeatSource::General,
        index: 0,
    }
}

pub fn class_slot(level: u8) -> SlotId {
    SlotId::Feat {
        level,
        source: FeatSource::Class,
        index: 0,
    }
}

/// The values a feat is held for at the end of a level.
pub fn values_at(build: &Build, level: u8, key: &str) -> BTreeSet<Parameter> {
    engine()
        .evaluate(build)
        .at(level)
        .unwrap()
        .feat_values
        .get(&FeatKey::new(key))
        .map(|values| values.keys().cloned().collect())
        .unwrap_or_default()
}

pub fn class(key: &str) -> &'static rules::Class {
    DATA.class(&ClassKey::new(key))
        .unwrap_or_else(|| panic!("no class {key}"))
}

/// One Level-up per class key, with no picks.
pub fn classes(keys: &[&str]) -> Build {
    let mut build = Build::default();
    for key in keys {
        build.levels.push(LevelUp::new(ClassKey::new(*key)));
    }
    build
}

/// `build` with `option` picked at the selection slot `index` of `level`.
pub fn with_option(mut build: Build, level: u8, index: u8, option: &str) -> Build {
    build
        .picks
        .insert(SlotId::Selection { level, index }, pick(option));
    faithful(build)
}

pub fn at(build: &Build, level: u8) -> CharacterState {
    engine().evaluate(build).at(level).cloned().unwrap()
}

pub fn holds(state: &CharacterState, feature: &str) -> bool {
    state.feature_count(&FeatureKey::new(feature)) > 0
}

/// The feats a feat slot offers, by key.
pub fn feats_offered(build: &Build, slot: &SlotId) -> BTreeSet<String> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("no slot {slot:?}"))
        .into_iter()
        .map(|option| option.pick.key().to_owned())
        .collect()
}

pub const ABYSSAL: &str = "sorcerer/bloodline/abyssal";

pub fn option_slot(level: u8, index: u8) -> SlotId {
    SlotId::OptionSelection { level, index }
}

pub fn with_pick(mut build: Build, slot: SlotId, option: &str) -> Build {
    build.picks.insert(slot, pick(option));
    faithful(build)
}

/// The options a slot offers, by key, each once.
pub fn offered_at(build: &Build, slot: &SlotId) -> BTreeSet<String> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("no slot {slot:?}"))
        .into_iter()
        .map(|option| option.pick.key().to_owned())
        .collect()
}

pub fn has_class_skill(state: &CharacterState, skill: &str) -> bool {
    state.class_skills.contains(&SkillKey::new(skill))
}

/// Holds `skill` as a class skill from `level` on, not before.
pub fn class_skill_from_level(build: &Build, level: u8, skill: &str) {
    assert!(
        !has_class_skill(&at(build, level - 1), skill),
        "{skill} before {level}"
    );
    assert!(
        has_class_skill(&at(build, level), skill),
        "{skill} at {level}"
    );
}

/// Holds `feature` from `level` on, not before.
pub fn from_level(build: &Build, level: u8, feature: &str) {
    assert!(
        !holds(&at(build, level - 1), feature),
        "{feature} before {level}"
    );
    assert!(holds(&at(build, level), feature), "{feature} at {level}");
}

pub fn cleric(domains: [&str; 2], levels: usize) -> Build {
    let build = build("cleric", levels);
    let build = with_option(build, 1, 0, &format!("cleric/domain/{}", domains[0]));
    with_option(build, 1, 1, &format!("cleric/domain/{}", domains[1]))
}

/// A true neutral druid.
pub fn druid(archetype: Option<&str>, levels: usize) -> Build {
    let mut druid = match archetype {
        Some(archetype) => archetype_build("druid", archetype, levels),
        None => build("druid", levels),
    };
    druid.start.alignment = Some(rules::Alignment::TrueNeutral);
    druid.start.deity = None;
    faithful(druid)
}

pub fn oracle(mystery: &str, levels: usize) -> Build {
    with_option(
        build("oracle", levels),
        1,
        0,
        &format!("oracle/mystery/{mystery}"),
    )
}

/// The oracle's level-1 slots: mystery, curse, revelation.
pub fn revelation_slot(level: u8) -> SlotId {
    SlotId::Selection {
        level,
        index: if level == 1 { 2 } else { 0 },
    }
}

pub fn kineticist(element: &str, levels: usize) -> Build {
    with_option(
        build("kineticist", levels),
        1,
        0,
        &format!("kineticist/element/{element}"),
    )
}

pub fn race_slot() -> SlotId {
    SlotId::Feat {
        level: 1,
        source: FeatSource::Race,
        index: 0,
    }
}

pub fn possible(build: &Build) -> bool {
    engine().is_possible(build)
}

pub fn feat_pick(key: &str, value: Parameter) -> Pick {
    Pick::feat_for(FeatKey::new(key), value)
}

pub fn mythic_feat(key: &str) -> &'static rules::MythicFeat {
    DATA.mythic_feats
        .iter()
        .find(|feat| feat.key.as_str() == key)
        .unwrap_or_else(|| panic!("no mythic feat {key}"))
}

pub fn unmet(needs: Prerequisite, has: Option<u8>) -> Option<Issue> {
    Some(Issue::Unmet(Unmet::Condition { needs, has }))
}

pub fn crossblooded(first: &str, second: &str) -> Build {
    let build = archetype_build("sorcerer", "sorcerer/crossblooded", 1);
    let build = with_option(build, 1, 0, &format!("sorcerer/bloodline/{first}"));
    with_option(build, 1, 1, &format!("sorcerer/bloodline/{second}"))
}

pub fn separatist(levels: usize) -> Build {
    let build = archetype_build("cleric", "cleric/separatist", levels);
    let build = with_option(build, 1, 0, "cleric/domain/fire");
    with_option(build, 1, 1, "cleric/domain/luck")
}

/// Every prerequisite in the real data: feats', classes', archetypes', selection options', mythic abilities' and
/// mythic feats'.
pub fn data_prerequisites(data: &GameData) -> Vec<&Prerequisite> {
    let prerequisites = data
        .feats
        .iter()
        .filter_map(|feat| feat.prerequisite.as_ref())
        .chain(
            data.all_classes()
                .filter_map(|class| class.prerequisite.as_ref()),
        )
        .chain(data.archetypes().filter_map(|a| a.prerequisite.as_ref()))
        .chain(
            data.selections()
                .flat_map(|selection| &selection.options)
                .filter_map(|option| option.prerequisite.as_ref()),
        )
        .chain(
            data.mythic_abilities
                .iter()
                .filter_map(|a| a.prerequisite.as_ref()),
        )
        .chain(
            data.mythic_feats
                .iter()
                .filter_map(|f| f.prerequisite.as_ref()),
        );
    prerequisites.collect()
}

pub fn selection_option(key: &str) -> &'static rules::SelectionOption {
    DATA.selection_option(&SelectionOptionKey::new(key))
        .unwrap_or_else(|| panic!("no option {key}"))
}

pub fn barbarian(levels: usize) -> Build {
    let mut build = build("barbarian", levels);
    align(&mut build, rules::Alignment::ChaoticNeutral);
    build
}

/// The status of `feat` in a feat slot.
pub fn feat_status(build: &Build, slot: &SlotId, feat: &str) -> Result<(), Issue> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("no slot {slot:?}"))
        .into_iter()
        .find(|option| option.pick.key() == feat)
        .unwrap_or_else(|| panic!("{feat} isn't offered in {slot:?}"))
        .status
}

pub const WILD_TALENTS: &str = "kineticist/wild-talents";

pub fn talent(name: &str) -> String {
    format!("{WILD_TALENTS}/{name}")
}

/// A kineticist of `element` with `levels` levels and these picks at the selection slots of their levels (a wild
/// talent at an even level, Expanded Element at 7 and 15).
pub fn kineticist_with(element: &str, levels: usize, picks: &[(u8, &str)]) -> Build {
    let mut build = kineticist(element, levels);
    for (level, key) in picks {
        build.picks.insert(selection_slot(*level), pick(key));
    }
    build
}

pub const ROGUE_TALENTS: &str = "rogue/rogue-talents";

pub fn rogue_talent(name: &str) -> String {
    format!("{ROGUE_TALENTS}/{name}")
}

/// A Build of one class with these picks at the selection slots of their levels.
pub fn with_picks(mut build: Build, picks: &[(u8, Pick)]) -> Build {
    for (level, pick) in picks {
        build.picks.insert(selection_slot(*level), pick.clone());
    }
    faithful(build)
}

pub fn extra_slot(level: u8, index: u8) -> SlotId {
    SlotId::FeatSelection { level, index }
}

pub fn selection_kind(key: &str) -> SlotKind {
    SlotKind::Selection(rules::FeatureSelectionKey::new(key))
}

/// A chaotic neutral Build taking `classes`, one per level, with no picks.
pub fn classes_build(classes: &[&str]) -> Build {
    let mut build = Build::default();
    align(&mut build, rules::Alignment::ChaoticNeutral);
    for class in classes {
        build
            .class_choices
            .entry(ClassKey::new(*class))
            .or_default();
        build.levels.push(LevelUp::new(ClassKey::new(*class)));
    }
    faithful(build)
}

/// The Build with every empty Feature selection slot filled, in slot order, with the first option its picker
/// leaves open.
pub fn with_selections_filled(mut build: Build) -> Build {
    let engine = engine();
    loop {
        let sheet = engine.evaluate(&build);
        let empty = sheet
            .levels
            .iter()
            .flat_map(|level| &level.slots)
            .filter(|slot| {
                matches!(slot.kind, SlotKind::Selection(_)) && !build.picks.contains_key(&slot.id)
            })
            .map(|slot| slot.id.clone())
            .find(|id| {
                engine
                    .options(&build, id)
                    .is_some_and(|options| options.iter().any(|option| option.status.is_ok()))
            });
        let Some(id) = empty else {
            return build;
        };
        let open = engine
            .options(&build, &id)
            .unwrap()
            .into_iter()
            .find(|option| option.status.is_ok())
            .unwrap();
        build.picks.insert(id, open.pick);
    }
}

/// The slot kind the Build's sheet has at `slot`, `None` when it has none.
pub fn kind_at(build: &Build, slot: &SlotId) -> Option<SlotKind> {
    engine()
        .evaluate(build)
        .slot(slot)
        .map(|slot| slot.kind.clone())
}

/// An unparameterised feat.
pub fn feat_pick_plain(key: &str) -> Pick {
    Pick::feat(FeatKey::new(key))
}

pub fn chaotic(mut build: Build) -> Build {
    build.start.alignment = Some(rules::Alignment::ChaoticNeutral);
    build.start.deity = None;
    faithful(build)
}

/// A lawful good paladin of `levels` levels, with no picks.
pub fn paladin(levels: usize) -> Build {
    let mut build = build("paladin", levels);
    align(&mut build, rules::Alignment::LawfulGood);
    build
}

/// A chaotic neutral Arcane Enforcer of `levels` levels, with no picks.
pub fn enforcer(levels: usize) -> Build {
    chaotic(archetype_build("slayer", "slayer/arcane-enforcer", levels))
}

/// The levels at which the Build's sheet has a slot of `selection`, in order.
pub fn levels_with_slot_of(build: &Build, selection: &str) -> Vec<u8> {
    let sheet = engine().evaluate(build);
    sheet
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .filter(|slot| slot.kind == selection_kind(selection))
        .map(|slot| match slot.id {
            SlotId::Selection { level, .. } => level,
            ref other => panic!("a {selection} slot at {other:?}"),
        })
        .collect()
}

/// The options the selection slot of `level` leaves open, by key.
pub fn open_at(build: &Build, level: u8) -> BTreeSet<String> {
    offered(build, level)
        .into_iter()
        .filter(|(_, status)| status.is_ok())
        .map(|(key, _)| key)
        .collect()
}

/// Why the selection slot of `level` locks `option`, as the sheet says it.
pub fn lock_reason(build: &Build, level: u8, option: &str) -> String {
    let status = offered(build, level)
        .remove(option)
        .unwrap_or_else(|| panic!("{option} isn't offered at {level}"));
    engine().issue_text(&status.expect_err("locked")).unwrap()
}

/// A chaotic neutral Exploiter Wizard of `levels` levels, with no picks.
pub fn exploiter(levels: usize) -> Build {
    chaotic(archetype_build("wizard", "wizard/exploiter-wizard", levels))
}

/// The one slot of `selection` the Build's sheet has at `level`.
pub fn slot_of(build: &Build, level: u8, selection: &str) -> SlotId {
    let found: Vec<SlotId> = engine()
        .evaluate(build)
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .filter(|slot| {
            slot.kind == selection_kind(selection)
                && matches!(slot.id, SlotId::Selection { level: at, .. } if at == level)
        })
        .map(|slot| slot.id.clone())
        .collect();
    let [slot] = found.as_slice() else {
        panic!("{} slots of {selection} at {level}", found.len())
    };
    slot.clone()
}

/// The options `slot`'s picker offers, by key, with their status.
pub fn offered_in(build: &Build, slot: &SlotId) -> HashMap<String, Result<(), Issue>> {
    engine()
        .options(build, slot)
        .unwrap_or_else(|| panic!("no slot {slot:?}"))
        .into_iter()
        .map(|SlotOption { pick, status, .. }| (pick.key().to_owned(), status))
        .collect()
}

/// Why `slot`'s picker locks `option`, as the sheet says it.
pub fn reason_in(build: &Build, slot: &SlotId, option: &str) -> String {
    let status = offered_in(build, slot)
        .remove(option)
        .unwrap_or_else(|| panic!("{option} isn't offered in {slot:?}"));
    engine().issue_text(&status.expect_err("locked")).unwrap()
}

/// The features of `class`'s archetype `archetype`, by key.
pub fn archetype_features(class_key: &str, archetype: &str) -> BTreeSet<String> {
    class(class_key)
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == archetype)
        .unwrap_or_else(|| panic!("no {archetype}"))
        .features
        .iter()
        .map(|f| f.key.to_string())
        .collect()
}

/// A lawful neutral monk of `levels` levels, with no picks.
pub fn monk(levels: usize) -> Build {
    let mut build = build("monk", levels);
    align(&mut build, rules::Alignment::LawfulNeutral);
    build
}

pub const INFUSIONS_SELECTION: &str = "kineticist/infusions";
pub const SHARPSHOOTER: &str = "kineticist/kinetic-sharpshooter";

pub fn infusion(name: &str) -> String {
    format!("{INFUSIONS_SELECTION}/{name}")
}

/// Who may take an infusion, by the Kinetic Sharpshooter's section: anyone, only the Sharpshooter ("exclusive to the
/// Sharpshooter"), or anyone but the Sharpshooter ("infusions that the Sharpshooter cannot use").
#[derive(Clone, Copy)]
pub enum Sharpshooter {
    Any,
    Only,
    Not,
}

pub fn infusion_reason(build: &Build, level: u8, name: &str) -> String {
    lock_reason(build, level, &infusion(name))
}

pub const SLAYER_TALENTS: &str = "slayer/slayer-talents";
pub const WITCH_HEX: &str = "witch/hex";
pub const FEYFORM: &str = "shifter/feyform-shifter";
pub const FEYFORM_TALENT: &str = "shifter/feyform-shifter/rogue-talent";
pub const VIVISECTIONIST: &str = "alchemist/vivisectionist";
pub const MEDICAL_DISCOVERY: &str = "alchemist/vivisectionist/medical-discovery";
pub const SANCTIFIED: &str = "inquisitor/sanctified-slayer";
pub const HEXCRAFTER: &str = "magus/hexcrafter";
pub const HEX_ARCANA: &str = "magus/hexcrafter/hex-arcana";
pub const ARCHAEOLOGIST: &str = "bard/archaeologist";

/// `alchemist#medical-discovery`'s three lists, by key.
pub const MEDICAL_DISCOVERIES: [&str; 25] = [
    "alchemist/discovery/cognatogen",
    "alchemist/discovery/enhance-potion",
    "alchemist/discovery/extend-potion",
    "alchemist/discovery/feral-mutagen",
    "alchemist/discovery/infusion",
    "alchemist/discovery/preserve-organs",
    "alchemist/discovery/spontaneous-healing",
    "alchemist/discovery/feral-wings",
    "alchemist/discovery/mummification",
    "alchemist/discovery/nauseating-flesh",
    "alchemist/discovery/greater-mutagen",
    "alchemist/discovery/grand-mutagen",
    "alchemist/discovery/greater-cognatogen",
    "alchemist/discovery/grand-cognatogen",
    "rogue/rogue-talents/combat-trick",
    "rogue/rogue-talents/fast-stealth",
    "rogue/rogue-talents/focusing-attack-confused",
    "rogue/rogue-talents/focusing-attack-shaken",
    "rogue/rogue-talents/focusing-attack-sickened",
    "rogue/rogue-talents/iron-guts",
    "rogue/rogue-talents/slow-reactions",
    "rogue/rogue-talents/weakening-wound",
    "rogue/rogue-talents/confounding-blades",
    "rogue/rogue-talents/crippling-strike",
    "rogue/rogue-talents/dispelling-attack",
];

/// A chaotic neutral Build taking each class in turn for its levels, with its archetype if any, with no picks.
pub fn multiclass(parts: &[(&str, Option<&str>, usize)]) -> Build {
    let mut build = Build::default();
    align(&mut build, rules::Alignment::ChaoticNeutral);
    for (class, archetype, levels) in parts {
        build.class_choices.insert(
            ClassKey::new(*class),
            ClassChoice {
                archetype: archetype.map(ArchetypeKey::new),
                ..ClassChoice::default()
            },
        );
        for _ in 0..*levels {
            build.levels.push(LevelUp::new(ClassKey::new(*class)));
        }
    }
    faithful(build)
}

/// A chaotic neutral Build of one class (with its archetype if any), `levels` levels, with no picks.
pub fn borrower(class: &str, archetype: Option<&str>, levels: usize) -> Build {
    multiclass(&[(class, archetype, levels)])
}

pub fn keys(selection: &rules::FeatureSelection) -> BTreeSet<String> {
    selection
        .options
        .iter()
        .map(|o| o.key.to_string())
        .chain(selection.shared_options.iter().map(|k| k.to_string()))
        .collect()
}

pub const WITCH_WANDERING_HEX: &str = "witch-wandering-hex";
pub const BENEFICIAL_CURSE: &str = "beneficial-curse";
pub const CURSES: &str = "oracle/oracles-curse";

/// `build` with `ability` as MR 1's general pick (MR 1 is at level 6).
pub fn with_mr_1_ability(mut build: Build, ability: &str) -> Build {
    build.picks.insert(
        SlotId::Mythic { rank: 1 },
        Pick::mythic_ability(rules::MythicAbilityKey::new(ability)),
    );
    build
}

/// The slot MR 1's mythic ability grants.
pub fn mr_1_selection() -> SlotId {
    SlotId::MythicSelection { rank: 1, index: 0 }
}
