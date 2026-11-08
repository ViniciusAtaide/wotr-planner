//! The levels grid: a row per character level with its class, one chip per slot, the ability increase, skill
//! ranks and a badge for the Mythic ranks reached there; then the summary of where the Build ends.

use leptos::prelude::*;
use rules::{AbilityScores, Build, Choice, Engine, LevelSheet, Sheet, Slot};

use crate::icons::feat_icon;
use crate::planner::{Tab, Target, use_planner};

#[component]
pub fn LevelsGrid() -> AnyView {
    let planner = use_planner();
    (move || {
        let engine = planner.engine();
        let sheet = planner.sheet.get();
        planner.build.with(|build| {
            let rows = sheet
                .levels
                .iter()
                .map(|level| level_row(&engine, build, level))
                .collect_view();
            let next = u8::try_from(build.levels.len() + 1).unwrap_or(u8::MAX);
            let can_add = next <= sheet.level_cap;
            let last = u8::try_from(build.levels.len()).unwrap_or(u8::MAX);
            // The last level's withdrawal: Remove level.
            let remove = planner.withdrawal(&Choice::CharacterLevel { level: last });
            view! {
                <Summary sheet=sheet.clone()/>
                <div class="grid" role="list">
                    {rows}
                </div>
                <div class="grid-actions">
                    {can_add
                        .then(|| {
                            view! {
                                <button
                                    class="add-level"
                                    on:click=move |_| planner.open(Target::Class(next))
                                >
                                    {format!("+ Level {next}")}
                                </button>
                            }
                        })}
                    {remove
                        .map(|(edit, removable)| {
                            let edit = StoredValue::new(edit);
                            view! {
                                <button
                                    class="remove-level"
                                    disabled=!removable
                                    on:click=move |_| planner.edit(edit.get_value())
                                >
                                    {format!("Remove level {last}")}
                                </button>
                            }
                        })}
                    {(!can_add)
                        .then(|| {
                            view! {
                                <p class="muted">
                                    {format!("Level {} is the last level.", sheet.level_cap)}
                                </p>
                            }
                        })}
                </div>
            }
        })
    })
    .into_any()
}

fn level_row(engine: &Engine, build: &Build, level: &LevelSheet) -> impl IntoView + use<> {
    let planner = use_planner();
    let n = level.character_level;
    let class_name = engine.class_name(&level.class).to_owned();
    let archetype = build
        .class_choices
        .get(&level.class)
        .and_then(|choice| choice.archetype.as_ref())
        .filter(|_| level.class_level == 1)
        .map(|archetype| engine.archetype_name(archetype).to_owned());
    let changeable = planner.class_removable(n);
    let chips = level
        .slots
        .iter()
        .map(|slot| slot_chip(engine, slot, false))
        .collect_view();
    let increase = level.ability_increase.then(|| {
        let ability = build.levels[usize::from(n) - 1].ability_increase;
        let value = ability.map_or("+".to_owned(), |ability| {
            format!("+1 {}", ability.short_name())
        });
        view! {
            <button
                class="chip"
                class:empty=ability.is_none()
                on:click=move |_| planner.open(Target::AbilityIncrease(n))
            >
                <span class="chip-label">"Ability"</span>
                <span class="chip-value">{value}</span>
            </button>
        }
    });
    let points = level.skill_points;
    let skills = view! {
        <button
            class="chip skills"
            class:empty=points.spent < points.available
            on:click=move |_| planner.open(Target::Skills(n))
        >
            <span class="chip-label">"Skills"</span>
            <span class="chip-value">{format!("{}/{}", points.spent, points.available)}</span>
        </button>
    };
    let mythic = (!level.mythic_ranks.is_empty()).then(|| {
        let ranks: Vec<String> = level.mythic_ranks.iter().map(u8::to_string).collect();
        view! {
            <button
                class="chip mythic-badge"
                title="Open the Mythic track"
                on:click=move |_| planner.tab.set(Tab::Mythic)
            >
                <span class="chip-label">"Mythic"</span>
                <span class="chip-value">{format!("✦ MR {}", ranks.join(" · "))}</span>
            </button>
        }
    });
    view! {
        <div class="row" role="listitem">
            <span class="row-number">{n}</span>
            <button
                class="class-button"
                disabled=!changeable
                title=(!changeable).then_some("Later choices depend on this class")
                on:click=move |_| planner.open(Target::Class(n))
            >
                <span class="class-line">
                    <span class="class-name">{class_name}</span>
                    " "
                    <span class="class-level">{level.class_level}</span>
                    {(!changeable)
                        .then(|| view! { <span class="lock" aria-label="Locked">"🔒"</span> })}
                </span>
                {archetype.map(|archetype| view! { <span class="class-archetype">{archetype}</span> })}
            </button>
            <div class="chips">{chips} {increase} {skills} {mythic}</div>
        </div>
    }
}

/// A slot's chip: what it takes, and its pick or "+". `lost` marks a pick a Path switch took away.
pub fn slot_chip(engine: &Engine, slot: &Slot, lost: bool) -> impl IntoView + use<> {
    let planner = use_planner();
    let label = engine.slot_label(slot);
    let title = label.clone();
    let value = slot.pick.as_ref().map(|pick| engine.pick_name(pick));
    let icon = slot
        .pick
        .as_ref()
        .and_then(|pick| engine.feat_category(pick));
    let empty = value.is_none();
    let id = slot.id.clone();
    view! {
        <button
            class="chip"
            class:empty=empty
            class:lost=lost
            title=title
            on:click=move |_| planner.open(Target::Slot(id.clone()))
        >
            // The feat type's icon sits in the label line, which never sets the chip's width, so the grid stays as
            // dense as without it.
            <span class="chip-label">{icon.map(feat_icon)}{label}</span>
            <span class="chip-value">
                {value.unwrap_or_else(|| "+".to_owned())}
                {lost.then(|| view! { <span class="lost-tag">"lost"</span> })}
            </span>
        </button>
    }
}

/// Where the Build ends: classes, BAB, saves, ability scores, class skills and the notes the sheet shows.
#[component]
fn Summary(sheet: std::sync::Arc<Sheet>) -> impl IntoView {
    let planner = use_planner();
    let engine = planner.engine();
    let Some(last) = sheet.levels.last() else {
        return view! {
            <p class="summary muted">
                "Choose the Build start, then add level 1 below."
            </p>
        }
        .into_any();
    };
    let state = &last.state;
    let classes: Vec<String> = state
        .classes
        .iter()
        .map(|class| format!("{} {}", engine.class_name(&class.class), class.level))
        .collect();
    let scores: Vec<String> = AbilityScores::ABILITIES
        .into_iter()
        .map(|ability| {
            format!(
                "{} {}",
                ability.short_name(),
                state.ability_scores.get(ability)
            )
        })
        .collect();
    let class_skills: Vec<&str> = state
        .class_skills
        .iter()
        .map(|skill| engine.skill_name(skill))
        .collect();
    let daily: Vec<String> = engine
        .daily_choices(state)
        .iter()
        .map(|feature| engine.feature_name(feature).to_owned())
        .collect();
    let faith = state.faith.as_ref().map(|faith| {
        let name = engine.faith_name(faith).to_owned();
        match engine.favoured_weapons_note(state) {
            Some(note) => format!("{name} · {note}"),
            None => name,
        }
    });
    let mythic = (state.mythic_rank > 0).then(|| {
        let path = state
            .mythic_path
            .as_ref()
            .map(|path| format!(" {}", engine.path_name(path)))
            .unwrap_or_default();
        format!(" · MR {}{path}", state.mythic_rank)
    });
    view! {
        <section class="summary" aria-label="Summary">
            <p class="summary-classes">
                <strong>{format!("Level {}", state.character_level)}</strong>
                {format!(" · {}", classes.join(" / "))}
                {mythic}
            </p>
            <p class="summary-numbers">
                {format!(
                    "BAB +{} · Fort +{} · Ref +{} · Will +{}",
                    state.bab,
                    state.fort,
                    state.reflex,
                    state.will,
                )}
            </p>
            <p class="summary-numbers">{scores.join(" · ")}</p>
            {faith.map(|faith| view! { <p class="summary-note">{format!("Faith: {faith}")}</p> })}
            {(!class_skills.is_empty())
                .then(|| {
                    view! {
                        <p class="summary-note">
                            {format!("Class skills: {}", class_skills.join(", "))}
                        </p>
                    }
                })}
            {(!daily.is_empty())
                .then(|| {
                    view! {
                        <p class="summary-note">
                            {format!("Chosen each day in the game: {}", daily.join(", "))}
                        </p>
                    }
                })}
        </section>
    }
    .into_any()
}
