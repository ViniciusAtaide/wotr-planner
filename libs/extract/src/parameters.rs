//! Parameterised and repeatable feats and options, from the guide's prose: what a feat is taken for ("Choose a
//! single weapon"), how many times it or an option can be taken ("You can take this feat up to 3 times"), the
//! prerequisites that name a value ("Weapon Focus (Dueling Sword)") or the pick's own ("for which you have already
//! selected Weapon Focus"), the weapons a feat can be taken for, and the bonus feats of the player's choosing (a
//! one-feat bonus list: the pick names the value). The tables quote the guide and are the same as ticket 27's
//! one-off rewrite of `libs/game-data/`, with ticket 32's decisions: every parameterised feat repeats once per value
//! (27-1), and the mythic feats and the Trickster's critical chain that take a value (27-5). The guide sentences
//! those decisions rest on are asserted where ticket 32's providers are read (`follow_ups::provide`).

use std::collections::BTreeSet;

use rules::{
    ArmourType, Class, DataAt, FeatKey, FeatList, FeatListKey, FeatSlot, FeatureKey, GameData,
    GuideRef, MythicFeatKey, Parameter, ParameterKind, Prerequisite, Repeat, SelectionOption,
    SelectionOptionKey, SkillKey, SpellSchool, Weapon, WeaponKey, WeaponProficiency,
};

use crate::guide::Guide;
use crate::names::slug;

/// The feats page's proficiency feats, each listing its weapons: "Become proficient in all of the following
/// weapons:" (simple, martial) or "... one of the following weapons:" (exotic).
const PROFICIENCY_LISTS: [(&str, WeaponProficiency); 3] = [
    ("simple-weapon-proficiency", WeaponProficiency::Simple),
    ("martial-weapons-proficiency", WeaponProficiency::Martial),
    ("exotic-weapon-proficiency", WeaponProficiency::Exotic),
];

/// Weapons no proficiency list names. Weapon Finesse's list: "Exotic: Dueling Sword, Elven Curve Blade, Estoc, Kama,
/// Sai, Swatooth Saber." (spelt "Sawtooth Saber" on the warpriest page). Weapon Focus: "This can also be selected
/// for unarmed strike (for Monks), natural weapons (for Wildshaped Druids) and touch attacks and rays (for
/// spellcasters)".
const OTHER_WEAPONS: [&str; 5] = [
    "Sawtooth Saber",
    "Unarmed Strike",
    "Natural Weapons",
    "Touch Attacks",
    "Rays",
];

/// The weapons a parameterised feat can be taken for: the three proficiency lists, then the others.
pub fn weapons(guide: &Guide) -> Vec<Weapon> {
    let feats = guide.page("feats");
    let mut weapons = Vec::new();
    for (anchor, proficiency) in PROFICIENCY_LISTS {
        let section = feats
            .root
            .find(anchor)
            .unwrap_or_else(|| panic!("the feats page has no #{anchor}"));
        let lines: Vec<&String> = section.paragraphs().flatten().collect();
        let list = lines
            .iter()
            .position(|line| line.starts_with("Become proficient in"))
            .and_then(|at| lines.get(at + 1))
            .unwrap_or_else(|| panic!("#{anchor} lists no weapons"));
        for name in list.trim_end_matches('.').split(',') {
            weapons.push(weapon(name.trim(), Some(proficiency)));
        }
    }
    weapons.extend(OTHER_WEAPONS.iter().map(|name| weapon(name, None)));
    weapons
}

fn weapon(name: &str, proficiency: Option<WeaponProficiency>) -> Weapon {
    Weapon {
        key: WeaponKey::new(slug(name)),
        name: name.to_owned(),
        proficiency,
    }
}

/// What a feat is taken for, or how often it can be taken: a feat taken for a value repeats once per value (27-1,
/// the user's decision), whatever the guide says of repeating. A feat left out takes no value and is taken once.
#[derive(Clone, Copy)]
enum Shape {
    /// Taken for a value of this kind, once per value.
    Value(ParameterKind),
    /// Taken for no value, this often.
    Repeats(Repeat),
}

const FEAT_SHAPES: &[(&str, Shape)] = {
    use ParameterKind as K;
    use Repeat::*;
    use Shape::*;
    &[
        // "Choose a single weapon ... may be selected multiple times, although it must be applied to a different
        // weapon each time."
        ("weapon-focus", Value(K::Weapon)),
        // "Choose a single weapon for which you have already selected Weapon Focus" (Greater Weapon Focus, Weapon
        // Specialization), "... Weapon Specialization" (Greater Weapon Specialization).
        ("greater-weapon-focus", Value(K::Weapon)),
        ("weapon-specialization", Value(K::Weapon)),
        ("greater-weapon-specialization", Value(K::Weapon)),
        // "Select one weapon type. When using that weapon, the threat range is doubled."
        ("improved-critical", Value(K::Weapon)),
        // The Trickster's chain: "When using the selected weapon, its critical threat range is increased by +1"
        // (27-5, the user's decision).
        ("improved-improved-critical", Value(K::Weapon)),
        ("improved-improved-improved-critical", Value(K::Weapon)),
        (
            "improved-improved-improved-critical-improved",
            Value(K::Weapon),
        ),
        // "Become proficient in one of the following weapons: Bastard Sword, ..."
        ("exotic-weapon-proficiency", Value(K::ExoticWeapon)),
        // "Select one skill"; per skill: user decision R4 (ticket 21), the Master of All's three Skill Focus.
        ("skill-focus", Value(K::Skill)),
        // "Choose a school of magic" / "... for which you have already selected Spell Focus".
        ("spell-focus", Value(K::SpellSchool)),
        ("greater-spell-focus", Value(K::SpellSchool)),
        // "Choose one energy type (acid, cold, electricity or fire)" / "... for which you have already selected
        // Elemental Focus".
        ("elemental-focus", Value(K::Energy)),
        ("greater-elemental-focus", Value(K::Energy)),
        // "Select one type of armour (light, medium, or heavy)".
        ("armor-focus", Value(K::ArmourType)),
        // "You can take this feat up to 3 times."
        ("extra-arcana", Repeats(Times(3))),
        ("extra-arcanist-exploit", Repeats(Times(3))),
        ("extra-discovery", Repeats(Times(3))),
        ("extra-hex", Repeats(Times(3))),
        ("extra-mercy", Repeats(Times(3))),
        ("extra-rogue-talent", Repeats(Times(3))),
        ("extra-slayer-talent", Repeats(Times(3))),
        ("extra-wild-talent", Repeats(Times(3))),
        // "This feat may be taken multiple times."
        ("extra-arcane-pool", Repeats(Unlimited)),
        ("extra-bombs", Repeats(Unlimited)),
        ("extra-channel", Repeats(Unlimited)),
        ("extra-ki", Repeats(Unlimited)),
        ("extra-lay-on-hands", Repeats(Unlimited)),
        ("extra-performance", Repeats(Unlimited)),
        ("extra-rage", Repeats(Unlimited)),
        ("extra-rage-power", Repeats(Unlimited)),
        ("extra-bloodrage", Repeats(Unlimited)),
        ("extra-reservoir", Repeats(Unlimited)),
        // "You may take this feat up to eight times".
        ("magical-tail", Repeats(Times(8))),
    ]
};

/// The mythic feats taken for a value, each once per value (27-5, the user's decision).
const MYTHIC_FEAT_SHAPES: &[(&str, ParameterKind)] = {
    use ParameterKind as K;
    &[
        // "The critical multiplier of your chosen weapon ..."
        ("improved-critical-mythic", K::Weapon),
        // "Select a school in which you have taken a Spell Focus ..."
        ("spell-focus-mythic", K::SpellSchool),
        // "Select a skill in which you have taken a Skill Focus ..."
        ("skill-focus-mythic", K::Skill),
        // "Select one school of magic ..."
        ("expanded-arsenal", K::SpellSchool),
        ("school-mastery", K::SpellSchool),
    ]
};

/// A value a prerequisite names.
#[derive(Clone, Copy)]
enum Value {
    Weapon(&'static str),
    Skill(&'static str),
    School(SpellSchool),
    Armour(ArmourType),
}

impl Value {
    fn parameter(self) -> Parameter {
        match self {
            Value::Weapon(key) => Parameter::Weapon(WeaponKey::new(key)),
            Value::Skill(key) => Parameter::Skill(SkillKey::new(key)),
            Value::School(school) => Parameter::SpellSchool(school),
            Value::Armour(armour) => Parameter::ArmourType(armour),
        }
    }
}

/// What a prerequisite's bare `Feat` leaf becomes.
#[derive(Clone, Copy)]
enum Needs {
    /// The feat for the pick's own value.
    SameValue,
    /// The feat for any of these values (one: that value).
    AnyOf(&'static [Value]),
}

/// Whose prerequisite.
#[derive(Clone, Copy)]
enum Owner {
    Feat(&'static str),
    MythicFeat(&'static str),
    PrestigeClass(&'static str),
}

const PREREQUISITES: &[(Owner, &str, Needs)] = {
    use Needs::*;
    use Owner::*;
    use Value::*;
    &[
        // "Choose a single weapon for which you have already selected Weapon Focus" and the like.
        (Feat("greater-weapon-focus"), "weapon-focus", SameValue),
        (Feat("weapon-specialization"), "weapon-focus", SameValue),
        (
            Feat("greater-weapon-specialization"),
            "weapon-specialization",
            SameValue,
        ),
        (Feat("greater-spell-focus"), "spell-focus", SameValue),
        (
            Feat("greater-elemental-focus"),
            "elemental-focus",
            SameValue,
        ),
        // The Trickster's chain: each needs the one before in "the selected weapon" (27-5).
        (
            Feat("improved-improved-critical"),
            "improved-critical",
            SameValue,
        ),
        (
            Feat("improved-improved-improved-critical"),
            "improved-improved-critical",
            SameValue,
        ),
        (
            Feat("improved-improved-improved-critical-improved"),
            "improved-improved-improved-critical",
            SameValue,
        ),
        // The mythic feats need their base feat in the value they're taken for (27-5).
        (
            MythicFeat("improved-critical-mythic"),
            "improved-critical",
            SameValue,
        ),
        (MythicFeat("spell-focus-mythic"), "spell-focus", SameValue),
        (MythicFeat("skill-focus-mythic"), "skill-focus", SameValue),
        // "Weapon Focus (Dueling Sword)".
        (
            Feat("aldori-dueling-mastery"),
            "weapon-focus",
            AnyOf(&[Weapon("dueling-sword")]),
        ),
        // "Weapon Focus (One-Handed Piercing)" ... "(i.e. Dagger, Estoc, Punching Dagger, Rapier, Sai, Starknife)".
        (
            Feat("fencing-grace"),
            "weapon-focus",
            AnyOf(&[
                Weapon("dagger"),
                Weapon("estoc"),
                Weapon("punching-dagger"),
                Weapon("rapier"),
                Weapon("sai"),
                Weapon("starknife"),
            ]),
        ),
        // "Skill Focus - Mobility" (Acrobatic Spellcaster), "Skill Focus - Mobility or Slayer Proficiencies".
        (
            Feat("acrobatic-spellcaster"),
            "skill-focus",
            AnyOf(&[Skill("mobility")]),
        ),
        (
            Feat("slayers-feint"),
            "skill-focus",
            AnyOf(&[Skill("mobility")]),
        ),
        // "Spell Focus (Conjuration)".
        (
            Feat("augment-summoning"),
            "spell-focus",
            AnyOf(&[School(SpellSchool::Conjuration)]),
        ),
        // "Required feats: Dazzling Display, Weapon Finesse, Weapon Focus (Dueling Sword)".
        (
            PrestigeClass("aldori-swordlord"),
            "weapon-focus",
            AnyOf(&[Weapon("dueling-sword")]),
        ),
        // "Required feats: Metamagic (any), Skill Focus (Knowledge: Arcana or Knowledge: World)".
        (
            PrestigeClass("loremaster"),
            "skill-focus",
            AnyOf(&[Skill("knowledge-arcana"), Skill("knowledge-world")]),
        ),
        // "Required feats: Skill Focus (Knowledge or Lore skill), Combat Expertise, Dodge".
        (
            PrestigeClass("student-of-war"),
            "skill-focus",
            AnyOf(&[
                Skill("knowledge-arcana"),
                Skill("knowledge-world"),
                Skill("lore-nature"),
                Skill("lore-religion"),
            ]),
        ),
        // "Mythic Armor Focus (Light Armor) - Assault ... Prerequisites: Armor Focus (Light)", and so on.
        (
            MythicFeat("mythic-armor-focus-light-armor-assault"),
            "armor-focus",
            AnyOf(&[Armour(ArmourType::Light)]),
        ),
        (
            MythicFeat("mythic-armor-focus-light-armor-avoidance"),
            "armor-focus",
            AnyOf(&[Armour(ArmourType::Light)]),
        ),
        (
            MythicFeat("mythic-armor-focus-light-armor-endurance"),
            "armor-focus",
            AnyOf(&[Armour(ArmourType::Light)]),
        ),
        (
            MythicFeat("mythic-armor-focus-medium-armor-assault"),
            "armor-focus",
            AnyOf(&[Armour(ArmourType::Medium)]),
        ),
        (
            MythicFeat("mythic-armor-focus-medium-armor-avoidance"),
            "armor-focus",
            AnyOf(&[Armour(ArmourType::Medium)]),
        ),
        (
            MythicFeat("mythic-armor-focus-medium-armor-endurance"),
            "armor-focus",
            AnyOf(&[Armour(ArmourType::Medium)]),
        ),
        (
            MythicFeat("mythic-armor-focus-heavy-armor-assault"),
            "armor-focus",
            AnyOf(&[Armour(ArmourType::Heavy)]),
        ),
        (
            MythicFeat("mythic-armor-focus-heavy-armor-avoidance"),
            "armor-focus",
            AnyOf(&[Armour(ArmourType::Heavy)]),
        ),
        (
            MythicFeat("mythic-armor-focus-heavy-armor-endurance"),
            "armor-focus",
            AnyOf(&[Armour(ArmourType::Heavy)]),
        ),
    ]
};

/// Options taken more than once.
const OPTION_REPEATS: &[(&str, Repeat)] = {
    use Repeat::*;
    &[
        // Barbarian: "This power may be selected up to three times." / "This ability may be selected up to three
        // times."
        ("barbarian/rage-power/increased-damage-reduction", Times(3)),
        ("barbarian/rage-power/swift-foot", Times(3)),
        // Stalwart Defender: "This ability may be selected twice."
        (
            "stalwart-defender/defensive-powers/increased-damage-reduction",
            Times(2),
        ),
        // Alchemist: "This may be taken up to three times".
        ("alchemist/discovery/preserve-organs", Times(3)),
        // Shaman: "This mutation may be selected multiple times."
        (
            "shaman/prophet-of-pestilence/carrier-of-disease/plague-mutation-new-strain",
            Unlimited,
        ),
        // Incense Synthesizer: "You can select this up to three times." / "... up to two times."
        (
            "alchemist/incense-synthesizer/incense-fog/incense-fog-clarifying-incense",
            Times(3),
        ),
        (
            "alchemist/incense-synthesizer/incense-fog/incense-fog-neutralizing-incense",
            Times(3),
        ),
        (
            "alchemist/incense-synthesizer/incense-fog/incense-fog-rallying-incense",
            Times(3),
        ),
        (
            "alchemist/incense-synthesizer/incense-fog/incense-fog-toxic-incense",
            Times(3),
        ),
        (
            "alchemist/incense-synthesizer/incense-fog/incense-fog-unsettling-fog",
            Times(3),
        ),
        (
            "alchemist/incense-synthesizer/incense-fog/incense-fog-improved-incense",
            Times(2),
        ),
    ]
};

/// Rogue talents that are "the feat of the same name", a parameterised one: the pick names the value.
const CHOSEN_FEATS: [(&str, &str); 2] = [
    ("rogue/rogue-talents/weapon-focus", "weapon-focus"),
    ("rogue/rogue-talents/skill-focus", "skill-focus"),
];

/// A bonus feat of the player's choosing: a feat the guide's table grants where its prose says the player picks
/// the value, or a feature that is such a grant. It becomes a one-feat bonus list of the class or archetype.
struct Choice {
    class: &'static str,
    /// The list's owner: the class, or one of its archetypes.
    owner: &'static str,
    list: &'static str,
    name: &'static str,
    guide: &'static str,
    feat: &'static str,
    /// What the rows grant now: the feat outright, or a feature standing in for it.
    granted: Granted,
}

enum Granted {
    Feat,
    Feature(&'static str),
}

const CHOICES: [Choice; 2] = [
    // Warpriest, Sacred Weapon: "At 1st level, a Warpriest gains Weapon Focus as a bonus feat in a weapon of his
    // choosing."
    Choice {
        class: "warpriest",
        owner: "warpriest",
        list: "warpriest/weapon-focus",
        name: "Weapon Focus",
        guide: "warpriest#sacred-weapon",
        feat: "weapon-focus",
        granted: Granted::Feat,
    },
    // Rogue, Master of All: "At 3rd, 9th and 15th levels, a Master of All receives a Skill Focus as a bonus feat."
    Choice {
        class: "rogue",
        owner: "rogue/master-of-all",
        list: "rogue/master-of-all/skill-focus",
        name: "Skill Focus (Master of All)",
        guide: "rogue#skill-focus-master-of-all",
        feat: "skill-focus",
        granted: Granted::Feature("rogue/master-of-all/skill-focus-master-of-all"),
    },
];

/// What the guide says a feat is taken for and how often; `None` for a feat it says neither of.
pub fn feat_shape(feat: &FeatKey) -> Option<(Option<ParameterKind>, Repeat)> {
    FEAT_SHAPES
        .iter()
        .find(|(key, ..)| feat.as_str() == *key)
        .map(|&(_, shape)| match shape {
            Shape::Value(kind) => (Some(kind), Repeat::PerValue),
            Shape::Repeats(repeat) => (None, repeat),
        })
}

/// What a mythic feat is taken for (once per value); `None` for one taken for no value.
pub fn mythic_feat_shape(feat: &MythicFeatKey) -> Option<ParameterKind> {
    MYTHIC_FEAT_SHAPES
        .iter()
        .find(|(key, _)| feat.as_str() == *key)
        .map(|&(_, kind)| kind)
}

/// How often the guide says an option can be taken; `None` for once.
pub fn option_repeat(option: &SelectionOptionKey) -> Option<Repeat> {
    OPTION_REPEATS
        .iter()
        .find(|(key, _)| option.as_str() == *key)
        .map(|&(_, repeat)| repeat)
}

/// The parameterised feat an option grants for its pick's value, when it's one of those.
pub fn chosen_feat(option: &SelectionOptionKey) -> Option<FeatKey> {
    CHOSEN_FEATS
        .iter()
        .find(|(key, _)| option.as_str() == *key)
        .map(|&(_, feat)| FeatKey::new(feat))
}

/// Rewrites the `Feat` leaves in `owner`'s prerequisite that the guide gives a value: for the pick's own, or for one
/// value (or any of several). Returns how many leaves it rewrote.
pub fn rewrite_prerequisite(owner: &DataAt, prerequisite: &mut Prerequisite) -> usize {
    let mut rewritten = 0;
    for (rule_owner, feat, needs) in PREREQUISITES {
        let matches = match (rule_owner, owner) {
            (Owner::Feat(key), DataAt::Feat(at)) => at.as_str() == *key,
            (Owner::MythicFeat(key), DataAt::MythicFeat(at)) => at.as_str() == *key,
            (Owner::PrestigeClass(key), DataAt::Class(at)) => at.as_str() == *key,
            _ => false,
        };
        if !matches {
            continue;
        }
        let feat = FeatKey::new(*feat);
        let new = match needs {
            Needs::SameValue => Prerequisite::FeatForSameValue(feat.clone()),
            Needs::AnyOf(values) => {
                let mut any: Vec<Prerequisite> = values
                    .iter()
                    .map(|value| Prerequisite::FeatFor {
                        feat: feat.clone(),
                        value: value.parameter(),
                    })
                    .collect();
                if any.len() == 1 {
                    any.remove(0)
                } else {
                    Prerequisite::Any(any)
                }
            }
        };
        rewritten += replace_leaf(prerequisite, &feat, &new);
    }
    rewritten
}

/// Marks the feats and options and rewrites the prerequisites and the rows, checking each rule finds what it
/// names. Runs on full rows, before the archetype rows are made sparse; the lists themselves come later
/// ([`add_lists`]), after the bonus lists are named.
pub fn apply(data: &mut GameData) {
    let mut shaped = 0;
    for feat in &mut data.feats {
        if let Some((parameter, repeat)) = feat_shape(&feat.key) {
            feat.parameter = parameter;
            feat.repeat = repeat;
            shaped += 1;
        }
    }
    assert_eq!(shaped, FEAT_SHAPES.len(), "every feat shape names a feat");
    let mut shaped = 0;
    for feat in &mut data.mythic_feats {
        if let Some(kind) = mythic_feat_shape(&feat.key) {
            feat.parameter = Some(kind);
            feat.repeat = Repeat::PerValue;
            shaped += 1;
        }
    }
    assert_eq!(
        shaped,
        MYTHIC_FEAT_SHAPES.len(),
        "every mythic feat shape names a mythic feat"
    );

    let mut rewritten = 0;
    let mut rewrite = |at: DataAt, prerequisite: &mut Option<Prerequisite>| {
        if let Some(prerequisite) = prerequisite {
            rewritten += rewrite_prerequisite(&at, prerequisite);
        }
    };
    for feat in &mut data.feats {
        rewrite(DataAt::Feat(feat.key.clone()), &mut feat.prerequisite);
    }
    for feat in &mut data.mythic_feats {
        rewrite(DataAt::MythicFeat(feat.key.clone()), &mut feat.prerequisite);
    }
    for class in &mut data.prestige_classes {
        rewrite(DataAt::Class(class.key.clone()), &mut class.prerequisite);
    }
    assert_eq!(
        rewritten,
        PREREQUISITES.len(),
        "every prerequisite rule rewrites one leaf"
    );

    let (mut repeats, mut chosen) = (0, 0);
    for option in options_mut(data) {
        if let Some(repeat) = option_repeat(&option.key) {
            option.repeat = repeat;
            repeats += 1;
        }
        if let Some(feat) = chosen_feat(&option.key) {
            assert!(option.feats.remove(&feat), "{}: grants {feat}", option.key);
            option.chosen_feat = Some(feat);
            chosen += 1;
        }
    }
    assert_eq!(
        (repeats, chosen),
        (OPTION_REPEATS.len(), CHOSEN_FEATS.len())
    );

    for class in &mut data.classes {
        apply_choices(class);
    }
}

/// In a class's rows (full or sparse), each bonus feat of the player's choosing becomes a slot of its one-feat list:
/// the rows granting its feat (or the feature standing in for it) get the slot instead, and the stand-in feature
/// is gone. A class without one is left alone.
pub fn apply_choices(class: &mut Class) {
    let key = class.key.to_string();
    for choice in CHOICES.iter().filter(|choice| choice.class == key) {
        apply_choice(class, choice);
    }
}

/// The rows granting the choice's feat (or its stand-in feature) get its list's slot instead.
fn apply_choice(class: &mut Class, choice: &Choice) {
    let feat = FeatKey::new(choice.feat);
    let slot = FeatSlot::OneOf {
        list: FeatListKey::new(choice.list),
    };
    let mut rows: Vec<(
        &mut Vec<FeatureKey>,
        &mut BTreeSet<FeatKey>,
        &mut Vec<FeatSlot>,
    )> = class
        .levels
        .iter_mut()
        .map(|row| (&mut row.features, &mut row.feats, &mut row.feat_slots))
        .collect();
    for archetype in &mut class.archetypes {
        rows.extend(
            archetype
                .levels
                .iter_mut()
                .map(|row| (&mut row.features, &mut row.feats, &mut row.feat_slots)),
        );
    }
    let mut changed = 0;
    for (features, feats, feat_slots) in rows {
        let grants = match choice.granted {
            Granted::Feat => feats.remove(&feat),
            Granted::Feature(feature) => {
                let before = features.len();
                features.retain(|key| key.as_str() != feature);
                features.len() < before
            }
        };
        if grants {
            feat_slots.push(slot.clone());
            changed += 1;
        }
    }
    assert!(
        changed > 0,
        "{}: no row grants {}",
        choice.owner,
        choice.feat
    );
    if let Granted::Feature(feature) = choice.granted {
        class
            .features
            .retain(|defined| defined.key.as_str() != feature);
        for archetype in &mut class.archetypes {
            archetype
                .features
                .retain(|defined| defined.key.as_str() != feature);
        }
    }
}

/// Adds each choice's one-feat bonus list to its owner. Runs after the bonus lists are named, which replaces each
/// owner's lists.
pub fn add_lists(data: &mut GameData) {
    for class in &mut data.classes {
        add_class_lists(class);
    }
}

/// Adds the one-feat bonus lists a class and its archetypes own.
pub fn add_class_lists(class: &mut Class) {
    let key = class.key.to_string();
    for choice in CHOICES.iter().filter(|choice| choice.class == key) {
        let list = FeatList {
            key: FeatListKey::new(choice.list),
            name: choice.name.to_owned(),
            guide: GuideRef::new(choice.guide),
            ignores_prerequisites: true,
            every_feat: false,
            feats: [FeatKey::new(choice.feat)].into(),
        };
        if choice.owner == choice.class {
            class.feat_lists.push(list);
        } else {
            class
                .archetypes
                .iter_mut()
                .find(|archetype| archetype.key.as_str() == choice.owner)
                .expect("the archetype")
                .feat_lists
                .push(list);
        }
    }
}

/// Replaces every `Feat(feat)` leaf with `new`; returns how many there were.
fn replace_leaf(tree: &mut Prerequisite, feat: &FeatKey, new: &Prerequisite) -> usize {
    match tree {
        Prerequisite::Feat(key) if key == feat => {
            *tree = new.clone();
            1
        }
        Prerequisite::All(children)
        | Prerequisite::Any(children)
        | Prerequisite::AtLeast(_, children) => children
            .iter_mut()
            .map(|child| replace_leaf(child, feat, new))
            .sum(),
        _ => 0,
    }
}

/// Every option of every class's, archetype's and path's selections.
fn options_mut(data: &mut GameData) -> impl Iterator<Item = &mut SelectionOption> {
    let classes = data
        .classes
        .iter_mut()
        .chain(&mut data.prestige_classes)
        .flat_map(|class| {
            class.selections.iter_mut().chain(
                class
                    .archetypes
                    .iter_mut()
                    .flat_map(|archetype| &mut archetype.selections),
            )
        });
    classes
        .chain(
            data.mythic_paths
                .iter_mut()
                .flat_map(|path| &mut path.selections),
        )
        .flat_map(|selection| &mut selection.options)
}
