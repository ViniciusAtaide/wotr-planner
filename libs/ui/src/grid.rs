//! The levels grid: a row per character level with its class, one chip per slot, the ability increase, skill
//! ranks and a badge for the Mythic ranks taken there (a rank taken later than the rank table's level has its own,
//! with the control that moves it); then the summary of where the Build ends, with each rank's level once one is
//! moved.

use leptos::prelude::*;
use rules::{
    AbilityScores, Build, Choice, Engine, FeatCategory, LevelSheet, Save, Sheet, Slot, SpellSchool,
    SpellTraits, WeaponKey,
};

use crate::icons::feat_icon;
use crate::planner::{AttackWeapon, Tab, Target, use_planner};

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
                .map(|level| level_row(&engine, build, &sheet, level))
                .collect_view();
            let next = u8::try_from(build.levels.len() + 1).unwrap_or(u8::MAX);
            let can_add = next <= sheet.level_cap;
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
                                    {if next == 1 {
                                        "Choose your first level".to_owned()
                                    } else {
                                        format!("+ Level {next}")
                                    }}
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

/// Why a level's class can't be changed now: a tap on its button shows it.
const LOCKED_CLASS: &str = "Later choices depend on this class";

fn level_row(
    engine: &Engine,
    build: &Build,
    sheet: &Sheet,
    level: &LevelSheet,
) -> impl IntoView + use<> {
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
    // The last level's withdrawal, Remove level: an × on the row's left edge.
    let remove = (usize::from(n) == build.levels.len())
        .then(|| planner.withdrawal(&Choice::CharacterLevel { level: n }))
        .flatten()
        .map(|(edit, removable)| {
            let edit = StoredValue::new(edit);
            let label = format!("Remove level {n}");
            view! {
                <button
                    class="remove-level"
                    aria-label=label.clone()
                    title=label
                    disabled=!removable
                    on:click=move |_| planner.edit(edit.get_value())
                >
                    "×"
                </button>
            }
        });
    // A class whose entry only the gear meets says which items, under its archetype.
    let gear = planner.gear_note(&Choice::Class { level: n });
    let chips = level
        .slots
        .iter()
        .map(|slot| slot_chip(engine, slot, false))
        .collect_view();
    let increase = level.ability_increase.then(|| {
        let ability = build.levels[usize::from(n) - 1].ability_increase;
        let value = ability.map_or("+".to_owned(), |ability| ability.increase_short_name());
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
    // The Mythic ranks taken at this level. Those at their rank table's level share one badge, as they always did;
    // a rank taken later (ticket 171) has a badge of its own that says where it came from, with the control that
    // moves it. Either badge opens the Mythic track, where every reached rank has that control.
    // The Build's path at this level, if it has one, sets the badges' colour.
    let path = level
        .state
        .mythic_path
        .as_ref()
        .map(|key| key.as_str().to_owned());
    let moved_from = |rank: u8| {
        engine
            .rank_table_level(rank)
            .filter(|earliest| n > *earliest)
    };
    let mut mythic = Vec::new();
    let mut ranks = level.mythic_ranks.iter().copied().peekable();
    while let Some(rank) = ranks.next() {
        let path = path.clone();
        match moved_from(rank) {
            Some(earliest) => {
                let move_control = rank_move(engine, build, sheet, rank);
                mythic.push(
                    view! {
                        <span class="rank-badge">
                            <button
                                class="chip mythic-badge moved"
                                data-path=path
                                data-rank=rank
                                title="Open the Mythic track"
                                on:click=move |_| planner.tab.set(Tab::Mythic)
                            >
                                <span class="chip-label">"Mythic"</span>
                                <span class="chip-value">{format!("✦ MR {rank}")}</span>
                                <span class="chip-moved">{format!("from level {earliest}")}</span>
                            </button>
                            {move_control}
                        </span>
                    }
                    .into_any(),
                );
            }
            None => {
                let mut together = vec![rank.to_string()];
                while let Some(next) = ranks.next_if(|next| moved_from(*next).is_none()) {
                    together.push(next.to_string());
                }
                mythic.push(
                    view! {
                        <button
                            class="chip mythic-badge"
                            data-path=path
                            title="Open the Mythic track"
                            on:click=move |_| planner.tab.set(Tab::Mythic)
                        >
                            <span class="chip-label">"Mythic"</span>
                            <span class="chip-value">{format!("✦ MR {}", together.join(" · "))}</span>
                        </button>
                    }
                    .into_any(),
                );
            }
        }
    }
    // A locked class stays tappable: a tap shows why it's locked under its name, and another hides it again, since a
    // touch screen never shows a title.
    let why = RwSignal::new(false);
    let locked = !changeable;
    view! {
        <div class="row" role="listitem">
            {remove}
            <span class="row-number">{n}</span>
            <button
                class="class-button"
                class:locked=locked
                aria-disabled=locked.then_some("true")
                title=locked.then_some(LOCKED_CLASS)
                on:click=move |_| {
                    if locked {
                        why.update(|why| *why = !*why);
                    } else {
                        planner.open(Target::Class(n));
                    }
                }
            >
                <span class="class-line">
                    <span class="class-name">{class_name}</span>
                    " "
                    <span class="class-level">{level.class_level}</span>
                    {locked
                        .then(|| view! { <span class="lock" aria-label="Locked">"🔒"</span> })}
                </span>
                {archetype.map(|archetype| view! { <span class="class-archetype">{archetype}</span> })}
                {gear.map(|gear| view! { <span class="class-gear">{gear}</span> })}
                {move || {
                    why.get()
                        .then(|| view! { <span class="class-locked">{LOCKED_CLASS}</span> })
                }}
            </button>
            <div class="chips">{chips} {increase} {skills} {mythic}</div>
        </div>
    }
}

/// The control that moves a Mythic rank to another level: a thumb-sized button beside the rank that draws a small
/// round mark, which opens the rank's level picker. `None` for a rank with nowhere to go (not reached, or reached only at the Build's last level with
/// nothing to come back from).
pub fn rank_move(
    engine: &Engine,
    build: &Build,
    sheet: &Sheet,
    rank: u8,
) -> Option<impl IntoView + use<>> {
    let planner = use_planner();
    let status = sheet.ranks.iter().find(|sheet| sheet.rank == rank)?.status;
    let level = engine.rank_level(build, rank)?;
    engine.rank_can_move(build, rank, status).then(|| {
        let label = format!("Move MR {rank}: taken at level {level}");
        view! {
            <button
                class="rank-move"
                aria-label=label.clone()
                title=label
                on:click=move |_| planner.open(Target::RankLevel(rank))
            >
                <span class="rank-move-mark" aria-hidden="true">
                    "⇅"
                </span>
            </button>
        }
    })
}

/// A slot's chip: what it takes, and its pick or "+". `lost` marks a pick a Path switch took away. A pick whose value
/// is still open (Favorite Metamagic from an older link) shows "(?)", and the title says what to choose.
pub fn slot_chip(engine: &Engine, slot: &Slot, lost: bool) -> impl IntoView + use<> {
    let planner = use_planner();
    let label = engine.slot_label(slot);
    let left_open = slot
        .pick
        .as_ref()
        .and_then(|pick| engine.value_left_open(pick));
    let title = match left_open {
        Some(kind) => format!("{label}: choose {}", kind.with_article()),
        None => label.clone(),
    };
    let value = slot.pick.as_ref().map(|pick| match left_open {
        Some(_) => format!("{} (?)", engine.pick_name(pick)),
        None => engine.pick_name(pick),
    });
    let feat_category = slot
        .pick
        .as_ref()
        .and_then(|pick| engine.feat_category(pick));
    let empty = value.is_none();
    // A pick allowed only by the Build's gear says which items, under its value (where a story note goes).
    let gear = planner.gear_note(&Choice::Slot(slot.id.clone()));
    let title = match &gear {
        Some(gear) => format!("{title} · {gear}"),
        None => title,
    };
    let id = slot.id.clone();
    let with_gear = gear.is_some();
    view! {
        <button
            class="chip"
            class:empty=empty
            class:lost=lost
            class:with-gear=with_gear
            title=title
            on:click=move |_| planner.open(Target::Slot(id.clone()))
        >
            // The feat category's icon sits in the label line, which never sets the chip's width, so the grid stays
            // as dense as without it.
            <span class="chip-label">{feat_category.map(feat_icon)}{label}</span>
            <span class="chip-value">
                {value.unwrap_or_else(|| "+".to_owned())}
                {lost.then(|| view! { <span class="lost-tag">"lost"</span> })}
            </span>
            {gear.map(|gear| view! { <span class="chip-gear">{gear}</span> })}
        </button>
    }
}

/// Where the Build ends: classes, BAB, saves, ability scores, class skills and the notes the sheet shows. A Build
/// with no levels has none: the "Choose your first level" button under it is all the empty grid needs.
#[component]
fn Summary(sheet: std::sync::Arc<Sheet>) -> impl IntoView {
    let planner = use_planner();
    let engine = planner.engine();
    let Some(last) = sheet.levels.last() else {
        return ().into_any();
    };
    let state = &last.state;
    let classes: Vec<String> = state
        .classes
        .iter()
        .map(|class| format!("{} {}", engine.class_name(&class.class), class.level))
        .collect();
    // With gear, each score it changes has its naked value beside it, smaller; without gear there's none.
    let naked = planner.naked.get();
    let naked_scores = (naked.as_ref())
        .and_then(|naked| naked.levels.last())
        .map(|level| level.state.ability_scores);
    let scores = AbilityScores::ABILITIES
        .into_iter()
        .enumerate()
        .map(|(index, ability)| {
            let score = state.ability_scores.get(ability);
            let bare = naked_scores
                .map(|scores| scores.get(ability))
                .filter(|bare| *bare != score);
            view! {
                {(index > 0).then_some(" · ")}
                {format!("{} {score}", ability.short_name())}
                {bare.map(|bare| naked_number(bare.to_string()))}
            }
        })
        .collect_view();
    // With gear, every other total it changes, with both numbers: none without gear, and nothing when none differs.
    let gear_totals = naked
        .as_ref()
        .and_then(|naked| naked.levels.last())
        .map(|bare| gear_totals(&engine, last, bare))
        .filter(|totals| !totals.is_empty())
        .map(|totals| {
            let totals = totals
                .into_iter()
                .enumerate()
                .map(|(index, total)| {
                    view! {
                        {(index > 0).then_some(" · ")}
                        <span class="gear-total">
                            {format!("{} {}", total.label, total.with)}
                            {naked_number(total.naked)}
                        </span>
                    }
                })
                .collect_view();
            view! {
                <p class="summary-numbers gear-totals">
                    <span class="gear-totals-label">"Totals with gear: "</span>
                    {totals}
                </p>
            }
        });
    let gear_dependents: Vec<String> = planner
        .gear_dependents()
        .into_iter()
        .map(|(name, note)| format!("{name} · {note}"))
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
    // Each reached rank's level, once the Build takes a rank later than the rank table's level, each such rank
    // saying so (ticket 171). A Build that moves no rank has no such line.
    let later = |rank: &rules::RankSheet| {
        engine
            .rank_table_level(rank.rank)
            .filter(|earliest| rank.character_level > *earliest)
    };
    let reached =
        || (sheet.ranks.iter()).filter(|rank| rank.status != rules::RankStatus::NotReached);
    let rank_levels: Vec<String> = match reached().any(|rank| later(rank).is_some()) {
        false => Vec::new(),
        true => reached()
            .map(|rank| {
                let later = later(rank)
                    .map(|earliest| format!(" (from {earliest})"))
                    .unwrap_or_default();
                format!("MR {} at {}{later}", rank.rank, rank.character_level)
            })
            .collect(),
    };
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
            <div class="summary-info">
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
                <p class="summary-numbers">{scores}</p>
                {gear_totals}
                {(!gear_dependents.is_empty())
                    .then(|| {
                        view! {
                            <p class="summary-note gear-note">
                                {format!("Gear-dependent: {}", gear_dependents.join("; "))}
                            </p>
                        }
                    })}
                {(!rank_levels.is_empty())
                    .then(|| {
                        view! {
                            <p class="summary-note rank-levels">
                                {format!("Mythic ranks by level: {}", rank_levels.join(" · "))}
                            </p>
                        }
                    })}
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
            </div>
            <AttackBlock sheet=sheet.clone()/>
        </section>
    }
    .into_any()
}

/// The attack line at the Build's last level, for the weapon chosen beside it (by default the Build's first Weapon
/// Focus): the engine's numbers, what they count and what they leave out.
#[component]
fn AttackBlock(sheet: std::sync::Arc<Sheet>) -> impl IntoView {
    let planner = use_planner();
    let engine = planner.engine();
    let default = engine.default_attack_weapon(&sheet);
    let weapon = Memo::new(move |_| match planner.attack_weapon.get() {
        AttackWeapon::Default => default.clone(),
        AttackWeapon::NoWeapon => None,
        AttackWeapon::Weapon(key) => Some(key),
    });
    let options = engine
        .attack_weapons()
        .into_iter()
        .map(|option| {
            let key = option.key.as_str().to_owned();
            let selected = {
                let key = key.clone();
                move || {
                    weapon
                        .with(|weapon| weapon.as_ref().is_some_and(|weapon| weapon.as_str() == key))
                }
            };
            view! { <option value=key selected=selected>{option.name.clone()}</option> }
        })
        .collect_view();
    let line = {
        let sheet = sheet.clone();
        move || {
            let key = weapon.get()?;
            let state = &sheet.levels.last()?.state;
            let line = engine.attack_line(state, &key)?;
            let power_attack = line.power_attack.map(|trade| {
                format!(
                    "Power Attack: -{} to attack, +{} damage",
                    trade.attack_penalty, trade.damage_bonus
                )
            });
            let counts: Vec<String> = line
                .parts
                .iter()
                .map(|part| match (part.attack, part.damage) {
                    (attack, 0) => format!("{} {attack:+} attack", part.name),
                    (0, damage) => format!("{} {damage:+} damage", part.name),
                    (attack, damage) => {
                        format!("{} {attack:+} attack, {damage:+} damage", part.name)
                    }
                })
                .collect();
            let leaves_out = line
                .leaves_out
                .iter()
                .map(|item| view! { <li>{*item}</li> })
                .collect_view();
            // The same weapon's line without the gear, when it differs.
            let texts = |line: &rules::AttackLine| {
                (
                    line.attacks_text(),
                    line.damage_text(),
                    line.critical_text(),
                )
            };
            let bare = (planner.naked.get())
                .and_then(|naked| engine.attack_line(&naked.levels.last()?.state, &key))
                .map(|bare| texts(&bare))
                .filter(|bare| *bare != texts(&line))
                .map(|(attacks, damage, critical)| {
                    view! {
                        <p class="summary-numbers naked-line">
                            {naked_number(format!("{attacks} · {damage} · {critical}"))}
                        </p>
                    }
                });
            Some(view! {
                <p class="summary-numbers attack-numbers">
                    <strong>{line.attacks_text()}</strong>
                    {format!(" · {} · {}", line.damage_text(), line.critical_text())}
                </p>
                {bare}
                {power_attack.map(|text| view! { <p class="summary-note">{text}</p> })}
                <p class="summary-note">{format!("Counts: BAB; {}", counts.join("; "))}</p>
                <details class="attack-leaves-out">
                    <summary>"Not counted"</summary>
                    <ul>{leaves_out}</ul>
                </details>
            })
        }
    };
    view! {
        <div class="attack">
            <label class="attack-weapon" title="Attack with">
                <span class="attack-label">{feat_icon(FeatCategory::Weapon)}</span>
                <select aria-label="Attack with" on:change=move |event| {
                    let value = event_target_value(&event);
                    planner
                        .attack_weapon
                        .set(if value.is_empty() {
                            AttackWeapon::NoWeapon
                        } else {
                            AttackWeapon::Weapon(WeaponKey::new(&value))
                        });
                }>
                    <option value="" selected=move || weapon.with(Option::is_none)>
                        "No weapon"
                    </option>
                    {options}
                </select>
            </label>
            {line}
        </div>
    }
}

/// A naked number beside its number with gear: smaller, labelled "naked" (the number without the Build's gear).
fn naked_number(text: String) -> impl IntoView {
    view! {
        <span class="naked" title="Without gear">
            <span class="naked-label">" naked "</span>
            {text}
        </span>
    }
}

/// A highest spell level's DC changed by gear: the spell level, the DC with gear and naked.
type DcChange = (u8, i16, i16);

/// A total that gear changes: its label, and its number with gear and naked.
struct GearTotal {
    label: String,
    with: String,
    naked: String,
}

/// Every total the gear changes at the Build's last level (`level`, and `bare`, the same level of the naked sheet),
/// in the summary's order: the saves, initiative, each skill, then the caster line's caster level, spell
/// penetration, concentration and its highest spell level's DC. Only those that differ.
fn gear_totals(engine: &Engine, level: &LevelSheet, bare: &LevelSheet) -> Vec<GearTotal> {
    let mut totals = Vec::new();
    let mut add = |label: &str, with: i16, naked: i16, signed: bool| {
        if with != naked {
            let text = |number: i16| {
                if signed {
                    format!("{number:+}")
                } else {
                    number.to_string()
                }
            };
            totals.push(GearTotal {
                label: label.to_owned(),
                with: text(with),
                naked: text(naked),
            });
        }
    };
    let (with, naked) = (engine.totals(level), engine.totals(bare));
    for (save, label) in [
        (Save::Fortitude, "Fortitude"),
        (Save::Reflex, "Reflex"),
        (Save::Will, "Will"),
    ] {
        add(label, with.save(save).total, naked.save(save).total, true);
    }
    add(
        "Initiative",
        with.initiative.total,
        naked.initiative.total,
        true,
    );
    for skill in &with.skills {
        if let Some(bare) = naked.skill(&skill.skill) {
            add(
                engine.skill_name(&skill.skill),
                skill.total.total,
                bare.total.total,
                true,
            );
        }
    }
    // The caster line of a spell of each school: the caster level, spell penetration and concentration are the
    // same for every school, a DC may not be (a bonus to one school's).
    let lines = |level: &LevelSheet| {
        SpellSchool::ALL.map(|school| {
            let spell = SpellTraits {
                school,
                subschool: None,
                energy: None,
                metamagic: false,
            };
            engine.caster_line(level, spell)
        })
    };
    let (with_lines, naked_lines) = (lines(level), lines(bare));
    if let (Some(with_line), Some(naked_line)) = (&with_lines[0], &naked_lines[0]) {
        add(
            "Caster level",
            i16::from(with_line.caster_level),
            i16::from(naked_line.caster_level),
            false,
        );
        add(
            "Spell penetration",
            with_line.spell_penetration,
            naked_line.spell_penetration,
            true,
        );
        add(
            "Concentration",
            with_line.concentration,
            naked_line.concentration,
            true,
        );
        // The highest spell level's DC, once for the schools that change alike: "Spell DC (level 3)" when every
        // school does, else naming them.
        let mut groups: Vec<(DcChange, Vec<&str>)> = Vec::new();
        for (school, (with_line, naked_line)) in SpellSchool::ALL
            .into_iter()
            .zip(with_lines.iter().zip(&naked_lines))
        {
            let (Some(with_dc), Some(naked_dc)) = (
                with_line.as_ref().and_then(|line| line.dcs.last()),
                naked_line.as_ref().and_then(|line| line.dcs.last()),
            ) else {
                continue;
            };
            if with_dc.spell_level != naked_dc.spell_level || with_dc.dc == naked_dc.dc {
                continue;
            }
            let key = (with_dc.spell_level, with_dc.dc, naked_dc.dc);
            match groups.iter_mut().find(|(same, _)| *same == key) {
                Some((_, schools)) => schools.push(school.name()),
                None => groups.push((key, vec![school.name()])),
            }
        }
        for ((spell_level, with_dc, naked_dc), schools) in groups {
            let label = if schools.len() == SpellSchool::ALL.len() {
                format!("Spell DC (level {spell_level})")
            } else {
                format!("Spell DC (level {spell_level}, {})", schools.join(", "))
            };
            add(&label, with_dc, naked_dc, false);
        }
    }
    totals
}
