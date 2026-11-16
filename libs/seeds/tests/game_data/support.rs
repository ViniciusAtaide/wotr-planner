//! The helpers the real-data tests share: the data and an engine over it, Builds, slots and picks, and lookups.

use std::collections::{BTreeSet, HashMap};
use std::sync::LazyLock;

use rules::{
    ArchetypeKey, Build, CharacterState, CheckedGameData, ClassChoice, ClassKey, Engine, FeatKey,
    FeatSource, FeatureKey, GameData, Issue, LevelUp, MythicPathKey, Parameter, Pick, Prerequisite,
    SelectionOptionKey, SkillKey, SlotId, SlotKind, SlotOption, Unmet, WeaponKey,
};

/// The game data as every reader holds it: [`CHECKED`]'s, each entry's `Requires` compiled into its `prerequisite`
/// (ticket 296).
pub static DATA: LazyLock<GameData> = LazyLock::new(|| GameData::clone(&CHECKED));

/// Every entry's blueprint, by where it is (ticket 237).
pub static BLUEPRINTS: LazyLock<std::collections::BTreeMap<rules::DataAt, Option<&'static str>>> =
    LazyLock::new(|| {
        DATA.blueprints()
            .into_iter()
            .map(|(at, guid)| (at, guid.map(rules::BlueprintGuid::as_str)))
            .collect()
    });

/// The files read and checked once, for [`DATA`] and every engine the tests build.
static CHECKED: LazyLock<CheckedGameData> = LazyLock::new(|| {
    seeds::check_game_data_files(&seeds::game_data_dir()).expect("game-data checks")
});

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

/// A Build from a link made before ticket 251, without the weapon feats in it that the game refuses for a weapon the
/// character isn't proficient with (`RequireProficiency`, which `BlueprintParametrizedFeature.CanSelect` checks,
/// WrathDecompiled 6ce4a6b): those make the link stale. Panics when any other choice fails.
pub fn without_unproficient_picks(mut build: Build) -> Build {
    for failure in engine().failures(&build) {
        match (failure.choice, failure.issue) {
            (rules::Choice::Slot(slot), Issue::NotProficient { .. }) => {
                build.picks.remove(&slot);
            }
            (choice, issue) => panic!("{choice:?} fails: {issue:?}"),
        }
    }
    build
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
        .chain(
            data.archetypes
                .iter()
                .filter_map(|a| a.prerequisite.as_ref()),
        )
        .chain(
            data.options
                .iter()
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

/// The features `class`'s archetype `archetype` grants on its level rows, by key.
pub fn archetype_features(class_key: &str, archetype: &str) -> BTreeSet<String> {
    DATA.archetypes_of(&class(class_key).key)
        .find(|a| a.key.as_str() == archetype)
        .unwrap_or_else(|| panic!("no {archetype}"))
        .levels
        .iter()
        .flat_map(|row| &row.features)
        .map(|f| f.to_string())
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
        .map(|o| o.to_string())
        .chain(selection.shared_options.iter().map(|k| k.to_string()))
        .collect()
}

pub const WITCH_WANDERING_HEX: &str = "witch-wandering-hex";
pub const BENEFICIAL_CURSE: &str = "beneficial-curse";
pub const CURSES: &str = "oracle/oracles-curse";
/// The selection Beneficial Curse grants (ticket 302), standing for [`CURSES`].
pub const BENEFICIAL_CURSE_SELECTION: &str = "oracle/beneficial-curse";

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

/// The benchmark's Builds (ticket 48's), one `name<TAB>link` line each; an empty link is the blank Build.
const BENCH: &str = include_str!("../../../../tools/corpus/bench-links.tsv");

/// The bench Builds by name, opened as the planner opens a link. Those made before deities (an alignment and no
/// faith, a stale link since ticket 94) are atheists, as the corpus's bench Builds are.
pub fn bench() -> Vec<(String, Build)> {
    let engine = engine();
    let keys = rules::LinkKeys::new(&DATA);
    BENCH
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let (name, link) = line.split_once('\t').unwrap_or((line, ""));
            let build = if link.is_empty() {
                Build::default()
            } else {
                engine.open_link(link).unwrap_or_else(|_| {
                    let mut build =
                        rules::decode_build(link, &keys).expect("the bench links decode");
                    build.start.deity = Some(rules::Faith::Atheism);
                    build
                })
            };
            assert!(engine.is_possible(&build), "{name}");
            (name.to_owned(), build)
        })
        .collect()
}

/// Some features and Feature selections. An entry no longer says its owner (ticket 244): the classes' and the Mythic
/// paths' are told apart by what grants them.
#[derive(Default)]
pub struct Grants {
    pub features: Vec<&'static rules::Feature>,
    pub selections: Vec<&'static rules::FeatureSelection>,
}

/// One level row's grants: the features and the Feature selections it names.
type RowGrants = (&'static [FeatureKey], &'static [rules::FeatureSelectionKey]);

/// What `rows` grant, in key order: the features and selections the rows name, and the ones the own options of those
/// selections name on their rows, all the way down.
pub fn grants(rows: impl IntoIterator<Item = RowGrants>) -> Grants {
    let data: &'static GameData = &DATA;
    let mut features = BTreeSet::new();
    let mut selections = BTreeSet::new();
    let mut waiting: Vec<RowGrants> = rows.into_iter().collect();
    while let Some((row_features, row_selections)) = waiting.pop() {
        features.extend(row_features);
        for key in row_selections {
            if !selections.insert(key) {
                continue;
            }
            let Some(selection) = data.selection(key) else {
                continue;
            };
            for option in data.options_of(selection) {
                waiting.extend(
                    option
                        .levels
                        .iter()
                        .map(|row| (&row.features[..], &row.selections[..])),
                );
            }
        }
    }
    Grants {
        features: features
            .into_iter()
            .filter_map(|k| data.feature(k))
            .collect(),
        // The selections of options (ticket 301: a row's selections of feats were feat slots).
        selections: selections
            .into_iter()
            .filter_map(|k| data.selection(k))
            .filter(|s| !s.offers_feats())
            .collect(),
    }
}

/// The rows of `class` and of its archetypes.
pub fn class_rows(class: &'static rules::Class) -> Vec<RowGrants> {
    let own = class
        .levels
        .iter()
        .map(|row| (&row.features[..], &row.selections[..]));
    let archetypes = DATA
        .archetypes
        .iter()
        .filter(move |a| a.class == class.key)
        .flat_map(|a| a.levels.iter())
        .map(|row| (&row.features[..], &row.selections[..]));
    own.chain(archetypes).collect()
}

/// The classes' features and selections (base and prestige classes', archetypes'): every one the Mythic paths don't
/// grant ([`all_path_grants`]). Some are granted from elsewhere than a class's rows, so reaching them from the rows
/// would miss them: Bloodline Ascendance and Second Mystery by a mythic ability, for one.
pub fn all_class_grants() -> Grants {
    let data: &'static GameData = &DATA;
    let paths = all_path_grants();
    let path_features: BTreeSet<&FeatureKey> = paths.features.iter().map(|f| &f.key).collect();
    let path_selections: BTreeSet<&rules::FeatureSelectionKey> =
        paths.selections.iter().map(|s| &s.key).collect();
    Grants {
        features: data
            .features
            .iter()
            .filter(|f| !path_features.contains(&f.key))
            .collect(),
        selections: data
            .selections
            .iter()
            .filter(|s| !s.offers_feats() && !path_selections.contains(&s.key))
            .collect(),
    }
}

/// What the Mythic paths grant: their rank rows' grants, and the features a switch to a path grants.
pub fn all_path_grants() -> Grants {
    let data: &'static GameData = &DATA;
    let mut grants = grants(data.mythic_paths.iter().flat_map(|path| {
        path.ranks
            .iter()
            .map(|row| (&row.features[..], &row.selections[..]))
    }));
    let switched = data
        .mythic_paths
        .iter()
        .filter_map(|path| path.switch.as_ref())
        .flat_map(|switch| &switch.features)
        .filter_map(|key| data.feature(key));
    grants.features.extend(switched);
    grants.features.sort_by(|a, b| a.key.cmp(&b.key));
    grants.features.dedup_by(|a, b| a.key == b.key);
    grants
}

// ---- feat slots and feat lists as selections (ticket 301) ------------------------------------------------------

/// The general and category selections of feats: what the `General`, `CombatFeat` and `Category` feat slots were.
pub const FEAT_POOLS: [&str; 5] = [
    rules::BASIC_FEAT_SELECTION,
    "combat-feats",
    "magic-feats",
    "teamwork-feats",
    "mounted-combat-feats",
];

/// A Feature selection by key.
/// The selections a feat's pick may be a slot of, as the planner holds them (ticket 302): those its selection needs a
/// slot of first ([`needs_slot_of`](rules::FeatureSelection::needs_slot_of)), which every Extra feat's names. Empty
/// for a feat that grants none.
pub fn picks_of(feat: &rules::Feat) -> BTreeSet<rules::FeatureSelectionKey> {
    feat.selection
        .as_ref()
        .map(|key| any_selection(key.as_str()).needs_slot_of.clone())
        .unwrap_or_default()
}

pub fn any_selection(key: &str) -> &'static rules::FeatureSelection {
    DATA.selection(&rules::FeatureSelectionKey::new(key))
        .unwrap_or_else(|| panic!("no selection {key}"))
}

/// Whether a selection is what a feat list was: one listing its own feats, not a pool ([`FEAT_POOLS`]).
pub fn is_feat_list(selection: &rules::FeatureSelection) -> bool {
    !selection.feats.is_empty() && !FEAT_POOLS.contains(&selection.key.as_str())
}

/// Every selection that was a feat list ([`is_feat_list`]), in key order.
pub fn feat_lists() -> impl Iterator<Item = &'static rules::FeatureSelection> {
    DATA.selections
        .iter()
        .filter(|selection| is_feat_list(selection))
}

/// The selections of feats among a row's selections: what its feat slots were.
pub fn feat_selections(
    keys: &[rules::FeatureSelectionKey],
) -> impl Iterator<Item = &rules::FeatureSelectionKey> {
    keys.iter()
        .filter(|key| DATA.selection(key).is_some_and(|s| s.offers_feats()))
}

/// The slot kind of a selection of feats.
pub fn feat_kind(key: &str) -> SlotKind {
    SlotKind::Feat(rules::FeatureSelectionKey::new(key))
}

/// `prerequisite` with an entry's own repeat condition beside it, `Not` of `own` (ticket 303: a selection's own
/// `PrerequisiteNoFeature` or cap on its rank), as the data's canonical tree holds them: an `All` of both, its
/// children sorted, flattened when `prerequisite` is an `All` itself.
pub fn and_not(prerequisite: Prerequisite, own: Prerequisite) -> Prerequisite {
    let mut children = match prerequisite {
        Prerequisite::All(children) => children,
        other => vec![other],
    };
    children.push(Prerequisite::Not(Box::new(own)));
    let mut all = Prerequisite::All(children);
    all.canonicalize();
    all
}

/// An Extra feat's cap on its own rank (ticket 250; ticket 303's form): `Not` of it held `rank` times.
pub fn own_rank(feat: &str, rank: u8) -> Prerequisite {
    Prerequisite::Rank {
        fact: rules::Fact::Feat(FeatKey::new(feat)),
        rank,
    }
}
