//! The game data: the server serves it as one cacheable JSON document at a URL that names its content hash, and the
//! page's head preloads that URL, so the download starts with the wasm's rather than after hydration. The sheet
//! then reads the same response, from the preload or the browser's cache.

use leptos::prelude::*;
use leptos::wasm_bindgen::{JsCast, JsValue};
use rules::GameData;
use wasm_bindgen_futures::JsFuture;

/// The game data's URL, relative to the page (`./game-data/<hash>.json`), so it works under any path prefix. The
/// server provides it as context to the shell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameDataUrl(pub String);

/// The id of the head's preload link, which tells the sheet where the game data is.
const LINK_ID: &str = "game-data";

/// The head's preload link for the game data; nothing when the server gave no URL (an error page).
#[component]
pub fn GameDataPreload() -> impl IntoView {
    use_context::<GameDataUrl>().map(|GameDataUrl(url)| {
        // `crossorigin` makes it a CORS request, as the sheet's `fetch` is, so the fetch takes the preload's response.
        view! { <link id=LINK_ID rel="preload" href=url r#as="fetch" crossorigin="anonymous"/> }
    })
}

/// Downloads the game data from the URL the head's preload link names. Browser only.
pub async fn load() -> Result<GameData, String> {
    let url = document()
        .get_element_by_id(LINK_ID)
        .and_then(|link| link.get_attribute("href"))
        .ok_or("the page names no game data")?;
    let response = JsFuture::from(window().fetch_with_str(&url))
        .await
        .map_err(js_error)?
        .dyn_into::<web_sys::Response>()
        .map_err(js_error)?;
    if !response.ok() {
        return Err(format!("{} {}", response.status(), response.status_text()));
    }
    let text = JsFuture::from(response.text().map_err(js_error)?)
        .await
        .map_err(js_error)?
        .as_string()
        .ok_or("the game data isn't text")?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn js_error(error: impl Into<JsValue>) -> String {
    let error = error.into();
    error
        .dyn_ref::<js_sys::Error>()
        .map(|error| String::from(error.message()))
        .or_else(|| error.as_string())
        .unwrap_or_else(|| format!("{error:?}"))
}
