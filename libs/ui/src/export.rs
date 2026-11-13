//! The build sheet's actions (ticket 87): copy the Build as a readable build sheet ([`rules::Engine::build_sheet`])
//! to the clipboard, or save it as a `.txt` file, each confirmed by a short message. They sit in a small menu at the
//! end of the header, so they take no row of their own; its items are plain buttons, so they work by touch on both
//! Fold screens.

use std::time::Duration;

use leptos::prelude::*;

use crate::browser;
use crate::planner::use_planner;

/// How long a confirmation stays.
const CONFIRMATION: Duration = Duration::from_secs(3);

#[component]
pub fn BuildSheetActions() -> AnyView {
    let planner = use_planner();
    let open = RwSignal::new(false);
    let status = RwSignal::new(None::<&'static str>);
    // Each message clears itself unless a later one replaced it.
    let confirm = move |message: &'static str| {
        // The copy's answer may come after the sheet is gone.
        if status.is_disposed() {
            return;
        }
        status.set(Some(message));
        set_timeout(
            move || {
                if !status.is_disposed() && status.get_untracked() == Some(message) {
                    status.set(None);
                }
            },
            CONFIRMATION,
        );
    };
    let text = move || {
        let page = browser::page_url();
        planner.ask("build_sheet", |engine, build| {
            engine.build_sheet(build, &page)
        })
    };
    let copy = move |_| {
        open.set(false);
        let text = text();
        leptos::task::spawn_local(async move {
            if browser::copy_text(&text).await {
                confirm("Build sheet copied");
            } else {
                confirm("Couldn't copy here: try Download .txt");
            }
        });
    };
    let download = move |_| {
        open.set(false);
        let name = planner
            .build
            .with_untracked(|build| file_name(build.name.as_deref()));
        if browser::download_text(&name, &text()) {
            confirm("Build sheet saved");
        } else {
            confirm("Couldn't save the file");
        }
    };
    view! {
        <div
            class="export"
            on:keydown=move |event| {
                if event.key() == "Escape" {
                    open.set(false);
                }
            }
        >
            <button
                class="icon export-toggle"
                aria-label="Export build sheet"
                title="Export build sheet"
                aria-haspopup="menu"
                aria-expanded=move || open.get().to_string()
                on:click=move |_| open.update(|open| *open = !*open)
            >
                "⋯"
            </button>
            {move || {
                open.get()
                    .then(|| {
                        view! {
                            // A click anywhere outside the menu closes it.
                            <div class="export-backdrop" on:click=move |_| open.set(false)></div>
                            <div class="export-menu" role="menu">
                                <button class="export-copy" role="menuitem" on:click=copy>
                                    "Copy build sheet"
                                </button>
                                <button class="export-download" role="menuitem" on:click=download>
                                    "Download .txt"
                                </button>
                            </div>
                        }
                    })
            }}
            <span class="export-status" role="status" aria-live="polite">
                {move || status.get()}
            </span>
        </div>
    }
    .into_any()
}

/// The saved file's name: the Build's name in lower case with each run of other characters than letters and digits
/// as one `-` ("Spear of the Balance" gives `spear-of-the-balance.txt`), or `wotr-build.txt`.
fn file_name(name: Option<&str>) -> String {
    let words: Vec<String> = name
        .unwrap_or_default()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_lowercase)
        .collect();
    if words.is_empty() {
        "wotr-build.txt".to_owned()
    } else {
        format!("{}.txt", words.join("-"))
    }
}
