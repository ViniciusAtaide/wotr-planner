//! Timings of the engine calls, for the smoke checks and benchmarks: when the page defines `window.__wotrTimings`
//! as an array, each [`timed`] call appends `[label, milliseconds]` to it.
//!
//! Development only. A dev build records them; a release build gets a pass-through that the compiler inlines away,
//! unless it's built with the `timings` feature to benchmark it (`cargo leptos build --release --lib-features
//! timings`).

#[cfg(any(debug_assertions, feature = "timings"))]
mod recording {
    use leptos::prelude::window;
    use leptos::wasm_bindgen::{JsCast, JsValue};

    /// Milliseconds since the page started.
    fn now() -> f64 {
        window()
            .performance()
            .map_or(0.0, |performance| performance.now())
    }

    /// Runs `f`, and when the page asks for timings records how long it took under `label`.
    pub fn timed<T>(label: &str, f: impl FnOnce() -> T) -> T {
        let start = now();
        let value = f();
        let elapsed = now() - start;
        let log = js_sys::Reflect::get(&window(), &JsValue::from_str("__wotrTimings"));
        if let Ok(log) = log
            && js_sys::Array::is_array(&log)
        {
            let entry = js_sys::Array::of2(&JsValue::from_str(label), &JsValue::from_f64(elapsed));
            log.unchecked_ref::<js_sys::Array>().push(&entry);
        }
        value
    }
}

#[cfg(any(debug_assertions, feature = "timings"))]
pub use recording::timed;

/// Runs `f`. Release builds record nothing.
#[cfg(not(any(debug_assertions, feature = "timings")))]
#[inline(always)]
pub fn timed<T>(_label: &str, f: impl FnOnce() -> T) -> T {
    f()
}
