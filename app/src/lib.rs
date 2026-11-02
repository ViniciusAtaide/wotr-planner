use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

mod browser;
mod grid;
mod mythic;
mod picker;
mod planner;
mod start;
mod timings;

/// The HTML document the server renders around [`App`].
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="color-scheme" content="light dark"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

/// All the seeded game data. The browser loads it once and runs the rules engine on it.
///
/// The server loads the game data once at startup and provides it as an `Arc<GameData>` context;
/// this returns that copy without touching the database. In the wasm build the body becomes an
/// HTTP call.
#[server]
pub async fn get_game_data() -> Result<rules::GameData, ServerFnError> {
    let game_data = use_context::<std::sync::Arc<rules::GameData>>()
        .ok_or_else(|| ServerFnError::new("the server has no game data loaded"))?;
    Ok(rules::GameData::clone(&game_data))
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/wotr-planner.css"/>
        <Title text="WotR Planner"/>
        <Router>
            <Routes fallback=|| "Not found.".into_view()>
                <Route path=path!("/") view=planner::SheetPage/>
            </Routes>
        </Router>
    }
}
