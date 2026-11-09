//! The browser API the sheet needs: the URL fragment that holds the Build, and the clipboard and a file download
//! for the build sheet.
//!
//! Everything here runs only in the browser: the sheet renders only once the game data has loaded there.

use std::time::Duration;

use leptos::prelude::{document, set_timeout, window};
use leptos::wasm_bindgen::{JsCast, JsValue};
use rules::{Build, encode_build};
use wasm_bindgen_futures::JsFuture;

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

/// The page's address before the fragment (`https://…/wotr-planner/`), which a Build's link adds its fragment to.
pub fn page_url() -> String {
    let location = window().location();
    let origin = location.origin().unwrap_or_default();
    let path = location.pathname().unwrap_or_else(|_| "/".into());
    format!("{origin}{path}")
}

/// Copies text to the clipboard. `false` when the browser has no clipboard here (a page not served over HTTPS or
/// from localhost) or refuses.
pub async fn copy_text(text: &str) -> bool {
    let navigator = window().navigator();
    // `navigator.clipboard` is undefined outside a secure context: calling it then would throw.
    let has_clipboard = js_sys::Reflect::get(&navigator, &JsValue::from_str("clipboard"))
        .is_ok_and(|clipboard| !clipboard.is_undefined());
    if !has_clipboard {
        return false;
    }
    JsFuture::from(navigator.clipboard().write_text(text))
        .await
        .is_ok()
}

/// Saves text as a plain-text file named `file_name`, through a link to it that is clicked and dropped. `false` when
/// the browser refuses.
pub fn download_text(file_name: &str, text: &str) -> bool {
    let download = || -> Result<(), JsValue> {
        let options = web_sys::BlobPropertyBag::new();
        options.set_type("text/plain;charset=utf-8");
        let parts = js_sys::Array::of1(&JsValue::from_str(text));
        let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options)?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)?;
        let link: web_sys::HtmlAnchorElement = document().create_element("a")?.unchecked_into();
        link.set_href(&url);
        link.set_download(file_name);
        link.click();
        // Revoked later, not at once: some browsers read the file only after the click returns.
        set_timeout(
            move || {
                let _ = web_sys::Url::revoke_object_url(&url);
            },
            Duration::from_secs(10),
        );
        Ok(())
    };
    download().is_ok()
}
