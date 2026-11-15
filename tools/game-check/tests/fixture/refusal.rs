//! A dump that isn't the pinned one is refused before anything is read from it.

use game_check::{Error, Known, binz, check};

use crate::support::{binz as write_binz, dump, pins_for, planner, records};

fn read(bytes: &[u8], pinned: &[u8]) -> Result<game_check::Dump, Error> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fixture.binz");
    std::fs::write(&path, bytes).unwrap();
    binz::read(&path, &pins_for(pinned))
}

#[test]
fn a_dump_whose_hash_is_not_the_pin_is_refused_naming_both() {
    let pinned = write_binz(&records(), 2);
    let mut changed = pinned.clone();
    let last = changed.len() - 1;
    changed[last] ^= 1;
    let error = read(&changed, &pinned).unwrap_err();
    let Error::Refused {
        what,
        expected,
        found,
    } = &error
    else {
        panic!("not refused: {error}");
    };
    assert_eq!(*what, "SHA-256");
    assert_eq!(*expected, binz::sha256(&pinned));
    assert_eq!(*found, binz::sha256(&changed));
    let message = error.to_string();
    assert!(
        message.contains(expected.as_str()) && message.contains(found.as_str()),
        "{message}"
    );
}

#[test]
fn a_truncated_dump_is_refused_on_its_size() {
    let pinned = write_binz(&records(), 2);
    let error = read(&pinned[..pinned.len() - 10], &pinned).unwrap_err();
    assert!(
        matches!(error, Error::Refused { what: "size", .. }),
        "{error}"
    );
}

#[test]
fn the_pinned_dump_reads_every_record() {
    let pinned = write_binz(&records(), 2);
    let dump = read(&pinned, &pinned).unwrap();
    assert_eq!(dump.len(), records().len());
}

#[test]
fn a_blueprint_whose_json_does_not_parse_is_an_error_naming_it() {
    let mut records = records();
    let steady_hand = records
        .iter_mut()
        .find(|record| record.name == "SteadyHandFeat")
        .unwrap();
    steady_hand.raw = "{\"Components\": [".to_owned();
    let (pins, dump) = dump(&write_binz(&records, 2));
    let error = check(&pins, &dump, &planner(), &Known::default()).unwrap_err();
    let message = error.to_string();
    assert!(
        matches!(error, Error::Format(_))
            && message.contains("SteadyHandFeat 000000000000000000000000000000a1"),
        "{message}"
    );
}
