//! The Mythic track: MR 1–10 at their fixed character levels, the first ascension at MR 1, the path at MR 3 and the
//! Path switch at MR 8, and under each rank the features its path row gives (MR 1 and 2: the first ascension's). A
//! rank is reached, partly dropped or dropped by a switch, or not reached (shown greyed, with no label); the picks a
//! switch took away stay, marked lost from the switch on, and an empty slot there still has a picker.

use leptos::prelude::*;
use rules::{
    Address, FIRST_ASCENSION_RANK, MythicPathKey, PATH_CHOICE_RANK, PATH_SWITCH_RANK, RankSheet,
    RankStatus,
};

use crate::grid::slot_chip;
use crate::planner::{Target, use_planner};

#[component]
pub fn MythicTrack() -> AnyView {
    let planner = use_planner();
    (move || {
        let engine = planner.engine();
        let sheet = planner.sheet.get();
        let levels = planner.build.with(|build| build.levels.len());
        let chosen_path = |key: &MythicPathKey| Chosen {
            name: engine.path_name(key).to_owned(),
            path: key.clone(),
        };
        let (path, switch) = planner.build.with(|build| {
            (
                build.mythic_path.as_ref().map(chosen_path),
                build.path_switch.as_ref().map(chosen_path),
            )
        });
        let rows = sheet
            .ranks
            .iter()
            .map(|rank| {
                let ascends = usize::from(rank.character_level) <= levels;
                // The first ascension shows the path-linked ability the rank takes: the Build's own, or its MR 3
                // path's.
                let first = rank.path.as_ref().map(|key| Chosen {
                    name: engine.first_ascension_name(key),
                    path: key.clone(),
                });
                rank_row(
                    rank,
                    ascends,
                    (rank.rank == FIRST_ASCENSION_RANK).then_some(first),
                    (rank.rank == PATH_CHOICE_RANK).then(|| path.clone()),
                    (rank.rank == PATH_SWITCH_RANK).then(|| switch.clone()),
                )
            })
            .collect_view();
        view! {
            <p class="summary muted">
                {format!(
                    "Each Mythic ascension happens at a fixed character level. The first ascension at MR {FIRST_ASCENSION_RANK} takes one path's path-linked ability, with its follow-ups at MR {}; the path is chosen at MR {PATH_CHOICE_RANK}; a Path switch can come at MR {PATH_SWITCH_RANK}.",
                    FIRST_ASCENSION_RANK + 1,
                )}
            </p>
            <div class="grid mythic" role="list">
                {rows}
            </div>
        }
    })
    .into_any()
}

/// What a path chip shows: the chosen one's name, and the path it belongs to, whose colour the chip takes.
#[derive(Clone)]
struct Chosen {
    name: String,
    path: MythicPathKey,
}

/// One rank's row. `first`, `path` and `switch` are set on the ranks that choose them: the chosen one, if any.
/// `ascends`: the Build has the rank's character level, so its first ascension, path or switch can be chosen. The
/// row, its path's name and each chosen path chip carry their path's key as `data-path`, which sets their colour.
fn rank_row(
    rank: &RankSheet,
    ascends: bool,
    first: Option<Option<Chosen>>,
    path: Option<Option<Chosen>>,
    switch: Option<Option<Chosen>>,
) -> impl IntoView + use<> {
    let planner = use_planner();
    let engine = planner.engine();
    let path_key = rank.path.as_ref().map(|key| key.as_str().to_owned());
    let path_name = rank
        .path
        .as_ref()
        .map(|key| (engine.path_name(key).to_owned(), key.as_str().to_owned()));
    // A rank not reached says so by its greyed level alone.
    let status = match rank.status {
        RankStatus::Reached | RankStatus::NotReached => None,
        RankStatus::PartlyDropped => Some("Partly lost to the Path switch"),
        RankStatus::Dropped => Some("Lost to the Path switch"),
    };
    let not_reached = rank.status == RankStatus::NotReached;
    let chooser = |address: Address, value: Option<Chosen>, target: Target| {
        let label = engine.choice_label(&address, None);
        let empty = value.is_none();
        let (name, key) = value.map_or(("+".to_owned(), None), |chosen| {
            (chosen.name, Some(chosen.path.as_str().to_owned()))
        });
        view! {
            <button
                class="chip path-chip"
                class:empty=empty
                data-path=key
                disabled=!ascends
                on:click=move |_| planner.open(target.clone())
            >
                <span class="chip-label">{label}</span>
                <span class="chip-value">{name}</span>
            </button>
        }
    };
    let first_chip =
        first.map(|value| chooser(Address::FirstAscension, value, Target::FirstAscension));
    let path_chip = path.map(|value| chooser(Address::Path, value, Target::Path));
    let features: Vec<&str> = engine
        .rank_features(rank)
        .into_iter()
        .map(|feature| engine.feature_name(feature))
        .collect();
    let features = (!features.is_empty())
        .then(|| view! { <span class="rank-features">{features.join(" · ")}</span> });
    let switch_chip = switch.map(|value| chooser(Address::Switch, value, Target::Switch));
    let chips = rank
        .slots
        .iter()
        .map(|slot| slot_chip(&engine, slot, rank.dropped_slots.contains(&slot.id)))
        .collect_view();
    view! {
        <div
            class="row rank"
            class:not-reached=not_reached
            class:dropped=rank.status == RankStatus::Dropped
            data-path=path_key
            role="listitem"
        >
            <span class="row-number mythic-number">{format!("MR {}", rank.rank)}</span>
            <span class="rank-level">
                {format!("Level {}", rank.character_level)}
                {path_name
                    .map(|(name, key)| view! { <span class="rank-path" data-path=key>{name}</span> })}
            </span>
            {features}
            <div class="chips">
                {first_chip} {path_chip} {switch_chip} {chips}
                {status.map(|status| view! { <span class="rank-status">{status}</span> })}
            </div>
        </div>
    }
}
