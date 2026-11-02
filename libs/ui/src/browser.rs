//! The browser API the sheet needs: the URL fragment that holds the Build.
//!
//! Everything here runs only in the browser: the sheet renders only once the game data has loaded there.

use leptos::prelude::window;
use leptos::wasm_bindgen::JsValue;
use rules::{Build, encode_build};

/// What the fragment holds: `#b=<code>`.
const FRAGMENT_PREFIX: &str = "#b=";

/// What the URL fragment says to open.
pub enum Fragment {
    /// No fragment: a blank Build.
    Blank,
    /// A link's code, still to be opened.
    Code(String),
    /// A fragment that isn't a Build's: it opens nothing, like a stale link.
    Other,
}

/// Reads the URL fragment.
pub fn fragment() -> Fragment {
    let hash = window().location().hash().unwrap_or_default();
    if hash.is_empty() || hash == "#" {
        Fragment::Blank
    } else if let Some(code) = hash.strip_prefix(FRAGMENT_PREFIX) {
        Fragment::Code(code.to_owned())
    } else {
        Fragment::Other
    }
}

/// The fragment for a Build: none for a blank one.
pub fn fragment_for(build: &Build) -> String {
    if *build == Build::default() {
        String::new()
    } else {
        format!("{FRAGMENT_PREFIX}{}", encode_build(build))
    }
}

/// Rewrites the URL to hold the Build, without a new history entry or a `hashchange` event.
pub fn write_fragment(build: &Build) {
    let window = window();
    let path = window.location().pathname().unwrap_or_else(|_| "/".into());
    let url = format!("{path}{}", fragment_for(build));
    if let Ok(history) = window.history() {
        // A failure leaves the old URL; the Build on the sheet is unaffected.
        let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&url));
    }
}
