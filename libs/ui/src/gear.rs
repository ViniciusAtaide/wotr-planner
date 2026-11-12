//! The Gear tab (ticket 167): a row per place a Build wears an item in, in the guide's slot order (two ring rows,
//! one body row for a robe or an armour), then a Tomes row for the usable items, each a tick. A row shows its item's
//! name with its summary below, as races show theirs, or "None"; an item with a conditional effect has an info button (or
//! a pointer resting on it) that unfolds those effects and its guide link, and one without has a plain "Guide" link. Tapping a row opens its picker.
//!
//! Gear belongs to no level, so the tab is independent of the levels: the engine counts an item at every one. An
//! item that a choice needs can't be removed or replaced: the engine locks those options, with the reason naming the
//! choice ([`rules::Engine::gear_options`]), and the row says which choice needs it.

use std::sync::Arc;

use leptos::prelude::*;
use rules::{Choice, Edit, Engine, GearItem, GearKey, GearOption, GearPlace, GearSpot, GuideRef};

use crate::picker::{
    Head, Item, ItemInfo, ItemList, after_paint, heading, item_detail, item_row, loading,
};
use crate::planner::{Target, use_planner};
use crate::timings::timed;

/// What an item does only in some circumstance, and its guide link: its detail. `None` for an item with no
/// conditional line: what it always does is in its summary, and the guide link alone makes no detail.
fn item_info(item: &GearItem) -> Option<ItemInfo> {
    ItemInfo::new(
        item.other_effects.clone(),
        item.guide.as_ref().map(GuideRef::url),
    )
}

#[component]
pub fn GearTab() -> AnyView {
    let planner = use_planner();
    (move || {
        let engine = planner.engine();
        let gear = planner.build.with(|build| build.gear.clone());
        let rows = GearPlace::ALL
            .into_iter()
            .map(|place| place_row(&engine, place, gear.worn.get(&place)))
            .collect_view();
        let used: Vec<GearKey> = gear.used.iter().cloned().collect();
        view! {
            <p class="summary muted">
                "Gear belongs to no level: an item's bonuses count at every level, for prerequisites and every number, but never for a level's skill points."
            </p>
            <div class="grid gear" role="list">
                {rows}
                {tomes_row(&engine, &used)}
            </div>
        }
    })
    .into_any()
}

/// The lock on an item the Build holds when some choice can't do without it: "Greater Two-Weapon Fighting needs it".
fn needed(item: &GearKey) -> Option<impl IntoView + use<>> {
    let names = use_planner().needing(item)?;
    Some(view! {
        <span class="gear-needed">
            <span class="lock-mark" aria-label="Locked">
                "🔒"
            </span>
            {format!(" {names} needs it")}
        </span>
    })
}

/// A place's row: its name, the item worn there (or "None"), which opens the place's picker, and the item's detail.
fn place_row(engine: &Engine, place: GearPlace, worn: Option<&GearKey>) -> impl IntoView + use<> {
    let planner = use_planner();
    let open = RwSignal::new(false);
    let item = worn.and_then(|key| engine.gear_item(key));
    let info = item.and_then(item_info);
    let has_info = info.is_some();
    let has_guide = !has_info && item.is_some_and(|item| item.guide.is_some());
    // With no detail to open, the guide link is a small plain link on the row.
    let guide = (!has_info)
        .then(|| item.and_then(|item| item.guide.as_ref()))
        .flatten()
        .map(|guide| {
            view! {
                <a class="guide-link plain" href=guide.url() target="_blank" rel="noopener noreferrer">
                    "Guide"
                </a>
            }
        });
    let value = match worn {
        Some(key) => view! {
            <span class="gear-name">{engine.gear_name(key).to_owned()}</span>
            <span class="gear-summary">{engine.gear_summary(key)}</span>
            {needed(key)}
        }
        .into_any(),
        None => view! { <span class="gear-none">"None"</span> }.into_any(),
    };
    let detail = info.map(|info| {
        let name = item.map_or_else(String::new, |item| item.name.clone());
        item_detail(open, &name, info)
    });
    view! {
        <div
            class="row gear-row item-row"
            class:has-info=has_info
            class:has-guide=has_guide
            class:open=move || open.get()
            role="listitem"
        >
            <span class="gear-place">{place.name()}</span>
            <button
                class="gear-value"
                class:empty=worn.is_none()
                aria-label=format!("{}: choose an item", place.name())
                on:click=move |_| planner.open(Target::Gear(place))
            >
                {value}
            </button>
            {detail}
            {guide}
        </div>
    }
}

/// The Tomes row: each usable item used, with its summary, or "None"; it opens the ticks.
fn tomes_row(engine: &Engine, used: &[GearKey]) -> impl IntoView + use<> {
    let planner = use_planner();
    let value = if used.is_empty() {
        view! { <span class="gear-none">"None"</span> }.into_any()
    } else {
        used.iter()
            .map(|key| {
                view! {
                    <span class="gear-used">
                        <span class="gear-name">{engine.gear_name(key).to_owned()}</span>
                        <span class="gear-summary">{engine.gear_summary(key)}</span>
                        {needed(key)}
                    </span>
                }
            })
            .collect_view()
            .into_any()
    };
    view! {
        <div class="row gear-row" role="listitem">
            <span class="gear-place">"Tomes"</span>
            <button
                class="gear-value"
                class:empty=used.is_empty()
                aria-label="Tomes: tick the ones used"
                on:click=move |_| planner.open(Target::Tomes)
            >
                {value}
            </button>
        </div>
    }
}

/// A gear option as a picker item: "None", or the item with its summary and detail. `category`: file it under its
/// slot (the body's robes and armours).
fn option_item(engine: &Engine, option: &GearOption, category: bool) -> Item {
    let item = match &option.item {
        None => Item::new("None"),
        Some(key) => {
            let data = engine.gear_item(key);
            Item {
                detail: engine.gear_summary(key),
                category: data
                    .filter(|_| category)
                    .map_or("", |data| data.slot.name())
                    .to_owned(),
                info: data.and_then(item_info),
                ..Item::new(engine.gear_name(key))
            }
        }
    };
    Item {
        locked: option.status.is_err(),
        reason: option.reason.clone(),
        current: option.held,
        ..item
    }
}

/// A place's picker: "None" first, then every item the place takes, each with its summary and detail. An item
/// that a choice needs can't be taken off or replaced by one that wouldn't keep the choice: those options are
/// locked with the engine's reason, said once above the list.
#[component]
pub fn GearPicker(place: GearPlace) -> impl IntoView {
    let planner = use_planner();
    let choice = Choice::Gear(GearSpot::Worn(place));
    let (title, heading_place) = heading(&planner.engine(), &choice);
    let step = RwSignal::new(None::<Arc<Vec<GearOption>>>);
    after_paint(step, move || {
        let build = planner.build.get_untracked();
        let engine = planner.engine();
        Arc::new(timed("gear_options", || engine.gear_options(&build, place)))
    });
    let list = move || {
        let Some(options) = step.get() else {
            return loading();
        };
        let engine = planner.engine();
        // Taking the item off is locked when a choice needs it: why, and what to do, above the list.
        let held = options
            .first()
            .filter(|none| none.item.is_none() && none.status.is_err())
            .and_then(|none| {
                let reason = none.reason.clone()?;
                let names = planner
                    .build
                    .with_untracked(|build| engine.choice_names(build, &none.needed_by));
                Some(format!("{reason}. Clear {names} first to take it off."))
            });
        let items = options
            .iter()
            .map(|option| option_item(&engine, option, place == GearPlace::Body))
            .collect();
        view! {
            {held
                .map(|note| {
                    view! {
                        <p class="picker-step locked-note">
                            <span aria-label="Locked">"🔒"</span>
                            {format!(" {note}")}
                        </p>
                    }
                })}
            <ItemList
                items
                on_pick=move |index: usize| planner.pick(options[index].edit.clone())
            />
        }
        .into_any()
    };
    view! {
        <Head title place=heading_place remove=Some(choice)/>
        {list}
    }
}

/// The tomes' picker: every usable item with a tick, its summary and detail. A tick uses the item or stops using it
/// at once, and the picker stays open for the next; one that a choice needs can't be unticked (locked with the
/// engine's reason).
#[component]
pub fn TomesPicker() -> impl IntoView {
    let planner = use_planner();
    let options = RwSignal::new(None::<Arc<Vec<GearOption>>>);
    let work = move || {
        let build = planner.build.get_untracked();
        let engine = planner.engine();
        Arc::new(timed("usable_options", || engine.usable_options(&build)))
    };
    after_paint(options, work);
    let tick = move |edit: Edit| {
        planner.edit(edit);
        options.set(Some(work()));
    };
    let list = move || {
        let Some(options) = options.get() else {
            return loading();
        };
        let engine = planner.engine();
        let rows = options
            .iter()
            .map(|option| {
                let item = Item {
                    tick: Some(option.held),
                    ..option_item(&engine, option, false)
                };
                let edit = option.edit.clone();
                item_row(item, move || tick(edit.clone()))
            })
            .collect_view();
        view! {
            <p class="picker-step">"A character can use every one: tick those used."</p>
            <div class="picker-list">
                <ul class="items">{rows}</ul>
            </div>
        }
        .into_any()
    };
    view! {
        <Head title="Tomes" place="Gear"/>
        {list}
    }
}
