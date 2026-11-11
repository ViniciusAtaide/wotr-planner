//! The Build start (race, alignment, deity, background, point buy) and a level's skill ranks.

use leptos::prelude::*;
use rules::{Address, Build, BuildStart, Choice, Edit, POINT_BUY_BUDGET, Step};

use crate::picker::{Head, heading};
use crate::planner::{Target, use_planner};
use crate::timings::timed;

/// The Build start, ahead of the grid: open while the Build has no levels, folded to one line after. It folds
/// itself when level 1 is added, so the grid's edits don't redo its checks, and opens again when the last level goes.
#[component]
pub fn BuildStartPanel() -> AnyView {
    let planner = use_planner();
    let has_levels = Memo::new(move |_| planner.build.with(|build| !build.levels.is_empty()));
    let open = RwSignal::new(!has_levels.get_untracked());
    Effect::new(move |had_levels: Option<bool>| {
        let has = has_levels.get();
        // Folds when the Build gains its first level, opens when it loses its last.
        if had_levels.is_some_and(|had| had != has) {
            open.set(!has);
        }
        has
    });
    let summary = move || {
        let engine = planner.engine();
        planner.build.with(|build| {
            let start = &build.start;
            let mut parts = Vec::new();
            parts.push(match (&start.subrace, &start.race) {
                (Some(subrace), _) => engine.subrace_name(subrace).to_owned(),
                (None, Some(race)) => engine.race_name(race).to_owned(),
                (None, None) => "No race".to_owned(),
            });
            parts.push(
                start
                    .alignment
                    .map_or("No alignment", |alignment| alignment.name())
                    .to_owned(),
            );
            parts.push(start.deity.as_ref().map_or("No deity".to_owned(), |faith| {
                engine.faith_name(faith).to_owned()
            }));
            parts.push(
                start
                    .background
                    .as_ref()
                    .map_or("No background".to_owned(), |key| {
                        engine.background_name(key).to_owned()
                    }),
            );
            parts.join(" · ")
        })
    };
    view! {
        <section class="start">
            <button
                class="start-toggle"
                aria-expanded=move || open.get().to_string()
                on:click=move |_| open.update(|open| *open = !*open)
            >
                <span class="section-title">"Build start"</span>
                <span class="start-summary">{summary}</span>
                <span class="chevron" class:open=move || open.get()>
                    "▾"
                </span>
            </button>
            {move || open.get().then(|| view! { <StartBody/> })}
        </section>
    }
    .into_any()
}

#[component]
fn StartBody() -> AnyView {
    let planner = use_planner();
    let engine = planner.engine();
    (move || {
        let build = planner.build.get();
        let start = build.start.clone();
        let race = match (&start.subrace, &start.race) {
            (Some(subrace), _) => Some(engine.subrace_name(subrace).to_owned()),
            (None, Some(race)) => Some(engine.race_name(race).to_owned()),
            (None, None) => None,
        };
        let alignment = start.alignment.map(|alignment| alignment.name().to_owned());
        let deity = start
            .deity
            .as_ref()
            .map(|faith| engine.faith_name(faith).to_owned());
        let background = start
            .background
            .as_ref()
            .map(|key| engine.background_name(key).to_owned());
        let note = start
            .background
            .as_ref()
            .map(|key| engine.background_note(key).to_owned())
            .filter(|note| !note.is_empty());
        let label = |address: Address| engine.choice_label(&address, None);
        // Only a race that chooses where its bonus goes has options.
        let chosen = racial_choice(&start);
        view! {
            <div class="start-body">
                {choice_row(label(Address::Race), race, Target::Race)}
                {chosen}
                // The faith before the alignment, which needs it (ticket 94).
                {choice_row(label(Address::Faith), deity, Target::Deity)}
                {choice_row(label(Address::Alignment), alignment, Target::Alignment)}
                {choice_row(label(Address::Background), background, Target::Background)}
                {note.map(|note| view! { <p class="note">{note}</p> })}
                <PointBuyRows current=build/>
            </div>
        }
    })
    .into_any()
}

fn choice_row(label: String, value: Option<String>, target: Target) -> impl IntoView + use<> {
    let planner = use_planner();
    let empty = value.is_none();
    view! {
        <div class="field">
            <span class="field-label">{label}</span>
            <button
                class="field-value"
                class:empty=empty
                on:click=move |_| planner.open(target.clone())
            >
                {value.unwrap_or_else(|| "Choose…".to_owned())}
            </button>
        </div>
    }
}

/// Where a race's "+2 to one ability score" goes: the engine's options, none for a race that doesn't choose.
fn racial_choice(start: &BuildStart) -> Option<impl IntoView + use<>> {
    let planner = use_planner();
    let options = planner.ask("racial_ability_options", |engine, build| {
        engine.racial_ability_options(build)
    });
    if options.is_empty() {
        return None;
    }
    let buttons = options
        .into_iter()
        .map(|option| {
            let on = start.racial_ability == Some(option.value);
            let edit = option.edit;
            view! {
                <button
                    class="seg"
                    class:on=on
                    disabled=option.status.is_err()
                    on:click=move |_| {
                        if !on {
                            planner.edit(edit.clone());
                        }
                    }
                >
                    {option.value.short_name()}
                </button>
            }
        })
        .collect_view();
    Some(view! {
        <div class="field">
            <span class="field-label">"+2 to"</span>
            <div class="segs">{buttons}</div>
        </div>
    })
}

/// A stepper's step's edit, when it can be taken now.
fn open_step(step: Option<Step>) -> Option<Edit> {
    step.filter(|step| step.status.is_ok())
        .map(|step| step.edit)
}

/// The point buy: each score's steps down and up, as the engine offers them (7 to 18, within the budget, breaking
/// nothing later).
#[component]
fn PointBuyRows(current: Build) -> impl IntoView {
    let build = current;
    let planner = use_planner();
    let engine = planner.engine();
    let start = build.start.clone();
    let point_buy = start.ability_scores;
    let left = i16::from(POINT_BUY_BUDGET) - i16::from(point_buy.cost());
    let steppers = planner.ask("point_buy_steps", |engine, build| {
        engine.point_buy_steps(build)
    });
    let rows = steppers
        .into_iter()
        .map(|stepper| {
            let ability = stepper.value;
            let (down, up) = (open_step(stepper.down), open_step(stepper.up));
            let score = point_buy.get(ability);
            let modifier = engine.racial_modifier(&start, ability);
            let total = i16::from(score) + i16::from(modifier);
            let racial = match modifier {
                0 => String::new(),
                m if m > 0 => format!("+{m}"),
                m => format!("{m}"),
            };
            view! {
                <div class="score">
                    <span class="score-name">{ability.short_name()}</span>
                    <button
                        class="step"
                        aria-label=format!("Lower {}", ability.name())
                        disabled=down.is_none()
                        on:click=move |_| {
                            if let Some(edit) = down.clone() {
                                planner.edit(edit);
                            }
                        }
                    >
                        "−"
                    </button>
                    <span class="score-value">{score}</span>
                    <button
                        class="step"
                        aria-label=format!("Raise {}", ability.name())
                        disabled=up.is_none()
                        on:click=move |_| {
                            if let Some(edit) = up.clone() {
                                planner.edit(edit);
                            }
                        }
                    >
                        "+"
                    </button>
                    <span class="score-racial">{racial}</span>
                    <span class="score-total">{total}</span>
                </div>
            }
        })
        .collect_view();
    view! {
        <div class="point-buy">
            <div class="point-buy-head">
                <span class="field-label">"Point buy"</span>
                <span class="points" class:spent=left == 0>
                    {format!("{left} of {POINT_BUY_BUDGET} points left")}
                </span>
            </div>
            <div class="scores">{rows}</div>
        </div>
    }
}

/// A level's skill ranks: ranks added per skill, within the level's skill points and the character level.
#[component]
pub fn SkillsPanel(level: u8) -> impl IntoView {
    let planner = use_planner();
    let engine = planner.engine();
    let (title, place) = heading(&engine, &Choice::SkillPoints { level });
    let index = usize::from(level) - 1;
    let rows = move || {
        let sheet = planner.sheet.get();
        let build = planner.build.get();
        let (Some(level_sheet), Some(level_up)) =
            (sheet.levels.get(index), build.levels.get(index))
        else {
            return view! { <p class="empty">"This level isn't in the Build any more."</p> }
                .into_any();
        };
        let points = level_sheet.skill_points;
        let state = &level_sheet.state;
        // Each skill's rank fewer and rank more, as the engine offers them: within the level's skill points, no total
        // above the character level here or later, and breaking nothing later.
        let Some(steppers) = timed("skill_steps", || engine.skill_steps(&build, level)) else {
            return view! { <p class="empty">"This level isn't in the Build any more."</p> }
                .into_any();
        };
        let skills = steppers
            .into_iter()
            .filter_map(|stepper| {
                let skill = engine
                    .skills()
                    .iter()
                    .find(|skill| skill.key == stepper.value)?;
                let here = level_up.skill_ranks.get(&skill.key).copied().unwrap_or(0);
                let (down, up) = (open_step(stepper.down), open_step(stepper.up));
                let total = state.skill_ranks(&skill.key);
                let class_skill = state.class_skills.contains(&skill.key);
                let name = skill.name.clone();
                Some(view! {
                    <li class="skill">
                        <span class="skill-name">
                            {name}
                            <span class="skill-ability">{skill.ability.short_name()}</span>
                            {class_skill
                                .then(|| {
                                    view! {
                                        <span class="class-skill" title="Class skill">
                                            "class"
                                        </span>
                                    }
                                })}
                        </span>
                        <span class="skill-total" title="Ranks after this level">
                            {total}
                        </span>
                        <button
                            class="step"
                            aria-label=format!("One rank fewer in {}", skill.name)
                            disabled=down.is_none()
                            on:click=move |_| {
                                if let Some(edit) = down.clone() {
                                    planner.edit(edit);
                                }
                            }
                        >
                            "−"
                        </button>
                        <span class="skill-here">{here}</span>
                        <button
                            class="step"
                            aria-label=format!("One rank more in {}", skill.name)
                            disabled=up.is_none()
                            on:click=move |_| {
                                if let Some(edit) = up.clone() {
                                    planner.edit(edit);
                                }
                            }
                        >
                            "+"
                        </button>
                    </li>
                })
            })
            .collect_view();
        view! {
            <p class="picker-step">
                {format!(
                    "{} of {} skill points spent · at most {level} ranks in a skill",
                    points.spent,
                    points.available,
                )}
            </p>
            <div class="picker-list">
                <div class="skill-head">
                    <span>"Skill"</span>
                    <span>"Total"</span>
                    <span class="skill-head-here">"This level"</span>
                </div>
                <ul class="skills">{skills}</ul>
            </div>
        }
        .into_any()
    };
    view! {
        <Head title place/>
        {rows}
    }
}
