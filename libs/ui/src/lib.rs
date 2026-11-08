use leptos::prelude::*;
use leptos::tachys::html::element::meta;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};

mod browser;
mod game_data;
mod grid;
mod icons;
mod mythic;
mod picker;
mod planner;
mod start;
mod timings;

pub use game_data::GameDataUrl;

/// What a link preview (WhatsApp, Slack, Discord, X…) and a search result say about the site.
const DESCRIPTION: &str = "Plan a Pathfinder: Wrath of the Righteous character level by level: classes, feats, \
                           skills and mythic paths, checked against the game's rules. Share any build as a link.";

/// An Open Graph tag, `<meta property=… content=…>` (`property` isn't an attribute the view macro knows).
fn og(property: &'static str, content: impl Into<String>) -> impl IntoView {
    meta().attr("property", property).content(content.into())
}

/// The HTML document the server renders around [`App`].
///
/// `site_url` is the site's public address, ending in `/` (for example `https://<user>.github.io/<repo>/`). Link
/// previews need the preview image by an absolute URL, so without it the page names no image (a dev server).
pub fn shell(options: LeptosOptions, site_url: Option<String>) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="color-scheme" content="light dark"/>
                <meta name="description" content=DESCRIPTION/>
                {og("og:type", "website")}
                {og("og:site_name", "WotR Planner")}
                {og("og:title", "WotR Planner")}
                {og("og:description", DESCRIPTION)}
                <meta name="twitter:card" content="summary_large_image"/>
                {site_url.map(|url| {
                    // `apps/frontend/public/social-preview.png`, made from `apps/frontend/social/preview.html`.
                    let image = format!("{url}social-preview.png");
                    view! {
                        {og("og:url", url)}
                        {og("og:image", image.clone())}
                        {og("og:image:type", "image/png")}
                        {og("og:image:width", "1200")}
                        {og("og:image:height", "630")}
                        {og(
                            "og:image:alt",
                            "WotR Planner: a character sheet with ability scores, levels, feats and a mythic rank",
                        )}
                        <meta name="twitter:image" content=image/>
                    }
                })}
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
