//! Ticket 151 took the Build start out of `Engine::totals` and `Engine::caster_line`. The harness compiles against
//! the crates on both sides of that change (a `rules` proof builds it against a ref's crates too), so it reads which
//! one it's built against from the rules crate beside it: `cfg(lines_take_start)` before ticket 151.

use std::path::Path;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(lines_take_start)");
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets the manifest dir");
    let totals = Path::new(&manifest).join("../../libs/rules/src/engine/totals.rs");
    println!("cargo::rerun-if-changed={}", totals.display());
    let text = std::fs::read_to_string(&totals).unwrap_or_default();
    if text.contains("start: &BuildStart") {
        println!("cargo::rustc-cfg=lines_take_start");
    }
}
