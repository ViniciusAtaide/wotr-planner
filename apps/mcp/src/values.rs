//! How a choice's value is written ([`ChoiceValue`]): the same for what the sheet shows a choice holds and for
//! what `options` offers, so a value copied from either matches.

use std::collections::BTreeSet;

use rules::{
    Ability, Alignment, ArchetypeKey, ClassKey, Engine, Faith, Parameter, Pick, RaceKey, SubraceKey,
};

use crate::address::ability_token;
use crate::shapes::ChoiceValue;

/// A Parameter as a value names it: a skill's, weapon's or metamagic feat's key, or a school, energy, armour type,
/// stat or element in lower case with dashes (`attacks-of-opportunity`, `negative-energy`).
pub(crate) fn parameter_text(value: &Parameter) -> String {
    match value {
        Parameter::Skill(key) => key.as_str().to_owned(),
        Parameter::Weapon(key) => key.as_str().to_owned(),
        Parameter::Metamagic(key) => key.as_str().to_owned(),
        other => other.key().to_lowercase().replace(' ', "-"),
    }
}

/// A pick's value: its key, with its parameter when it names one.
pub(crate) fn pick_value(pick: &Pick) -> ChoiceValue {
    match pick.parameter() {
        Some(parameter) => ChoiceValue::Parameterised {
            key: pick.key().to_owned(),
            parameter: parameter_text(parameter),
        },
        None => ChoiceValue::Key(pick.key().to_owned()),
    }
}

/// An ability's value: `str`, `dex`, …
pub(crate) fn ability_value(ability: Ability) -> ChoiceValue {
    ChoiceValue::Key(ability_token(ability))
}

/// An alignment's key: its name in lower case with dashes (`lawful-good`, `true-neutral`).
pub(crate) fn alignment_key(alignment: Alignment) -> String {
    alignment.name().to_lowercase().replace(' ', "-")
}

/// A faith's key: the deity's, or `atheism`.
pub(crate) fn faith_key(faith: &Faith) -> String {
    match faith {
        Faith::Deity(key) => key.as_str().to_owned(),
        Faith::Atheism => "atheism".to_owned(),
    }
}

/// The race picker's value: the subrace's key when there is one (it starts with the race's), else the race's.
pub(crate) fn race_value(race: &RaceKey, subrace: Option<&SubraceKey>) -> ChoiceValue {
    ChoiceValue::Key(subrace.map_or(race.as_str(), SubraceKey::as_str).to_owned())
}

/// A class as the class picker names it: "Fighter", "Armiger (Fighter)", "Eldritch Knight, advancing Wizard",
/// "Eldritch Knight, advancing nothing".
pub(crate) fn class_name(
    engine: &Engine,
    class: &ClassKey,
    archetype: Option<&ArchetypeKey>,
    advances: Option<&BTreeSet<ClassKey>>,
) -> String {
    let name = match archetype {
        Some(archetype) => format!(
            "{} ({})",
            engine.archetype_name(archetype),
            engine.class_name(class)
        ),
        None => engine.class_name(class).to_owned(),
    };
    match advances {
        Some(advances) if advances.is_empty() => format!("{name}, advancing nothing"),
        Some(advances) => {
            let names: Vec<&str> = advances.iter().map(|key| engine.class_name(key)).collect();
            format!("{name}, advancing {}", names.join(" and "))
        }
        None => name,
    }
}

/// A class's value: the class, its archetype, and for a prestige class that asks, the classes it advances.
pub(crate) fn class_value(
    class: &ClassKey,
    archetype: Option<&ArchetypeKey>,
    advances: Option<&BTreeSet<ClassKey>>,
) -> ChoiceValue {
    ChoiceValue::Class {
        class: class.as_str().to_owned(),
        archetype: archetype.map(|key| key.as_str().to_owned()),
        advances: advances.map(|set| set.iter().map(|key| key.as_str().to_owned()).collect()),
    }
}
