//! The Sheet: the one planning screen. It loads the game data once, runs the rules engine in the browser, and
//! keeps the Build in the URL fragment.

use std::collections::BTreeSet;
use std::sync::Arc;

use leptos::prelude::*;
use rules::{Build, Choice, Edit, Engine, Sheet, SlotId, StaleLink, WeaponKey};

use crate::browser::{self, Fragment};
use crate::export::BuildSheetActions;
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
    Race,
    Alignment,
    Deity,
    Background,
    AbilityIncrease(u8),
    Skills(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Levels,
    Mythic,
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
    pub picker: RwSignal<Option<Target>>,
    pub tab: RwSignal<Tab>,
    /// The weapon the summary's attack line is for: `None` until one is chosen (the engine's default, the Build's
    /// first Weapon Focus), then the choice (`Some(None)`: no weapon). Not part of the Build or its link.
    pub attack_weapon: RwSignal<Option<Option<WeaponKey>>>,
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
        Self {
            engine,
            build,
            sheet,
            removable_classes,
            picker: RwSignal::new(None),
            tab: RwSignal::new(Tab::Levels),
            attack_weapon: RwSignal::new(None),
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

    pub fn open(&self, target: Target) {
        self.picker.set(Some(target));
    }

    pub fn close(&self) {
        self.picker.set(None);
    }

    /// Replaces the whole Build (a link pasted into the address bar): what belonged to the old one goes with it.
    fn replace(&self, build: Build, error: Option<String>) {
        self.picker.set(None);
        self.attack_weapon.set(None);
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
            </header>
            <ErrorBanner/>
            <BuildSheetActions/>
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

/// The Levels and Mythic track tabs, and the one shown.
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
        </nav>
        {move || match tab.get() {
            Tab::Levels => view! { <LevelsGrid/> }.into_any(),
            Tab::Mythic => view! { <MythicTrack/> }.into_any(),
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
