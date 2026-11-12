//! Gear in the engine (ticket 166): the items the data has, what a place or the usable items offer, the Build
//! without its gear (the naked numbers), and the choices a Build can make only because of its gear.
//!
//! How the walk counts gear is in `walk/gear.rs`. What the web app and the MCP server call is here:
//!
//! - [`Engine::gear_options`] and [`Engine::usable_options`]: taking a place's item off and every item the place
//!   takes, or every usable item, each with its edit and whether it can be made now. An item that a choice needs
//!   can't be removed or replaced (the user, 2026-11-12): such an option is locked, with a reason naming the choice.
//! - [`Engine::naked_sheet`]: the sheet of the Build without its gear, for the numbers without gear. `None` for a
//!   Build that holds none, whose own sheet already is that one.
//! - [`Engine::gear_dependents`]: the choices allowed only because of gear, each with the items it needs.
//! - A slot's picker marks an option open only because of gear ([`SlotOption::with_gear`],
//!   [`SlotEntry::gear_note`]); [`Engine::gear_note`] words the mark.
//! - Every other picker that can change a score or a class marks an option that would leave some choice allowed
//!   only because of the gear (`leans_on_gear` on a [`Step`](super::Step), a [`ChoiceOption`](super::ChoiceOption)
//!   and a [`ClassOption`](super::ClassOption)); [`Engine::newly_gear_dependent`] answers the same for any edit.
//!
//! [`SlotOption::with_gear`]: super::SlotOption::with_gear
//! [`SlotEntry::gear_note`]: super::SlotEntry::gear_note

use std::cell::{OnceCell, RefCell};
use std::collections::{BTreeSet, HashMap};

use super::walk::{Cursor, before_slot, pick_fits};
use super::{Address, Engine, Sheet, SlotKind};
use crate::build::{Build, Edit, Gear, GearPlace, GearSpot, Pick, SlotId};
use crate::choice::{Choice, Issue};
use crate::gear::{GearItem, GearSlot};
use crate::keys::GearKey;
use crate::state::CharacterState;

/// One option of a gear picker ([`Engine::gear_options`], [`Engine::usable_options`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GearOption {
    /// The item; `None` for a place's "None", which takes the place's item off.
    pub item: Option<GearKey>,
    /// Whether the Build is that way already: the item is the one worn there (the place is empty, for "None"), or
    /// the usable item is used.
    pub held: bool,
    /// `Ok` when the edit can be made now; [`Issue::BreaksLater`] when it would refuse a choice the Build holds: an
    /// item that a choice needs can't be removed or replaced until the choice is cleared. What the Build holds is
    /// always `Ok`.
    pub status: Result<(), Issue>,
    /// The choices the edit would refuse, in walk order; empty for an open option.
    pub needed_by: Vec<Choice>,
    /// Why it is locked, naming those choices: "Greater Two-Weapon Fighting needs Belt of Incredible Dexterity
    /// +2", or, for an item whose own penalty is the cause, "Scholar's Headband would refuse Persuasive". `None`
    /// for an open option.
    pub reason: Option<String>,
    /// The edit: a place's wears the item instead of the one there ("None": takes it off); a usable item's uses
    /// it, or stops using it when it is used.
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

/// The items a choice is allowed only because of. At least one of the lists holds an item.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GearNeed {
    /// The items it needs, each one: taking any of them off alone refuses the choice.
    pub all_of: Vec<GearKey>,
    /// Beside those, it needs one more item, and any one of these is enough.
    pub one_of: Vec<GearKey>,
    /// Beside [`all_of`](Self::all_of), it needs two or more further items and no single one is enough: these are
    /// one set that is, though others may be too (any two of three belts' worth). Empty unless
    /// [`one_of`](Self::one_of) is.
    pub for_example: Vec<GearKey>,
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

    /// A place's picker: "None" first (the edit that takes the place's item off), then every item the place takes,
    /// in key order (the body: robes and armours together), each with the edit that wears it instead of the item
    /// there. A ring worn in the other ring place moves ([`Edit::Wear`]).
    ///
    /// An item that a choice needs can't be removed or replaced (the user, 2026-11-12): "None" and every item that
    /// wouldn't keep the Build possible are [`Issue::BreaksLater`], with the choices they would refuse and a reason
    /// naming them. An item that keeps every choice allowed is open. The same verdict as [`Engine::allows`] of the
    /// option's edit, and as the place's withdrawal ([`Engine::withdrawal`] of [`Choice::Gear`]).
    ///
    /// Costs a walk of the Build for each option but the one held, and for a locked one a second.
    pub fn gear_options(&self, build: &Build, place: GearPlace) -> Vec<GearOption> {
        let spot = GearSpot::Worn(place);
        let worn = build.gear.worn.get(&place);
        let judge = GearJudge::new(self, build, &spot);
        let wear = |item: Option<&GearKey>| Edit::Wear {
            place,
            item: item.cloned(),
        };
        let none = judge.option(None, worn.is_none(), wear(None));
        let items = (self.gear().iter())
            .filter(|item| place.takes(item.slot))
            .map(|item| {
                let held = worn == Some(&item.key);
                judge.option(Some(&item.key), held, wear(Some(&item.key)))
            });
        std::iter::once(none).chain(items).collect()
    }

    /// The usable items' picker (the manuals and tomes, an elixir, a potion): every one, in key order, as a tick
    /// each. They share no place: a Build uses any number, each once. An option's edit uses its item, or stops
    /// using it when it is used ([`GearOption::held`]); stopping one that a choice needs is locked, as in
    /// [`gear_options`](Self::gear_options).
    ///
    /// Costs a walk of the Build for each item, and for a locked one a second.
    pub fn usable_options(&self, build: &Build) -> Vec<GearOption> {
        (self.gear().iter())
            .filter(|item| item.slot == GearSlot::Usable)
            .map(|item| {
                let held = build.gear.used.contains(&item.key);
                let spot = GearSpot::Used(item.key.clone());
                let edit = Edit::Use {
                    item: item.key.clone(),
                    used: !held,
                };
                // The tick itself is the option: held or not, its edit changes the Build.
                let mut option =
                    GearJudge::new(self, build, &spot).option(Some(&item.key), false, edit);
                option.held = held;
                option
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
    /// The items such a choice needs can't be removed or replaced while the Build holds the choice
    /// ([`gear_options`](Self::gear_options)).
    ///
    /// Costs two walks of the Build; when some choice depends on gear, one more for each item held, and a few more
    /// for each choice that needs more than the items it can't do without.
    pub fn gear_dependents(&self, build: &Build) -> Vec<GearDependent> {
        let dependent = self.dependent_choices(build);
        if dependent.is_empty() {
            return Vec::new();
        }
        let views = GearViews::new(build);
        // The choices that fail with each set of the items, walked once per set.
        let failing: RefCell<HashMap<Vec<bool>, BTreeSet<Choice>>> = RefCell::default();
        dependent
            .into_iter()
            .map(|choice| {
                let needs = views.need(|kept| {
                    let mut failing = failing.borrow_mut();
                    let fails = failing
                        .entry(kept.to_vec())
                        .or_insert_with(|| self.failing_choices(&views.with(kept)));
                    !fails.contains(&choice)
                });
                GearDependent { choice, needs }
            })
            .collect()
    }

    /// The Build without its gear and what tells which of a picker's options would lean on the gear
    /// ([`GearLean`]); `None` for a Build without gear. A picker keeps the first and builds the second on it:
    /// `let naked = engine.naked_build(build); let lean = naked.as_ref().map(|naked| GearLean::new(..))`.
    pub(super) fn naked_build(&self, build: &Build) -> Option<Build> {
        (!build.gear.is_empty()).then(|| without_gear(build))
    }

    /// The choices an edit would leave allowed only because of the Build's gear, that don't need it today: what a
    /// step, an option or a withdrawal that [`Engine::allows`] accepts would quietly lean on the gear for (a
    /// Dexterity taken down to 17 under a belt, with a feat needing 19; a class entered on a headband's
    /// Intelligence). Empty when the edited Build holds no gear, at no cost. The pickers that can change a score or
    /// a class carry it on each open option (`leans_on_gear`); ask it of any other edit before offering it as safe.
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

    /// The choices that fail in a Build, as a set.
    pub(super) fn failing_choices(&self, build: &Build) -> BTreeSet<Choice> {
        (self.failures(build).into_iter())
            .map(|failure| failure.choice)
            .collect()
    }

    /// The choices that hold up with the Build's gear and fail without it, in walk order; none for a Build
    /// without gear, at no cost. Two walks.
    fn dependent_choices(&self, build: &Build) -> Vec<Choice> {
        if build.gear.is_empty() {
            return Vec::new();
        }
        let with_gear = self.failing_choices(build);
        let mut dependent: Vec<Choice> = Vec::new();
        for failure in self.failures(&without_gear(build)) {
            if !with_gear.contains(&failure.choice) && !dependent.contains(&failure.choice) {
                dependent.push(failure.choice);
            }
        }
        dependent
    }

    /// The mark on a choice open only because of gear, as the sheet and a picker show it where a story note goes:
    /// "With Belt of Incredible Dexterity +2"; "With A and B" when it needs both; "With A or B" when either is
    /// enough; "With A and one of B or C"; and, when it needs two or more items of which none is needed for sure,
    /// "With gear such as A and B", one set that is enough.
    pub fn gear_note(&self, needs: &GearNeed) -> String {
        let list = |items: &[GearKey], word: &str| -> String {
            let names: Vec<&str> = items.iter().map(|item| self.gear_name(item)).collect();
            match names.as_slice() {
                [] => String::new(),
                [one] => (*one).to_owned(),
                [rest @ .., last] => format!("{} {word} {last}", rest.join(", ")),
            }
        };
        let all_of = list(&needs.all_of, "and");
        match (
            needs.all_of.is_empty(),
            needs.one_of.is_empty(),
            needs.for_example.is_empty(),
        ) {
            (_, false, _) if needs.all_of.is_empty() => {
                format!("With {}", list(&needs.one_of, "or"))
            }
            (_, false, _) => format!("With {all_of} and one of {}", list(&needs.one_of, "or")),
            (true, true, false) => {
                format!("With gear such as {}", list(&needs.for_example, "and"))
            }
            (false, true, false) => format!(
                "With {all_of} and more gear, such as {}",
                list(&needs.for_example, "and")
            ),
            (_, true, true) => format!("With {all_of}"),
        }
    }

    /// The choices, named as a reason names them, joined: a pick by what it takes ("Greater Two-Weapon Fighting"),
    /// a level's class by its level and class ("Level 3: Wizard"), any other by its label
    /// ([`placed_label`](Self::placed_label)). `build` is the Build that holds them.
    pub fn choice_names(&self, build: &Build, choices: &[Choice]) -> String {
        let names: Vec<String> = choices
            .iter()
            .map(|choice| match choice {
                Choice::Slot(slot) => match build.picks.get(slot) {
                    Some(pick) => self.pick_name(pick),
                    None => self.placed_label(&Address::from(choice), None),
                },
                Choice::Class { level } => {
                    let class = usize::from(*level)
                        .checked_sub(1)
                        .and_then(|index| build.levels.get(index))
                        .map_or("Class", |level_up| self.class_name(&level_up.class));
                    format!("Level {level}: {class}")
                }
                choice => self.placed_label(&Address::from(choice), None),
            })
            .collect();
        match names.as_slice() {
            [] => String::new(),
            [one] => one.clone(),
            [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
        }
    }

    /// What tells which of a slot's options are open only because of gear: `None` for a Build without gear.
    pub(super) fn gear_at_slot(&self, build: &Build, slot: &SlotId) -> Option<GearAtSlot<'_>> {
        (!build.gear.is_empty()).then(|| GearAtSlot {
            engine: self,
            slot: slot.clone(),
            views: GearViews::new(build),
            slots: RefCell::default(),
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

/// The verdict on each option of one gear spot: whether its edit keeps the Build possible, and what it would refuse
/// when it doesn't.
struct GearJudge<'a> {
    engine: &'a Engine,
    build: &'a Build,
    /// The item the spot holds now.
    current: Option<&'a GearKey>,
    /// The choices that fail in the Build as it is (none, in a possible one).
    failing: OnceCell<BTreeSet<Choice>>,
    /// The choices that taking the spot's item off would refuse: those that need it.
    need_current: OnceCell<Vec<Choice>>,
    spot: &'a GearSpot,
}

impl<'a> GearJudge<'a> {
    fn new(engine: &'a Engine, build: &'a Build, spot: &'a GearSpot) -> Self {
        Self {
            engine,
            build,
            current: build.gear.at(spot),
            failing: OnceCell::new(),
            need_current: OnceCell::new(),
            spot,
        }
    }

    /// The choices an edit would refuse, in walk order: those that fail in the edited Build and not in the Build.
    fn refused_by(&self, edit: &Edit) -> Vec<Choice> {
        let engine = self.engine;
        let failing = self
            .failing
            .get_or_init(|| engine.failing_choices(self.build));
        let mut edited = self.build.clone();
        engine.apply(&mut edited, edit);
        let mut refused: Vec<Choice> = Vec::new();
        for failure in engine.failures(&edited) {
            if !failing.contains(&failure.choice) && !refused.contains(&failure.choice) {
                refused.push(failure.choice);
            }
        }
        refused
    }

    /// An option: open when the Build holds it already or its edit keeps the Build possible; otherwise locked, with
    /// what it would refuse and why. The reason blames the spot's item when the refused choices all need it, and
    /// the option's item otherwise (its own penalty refuses them).
    fn option(&self, item: Option<&GearKey>, held: bool, edit: Edit) -> GearOption {
        let (engine, build) = (self.engine, self.build);
        let open = held || engine.allows(build, &edit);
        let needed_by = if open {
            Vec::new()
        } else {
            self.refused_by(&edit)
        };
        let reason = (!open).then(|| {
            let names = engine.choice_names(build, &needed_by);
            let take_off = match self.spot {
                GearSpot::Worn(place) => Edit::Wear {
                    place: *place,
                    item: None,
                },
                GearSpot::Used(item) => Edit::Use {
                    item: item.clone(),
                    used: false,
                },
            };
            let need_current = self.need_current.get_or_init(|| match self.current {
                Some(_) => self.refused_by(&take_off),
                None => Vec::new(),
            });
            match (self.current, item) {
                (Some(current), _) if needed_by.iter().all(|c| need_current.contains(c)) => {
                    let verb = if needed_by.len() == 1 {
                        "needs"
                    } else {
                        "need"
                    };
                    format!("{names} {verb} {}", engine.gear_name(current))
                }
                (_, Some(item)) => format!("{} would refuse {names}", engine.gear_name(item)),
                (_, None) => format!("It would refuse {names}"),
            }
        });
        GearOption {
            item: item.cloned(),
            held,
            status: if open {
                Ok(())
            } else {
                Err(Issue::BreaksLater)
            },
            needed_by,
            reason,
            edit,
        }
    }
}

/// What tells, for a picker's open options, which would leave some choice allowed only because of the Build's gear:
/// the walk of the Build without its gear, asked each option's edit. Built once per picker, for a Build with gear;
/// what the Build leans on already is worked out once, when the first open option asks.
pub(super) struct GearLean<'a> {
    engine: &'a Engine,
    build: &'a Build,
    naked: &'a Build,
    cursor: Cursor<'a>,
    /// The choices whose failing without the gear is nothing new: those that fail in the Build as it is, and those
    /// that lean on the gear already.
    known: OnceCell<Vec<Choice>>,
}

impl<'a> GearLean<'a> {
    /// `naked` is `build` without its gear ([`without_gear`]).
    pub(super) fn new(engine: &'a Engine, build: &'a Build, naked: &'a Build) -> Self {
        Self {
            engine,
            build,
            naked,
            cursor: Cursor::new(engine, naked),
            known: OnceCell::new(),
        }
    }

    /// What an option with this status and edit leans on: nothing for a locked one, or without gear (`lean`).
    pub(super) fn lean(
        lean: Option<&Self>,
        status: &Result<(), Issue>,
        edit: &Edit,
    ) -> Vec<Choice> {
        match (lean, status) {
            (Some(lean), Ok(())) => lean.leaning(edit),
            _ => Vec::new(),
        }
    }

    /// The choices an open option's edit would newly leave allowed only because of the gear, in walk order: those
    /// the edit refuses in the Build without its gear and not in the Build with it. The edit's own choice counts (a
    /// class whose entry only the gear meets). Most edits refuse none, which one walk from the edit's level tells.
    fn leaning(&self, edit: &Edit) -> Vec<Choice> {
        let engine = self.engine;
        let known = self.known.get_or_init(|| {
            let mut known = engine.dependent_choices(self.build);
            known.extend(engine.failing_choices(self.build));
            known
        });
        if self.cursor.keeps(edit, known.clone()) {
            return Vec::new();
        }
        // What the edit refuses whatever is worn (an advancement its own edit leaves open) is no leaning.
        let mut with_gear = self.build.clone();
        engine.apply(&mut with_gear, edit);
        let refused = engine.failing_choices(&with_gear);
        let mut edited = self.naked.clone();
        engine.apply(&mut edited, edit);
        let mut leaning: Vec<Choice> = Vec::new();
        for failure in engine.failures(&edited) {
            let choice = failure.choice;
            if !known.contains(&choice) && !refused.contains(&choice) && !leaning.contains(&choice)
            {
                leaning.push(choice);
            }
        }
        leaning
    }
}

/// A Build's gear taken apart, to tell which items a choice needs: the Build with any set of its items.
struct GearViews {
    build: Build,
    held: Vec<(GearSpot, GearKey)>,
}

impl GearViews {
    fn new(build: &Build) -> Self {
        Self {
            build: build.clone(),
            held: (build.gear.items())
                .map(|(spot, item)| (spot, item.clone()))
                .collect(),
        }
    }

    /// The Build with only the items `kept` marks, in the items' order.
    fn with(&self, kept: &[bool]) -> Build {
        let mut gear = Gear::default();
        for ((spot, item), kept) in self.held.iter().zip(kept) {
            match spot {
                _ if !kept => {}
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
            ..self.build.clone()
        }
    }

    /// The items a gear-dependent choice needs, given whether it is allowed with each set of the items (`allowed`,
    /// asked a mark per item): the items it can't do without (taking one off alone refuses it); then, when those
    /// alone aren't enough, the items of which any one more is; and when no single one more is, one set of further
    /// items that is enough, found by dropping each other item that can go.
    fn need(&self, allowed: impl Fn(&[bool]) -> bool) -> GearNeed {
        let count = self.held.len();
        let keys = |at: &[usize]| -> Vec<GearKey> {
            at.iter().map(|at| self.held[*at].1.clone()).collect()
        };
        let marks =
            |kept: &[usize]| -> Vec<bool> { (0..count).map(|at| kept.contains(&at)).collect() };
        let required: Vec<usize> = (0..count)
            .filter(|at| {
                let rest: Vec<usize> = (0..count).filter(|other| other != at).collect();
                !allowed(&marks(&rest))
            })
            .collect();
        let all_of = keys(&required);
        if !required.is_empty() && allowed(&marks(&required)) {
            return GearNeed {
                all_of,
                ..GearNeed::default()
            };
        }
        let others: Vec<usize> = (0..count).filter(|at| !required.contains(at)).collect();
        let one_of: Vec<usize> = (others.iter().copied())
            .filter(|at| {
                let mut kept = required.clone();
                kept.push(*at);
                allowed(&marks(&kept))
            })
            .collect();
        if !one_of.is_empty() {
            return GearNeed {
                all_of,
                one_of: keys(&one_of),
                ..GearNeed::default()
            };
        }
        // Two or more further items: drop each that can go, and what is left is one set that is enough.
        let mut kept: Vec<usize> = (0..count).collect();
        for at in &others {
            let without: Vec<usize> = kept.iter().copied().filter(|k| k != at).collect();
            if allowed(&marks(&without)) {
                kept = without;
            }
        }
        kept.retain(|at| !required.contains(at));
        GearNeed {
            all_of,
            for_example: keys(&kept),
            ..GearNeed::default()
        }
    }
}

/// The state just before a slot is filled, and what the slot accepts.
type BeforeSlot = (CharacterState, SlotKind);

/// A slot just before it's filled, in the Build with any set of its items: which of the slot's open options are
/// open only because of gear, and of which items. Each set's state is walked once, when an option first asks.
pub(super) struct GearAtSlot<'a> {
    engine: &'a Engine,
    slot: SlotId,
    views: GearViews,
    /// The slot with each set of the items asked about so far; `None` where that Build has no such slot.
    slots: RefCell<HashMap<Vec<bool>, Option<BeforeSlot>>>,
}

impl GearAtSlot<'_> {
    /// Whether the slot takes `pick` in the Build with the items `kept` marks.
    fn fits(&self, kept: &[bool], pick: &Pick) -> bool {
        let engine = self.engine;
        let mut slots = self.slots.borrow_mut();
        let at = slots
            .entry(kept.to_vec())
            .or_insert_with(|| before_slot(engine, &self.views.with(kept), &self.slot));
        (at.as_ref()).is_some_and(|(state, kind)| pick_fits(engine, state, kind, pick))
    }

    /// The items a pick open in the slot needs; `None` for one the Build without gear takes there too.
    pub(super) fn need(&self, pick: &Pick) -> Option<GearNeed> {
        let none = vec![false; self.views.held.len()];
        if self.fits(&none, pick) {
            return None;
        }
        Some(self.views.need(|kept| self.fits(kept, pick)))
    }
}
