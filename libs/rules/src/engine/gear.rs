//! Gear in the engine (ticket 166): the items the data has, what a place or the usable items offer, the Build
//! without its gear (the naked numbers), and the choices a Build can make only because of its gear.
//!
//! How the walk counts gear is in `walk/gear.rs`. What the web app and the MCP server call is here:
//!
//! - [`Engine::gear_options`] and [`Engine::usable_options`]: every item a place takes, or every usable item, each
//!   with the edit that wears or uses it. None is ever locked.
//! - [`Engine::naked_sheet`]: the sheet of the Build without its gear, for the numbers without gear. `None` for a
//!   Build that holds none, whose own sheet already is that one.
//! - [`Engine::gear_dependents`]: the choices allowed only because of gear, each with the items it needs.
//! - [`Engine::newly_gear_dependent`]: the choices an edit would make lean on the gear.
//! - A slot's picker marks an option open only because of gear ([`SlotOption::with_gear`],
//!   [`SlotEntry::gear_note`]); [`Engine::gear_note`] words the mark.
//!
//! [`SlotOption::with_gear`]: super::SlotOption::with_gear
//! [`SlotEntry::gear_note`]: super::SlotEntry::gear_note

use std::collections::BTreeSet;

use super::walk::{before_slot, pick_fits};
use super::{Engine, Sheet, SlotKind};
use crate::build::{Build, Edit, Gear, GearPlace, GearSpot, Pick, SlotId};
use crate::choice::{Choice, Issue};
use crate::gear::{GearItem, GearSlot};
use crate::keys::GearKey;
use crate::state::CharacterState;

/// One item a gear picker offers ([`Engine::gear_options`], [`Engine::usable_options`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GearOption {
    pub item: GearKey,
    /// Whether the Build holds it there already.
    pub held: bool,
    /// The edit that wears it in the place, instead of the item there, or uses it.
    pub edit: Edit,
}

/// A choice a Build can make only because of its gear ([`Engine::gear_dependents`]): the walk allows it with the
/// gear and refuses it without.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GearDependent {
    /// The choice: a pick, or a class or archetype whose entry needs a score.
    pub choice: Choice,
    pub needs: GearNeed,
}

/// The items a choice is allowed only because of.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GearNeed {
    /// With [`all`](Self::all), each item whose removal alone refuses the choice. Otherwise no single removal
    /// does, and these are the items that are each enough alone (a belt or a tome, either giving the score); or,
    /// when none is enough alone either, every item the Build holds.
    pub items: Vec<GearKey>,
    /// Whether every one of the items is needed, rather than any one of them.
    pub all: bool,
}

impl Engine {
    /// Every gear item, in key order.
    pub fn gear(&self) -> &[GearItem] {
        &self.catalog.data().gear
    }

    /// A gear item by key.
    pub fn gear_item(&self, key: &GearKey) -> Option<&GearItem> {
        self.catalog.gear_item(key)
    }

    /// A gear item's name; its key for one the data lacks.
    pub fn gear_name<'a>(&'a self, key: &'a GearKey) -> &'a str {
        self.catalog
            .gear_item(key)
            .map_or(key.as_str(), |item| &item.name)
    }

    /// A gear item's summary, generated from its bonuses ("Dex +2 enhancement"); empty for one the data lacks.
    pub fn gear_summary(&self, key: &GearKey) -> String {
        self.catalog
            .gear_item(key)
            .map_or_else(String::new, |item| item.summary(self.skills()))
    }

    /// Whether a spot takes an item: a place an item of one of its slots ([`GearPlace::takes`]), and a usable
    /// item's own spot that item.
    fn spot_takes(&self, spot: &GearSpot, item: &GearItem) -> bool {
        match spot {
            GearSpot::Worn(place) => place.takes(item.slot),
            GearSpot::Used(key) => item.slot == GearSlot::Usable && *key == item.key,
        }
    }

    /// Whether a Build can hold an item at a spot: [`Issue::UnknownKey`] for a key the data lacks,
    /// [`Issue::NotAllowed`] for an item the spot doesn't take. The walk's rule.
    pub(super) fn gear_fit(&self, spot: &GearSpot, key: &GearKey) -> Result<&GearItem, Issue> {
        let item = self.catalog.gear_item(key).ok_or(Issue::UnknownKey)?;
        if self.spot_takes(spot, item) {
            Ok(item)
        } else {
            Err(Issue::NotAllowed)
        }
    }

    /// A place's picker: every item the place takes, in key order (the body: robes and armours together), each
    /// with the edit that wears it instead of the item there. None is ever locked: an item comes on or off
    /// whatever needs it (the user's decision, ticket 163), and [`Engine::allows`] says whether an edit would
    /// refuse another choice. A ring worn in the other ring place moves ([`Edit::Wear`]). Taking the place's item
    /// off is its withdrawal ([`Engine::withdrawal`] of [`Choice::Gear`]).
    ///
    /// Costs no walk.
    pub fn gear_options(&self, build: &Build, place: GearPlace) -> Vec<GearOption> {
        let worn = build.gear.worn.get(&place);
        (self.gear().iter())
            .filter(|item| place.takes(item.slot))
            .map(|item| GearOption {
                item: item.key.clone(),
                held: worn == Some(&item.key),
                edit: Edit::Wear {
                    place,
                    item: Some(item.key.clone()),
                },
            })
            .collect()
    }

    /// The usable items' picker (the manuals and tomes, an elixir, a potion): every one, in key order, each with
    /// the edit that uses it. They share no place: a Build uses any number, each once. Stopping one is its
    /// withdrawal ([`Engine::withdrawal`] of [`Choice::Gear`] with [`GearSpot::Used`]).
    ///
    /// Costs no walk.
    pub fn usable_options(&self, build: &Build) -> Vec<GearOption> {
        (self.gear().iter())
            .filter(|item| item.slot == GearSlot::Usable)
            .map(|item| GearOption {
                item: item.key.clone(),
                held: build.gear.used.contains(&item.key),
                edit: Edit::Use {
                    item: item.key.clone(),
                    used: true,
                },
            })
            .collect()
    }

    /// The sheet of the Build without its gear: what every number is naked. `None` for a Build that holds no
    /// gear, whose own sheet ([`Engine::evaluate`]) already is that one, so it costs nothing extra. The lines take
    /// its levels as they take the Build's own ([`Engine::totals`], [`Engine::caster_line`],
    /// [`Engine::attack_line`]).
    ///
    /// A choice that only the gear allows fails in this walk and isn't held ([`Engine::gear_dependents`] lists
    /// them), so the naked numbers also lack whatever that choice gave: they are the numbers of the same Build with
    /// no gear, exactly.
    ///
    /// Costs one walk of the Build.
    pub fn naked_sheet(&self, build: &Build) -> Option<Sheet> {
        (!build.gear.is_empty()).then(|| self.evaluate(&without_gear(build)))
    }

    /// The choices the Build can make only because of its gear, in walk order: each choice (a pick, mostly) that
    /// holds up with the gear and fails without it, with the items it needs ([`GearNeed`]). A choice that needs
    /// another gear-dependent one is here too, with the same items. Empty for a Build without gear, at no cost.
    ///
    /// When an item is taken off, what needed it stays in the Build and fails with its usual reason
    /// ([`Engine::failures`]).
    ///
    /// Costs two walks of the Build; when some choice depends on gear, one more for each item held, and one more
    /// again for each when some choice needs no single item.
    pub fn gear_dependents(&self, build: &Build) -> Vec<GearDependent> {
        let dependent = self.dependent_choices(build);
        if dependent.is_empty() {
            return Vec::new();
        }
        let failing = |build: &Build| -> BTreeSet<Choice> {
            (self.failures(build).into_iter())
                .map(|failure| failure.choice)
                .collect()
        };
        let views = GearViews::new(build);
        let without: Vec<BTreeSet<Choice>> = views.without.iter().map(&failing).collect();
        let only = std::cell::OnceCell::new();
        dependent
            .into_iter()
            .map(|choice| {
                let needs = views.need(
                    |at| without[at].contains(&choice),
                    |at| {
                        let only: &Vec<BTreeSet<Choice>> =
                            only.get_or_init(|| views.only.iter().map(&failing).collect());
                        !only[at].contains(&choice)
                    },
                );
                GearDependent { choice, needs }
            })
            .collect()
    }

    /// The choices an edit would leave allowed only because of the Build's gear, that don't need it today: what a
    /// step, an option or a withdrawal that [`Engine::allows`] accepts would quietly lean on the gear for (a
    /// Dexterity taken down to 17 under a belt, with a feat needing 19). Empty when the edited Build holds no gear,
    /// at no cost. The point buy's steps carry it ([`Step::with_gear`](super::Step::with_gear)); ask it of any
    /// other edit before offering it as safe.
    ///
    /// Costs four walks of the Build.
    pub fn newly_gear_dependent(&self, build: &Build, edit: &Edit) -> Vec<Choice> {
        let mut edited = build.clone();
        self.apply(&mut edited, edit);
        if edited.gear.is_empty() {
            return Vec::new();
        }
        let before = self.dependent_choices(build);
        let mut after = self.dependent_choices(&edited);
        after.retain(|choice| !before.contains(choice));
        after
    }

    /// The choices that hold up with the Build's gear and fail without it, in walk order; none for a Build
    /// without gear, at no cost. Two walks.
    fn dependent_choices(&self, build: &Build) -> Vec<Choice> {
        if build.gear.is_empty() {
            return Vec::new();
        }
        let with_gear: BTreeSet<Choice> = (self.failures(build).into_iter())
            .map(|failure| failure.choice)
            .collect();
        let mut dependent: Vec<Choice> = Vec::new();
        for failure in self.failures(&without_gear(build)) {
            if !with_gear.contains(&failure.choice) && !dependent.contains(&failure.choice) {
                dependent.push(failure.choice);
            }
        }
        dependent
    }

    /// The mark on a choice open only because of gear, as the sheet and a picker show it where a story note goes:
    /// "With Belt of Incredible Dexterity +2", "With A and B" when it needs both, "With A or B" when either is
    /// enough.
    pub fn gear_note(&self, needs: &GearNeed) -> String {
        let names: Vec<&str> = (needs.items.iter())
            .map(|item| self.gear_name(item))
            .collect();
        let joined = match names.as_slice() {
            [] => String::new(),
            [one] => (*one).to_owned(),
            [rest @ .., last] => {
                let word = if needs.all { "and" } else { "or" };
                format!("{} {word} {last}", rest.join(", "))
            }
        };
        format!("With {joined}")
    }

    /// What tells which of a slot's options are open only because of gear: `None` for a Build without gear.
    pub(super) fn gear_at_slot(&self, build: &Build, slot: &SlotId) -> Option<GearAtSlot<'_>> {
        (!build.gear.is_empty()).then(|| GearAtSlot {
            engine: self,
            slot: slot.clone(),
            naked: before_slot(self, &without_gear(build), slot),
            views: GearViews::new(build),
            without: std::cell::OnceCell::new(),
            only: std::cell::OnceCell::new(),
        })
    }
}

/// The Build with no gear.
fn without_gear(build: &Build) -> Build {
    Build {
        gear: Gear::default(),
        ..build.clone()
    }
}

/// A Build's gear taken apart, to tell which items a choice needs: the Build without each item, and with each
/// alone.
struct GearViews {
    items: Vec<GearKey>,
    /// The Build without each item, in the items' order.
    without: Vec<Build>,
    /// The Build with each item alone.
    only: Vec<Build>,
}

impl GearViews {
    fn new(build: &Build) -> Self {
        let held: Vec<(GearSpot, GearKey)> = (build.gear.items())
            .map(|(spot, item)| (spot, item.clone()))
            .collect();
        let with = |keep: &dyn Fn(usize) -> bool| -> Build {
            let mut gear = Gear::default();
            for (at, (spot, item)) in held.iter().enumerate() {
                if !keep(at) {
                    continue;
                }
                match spot {
                    GearSpot::Worn(place) => {
                        gear.worn.insert(*place, item.clone());
                    }
                    GearSpot::Used(_) => {
                        gear.used.insert(item.clone());
                    }
                }
            }
            Build {
                gear,
                ..build.clone()
            }
        };
        Self {
            without: (0..held.len())
                .map(|at| with(&|other| other != at))
                .collect(),
            only: (0..held.len())
                .map(|at| with(&|other| other == at))
                .collect(),
            items: held.into_iter().map(|(_, item)| item).collect(),
        }
    }

    /// The items a gear-dependent choice needs: those whose removal alone refuses it (`refused_without`, by the
    /// item's position); when none does, those that alone allow it (`allowed_with_only`); when none does that
    /// either, all of them.
    fn need(
        &self,
        refused_without: impl Fn(usize) -> bool,
        allowed_with_only: impl Fn(usize) -> bool,
    ) -> GearNeed {
        let pick = |wanted: &dyn Fn(usize) -> bool| -> Vec<GearKey> {
            (self.items.iter().enumerate())
                .filter(|(at, _)| wanted(*at))
                .map(|(_, item)| item.clone())
                .collect()
        };
        let needed = pick(&refused_without);
        if !needed.is_empty() {
            return GearNeed {
                items: needed,
                all: true,
            };
        }
        let enough = pick(&allowed_with_only);
        GearNeed {
            items: if enough.is_empty() {
                self.items.clone()
            } else {
                enough
            },
            all: false,
        }
    }
}

/// A slot just before it's filled, in the Build without its gear, without each item and with each alone: which of
/// the slot's open options are open only because of gear, and of which items. The states without each item and
/// with each alone are walked only once an option needs them.
pub(super) struct GearAtSlot<'a> {
    engine: &'a Engine,
    slot: SlotId,
    /// The slot in the Build without gear; `None` when only the gear gives the Build the slot.
    naked: Option<(CharacterState, SlotKind)>,
    views: GearViews,
    without: std::cell::OnceCell<Vec<Option<(CharacterState, SlotKind)>>>,
    only: std::cell::OnceCell<Vec<Option<(CharacterState, SlotKind)>>>,
}

impl GearAtSlot<'_> {
    /// The items a pick open in the slot needs; `None` for one the Build without gear takes there too.
    pub(super) fn need(&self, pick: &Pick) -> Option<GearNeed> {
        let engine = self.engine;
        let fits = |at: &Option<(CharacterState, SlotKind)>| {
            (at.as_ref()).is_some_and(|(state, kind)| pick_fits(engine, state, kind, pick))
        };
        if fits(&self.naked) {
            return None;
        }
        let slots = |builds: &[Build]| -> Vec<Option<(CharacterState, SlotKind)>> {
            (builds.iter())
                .map(|build| before_slot(engine, build, &self.slot))
                .collect()
        };
        Some(self.views.need(
            |at| !fits(&self.without.get_or_init(|| slots(&self.views.without))[at]),
            |at| fits(&self.only.get_or_init(|| slots(&self.views.only))[at]),
        ))
    }
}
