//! The Sheet: the one planning screen. It loads the game data once, runs the rules engine in the browser, and
//! keeps the Build in the URL fragment.

use std::collections::BTreeSet;
use std::sync::Arc;

use leptos::prelude::*;
use rules::{
    Ability, Build, Choice, Edit, Engine, GearDependent, GearKey, GearPlace, GearTarget, Sheet,
    SlotId, StaleLink, WeaponKey,
};

use crate::browser::{self, Fragment};
use crate::export::BuildSheetActions;
use crate::gear::GearTab;
use crate::grid::LevelsGrid;
use crate::mythic::MythicTrack;
use crate::picker::Picker;
use crate::start::BuildStartPanel;
use crate::timings::timed;

/// What the open picker is for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Slot(SlotId),
    /// The class (and archetype) at a character level; one past the end adds a level.
    Class(u8),
    /// The first ascension's path-linked ability (MR 1).
    FirstAscension,
    Path,
    Switch,
    /// The character level a Mythic rank is taken at.
    RankLevel(u8),
    Race,
    Alignment,
    Deity,
    Background,
    AbilityIncrease(u8),
    Skills(u8),
    /// The item worn in a gear place.
    Gear(GearPlace),
    /// The usable items (the manuals and tomes), a tick each.
    Tomes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Levels,
    Mythic,
    Gear,
}

/// The weapon the summary's attack line is for. Not part of the Build or its link.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum AttackWeapon {
    /// Nothing chosen yet: the engine's default, the Build's first Weapon Focus (no weapon when it has none).
    #[default]
    Default,
    /// "No weapon" chosen: no attack line.
    NoWeapon,
    /// This weapon chosen.
    Weapon(WeaponKey),
}

/// The sheet's state: the engine, the Build, and what's derived from it once per edit.
#[derive(Clone, Copy)]
pub struct Planner {
    engine: StoredValue<Arc<Engine>>,
    pub build: RwSignal<Build>,
    /// The engine's walk of the Build, once per edit.
    pub sheet: Memo<Arc<Sheet>>,
    /// The levels whose class may be changed now, once per edit (and only when something asks): the grid's class
    /// buttons. Any other choice is asked about one at a time ([`withdrawal`](Self::withdrawal)).
    removable_classes: Memo<Arc<BTreeSet<u8>>>,
    /// The sheet of the Build without its gear, for the naked numbers, once per edit (and only when something
    /// asks): `None` for a Build without gear, at no cost.
    pub naked: Memo<Option<Arc<Sheet>>>,
    /// The choices the Build makes only because of its gear, each with the items it needs, once per edit (and only
    /// when something asks): empty for a Build without gear, at no cost.
    gear_dependents: Memo<Arc<Vec<GearDependent>>>,
    pub picker: RwSignal<Option<Target>>,
    pub tab: RwSignal<Tab>,
    /// The weapon the summary's attack line is for: the engine's default until one is chosen.
    pub attack_weapon: RwSignal<AttackWeapon>,
    /// The one generic error a link that opens nothing shows.
    pub error: RwSignal<Option<String>>,
}

pub fn use_planner() -> Planner {
    expect_context::<Planner>()
}

impl Planner {
    fn new(engine: Arc<Engine>, build: Build, error: Option<String>) -> Self {
        let build = RwSignal::new(build);
        let engine = StoredValue::new(engine);
        let sheet = Memo::new_owning(move |_| {
            let engine = engine.get_value();
            let sheet = build.with(|build| timed("evaluate", || engine.evaluate(build)));
            (Arc::new(sheet), true)
        });
        let removable_classes = Memo::new_owning(move |_| {
            let engine = engine.get_value();
            let levels =
                build.with(|build| timed("removable_classes", || engine.removable_classes(build)));
            (Arc::new(levels), true)
        });
        let naked = Memo::new_owning(move |_| {
            let engine = engine.get_value();
            let naked = build.with(|build| timed("naked_sheet", || engine.naked_sheet(build)));
            (naked.map(Arc::new), true)
        });
        let gear_dependents = Memo::new_owning(move |_| {
            let engine = engine.get_value();
            let dependents =
                build.with(|build| timed("gear_dependents", || engine.gear_dependents(build)));
            (Arc::new(dependents), true)
        });
        Self {
            engine,
            build,
            sheet,
            removable_classes,
            naked,
            gear_dependents,
            picker: RwSignal::new(None),
            tab: RwSignal::new(Tab::Levels),
            attack_weapon: RwSignal::new(AttackWeapon::Default),
            error: RwSignal::new(error),
        }
    }

    pub fn engine(&self) -> Arc<Engine> {
        self.engine.get_value()
    }

    /// Makes an edit the way the sheet makes every edit: through `Engine::apply`, then into the URL.
    pub fn edit(&self, edit: Edit) {
        let engine = self.engine();
        self.build
            .update(|build| timed("apply", || engine.apply(build, &edit)));
        self.edited();
    }

    /// Makes a picker's pick: closes the picker, then makes the edit as [`edit`](Self::edit) does, unless it leaves
    /// the Build as it is (the current pick picked again), which leaves the Build, its URL and the error alone.
    pub fn pick(&self, edit: Edit) {
        self.close();
        let engine = self.engine();
        let mut after = self.build.get_untracked();
        timed("apply", || engine.apply(&mut after, &edit));
        if self.build.with_untracked(|build| *build != after) {
            self.build.set(after);
            self.edited();
        }
    }

    /// After an edit: the URL holds the new Build, and the link's error goes.
    fn edited(&self) {
        self.build.with_untracked(browser::write_fragment);
        self.error.set(None);
    }

    /// Asks the engine about the current Build (a picker's options, a panel's steps), timed under `label`.
    /// Untracked: the views that ask read the Build themselves, so they ask again after each edit.
    pub fn ask<T>(&self, label: &str, ask: impl FnOnce(&Engine, &Build) -> T) -> T {
        let engine = self.engine();
        self.build
            .with_untracked(|build| timed(label, || ask(&engine, build)))
    }

    /// Whether the class at a level may be changed now.
    pub fn class_removable(&self, level: u8) -> bool {
        self.removable_classes
            .with(|levels| levels.contains(&level))
    }

    /// A remove control's answer for a choice: `None` when the Build doesn't hold it, otherwise the edit that takes
    /// it back and whether it may be removed now. Tracked: asked again after each edit.
    pub fn withdrawal(&self, choice: &Choice) -> Option<(Edit, bool)> {
        let engine = self.engine();
        self.build
            .with(|build| timed("withdrawal", || engine.withdrawal(build, choice)))
    }

    /// The mark on a choice the Build makes only because of its gear ("With Belt of Incredible Dexterity +2");
    /// `None` for any other. Tracked: asked again after each edit.
    pub fn gear_note(&self, choice: &Choice) -> Option<String> {
        self.gear_dependents.with(|dependents| {
            (dependents.iter())
                .find(|dependent| dependent.choice == *choice)
                .map(|dependent| self.engine().gear_note(&dependent.needs))
        })
    }

    /// The choices the Build makes only because of gear that raises `ability`, grouped by their mark, each group's
    /// choices named and joined (`("With Belt of Incredible Dexterity +2", "Improved Two-Weapon Fighting and
    /// Greater Two-Weapon Fighting")`), in the order the Build makes them. A choice counts when an item it needs
    /// (any of its lists) gives a bonus to `ability`. Tracked.
    pub fn gear_dependents_raising(&self, ability: Ability) -> Vec<(String, String)> {
        let engine = self.engine();
        let raises = |item: &GearKey| {
            (engine.gear_item(item)).is_some_and(|item| {
                (item.bonuses.iter()).any(|bonus| bonus.target == GearTarget::Ability(ability))
            })
        };
        self.gear_dependents.with(|dependents| {
            let mut groups: Vec<(String, Vec<Choice>)> = Vec::new();
            for dependent in dependents.iter() {
                let needs = &dependent.needs;
                let items = (needs.all_of.iter())
                    .chain(&needs.one_of)
                    .chain(&needs.for_example);
                if !items.into_iter().any(raises) {
                    continue;
                }
                let note = engine.gear_note(needs);
                match groups.iter_mut().find(|(same, _)| *same == note) {
                    Some((_, choices)) => choices.push(dependent.choice.clone()),
                    None => groups.push((note, vec![dependent.choice.clone()])),
                }
            }
            self.build.with(|build| {
                (groups.into_iter())
                    .map(|(note, choices)| (note, engine.choice_names(build, &choices)))
                    .collect()
            })
        })
    }

    /// The choices that can't do without an item the Build holds, named and joined ("Greater Two-Weapon
    /// Fighting"): those whose every way to stay allowed needs it. `None` when none does. Tracked.
    pub fn needing(&self, item: &GearKey) -> Option<String> {
        let engine = self.engine();
        self.gear_dependents.with(|dependents| {
            let choices: Vec<Choice> = (dependents.iter())
                .filter(|dependent| dependent.needs.all_of.contains(item))
                .map(|dependent| dependent.choice.clone())
                .collect();
            (!choices.is_empty()).then(|| {
                self.build
                    .with(|build| engine.choice_names(build, &choices))
            })
        })
    }

    pub fn open(&self, target: Target) {
        self.picker.set(Some(target));
    }

    pub fn close(&self) {
        self.picker.set(None);
    }

    /// Replaces the whole Build (a link pasted into the address bar): what belonged to the old one goes with it.
    fn replace(&self, build: Build, error: Option<String>) {
        self.picker.set(None);
        self.attack_weapon.set(AttackWeapon::Default);
        self.build.set(build);
        self.error.set(error);
    }

    /// Opens what the URL fragment holds: a blank Build, or a link through `Engine::open_link`. A link that
    /// opens nothing gives the one generic error and a blank Build.
    fn open_fragment(engine: &Engine) -> (Build, Option<String>) {
        let opened = match browser::fragment() {
            Fragment::Blank => Ok(Build::default()),
            Fragment::Code(code) => timed("open_link", || engine.open_link(&code)),
            Fragment::Other => Err(StaleLink),
        };
        match opened {
            Ok(build) => (build, None),
            Err(stale) => (Build::default(), Some(stale.to_string())),
        }
    }
}

/// The page: loads the game data once, then shows the sheet.
#[component]
pub fn SheetPage() -> impl IntoView {
    let engine = LocalResource::new(|| async {
        crate::game_data::load()
            .await
            .map(|data| Arc::new(timed("engine", || Engine::new(data))))
    });
    move || match engine.get() {
        None => view! { <p class="loading">"Loading the game data…"</p> }.into_any(),
        Some(Err(error)) => view! {
            <p class="banner banner-error">"The game data didn't load: " {error}</p>
        }
        .into_any(),
        Some(Ok(engine)) => view! { <SheetView engine/> }.into_any(),
    }
}

#[component]
fn SheetView(engine: Arc<Engine>) -> impl IntoView {
    let (build, error) = Planner::open_fragment(&engine);
    let planner = Planner::new(engine, build, error);
    provide_context(planner);

    // Pasting another link into the address bar changes only the fragment, which reloads nothing: open it.
    let listener = window_event_listener(leptos::ev::hashchange, move |_| {
        let current = planner.build.with_untracked(browser::fragment_for);
        let hash = window().location().hash().unwrap_or_default();
        if hash.trim_start_matches('#') != current.trim_start_matches('#') {
            let (build, error) = Planner::open_fragment(&planner.engine());
            planner.replace(build, error);
        }
    });
    on_cleanup(move || listener.remove());

    // Escape closes the picker.
    let keys = window_event_listener(leptos::ev::keydown, move |event| {
        if event.key() == "Escape" {
            planner.close();
        }
    });
    on_cleanup(move || keys.remove());

    // Each part is its own type-erased view: the whole sheet as one view type is too deep for the compiler.
    view! {
        <div class="app">
            <header class="top">
                <h1>"WotR Planner"</h1>
                <BuildName/>
                <BuildSheetActions/>
            </header>
            <ErrorBanner/>
            <BuildStartPanel/>
            <Tabs/>
            <Picker/>
        </div>
    }
}

/// The one generic error a link that opens nothing shows, until dismissed or the next edit.
#[component]
fn ErrorBanner() -> AnyView {
    let planner = use_planner();
    (move || {
        planner.error.get().map(|error| {
            view! {
                <div class="banner banner-error" role="alert">
                    <span>{error}</span>
                    <button
                        class="icon"
                        aria-label="Dismiss"
                        on:click=move |_| planner.error.set(None)
                    >
                        "×"
                    </button>
                </div>
            }
        })
    })
    .into_any()
}

/// The Levels, Mythic track and Gear tabs, and the one shown.
#[component]
fn Tabs() -> AnyView {
    let tab = use_planner().tab;
    view! {
        <nav class="tabs" role="tablist">
            <button
                role="tab"
                class:on=move || tab.get() == Tab::Levels
                aria-selected=move || (tab.get() == Tab::Levels).to_string()
                on:click=move |_| tab.set(Tab::Levels)
            >
                "Levels"
            </button>
            <button
                role="tab"
                class:on=move || tab.get() == Tab::Mythic
                aria-selected=move || (tab.get() == Tab::Mythic).to_string()
                on:click=move |_| tab.set(Tab::Mythic)
            >
                "✦ Mythic track"
            </button>
            <button
                role="tab"
                class:on=move || tab.get() == Tab::Gear
                aria-selected=move || (tab.get() == Tab::Gear).to_string()
                on:click=move |_| tab.set(Tab::Gear)
            >
                "Gear"
            </button>
        </nav>
        {move || match tab.get() {
            Tab::Levels => view! { <LevelsGrid/> }.into_any(),
            Tab::Mythic => view! { <MythicTrack/> }.into_any(),
            Tab::Gear => view! { <GearTab/> }.into_any(),
        }}
    }
    .into_any()
}

/// The Build's optional name, carried in the link.
#[component]
fn BuildName() -> AnyView {
    let planner = use_planner();
    view! {
        <input
            class="build-name"
            type="text"
            placeholder="Unnamed build"
            aria-label="Build name"
            maxlength="60"
            prop:value=move || planner.build.with(|build| build.name.clone().unwrap_or_default())
            on:change=move |event| {
                let name = event_target_value(&event).trim().to_owned();
                planner.edit(Edit::SetName((!name.is_empty()).then_some(name)));
            }
        />
    }
    .into_any()
}
