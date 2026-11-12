//! Gear in the server (ticket 168): a Build holds gear only when the user states it. Everything here asks the
//! engine (ticket 166): a place's or a usable item's options and their locks ([`Engine::gear_options`],
//! [`Engine::usable_options`]), the choices only gear allows ([`Engine::gear_dependents`]), the numbers without gear
//! ([`Engine::naked_sheet`]) and the marks on an option that is open only because of gear or would lean on it.
//!
//! - `options` and `choose` take a place's items through [`gear_offers`], each with the engine's edit and verdict:
//!   an item a choice needs can't be taken off, or replaced by one that doesn't keep the choice, and the offer says
//!   which choice.
//! - `sheet` lists the gear held ([`gear_lines`]) and the choices it allows ([`dependent_lines`]), and gives the
//!   numbers without it where they differ (`sheet.rs`).
//! - `compare` and `improve` count the naked numbers unless asked ([`counted`]), and never add an item.

use std::collections::BTreeSet;

use rules::{
    Address, Build, Choice, Edit, Engine, GearDependent, GearItem, GearOption, GearPlace, GearSlot,
    GearSpot, Sheet,
};
use serde::Serialize;

use crate::error::ToolError;
use crate::options::Offer;
use crate::shapes::{ChoiceValue, GearDependentLine, GearLine, Numbers};

/// The value that takes a place's item off (or stops using a usable item) in `options` and `choose`.
pub(crate) const NONE: &str = "none";

/// How an open option stands with the Build's gear, when it does.
#[derive(Clone, Debug)]
pub(crate) enum GearMark {
    /// Open only because of the gear: the items it needs, worded ("With Belt of Incredible Dexterity +2").
    WithGear(String),
    /// Open, but taking it leaves these choices allowed only because of the gear, worded.
    LeansOnGear(String),
}

/// The mark of an open option whose edit would leave `choices` allowed only because of the gear (the engine's
/// `leans_on_gear`); `None` when there are none. The choices are named in the Build the edit makes, which holds them.
pub(crate) fn leaning(
    engine: &Engine,
    build: &Build,
    edit: &Edit,
    choices: &[Choice],
) -> Option<GearMark> {
    if choices.is_empty() {
        return None;
    }
    let mut edited = build.clone();
    engine.apply(&mut edited, edit);
    let names = engine.choice_names(&edited, choices);
    Some(GearMark::LeansOnGear(format!(
        "{names} would then be allowed only because of the gear"
    )))
}

/// A choice's address from the agent's text, or `no_such_choice` with the parser's reason; text that starts with
/// `gear` also gets the gear addresses.
pub(crate) fn parse_address(text: &str) -> Result<Address, ToolError> {
    text.parse().map_err(|error: rules::NotAnAddress| {
        let mut reason = error.to_string();
        if text.trim().starts_with("gear") {
            let places: Vec<String> = GearPlace::ALL
                .into_iter()
                .map(|place| format!("`{}`", Address::Gear(GearSpot::Worn(place))))
                .collect();
            reason.push_str(&format!(
                " The gear addresses are the places {} (`gear.body` takes a robe or an armour), and \
                 `gear.usable.<item key>` for each usable item (`search` with `kinds: [\"gear\"]` and `category: \
                 \"Usable\"` lists them).",
                places.join(", ")
            ));
        }
        ToolError::no_such_choice(reason)
    })
}

/// The usable item a `gear.usable.<key>` address names, or why there's none.
fn usable<'a>(engine: &'a Engine, key: &rules::GearKey) -> Result<&'a GearItem, ToolError> {
    match engine.gear_item(key) {
        Some(item) if item.slot == GearSlot::Usable => Ok(item),
        Some(item) => Err(ToolError::no_such_choice(format!(
            "{} isn't a usable item: it's worn at {}.",
            item.name,
            places(item).join(" or ")
        ))),
        None => Err(ToolError::no_such_choice(format!(
            "There is no gear item `{}`: `search` with `kinds: [\"gear\"]` finds items by name or bonus (`category: \
             \"Usable\"` lists the usable ones).",
            key.as_str()
        ))),
    }
}

/// The options at a gear address, in the engine's order: "None" (`none`) first, then each item the place takes,
/// each with its edit and the engine's verdict. A usable item's address offers "None" (stop using it) and the item
/// itself. An item that a choice needs can't be taken off, or replaced by one that doesn't keep the choice: such an
/// offer is `breaks_later`, its reason naming
/// the choice ("Greater Two-Weapon Fighting needs Belt of Incredible Dexterity +2").
pub(crate) fn gear_offers(
    engine: &Engine,
    build: &Build,
    spot: &GearSpot,
) -> Result<Vec<Offer>, ToolError> {
    match spot {
        GearSpot::Worn(place) => Ok(engine
            .gear_options(build, *place)
            .into_iter()
            .map(|option| offer(engine, option))
            .collect()),
        GearSpot::Used(key) => {
            let item = usable(engine, key)?;
            let tick = engine
                .usable_options(build)
                .into_iter()
                .find(|option| option.item.as_ref() == Some(&item.key))
                .expect("the usable items' picker lists every usable item");
            // The tick's verdict is its edit's: using the item when it isn't used, stopping when it is. The other
            // option is what the Build holds already.
            let held = |used: bool| GearOption {
                item: used.then(|| item.key.clone()),
                held: true,
                status: Ok(()),
                needed_by: Vec::new(),
                reason: None,
                edit: Edit::Use {
                    item: item.key.clone(),
                    used,
                },
            };
            let (none, with) = if tick.held {
                let stop = GearOption {
                    item: None,
                    held: false,
                    ..tick
                };
                (stop, held(true))
            } else {
                (
                    held(false),
                    GearOption {
                        held: false,
                        ..tick
                    },
                )
            };
            Ok(vec![offer(engine, none), offer(engine, with)])
        }
    }
}

/// One gear option as an offer.
fn offer(engine: &Engine, option: GearOption) -> Offer {
    let GearOption {
        item,
        held,
        status,
        reason,
        edit,
        ..
    } = option;
    let mut offer = match &item {
        Some(key) => {
            let item = engine
                .gear_item(key)
                .expect("the engine offers items it has");
            let mut offer = Offer::new(
                ChoiceValue::Key(key.as_str().to_owned()),
                item.name.clone(),
                status,
                edit,
            );
            let summary = engine.gear_summary(key);
            offer.summary = (!summary.is_empty()).then_some(summary);
            offer.other_effects = item.other_effects.clone();
            offer.uncounted = item.always_on.clone();
            offer.category = Some(item.slot.name().to_owned());
            offer
        }
        None => Offer::new(
            ChoiceValue::Key(NONE.to_owned()),
            "None".to_owned(),
            status,
            edit,
        ),
    };
    offer.reason = reason;
    offer.current = held;
    offer
}

/// Why the item at a gear address can't come off, when a choice needs it: the engine's reason on taking it off
/// ("Greater Two-Weapon Fighting needs Belt of Incredible Dexterity +2").
pub(crate) fn needed_reason(engine: &Engine, build: &Build, spot: &GearSpot) -> Option<String> {
    gear_offers(engine, build, spot)
        .ok()?
        .into_iter()
        .find(|offer| offer.value == ChoiceValue::Key(NONE.to_owned()))
        .and_then(|offer| offer.reason)
}

/// The addresses where an item goes: `gear.belt`; both ring places for a ring; `gear.body` for a robe or an armour;
/// `gear.usable.<its key>` for a usable item.
pub(crate) fn places(item: &GearItem) -> Vec<String> {
    match item.slot {
        GearSlot::Usable => vec![Address::Gear(GearSpot::Used(item.key.clone())).to_string()],
        slot => GearPlace::ALL
            .into_iter()
            .filter(|place| place.takes(slot))
            .map(|place| Address::Gear(GearSpot::Worn(place)).to_string())
            .collect(),
    }
}

/// The gear a Build holds, in the order its link writes it (places in order, then the usable items by key); with
/// `removable`, whether each can come off now.
pub(crate) fn gear_lines(
    engine: &Engine,
    build: &Build,
    removable: Option<&BTreeSet<Choice>>,
) -> Vec<GearLine> {
    build
        .gear
        .items()
        .map(|(spot, key)| {
            let address = Address::Gear(spot.clone());
            GearLine {
                choice: address.to_string(),
                label: engine.placed_label(&address, None),
                value: key.as_str().to_owned(),
                name: engine.gear_name(key).to_owned(),
                summary: engine.gear_summary(key),
                other_effects: engine
                    .gear_item(key)
                    .map(|item| item.other_effects.clone())
                    .unwrap_or_default(),
                uncounted: engine
                    .gear_item(key)
                    .map(|item| item.always_on.clone())
                    .unwrap_or_default(),
                removable: removable.map(|set| set.contains(&Choice::Gear(spot))),
            }
        })
        .collect()
}

/// The choices a Build can make only because of its gear, worded: each by its address, label and what it holds,
/// with the items it needs. None for a Build without gear, at no cost.
pub(crate) fn dependent_lines(
    engine: &Engine,
    build: &Build,
    sheet: &Sheet,
) -> Vec<GearDependentLine> {
    engine
        .gear_dependents(build)
        .into_iter()
        .map(|GearDependent { choice, needs }| {
            let address = match &choice {
                Choice::CharacterLevel { level } => Address::Class(*level),
                choice => Address::from(choice),
            };
            let slot = match &address {
                Address::Slot(id) => sheet.slot(id),
                _ => None,
            };
            GearDependentLine {
                choice: address.to_string(),
                label: engine.placed_label(&address, slot),
                name: engine.choice_names(build, std::slice::from_ref(&choice)),
                needs: engine.gear_note(&needs),
                items: (needs.all_of.iter())
                    .chain(&needs.one_of)
                    .chain(&needs.for_example)
                    .map(|key| key.as_str().to_owned())
                    .collect(),
            }
        })
        .collect()
}

/// The sheet whose numbers `compare` and `improve` count: the naked one ([`Engine::naked_sheet`], the Build with no
/// gear) unless `with_gear`, and the Build's own for a Build without gear.
pub(crate) fn counted(engine: &Engine, build: &Build, sheet: &Sheet, numbers: Numbers) -> Sheet {
    match numbers {
        Numbers::WithGear => sheet.clone(),
        Numbers::Naked => engine.naked_sheet(build).unwrap_or_else(|| sheet.clone()),
    }
}

/// `other` when it differs from `counted`, as the tools print them: the numbers beside the counted ones.
pub(crate) fn if_differs<T: Serialize>(counted: Option<&T>, other: Option<T>) -> Option<T> {
    let json = |value: &T| serde_json::to_value(value).expect("a shape serialises");
    match (counted, other) {
        (Some(counted), Some(other)) if json(counted) == json(&other) => None,
        (_, other) => other,
    }
}
