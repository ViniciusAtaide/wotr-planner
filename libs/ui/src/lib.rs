use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

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
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <game_data::GameDataPreload/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
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
