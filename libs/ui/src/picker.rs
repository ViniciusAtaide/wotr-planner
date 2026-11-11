//! The picker: a bottom sheet on a narrow screen (the Fold's cover), a centred modal from 600px up. One list
//! behaviour for every choice: search, category chips, the locked options greyed behind "Show N locked" with
//! their reason (none for one that would break a later choice: it just stays grey), and a Remove button
//! only for a choice the Build holds and may remove now (`Engine::withdrawal`), which takes it back by its edit.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use leptos::prelude::*;
use rules::{
    Address, Alignment, Build, Choice, ClassOption, Edit, Engine, FIRST_ASCENSION_RANK, Faith,
    FeatCategory, Issue, Place, SlotEntry, SlotId, SlotOption,
};

use crate::icons::feat_icon;
use crate::planner::{Target, use_planner};
use crate::timings::timed;

/// One row of a picker list.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub name: String,
    pub detail: String,
    /// The chip it's filed under; empty for none.
    pub category: String,
    /// The category of the feat it takes, which picks its icon; `None` for an item that takes no feat.
    pub feat_category: Option<FeatCategory>,
    /// It can't be taken: greyed out, behind "Show N locked".
    pub locked: bool,
    /// Why not, when the sheet says: an option that would break a later choice stays grey with none.
    pub reason: Option<String>,
    /// What the Build holds now.
    pub current: bool,
    /// Opens a list of values instead of picking.
    pub more: bool,
}

impl Item {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            detail: String::new(),
            category: String::new(),
            feat_category: None,
            locked: false,
            reason: None,
            current: false,
            more: false,
        }
    }
}

/// The open picker, if any.
#[component]
pub fn Picker() -> AnyView {
    let planner = use_planner();
    (move || {
        planner.picker.get().map(|target| {
            view! {
                <div class="scrim" on:click=move |_| planner.close()></div>
                <section class="picker" role="dialog" aria-modal="true">
                    {body(target)}
                </section>
            }
        })
    })
    .into_any()
}

fn body(target: Target) -> AnyView {
    match target {
        Target::Slot(id) => view! { <SlotPicker id/> }.into_any(),
        Target::Class(level) => view! { <ClassPicker level/> }.into_any(),
        Target::FirstAscension => view! { <FirstAscensionPicker/> }.into_any(),
        Target::Path => view! { <PathPicker switch=false/> }.into_any(),
        Target::Switch => view! { <PathPicker switch=true/> }.into_any(),
        Target::Race => view! { <RacePicker/> }.into_any(),
        Target::Alignment => view! { <AlignmentPicker/> }.into_any(),
        Target::Deity => view! { <DeityPicker/> }.into_any(),
        Target::Background => view! { <BackgroundPicker/> }.into_any(),
        Target::AbilityIncrease(level) => view! { <AbilityIncreasePicker level/> }.into_any(),
        Target::Skills(level) => view! { <crate::start::SkillsPanel level/> }.into_any(),
    }
}

/// The picker's header: what's being chosen, where, a remove control when the Build holds the choice and may remove
/// it now, and close.
#[component]
pub fn Head(
    #[prop(into)] title: String,
    #[prop(into, optional)] place: String,
    /// The choice the remove control takes back, by the engine's edit; shown only while the Build holds it and it
    /// is removable.
    #[prop(default = None)]
    remove: Option<Choice>,
    /// Goes back a step.
    #[prop(default = None)]
    back: Option<Callback<()>>,
) -> impl IntoView {
    let planner = use_planner();
    let remove = remove.map(|choice| {
        move || {
            let (edit, true) = planner.withdrawal(&choice)? else {
                return None;
            };
            let edit = StoredValue::new(edit);
            Some(view! {
                <button
                    class="remove"
                    title="Remove this choice"
                    on:click=move |_| planner.pick(edit.get_value())
                >
                    "Remove"
                </button>
            })
        }
    });
    view! {
        <header class="picker-head">
            {back
                .map(|back| {
                    view! {
                        <button class="icon" aria-label="Back" on:click=move |_| back.run(())>
                            "‹"
                        </button>
                    }
                })}
            <div class="picker-title">
                <h2>{title}</h2>
                {(!place.is_empty()).then(|| view! { <p>{place}</p> })}
            </div>
            {remove}
            <button class="icon" aria-label="Close" on:click=move |_| planner.close()>
                "✕"
            </button>
        </header>
    }
}

/// A searchable list with category chips, the locked items behind a toggle.
#[component]
pub fn ItemList(
    items: Vec<Item>,
    #[prop(into)] on_pick: Callback<usize>,
    /// What an empty search says.
    #[prop(optional)]
    nothing: Option<&'static str>,
) -> impl IntoView {
    let searchable = items.len() > 8;
    // Each filter chip once, in name order, with a feat category's icon only when every item under it takes a feat
    // of that one category: a feat picker's "Weapon" chip, not a talent picker's "Rogue Talents", whose talents
    // grant different feats or none.
    let mut shared: BTreeMap<String, Option<FeatCategory>> = BTreeMap::new();
    for item in items.iter().filter(|item| !item.category.is_empty()) {
        shared
            .entry(item.category.clone())
            .and_modify(|category| {
                if *category != item.feat_category {
                    *category = None;
                }
            })
            .or_insert(item.feat_category);
    }
    let categories: Vec<(String, Option<FeatCategory>)> = shared.into_iter().collect();
    let items = StoredValue::new(items);
    let query = RwSignal::new(String::new());
    let category = RwSignal::new(None::<String>);
    let show_locked = RwSignal::new(false);
    let matching = move |locked: bool| -> Vec<(usize, Item)> {
        let query = query.get().to_lowercase();
        let category = category.get();
        items.with_value(|items| {
            items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.locked == locked)
                .filter(|(_, item)| category.as_ref().is_none_or(|c| &item.category == c))
                .filter(|(_, item)| {
                    query.is_empty()
                        || item.name.to_lowercase().contains(&query)
                        || item.detail.to_lowercase().contains(&query)
                })
                .map(|(index, item)| (index, item.clone()))
                .collect()
        })
    };
    let row = move |(index, item): (usize, Item)| {
        let locked = item.locked;
        view! {
            <li>
                <button
                    class="item"
                    class:current=item.current
                    disabled=locked
                    aria-current=item.current.then_some("true")
                    on:click=move |_| on_pick.run(index)
                >
                    <span class="item-name">
                        {item.feat_category.map(feat_icon)}
                        {item.name}
                        {item.current.then_some(" ✓")}
                        {item.more.then(|| view! { <span class="more">" ›"</span> })}
                    </span>
                    {(!item.detail.is_empty())
                        .then(|| view! { <span class="item-detail">{item.detail}</span> })}
                    {item.reason.map(|reason| view! { <span class="item-reason">{reason}</span> })}
                </button>
            </li>
        }
    };
    view! {
        <div class="picker-tools">
            {searchable
                .then(|| {
                    view! {
                        <input
                            class="search"
                            type="search"
                            placeholder="Search"
                            aria-label="Search"
                            prop:value=move || query.get()
                            on:input=move |event| query.set(event_target_value(&event))
                        />
                    }
                })}
            {(categories.len() > 1)
                .then(|| {
                    let chips = categories
                        .into_iter()
                        .map(|(name, feat_category)| {
                            let on = {
                                let name = name.clone();
                                move || category.get().as_ref() == Some(&name)
                            };
                            let set = name.clone();
                            view! {
                                <button
                                    class="chip-filter"
                                    class:on=on
                                    on:click=move |_| category.set(Some(set.clone()))
                                >
                                    {feat_category.map(feat_icon)}
                                    {name}
                                </button>
                            }
                        })
                        .collect_view();
                    view! {
                        <div class="filters" role="group" aria-label="Categories">
                            <button
                                class="chip-filter"
                                class:on=move || category.get().is_none()
                                on:click=move |_| category.set(None)
                            >
                                "All"
                            </button>
                            {chips}
                        </div>
                    }
                })}
        </div>
        <div class="picker-list">
            <ul class="items">{move || matching(false).into_iter().map(row).collect_view()}</ul>
            {move || {
                let locked = matching(true);
                let open = matching(false);
                let count = locked.len();
                view! {
                    {(open.is_empty() && count == 0)
                        .then(|| {
                            view! { <p class="empty">{nothing.unwrap_or("Nothing matches.")}</p> }
                        })}
                    {(count > 0)
                        .then(|| {
                            view! {
                                <button
                                    class="show-locked"
                                    aria-expanded=move || show_locked.get().to_string()
                                    on:click=move |_| show_locked.update(|shown| *shown = !*shown)
                                >
                                    {move || {
                                        if show_locked.get() {
                                            format!("Hide {count} locked")
                                        } else {
                                            format!("Show {count} locked")
                                        }
                                    }}
                                </button>
                            }
                        })}
                    {move || {
                        show_locked
                            .get()
                            .then(|| {
                                view! {
                                    <ul class="items locked">
                                        {locked.clone().into_iter().map(row).collect_view()}
                                    </ul>
                                }
                            })
                    }}
                }
            }}
        </div>
    }
}

/// A picker's step, which starts out loading and gets its list after the first paint ([`after_paint`]).
trait Step<T> {
    /// Moves from loading to the list; any other step stays. Whether the step changed.
    fn list(&mut self, list: T) -> bool;
}

/// A picker with one step: `None` while loading, then the list.
impl<T> Step<T> for Option<T> {
    fn list(&mut self, list: T) -> bool {
        let loading = self.is_none();
        if loading {
            *self = Some(list);
        }
        loading
    }
}

/// Works out a picker's list after the browser has painted, so the picker opens at once and shows that it's working
/// meanwhile, then moves its step from loading to the list. A picker closed before then has disposed `step` and every
/// value the work reads, so the work is skipped.
fn after_paint<T: 'static, S: Step<T> + Send + Sync + 'static>(
    step: RwSignal<S>,
    work: impl FnOnce() -> T + 'static,
) {
    request_animation_frame(move || {
        set_timeout(
            move || {
                if !step.is_disposed() {
                    let list = work();
                    step.maybe_update(|step| step.list(list));
                }
            },
            Duration::ZERO,
        )
    });
}

fn loading() -> AnyView {
    view! { <p class="empty">"Working out the options…"</p> }.into_any()
}

/// The reason a locked option shows, when it shows one.
fn reason(engine: &Engine, status: &Result<(), Issue>) -> Option<String> {
    status
        .as_ref()
        .err()
        .and_then(|issue| engine.issue_text(issue))
}

/// Where a choice sits, as a heading says it: "Build start", "Level 3" or "Mythic rank 4" ([`Place::heading`]).
fn place_heading(address: &Address) -> String {
    address.place().map_or_else(String::new, Place::heading)
}

/// A picker's heading for a choice: what the engine labels it ([`Engine::choice_label`]) and where it sits.
pub fn heading(engine: &Engine, choice: &Choice) -> (String, String) {
    let address = Address::from(choice);
    (engine.choice_label(&address, None), place_heading(&address))
}

/// A slot's picker: the engine's entries (one per feat, option or mythic pick), an entry of values opening the list of
/// its options.
#[component]
fn SlotPicker(id: SlotId) -> impl IntoView {
    let slot = id;
    let planner = use_planner();
    let engine = planner.engine();
    let Some(found) = planner
        .sheet
        .with_untracked(|sheet| sheet.slot(&slot).cloned())
    else {
        return view! {
            <Head title="Not in the Build"/>
            <p class="empty">"This slot isn't in the Build any more."</p>
        }
        .into_any();
    };
    let current = found.pick.clone();
    let title = engine.choice_label(&Address::Slot(slot.clone()), Some(&found));
    let remove = Some(Choice::Slot(slot.clone()));

    let step = RwSignal::new(SlotStep::Loading);
    {
        let slot = slot.clone();
        after_paint(step, move || {
            let build = planner.build.get_untracked();
            let entries =
                timed("options", || engine.slot_entries(&build, &slot)).unwrap_or_default();
            Arc::new(entries)
        });
    }
    let place = place_heading(&Address::Slot(slot.clone()));
    let current = StoredValue::new(current);
    let is_current = move |option: &SlotOption| {
        current.with_value(|current| current.as_ref() == Some(&option.pick))
    };
    let pick = move |option: &SlotOption| planner.pick(option.edit.clone());

    let list = move || match step.get() {
        SlotStep::Loading => loading(),
        SlotStep::List(entries) => {
            let items = entries
                .iter()
                .map(|entry| Item {
                    name: entry.name.clone(),
                    detail: entry.detail(),
                    category: entry.category.clone(),
                    feat_category: entry.feat_category,
                    locked: !entry.open,
                    reason: entry.reason.clone(),
                    current: entry
                        .options
                        .iter()
                        .any(|option| is_current(&option.option)),
                    more: entry.noun.is_some(),
                })
                .collect();
            view! {
                <ItemList
                    items
                    on_pick=move |index: usize| {
                        let entry = &entries[index];
                        if entry.noun.is_some() {
                            step.set(SlotStep::Values {
                                entries: entries.clone(),
                                entry: index,
                            });
                        } else {
                            pick(&entry.options[0].option);
                        }
                    }
                />
            }
            .into_any()
        }
        SlotStep::Values { entries, entry } => {
            let name = entries[entry].name.clone();
            let items = entries[entry]
                .options
                .iter()
                .map(|option| Item {
                    locked: option.option.status.is_err(),
                    reason: option.reason.clone(),
                    current: is_current(&option.option),
                    ..Item::new(option.value.clone())
                })
                .collect();
            view! {
                <p class="picker-step">{name}</p>
                <ItemList
                    items
                    on_pick=move |option: usize| pick(&entries[entry].options[option].option)
                />
            }
            .into_any()
        }
    };
    // The header changes only between the list and an entry's values (back), not when the list arrives.
    let values = Memo::new(move |_| matches!(step.get(), SlotStep::Values { .. }));
    view! {
        {move || {
            let back = values.get().then(|| Callback::new(move |()| step.update(SlotStep::back)));
            view! { <Head title=title.clone() place=place.clone() remove=remove.clone() back/> }
        }}
        {list}
    }
    .into_any()
}

/// A slot picker's step: its entries being worked out, then their list, then an entry's values.
#[derive(Clone)]
enum SlotStep {
    Loading,
    List(Arc<Vec<SlotEntry>>),
    Values {
        entries: Arc<Vec<SlotEntry>>,
        entry: usize,
    },
}

impl SlotStep {
    /// Back from an entry's values to the list.
    fn back(&mut self) {
        if let Self::Values { entries, .. } = self {
            *self = Self::List(entries.clone());
        }
    }
}

impl Step<Arc<Vec<SlotEntry>>> for SlotStep {
    fn list(&mut self, entries: Arc<Vec<SlotEntry>>) -> bool {
        let loading = matches!(self, Self::Loading);
        if loading {
            *self = Self::List(entries);
        }
        loading
    }
}

/// Whether the Build holds this class option at the level now: the level's class, with the option's archetype.
fn holds_class_option(build: &Build, level: u8, option: &ClassOption) -> bool {
    usize::from(level)
        .checked_sub(1)
        .and_then(|index| build.levels.get(index))
        .is_some_and(|level_up| {
            level_up.class == option.class
                && build
                    .class_choices
                    .get(&option.class)
                    .map(|choice| &choice.archetype)
                    == Some(&option.archetype)
        })
}

#[component]
fn ClassPicker(level: u8) -> impl IntoView {
    let planner = use_planner();
    let engine = StoredValue::new(planner.engine());
    let step = RwSignal::new(ClassStep::Loading);
    after_paint(step, move || {
        let build = planner.build.get_untracked();
        let options = engine.with_value(|engine| {
            timed("class_options", || engine.class_options(&build, level)).unwrap_or_default()
        });
        Arc::new(options)
    });
    let adding = planner
        .build
        .with_untracked(|build| usize::from(level) > build.levels.len());
    let (title, place) = engine.with_value(|engine| heading(engine, &Choice::Class { level }));
    let title = if adding {
        "Add a class".to_owned()
    } else {
        title
    };
    // The advancement step keeps the level: "Level 7: Spellcasting advancement".
    let advancement_place = Address::Class(level).placed("Spellcasting advancement");

    // A prestige class first taken here asks which classes it advances.
    let choose = move |options: Arc<Vec<ClassOption>>, option: ClassOption| {
        let asks_advances = planner.build.with_untracked(|build| {
            engine.with_value(|engine| engine.asks_advances(build, level, &option.class))
        });
        if asks_advances {
            step.set(ClassStep::Advances {
                options,
                option: Box::new(option),
            });
        } else {
            planner.pick(option.edit);
        }
    };

    move || {
        let list = match step.get() {
            ClassStep::Advances { options, option } => {
                return view! {
                    <Head
                        title=title.clone()
                        place=advancement_place.clone()
                        back=Some(Callback::new(move |()| step.set(ClassStep::List(options.clone()))))
                    />
                    <Advances level option=*option/>
                }
                .into_any();
            }
            ClassStep::Loading => loading(),
            ClassStep::List(options) => {
                let build = planner.build.get_untracked();
                let items = engine.with_value(|engine| {
                    options
                        .iter()
                        .map(|option| class_item(engine, &build, level, option))
                        .collect()
                });
                view! {
                    <ItemList
                        items
                        on_pick=move |index: usize| choose(options.clone(), options[index].clone())
                    />
                }
                .into_any()
            }
        };
        view! {
            <Head title=title.clone() place=place.clone()/>
            {list}
        }
        .into_any()
    }
}

/// A class picker's step: its options being worked out, then their list, then which classes the chosen prestige
/// class advances.
#[derive(Clone)]
enum ClassStep {
    Loading,
    List(Arc<Vec<ClassOption>>),
    Advances {
        options: Arc<Vec<ClassOption>>,
        /// Boxed: the other steps are a pointer each.
        option: Box<ClassOption>,
    },
}

impl Step<Arc<Vec<ClassOption>>> for ClassStep {
    fn list(&mut self, options: Arc<Vec<ClassOption>>) -> bool {
        let loading = matches!(self, Self::Loading);
        if loading {
            *self = Self::List(options);
        }
        loading
    }
}

fn class_item(engine: &Engine, build: &Build, level: u8, option: &ClassOption) -> Item {
    let prestige = engine.is_prestige(&option.class);
    let class_name = engine.class_name(&option.class);
    let (name, detail) = match &option.archetype {
        // The summary, or for an archetype without one, whose archetype it is (ticket 112).
        Some(archetype) => (
            engine.archetype_name(archetype).to_owned(),
            match engine.archetype_summary(archetype) {
                "" => format!("{class_name} archetype"),
                summary => summary.to_owned(),
            },
        ),
        None => (
            class_name.to_owned(),
            engine.class_summary(&option.class).to_owned(),
        ),
    };
    Item {
        detail,
        category: if prestige {
            "Prestige classes".into()
        } else {
            "Base classes".into()
        },
        locked: option.status.is_err(),
        reason: reason(engine, &option.status),
        current: holds_class_option(build, level, option),
        ..Item::new(name)
    }
}

/// Which held classes a prestige class advances: the choices the rules allow here ([`Engine::advances_options`]).
#[component]
fn Advances(level: u8, option: ClassOption) -> impl IntoView {
    let planner = use_planner();
    let engine = planner.engine();
    let build = planner.build.get_untracked();
    let possible = timed("advances", || {
        engine.advances_options(&build, level, &option)
    });
    let current = build
        .class_choices
        .get(&option.class)
        .map(|choice| choice.advances.clone());
    let items = possible
        .iter()
        .map(|possible| {
            let advances = &possible.advances;
            let names: Vec<&str> = advances
                .iter()
                .map(|class| engine.class_name(class))
                .collect();
            let name = if names.is_empty() {
                "None".to_owned()
            } else {
                names.join(" and ")
            };
            Item {
                detail: if advances.is_empty() {
                    "Its levels advance no class's spellcasting.".into()
                } else {
                    String::new()
                },
                current: current.as_ref() == Some(advances),
                ..Item::new(name)
            }
        })
        .collect();
    let prestige = engine.class_name(&option.class).to_owned();
    view! {
        <p class="picker-step">{format!("Which spellcasting does {prestige} advance?")}</p>
        <ItemList
            items
            on_pick=move |index: usize| planner.pick(possible[index].edit.clone())
        />
    }
}

/// The first ascension's picker (MR 1): each path's path-linked ability, with its path and its follow-ups at MR 2.
/// Without one of its own the Build takes its MR 3 path's, which shows as the current one.
#[component]
fn FirstAscensionPicker() -> impl IntoView {
    let planner = use_planner();
    let engine = planner.engine();
    let build = planner.build.get_untracked();
    let options = timed("first_ascension_options", || {
        engine.first_ascension_options(&build)
    });
    let current = planner
        .sheet
        .get_untracked()
        .ranks
        .iter()
        .find(|rank| rank.rank == FIRST_ASCENSION_RANK)
        .and_then(|rank| rank.path.clone());
    let items = options
        .iter()
        .map(|option| {
            let follow_ups: Vec<&str> = engine
                .path_rank_features(&option.path, FIRST_ASCENSION_RANK + 1)
                .into_iter()
                .map(|feature| engine.feature_name(feature))
                .collect();
            Item {
                detail: format!(
                    "{} · MR {}: {}",
                    engine.path_name(&option.path),
                    FIRST_ASCENSION_RANK + 1,
                    follow_ups.join(", ")
                ),
                locked: option.status.is_err(),
                reason: reason(&engine, &option.status),
                current: current.as_ref() == Some(&option.path),
                ..Item::new(engine.first_ascension_name(&option.path))
            }
        })
        .collect();
    let (title, place) = heading(&engine, &Choice::FirstAscension);
    view! {
        <Head title place remove=Some(Choice::FirstAscension)/>
        <ItemList
            items
            on_pick=move |index: usize| planner.pick(options[index].edit.clone())
        />
    }
}

#[component]
fn PathPicker(switch: bool) -> impl IntoView {
    let planner = use_planner();
    let engine = planner.engine();
    let build = planner.build.get_untracked();
    let options = if switch {
        timed("switch_options", || engine.switch_options(&build))
    } else {
        timed("path_options", || engine.path_options(&build))
    };
    let current = if switch {
        build.path_switch.clone()
    } else {
        build.mythic_path.clone()
    };
    let items = options
        .iter()
        .map(|option| Item {
            detail: engine.path_summary(&option.path).to_owned(),
            locked: option.status.is_err(),
            reason: reason(&engine, &option.status),
            current: current.as_ref() == Some(&option.path),
            ..Item::new(engine.path_name(&option.path))
        })
        .collect();
    let remove = if switch {
        Choice::PathSwitch
    } else {
        Choice::MythicPath
    };
    let (title, place) = heading(&engine, &remove);
    view! {
        <Head title place remove=Some(remove)/>
        <ItemList
            items
            on_pick=move |index: usize| planner.pick(options[index].edit.clone())
        />
    }
}

/// A Build start picker's list: the engine's options as items, each with the edit that takes it, filled after the
/// first paint as the slot picker's is. A locked option shows its reason (a faith and alignment that don't fit), or
/// none when it would break a later choice.
fn start_picker(
    remove: Choice,
    options: impl FnOnce(&Engine) -> Vec<(Item, Edit)> + 'static,
) -> impl IntoView {
    let planner = use_planner();
    let computed = RwSignal::new(None::<Arc<Vec<(Item, Edit)>>>);
    after_paint(computed, move || {
        let engine = planner.engine();
        Arc::new(timed("start_options", || options(&engine)))
    });
    let list = move || {
        let Some(options) = computed.get() else {
            return loading();
        };
        let items = options.iter().map(|(item, _)| item.clone()).collect();
        view! {
            <ItemList
                items
                on_pick=move |index: usize| planner.pick(options[index].1.clone())
            />
        }
        .into_any()
    };
    let (title, place) = heading(&planner.engine(), &remove);
    view! {
        <Head title place remove=Some(remove)/>
        {list}
    }
}

/// An item locked by an option's status, with its reason when it has one.
fn with_status(engine: &Engine, item: Item, status: &Result<(), Issue>) -> Item {
    Item {
        locked: status.is_err(),
        reason: reason(engine, status),
        ..item
    }
}

#[component]
fn RacePicker() -> impl IntoView {
    let planner = use_planner();
    start_picker(Choice::Race, move |engine| {
        let current = planner.build.with_untracked(|build| build.start.clone());
        let options = planner.ask("race_options", |engine, build| engine.race_options(build));
        options
            .into_iter()
            .filter_map(|option| {
                let race = engine
                    .races()
                    .iter()
                    .find(|race| race.key == option.value.race)?;
                let (name, summary) = match &option.value.subrace {
                    Some(key) => {
                        let subrace = race.subraces.iter().find(|subrace| &subrace.key == key)?;
                        (subrace.name.clone(), subrace.summary.clone())
                    }
                    None => (race.name.clone(), race.summary.clone()),
                };
                let item = Item {
                    detail: summary,
                    current: current.race.as_ref() == Some(&option.value.race)
                        && current.subrace == option.value.subrace,
                    ..Item::new(name)
                };
                Some((with_status(engine, item, &option.status), option.edit))
            })
            .collect()
    })
}

/// The alignment wheel, laid out as the game draws it: Neutral Good at the top, the other eight around it, True
/// Neutral in the middle; good tinted blue, neutral beige, evil red. Locked alignments are greyed out: all nine until
/// a faith is chosen, then those the deity doesn't allow, with the one reason the options share under the wheel.
#[component]
fn AlignmentPicker() -> impl IntoView {
    let planner = use_planner();
    let current = planner.build.with_untracked(|build| build.start.clone());
    // The name over the wheel: the one under the pointer or focus, else the current one.
    let shown = RwSignal::new(None::<Alignment>);
    let options = planner.ask("alignment_options", |engine, build| {
        engine.alignment_options(build)
    });
    let engine = planner.engine();
    // Why the wheel locks: "Choose a deity or atheism first", or the deity's limit; said once under the wheel.
    let misfit = options
        .iter()
        .find_map(|option| reason(&engine, &option.status));
    let nodes = options
        .into_iter()
        .map(|option| {
            let alignment = option.value;
            let on = current.alignment == Some(alignment);
            let edit = StoredValue::new(option.edit);
            view! {
                <button
                    class=format!("alignment-node {}", wheel_place(alignment))
                    class:on=on
                    role="radio"
                    aria-checked=on.to_string()
                    aria-label=alignment.name()
                    title=alignment.name()
                    disabled=option.status.is_err()
                    on:mouseenter=move |_| shown.set(Some(alignment))
                    on:focus=move |_| shown.set(Some(alignment))
                    on:mouseleave=move |_| shown.set(None)
                    on:blur=move |_| shown.set(None)
                    on:click=move |_| planner.pick(edit.get_value())
                >
                    {alignment.short_name()}
                </button>
            }
        })
        .collect_view();
    let name = move || {
        shown
            .get()
            .or(current.alignment)
            .map_or("Choose an alignment", Alignment::name)
    };
    let (title, place) = heading(&engine, &Choice::Alignment);
    view! {
        <Head title place remove=Some(Choice::Alignment)/>
        <p class="alignment-name">{name}</p>
        <div class="alignment-wheel" role="radiogroup" aria-label="Alignment">
            {nodes}
        </div>
        {misfit.map(|reason| view! { <p class="note alignment-reason">{reason}</p> })}
    }
}

/// The deity picker: atheism first, then every deity by name, each with its alignments, favoured weapons and
/// domains. A deity that doesn't allow the start's alignment is locked with the reason; one that would break a later
/// choice (a domain it doesn't offer) is locked with none.
#[component]
fn DeityPicker() -> impl IntoView {
    let planner = use_planner();
    start_picker(Choice::Deity, move |engine| {
        let current = planner
            .build
            .with_untracked(|build| build.start.deity.clone());
        let options = planner.ask("faith_options", |engine, build| engine.faith_options(build));
        let mut items: Vec<(Item, Edit)> = options
            .into_iter()
            .filter_map(|option| {
                let item = match &option.value {
                    Faith::Atheism => Item {
                        detail: "No deity; any alignment".to_owned(),
                        ..Item::new("Atheism")
                    },
                    Faith::Deity(key) => {
                        let deity = engine.deities().iter().find(|deity| &deity.key == key)?;
                        deity_item(engine, deity)
                    }
                };
                let item = Item {
                    current: current.as_ref() == Some(&option.value),
                    ..item
                };
                Some((with_status(engine, item, &option.status), option.edit))
            })
            .collect();
        // Atheism stays first; the deities go by name.
        if let Some(deities) = items.get_mut(1..) {
            deities.sort_by(|(a, _), (b, _)| a.name.cmp(&b.name));
        }
        items
    })
}

/// A deity's item: its name, then its alignments, favoured weapons and domains.
fn deity_item(engine: &Engine, deity: &rules::Deity) -> Item {
    let alignments: Vec<&str> = deity.alignments.iter().map(|a| a.short_name()).collect();
    let weapons: Vec<&str> = deity
        .favoured_weapons
        .iter()
        .map(|weapon| engine.weapon_name(weapon))
        .collect();
    let domains: Vec<&str> = deity
        .domains
        .iter()
        .map(|domain| engine.domain_name(domain))
        .collect();
    Item {
        detail: format!(
            "{} · {} · {}",
            alignments.join(" "),
            weapons.join(", "),
            domains.join(", ")
        ),
        ..Item::new(deity.name.clone())
    }
}

/// An alignment's place on the wheel, as a class: its spot on the ring (or the middle) and its moral tint.
fn wheel_place(alignment: Alignment) -> &'static str {
    match alignment {
        Alignment::NeutralGood => "at-n good",
        Alignment::ChaoticGood => "at-ne good",
        Alignment::ChaoticNeutral => "at-e neutral",
        Alignment::ChaoticEvil => "at-se evil",
        Alignment::NeutralEvil => "at-s evil",
        Alignment::LawfulEvil => "at-sw evil",
        Alignment::LawfulNeutral => "at-w neutral",
        Alignment::LawfulGood => "at-nw good",
        Alignment::TrueNeutral => "at-middle neutral",
    }
}

#[component]
fn BackgroundPicker() -> impl IntoView {
    let planner = use_planner();
    start_picker(Choice::Background, move |engine| {
        let current = planner
            .build
            .with_untracked(|build| build.start.background.clone());
        let options = planner.ask("background_options", |engine, build| {
            engine.background_options(build)
        });
        options
            .into_iter()
            .filter_map(|option| {
                let background = engine
                    .backgrounds()
                    .iter()
                    .find(|background| background.key == option.value)?;
                let item = Item {
                    detail: background.note.clone(),
                    category: background.group.map_or("", |group| group.name()).to_owned(),
                    current: current.as_ref() == Some(&option.value),
                    ..Item::new(background.name.clone())
                };
                Some((with_status(engine, item, &option.status), option.edit))
            })
            .collect()
    })
}

#[component]
fn AbilityIncreasePicker(level: u8) -> impl IntoView {
    let planner = use_planner();
    let engine = planner.engine();
    let current = planner.build.with_untracked(|build| {
        build
            .levels
            .get(usize::from(level) - 1)
            .and_then(|level_up| level_up.ability_increase)
    });
    let options = planner
        .ask("ability_increase_options", |engine, build| {
            engine.ability_increase_options(build, level)
        })
        .unwrap_or_default();
    let items: Vec<Item> = options
        .iter()
        .map(|option| {
            let item = Item {
                current: current == Some(option.value),
                ..Item::new(option.value.increase_name())
            };
            with_status(&engine, item, &option.status)
        })
        .collect();
    let (title, place) = heading(&engine, &Choice::AbilityIncrease { level });
    view! {
        <Head title place remove=Some(Choice::AbilityIncrease { level })/>
        <ItemList
            items
            on_pick=move |index: usize| planner.pick(options[index].edit.clone())
        />
    }
}
