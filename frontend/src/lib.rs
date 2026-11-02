#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    // Readable panic messages in the browser console, for development; release builds leave it out.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
