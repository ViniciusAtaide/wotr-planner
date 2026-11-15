//! The repo's own pins and known list parse.

use game_check::{Known, Pins};

#[test]
fn the_check_reads_the_dumps_pins_from_scripts_game_sources_pins() {
    let pins = Pins::embedded().unwrap();
    assert_eq!(pins.asset, "blueprints_raw_2.7.0x_0.binz");
    assert_eq!(pins.bytes, 136_905_792);
    assert_eq!(
        pins.sha256,
        "03ba6e29782c0c14e0c0383cd739560358641f7b5842f1dcfb690246e73c9c6a"
    );
}

#[test]
fn the_known_list_parses() {
    Known::embedded().unwrap();
}
