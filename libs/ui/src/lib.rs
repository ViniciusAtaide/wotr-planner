use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};

mod browser;
mod game_data;
mod grid;
mod mythic;
mod picker;
mod planner;
mod start;
mod timings;

pub use game_data::GameDataUrl;

/// The HTML document the server renders around [`App`].
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="color-scheme" content="light dark"/>
                // No icon: without this link a browser asks for `/favicon.ico`, at the host's root (outside a
                // GitHub project site), and gets a 404.
                <link rel="icon" href="data:,"/>
                <AutoReload options=options.clone()/>
                // `root="."`: the scripts and wasm load from `./pkg/…`, relative to the page.
                <HydrationScripts options root="."/>
                <game_data::GameDataPreload/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

/// The app: one page, the sheet, whatever the path. There's no router: the Build lives in the URL fragment, and the
/// page must also work under a path prefix (a GitHub project site is at `/<repo>/`), where a route for `/` would
/// match nothing. The server serves it at `/` only.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="./pkg/wotr-planner.css"/>
        <Title text="WotR Planner"/>
        <planner::SheetPage/>
    }
}
