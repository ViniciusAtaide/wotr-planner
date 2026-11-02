//! The picker: a bottom sheet on a narrow screen (the Fold's cover), a centred modal from 600px up. One list
//! behaviour for every choice: search, category chips, the locked options greyed behind "Show N locked" with
//! their reason (none for one that would break a later choice: it just stays grey), and a remove (×) control
//! only for a choice `Engine::removable` gives.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use leptos::prelude::*;
use rules::{
    Ability, AbilityModifier, AbilityScores, Alignment, Build, BuildStart, Choice, ClassChoice,
    ClassKey, ClassOption, Edit, Engine, Issue, Pick, SlotId, SlotOption,
};

use crate::browser::timed;
use crate::planner::{Target, use_planner};

/// One row of a picker list.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub name: String,
    pub detail: String,
    /// The chip it's filed under; empty for none.
    pub category: String,
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
        Target::Path => view! { <PathPicker switch=false/> }.into_any(),
        Target::Switch => view! { <PathPicker switch=true/> }.into_any(),
        Target::Race => view! { <RacePicker/> }.into_any(),
        Target::Alignment => view! { <AlignmentPicker/> }.into_any(),
        Target::Background => view! { <BackgroundPicker/> }.into_any(),
        Target::AbilityIncrease(level) => view! { <AbilityIncreasePicker level/> }.into_any(),
        Target::Skills(level) => view! { <crate::start::SkillsPanel level/> }.into_any(),
    }
}

/// The picker's header: what's being chosen, where, a remove control when the choice may be removed now, and
/// close.
#[component]
pub fn Head(
    #[prop(into)] title: String,
    #[prop(into, optional)] place: String,
    /// The choice the remove control takes back, and its edit; shown only while the choice is removable.
    #[prop(default = None)]
    remove: Option<(Choice, Edit)>,
    /// Goes back a step.
    #[prop(default = None)]
    back: Option<Callback<()>>,
) -> impl IntoView {
    let planner = use_planner();
    let remove = remove.map(|(choice, edit)| {
        let edit = StoredValue::new(edit);
        move || {
            planner.removable(&choice).then(|| {
                view! {
                    <button
                        class="remove"
                        title="Remove this choice"
                        on:click=move |_| {
                            planner.close();
                            planner.edit(edit.get_value());
                        }
                    >
                        "× Remove"
                    </button>
                }
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
    let categories: Vec<String> = items
        .iter()
        .map(|item| item.category.clone())
        .filter(|category| !category.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
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
                        .map(|name| {
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

/// Runs `work` after the browser has painted, so a picker opens at once and shows that it's working while the
/// engine computes its options.
fn after_paint(work: impl FnOnce() + 'static) {
    request_animation_frame(move || set_timeout(work, Duration::ZERO));
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

/// Where a slot is: "Level 3" or "Mythic rank 4".
pub fn slot_place(slot: &SlotId) -> String {
    match (slot.level(), slot.rank()) {
        (Some(level), _) => format!("Level {level}"),
        (_, Some(rank)) => format!("Mythic rank {rank}"),
        _ => String::new(),
    }
}

/// A slot's options, one entry per feat, option or mythic pick; a parameterised one holds each of its values.
struct Group {
    name: String,
    category: String,
    summary: String,
    /// Picks that name a value: the group opens a list of them.
    values: bool,
    options: Vec<SlotOption>,
}

fn groups(engine: &Engine, options: Vec<SlotOption>) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    let mut last_key: Option<String> = None;
    for option in options {
        let key = option.pick.key().to_owned();
        if last_key.as_deref() == Some(key.as_str())
            && let Some(group) = groups.last_mut()
        {
            group.options.push(option);
            continue;
        }
        let mut summary = engine.pick_summary(&option.pick).to_owned();
        let stories = engine.story_conditions(&option.pick);
        if !stories.is_empty() {
            let names: Vec<&str> = stories.iter().map(|story| story.name()).collect();
            if !summary.is_empty() {
                summary.push(' ');
            }
            summary.push_str(&format!("(Story: {})", names.join(", ")));
        }
        groups.push(Group {
            name: engine.pick_base_name(&option.pick).to_owned(),
            category: engine.pick_category(&option.pick).to_owned(),
            summary,
            values: option.pick.parameter().is_some(),
            options: vec![option],
        });
        last_key = Some(key);
    }
    groups
}

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
    let title = engine.slot_label(&found);
    let remove = current
        .is_some()
        .then(|| (Choice::Slot(slot.clone()), Edit::Clear(slot.clone())));

    let computed = RwSignal::new(None::<Arc<Vec<Group>>>);
    {
        let engine = engine.clone();
        let slot = slot.clone();
        after_paint(move || {
            let build = planner.build.get_untracked();
            let options = timed("options", || engine.options(&build, &slot)).unwrap_or_default();
            computed.set(Some(Arc::new(groups(&engine, options))));
        });
    }
    let open_group = RwSignal::new(None::<usize>);
    let slot = StoredValue::new(slot);
    let current = StoredValue::new(current);
    let pick = move |pick: Pick| {
        planner.close();
        if current.with_value(|current| current.as_ref() != Some(&pick)) {
            planner.edit(Edit::Pick {
                slot: slot.get_value(),
                pick,
            });
        }
    };
    let place = slot.with_value(slot_place);
    let engine = StoredValue::new(engine);

    let list = move || {
        let Some(groups) = computed.get() else {
            return loading();
        };
        let engine = engine.get_value();
        match open_group.get() {
            None => {
                let items = groups
                    .iter()
                    .map(|group| {
                        let open = group.options.iter().any(|option| option.status.is_ok());
                        let detail = if group.values {
                            let what = group.options[0]
                                .pick
                                .parameter()
                                .map(value_noun)
                                .unwrap_or("a value");
                            let choose = format!("Choose {what}.");
                            if group.summary.is_empty() {
                                choose
                            } else {
                                format!("{choose} {}", group.summary)
                            }
                        } else {
                            group.summary.clone()
                        };
                        Item {
                            name: group.name.clone(),
                            detail,
                            category: group.category.clone(),
                            locked: !open,
                            reason: (!open).then(|| group_reason(&engine, group)).flatten(),
                            current: current.with_value(|current| {
                                current.as_ref().is_some_and(|current| {
                                    group.options.iter().any(|option| &option.pick == current)
                                })
                            }),
                            more: group.values,
                        }
                    })
                    .collect();
                let groups = groups.clone();
                view! {
                    <ItemList
                        items
                        on_pick=move |index: usize| {
                            let group = &groups[index];
                            if group.values {
                                open_group.set(Some(index));
                            } else {
                                pick(group.options[0].pick.clone());
                            }
                        }
                    />
                }
                .into_any()
            }
            Some(index) => {
                let group = &groups[index];
                let items = group
                    .options
                    .iter()
                    .map(|option| Item {
                        locked: option.status.is_err(),
                        reason: reason(&engine, &option.status),
                        current: current
                            .with_value(|current| current.as_ref() == Some(&option.pick)),
                        ..Item::new(
                            option
                                .pick
                                .parameter()
                                .map(|value| engine.parameter_name(value).to_owned())
                                .unwrap_or_default(),
                        )
                    })
                    .collect();
                let picks: Vec<Pick> = group
                    .options
                    .iter()
                    .map(|option| option.pick.clone())
                    .collect();
                view! {
                    <p class="picker-step">{group.name.clone()}</p>
                    <ItemList items on_pick=move |index: usize| pick(picks[index].clone())/>
                }
                .into_any()
            }
        }
    };
    view! {
        {move || {
            let back = open_group
                .get()
                .is_some()
                .then(|| Callback::new(move |()| open_group.set(None)));
            view! { <Head title=title.clone() place=place.clone() remove=remove.clone() back/> }
        }}
        {list}
    }
    .into_any()
}

/// Why no value of a group can be taken: a value's reason other than already holding it, when one has one (Weapon
/// Focus held for a longsword but needing BAB +1 for the rest says the BAB), else the first value's. None when that
/// one shows no reason.
fn group_reason(engine: &Engine, group: &Group) -> Option<String> {
    let issues = group
        .options
        .iter()
        .filter_map(|option| option.status.as_ref().err());
    let held = |issue: &&Issue| matches!(issue, Issue::AlreadyHeld | Issue::MaxTimes { .. });
    issues
        .clone()
        .find(|issue| !held(issue))
        .or_else(|| issues.clone().next())
        .and_then(|issue| engine.issue_text(issue))
}

/// What a value is called, with its article: "a weapon", "a skill", "an energy type".
fn value_noun(value: &rules::Parameter) -> &'static str {
    match value {
        rules::Parameter::Skill(_) => "a skill",
        rules::Parameter::Weapon(_) => "a weapon",
        rules::Parameter::SpellSchool(_) => "a school",
        rules::Parameter::Energy(_) => "an energy type",
        rules::Parameter::ArmourType(_) => "an armour type",
    }
}

/// Whether a class is first taken at this level: no level before it takes it.
fn first_taken_at(build: &Build, class: &ClassKey, level: u8) -> bool {
    !build
        .levels
        .iter()
        .take(usize::from(level).saturating_sub(1))
        .any(|level_up| &level_up.class == class)
}

/// Whether a prestige class's rows advance another class's spellcasting.
fn advances_spellcasting(engine: &Engine, class: &ClassKey) -> bool {
    engine
        .data()
        .prestige_classes
        .iter()
        .find(|prestige| &prestige.key == class)
        .is_some_and(|prestige| {
            prestige
                .levels
                .iter()
                .any(|row| row.advances_arcane || row.advances_divine)
        })
}

/// The edit for taking a class option at a level: a class first taken here gets the option's archetype and the
/// advancement given (or the one it has).
fn class_edit(
    build: &Build,
    level: u8,
    option: &ClassOption,
    advances: Option<BTreeSet<ClassKey>>,
) -> Edit {
    let choice = first_taken_at(build, &option.class, level).then(|| ClassChoice {
        archetype: option.archetype.clone(),
        advances: advances.unwrap_or_else(|| {
            build
                .class_choices
                .get(&option.class)
                .map(|choice| choice.advances.clone())
                .unwrap_or_default()
        }),
    });
    Edit::Class {
        level,
        class: option.class.clone(),
        choice,
    }
}

#[component]
fn ClassPicker(level: u8) -> impl IntoView {
    let planner = use_planner();
    let engine = StoredValue::new(planner.engine());
    let computed = RwSignal::new(None::<Arc<Vec<ClassOption>>>);
    after_paint(move || {
        let build = planner.build.get_untracked();
        let options = engine.with_value(|engine| {
            timed("class_options", || engine.class_options(&build, level)).unwrap_or_default()
        });
        computed.set(Some(Arc::new(options)));
    });
    // A prestige class first taken here asks which classes it advances.
    let advancing = RwSignal::new(None::<ClassOption>);
    let adding = planner
        .build
        .with_untracked(|build| usize::from(level) > build.levels.len());
    let title = if adding {
        format!("Level {level}: add a class")
    } else {
        format!("Class at level {level}")
    };

    let choose = move |option: ClassOption| {
        let build = planner.build.get_untracked();
        let needs_advances = engine.with_value(|engine| {
            engine.is_prestige(&option.class)
                && advances_spellcasting(engine, &option.class)
                && first_taken_at(&build, &option.class, level)
        });
        if needs_advances {
            advancing.set(Some(option));
            return;
        }
        planner.close();
        let edit = class_edit(&build, level, &option, None);
        let unchanged = build.levels.get(usize::from(level) - 1).is_some_and(|l| {
            l.class == option.class
                && build.class_choices.get(&option.class).map(|c| &c.archetype)
                    == Some(&option.archetype)
        });
        if !unchanged {
            planner.edit(edit);
        }
    };

    move || {
        if let Some(option) = advancing.get() {
            return view! {
                <Head
                    title=title.clone()
                    place="Spellcasting advancement"
                    back=Some(Callback::new(move |()| advancing.set(None)))
                />
                <Advances level option/>
            }
            .into_any();
        }
        let list = match computed.get() {
            None => loading(),
            Some(options) => {
                let build = planner.build.get_untracked();
                let items = engine.with_value(|engine| {
                    options
                        .iter()
                        .map(|option| class_item(engine, &build, level, option))
                        .collect()
                });
                let options = options.clone();
                view! {
                    <ItemList items on_pick=move |index: usize| choose(options[index].clone())/>
                }
                .into_any()
            }
        };
        view! {
            <Head title=title.clone()/>
            {list}
        }
        .into_any()
    }
}

fn class_item(engine: &Engine, build: &Build, level: u8, option: &ClassOption) -> Item {
    let prestige = engine.is_prestige(&option.class);
    let class_name = engine.class_name(&option.class);
    let (name, detail) = match &option.archetype {
        Some(archetype) => (
            engine.archetype_name(archetype).to_owned(),
            format!("{class_name} archetype"),
        ),
        None => (class_name.to_owned(), String::new()),
    };
    let current = build
        .levels
        .get(usize::from(level).wrapping_sub(1))
        .is_some_and(|level_up| {
            level_up.class == option.class
                && build
                    .class_choices
                    .get(&option.class)
                    .map(|choice| &choice.archetype)
                    == Some(&option.archetype)
        });
    Item {
        detail,
        category: if prestige {
            "Prestige classes".into()
        } else {
            "Base classes".into()
        },
        locked: option.status.is_err(),
        reason: reason(engine, &option.status),
        current,
        ..Item::new(name)
    }
}

/// Which held classes a prestige class advances: the choices the rules allow here, and none.
#[component]
fn Advances(level: u8, option: ClassOption) -> impl IntoView {
    let planner = use_planner();
    let engine = planner.engine();
    let build = planner.build.get_untracked();
    let held: Vec<ClassKey> = build
        .levels
        .iter()
        .take(usize::from(level) - 1)
        .map(|level_up| level_up.class.clone())
        .fold(Vec::new(), |mut held, class| {
            if !held.contains(&class) {
                held.push(class);
            }
            held
        });
    let mut candidates: Vec<BTreeSet<ClassKey>> = held
        .iter()
        .map(|class| BTreeSet::from([class.clone()]))
        .collect();
    for (i, first) in held.iter().enumerate() {
        for second in &held[i + 1..] {
            candidates.push(BTreeSet::from([first.clone(), second.clone()]));
        }
    }
    candidates.push(BTreeSet::new());
    let possible: Vec<BTreeSet<ClassKey>> = timed("advances", || {
        candidates
            .into_iter()
            .filter(|advances| {
                let mut after = build.clone();
                engine.apply(
                    &mut after,
                    &class_edit(&build, level, &option, Some(advances.clone())),
                );
                engine.is_possible(&after)
            })
            .collect()
    });
    let current = build
        .class_choices
        .get(&option.class)
        .map(|choice| choice.advances.clone());
    let items = possible
        .iter()
        .map(|advances| {
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
            on_pick=move |index: usize| {
                let advances = possible[index].clone();
                planner.close();
                let build = planner.build.get_untracked();
                planner.edit(class_edit(&build, level, &option, Some(advances)));
            }
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
        .map(|option| {
            let summary = engine
                .data()
                .mythic_paths
                .iter()
                .find(|path| path.key == option.path)
                .map(|path| path.summary.clone())
                .unwrap_or_default();
            Item {
                detail: summary,
                locked: option.status.is_err(),
                reason: reason(&engine, &option.status),
                current: current.as_ref() == Some(&option.path),
                ..Item::new(engine.path_name(&option.path))
            }
        })
        .collect();
    let (title, place, remove) = if switch {
        (
            "Path switch",
            "Mythic rank 8",
            (Choice::PathSwitch, Edit::PathSwitch(None)),
        )
    } else {
        (
            "Mythic path",
            "Mythic rank 3",
            (Choice::MythicPath, Edit::MythicPath(None)),
        )
    };
    let paths: Vec<_> = options.into_iter().map(|option| option.path).collect();
    view! {
        <Head title place remove=current.is_some().then_some(remove)/>
        <ItemList
            items
            on_pick=move |index: usize| {
                let path = Some(paths[index].clone());
                planner.close();
                if path != current {
                    planner
                        .edit(if switch { Edit::PathSwitch(path) } else { Edit::MythicPath(path) });
                }
            }
        />
    }
}

/// The modifiers a Build start's race (or subrace, whose replace the race's) gives each ability.
pub fn racial_modifiers(engine: &Engine, start: &BuildStart) -> [i8; 6] {
    let mut modifiers = [0i8; 6];
    let Some(race) = start
        .race
        .as_ref()
        .and_then(|key| engine.data().races.iter().find(|race| &race.key == key))
    else {
        return modifiers;
    };
    let list = start
        .subrace
        .as_ref()
        .and_then(|key| race.subraces.iter().find(|subrace| &subrace.key == key))
        .map_or(&race.ability_modifiers, |subrace| {
            &subrace.ability_modifiers
        });
    for modifier in list {
        let (ability, amount) = match modifier {
            AbilityModifier::Fixed { ability, amount } => (Some(*ability), *amount),
            AbilityModifier::Chosen { amount } => (start.racial_ability, *amount),
        };
        if let Some(ability) = ability {
            let index = AbilityScores::ABILITIES
                .iter()
                .position(|a| *a == ability)
                .unwrap_or(0);
            modifiers[index] += amount;
        }
    }
    modifiers
}

/// Whether a Build start's race (or subrace) lets the player choose where "+2 to one ability score" goes.
pub fn has_chosen_modifier(engine: &Engine, start: &BuildStart) -> bool {
    let Some(race) = start
        .race
        .as_ref()
        .and_then(|key| engine.data().races.iter().find(|race| &race.key == key))
    else {
        return false;
    };
    let list = start
        .subrace
        .as_ref()
        .and_then(|key| race.subraces.iter().find(|subrace| &subrace.key == key))
        .map_or(&race.ability_modifiers, |subrace| {
            &subrace.ability_modifiers
        });
    list.iter()
        .any(|modifier| matches!(modifier, AbilityModifier::Chosen { .. }))
}

/// A Build start picker: each option is a whole new Build start, locked when `Engine::allows` refuses it.
fn start_items(
    planner: crate::planner::Planner,
    starts: &[(Item, BuildStart)],
    current: &BuildStart,
) -> Vec<Item> {
    starts
        .iter()
        .map(|(item, start)| {
            let allowed = start == current || planner.allows(&Edit::Start(start.clone()));
            Item {
                locked: !allowed,
                ..item.clone()
            }
        })
        .collect()
}

#[component]
fn StartPicker(
    title: &'static str,
    starts: Vec<(Item, BuildStart)>,
    remove: Option<(Choice, Edit)>,
) -> impl IntoView {
    let planner = use_planner();
    let current = planner.build.with_untracked(|build| build.start.clone());
    let items = planner
        .build
        .with_untracked(|_| timed("start_options", || start_items(planner, &starts, &current)));
    let starts: Vec<BuildStart> = starts.into_iter().map(|(_, start)| start).collect();
    view! {
        <Head title place="Build start" remove/>
        <ItemList
            items
            on_pick=move |index: usize| {
                let start = starts[index].clone();
                planner.close();
                if start != current {
                    planner.edit(Edit::Start(start));
                }
            }
        />
    }
}

#[component]
fn RacePicker() -> impl IntoView {
    let planner = use_planner();
    let engine = planner.engine();
    let current = planner.build.with_untracked(|build| build.start.clone());
    let mut starts = Vec::new();
    for race in &engine.data().races {
        let entries = std::iter::once((None, race.name.clone(), race.summary.clone())).chain(
            race.subraces.iter().map(|subrace| {
                (
                    Some(subrace.key.clone()),
                    subrace.name.clone(),
                    subrace.summary.clone(),
                )
            }),
        );
        for (subrace, name, summary) in entries {
            let mut start = BuildStart {
                race: Some(race.key.clone()),
                subrace: subrace.clone(),
                ..current.clone()
            };
            if !has_chosen_modifier(&engine, &start) {
                start.racial_ability = None;
            }
            let item = Item {
                detail: summary,
                current: current.race.as_ref() == Some(&race.key) && current.subrace == subrace,
                ..Item::new(name)
            };
            starts.push((item, start));
        }
    }
    let remove = current.race.is_some().then(|| {
        (
            Choice::Race,
            Edit::Start(BuildStart {
                race: None,
                subrace: None,
                racial_ability: None,
                ..current.clone()
            }),
        )
    });
    view! { <StartPicker title="Race" starts remove/> }
}

/// The alignment wheel, laid out as the game draws it: Neutral Good at the top, the other eight around it, True
/// Neutral in the middle; good tinted blue, neutral beige, evil red. Locked alignments are greyed out.
#[component]
fn AlignmentPicker() -> impl IntoView {
    let planner = use_planner();
    let current = planner.build.with_untracked(|build| build.start.clone());
    let remove = current.alignment.is_some().then(|| {
        (
            Choice::Alignment,
            Edit::Start(BuildStart {
                alignment: None,
                ..current.clone()
            }),
        )
    });
    // The name over the wheel: the one under the pointer or focus, else the current one.
    let shown = RwSignal::new(None::<Alignment>);
    let nodes = Alignment::ALL
        .into_iter()
        .map(|alignment| {
            let start = BuildStart {
                alignment: Some(alignment),
                ..current.clone()
            };
            let on = current.alignment == Some(alignment);
            let allowed = on || planner.allows(&Edit::Start(start.clone()));
            let start = StoredValue::new(start);
            view! {
                <button
                    class=format!("alignment-node {}", wheel_place(alignment))
                    class:on=on
                    role="radio"
                    aria-checked=on.to_string()
                    aria-label=alignment.name()
                    title=alignment.name()
                    disabled=!allowed
                    on:mouseenter=move |_| shown.set(Some(alignment))
                    on:focus=move |_| shown.set(Some(alignment))
                    on:mouseleave=move |_| shown.set(None)
                    on:blur=move |_| shown.set(None)
                    on:click=move |_| {
                        planner.close();
                        if !on {
                            planner.edit(Edit::Start(start.get_value()));
                        }
                    }
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
    view! {
        <Head title="Alignment" place="Build start" remove/>
        <p class="alignment-name">{name}</p>
        <div class="alignment-wheel" role="radiogroup" aria-label="Alignment">
            {nodes}
        </div>
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
    let engine = planner.engine();
    let current = planner.build.with_untracked(|build| build.start.clone());
    let starts = engine
        .data()
        .backgrounds
        .iter()
        .map(|background| {
            let item = Item {
                detail: background.note.clone(),
                category: background.group.map_or("", |group| group.name()).to_owned(),
                current: current.background.as_ref() == Some(&background.key),
                ..Item::new(background.name.clone())
            };
            (
                item,
                BuildStart {
                    background: Some(background.key.clone()),
                    ..current.clone()
                },
            )
        })
        .collect();
    let remove = current.background.is_some().then(|| {
        (
            Choice::Background,
            Edit::Start(BuildStart {
                background: None,
                ..current.clone()
            }),
        )
    });
    view! { <StartPicker title="Background" starts remove/> }
}

#[component]
fn AbilityIncreasePicker(level: u8) -> impl IntoView {
    let planner = use_planner();
    let current = planner.build.with_untracked(|build| {
        build
            .levels
            .get(usize::from(level) - 1)
            .and_then(|level_up| level_up.ability_increase)
    });
    let items: Vec<Item> = AbilityScores::ABILITIES
        .into_iter()
        .map(|ability| {
            let edit = Edit::AbilityIncrease {
                level,
                ability: Some(ability),
            };
            let allowed = current == Some(ability) || planner.allows(&edit);
            Item {
                current: current == Some(ability),
                locked: !allowed,
                ..Item::new(format!("+1 {}", ability.name()))
            }
        })
        .collect();
    let remove = current.map(|_| {
        (
            Choice::AbilityIncrease { level },
            Edit::AbilityIncrease {
                level,
                ability: None,
            },
        )
    });
    let place = format!("Level {level}");
    view! {
        <Head title="Ability increase" place remove/>
        <ItemList
            items
            on_pick=move |index: usize| {
                let ability: Ability = AbilityScores::ABILITIES[index];
                planner.close();
                if current != Some(ability) {
                    planner
                        .edit(Edit::AbilityIncrease {
                            level,
                            ability: Some(ability),
                        });
                }
            }
        />
    }
}
