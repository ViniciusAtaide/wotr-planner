//! The fixture: its blueprints written as a `.binz` (LZ4 chunks, as the dump's), its pins, its game data, and a run.

use std::path::Path;

use game_check::{Dump, Known, Pins, Report, binz, check};
use rules::GameData;
use serde_json::Value;

/// One record of the fixture's dump.
#[derive(Clone)]
pub struct Record {
    pub guid: String,
    pub name: String,
    pub type_name: String,
    pub raw: String,
}

/// The fixture's blueprints, in the file's order.
pub fn records() -> Vec<Record> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixture/blueprints.json");
    let text = std::fs::read_to_string(path).expect("the fixture's blueprints");
    let records: Vec<Value> =
        serde_json::from_str(&text).expect("the fixture's blueprints are JSON");
    records
        .into_iter()
        .map(|record| Record {
            guid: record["guid"].as_str().unwrap().to_owned(),
            name: record["name"].as_str().unwrap().to_owned(),
            type_name: record["type"].as_str().unwrap().to_owned(),
            raw: record["raw"].to_string(),
        })
        .collect()
}

/// The fixture's game data.
pub fn planner() -> GameData {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixture/planner.ron");
    let text = std::fs::read_to_string(path).expect("the fixture's game data");
    ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
        .from_str(&text)
        .expect("the fixture's game data parses")
}

/// A `.binz` of the records: a header chunk, then the records in `chunks` LZ4-compressed blueprint chunks, then a
/// stored chunk of another id.
pub fn binz(records: &[Record], chunks: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    chunk(&mut bytes, (32, 0), &[2, 0, 0, 0], false);
    let per_chunk = records.len().div_ceil(chunks.max(1)).max(1);
    for part in records.chunks(per_chunk) {
        let mut payload = Vec::new();
        for record in part {
            for text in [&record.guid, &record.name, &record.type_name, &record.raw] {
                string(&mut payload, text);
            }
        }
        chunk(&mut bytes, (33, 0), &payload, true);
    }
    chunk(&mut bytes, (34, 0), b"{\"strings\":{}}", false);
    bytes
}

fn chunk(bytes: &mut Vec<u8>, (id, sub): (u16, u16), payload: &[u8], compress: bool) {
    let stored = if compress {
        lz4_flex::block::compress(payload)
    } else {
        payload.to_vec()
    };
    bytes.extend(id.to_le_bytes());
    bytes.extend(sub.to_le_bytes());
    bytes.extend((payload.len() as u32).to_le_bytes());
    bytes.extend((if compress { stored.len() as u32 } else { 0 }).to_le_bytes());
    bytes.extend(0u32.to_le_bytes());
    bytes.extend(stored);
}

/// A 7-bit varint length, then the UTF-8.
fn string(bytes: &mut Vec<u8>, text: &str) {
    let mut length = text.len();
    loop {
        let byte = (length & 0x7f) as u8;
        length >>= 7;
        if length == 0 {
            bytes.push(byte);
            break;
        }
        bytes.push(byte | 0x80);
    }
    bytes.extend(text.as_bytes());
}

/// Pins for these bytes, as `scripts/game-sources.pins` writes them.
pub fn pins_for(bytes: &[u8]) -> Pins {
    Pins::parse(&format!(
        "# the fixture's\nBLUEPRINTS_ASSET=fixture.binz\nBLUEPRINTS_BYTES={}\nBLUEPRINTS_SHA256={}\n",
        bytes.len(),
        binz::sha256(bytes)
    ))
    .expect("the fixture's pins parse")
}

/// The fixture's dump read back through the file and its pins, as the check reads the real one.
pub fn dump(bytes: &[u8]) -> (Pins, Dump) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fixture.binz");
    std::fs::write(&path, bytes).unwrap();
    let pins = pins_for(bytes);
    let dump = binz::read(&path, &pins).expect("the fixture's dump reads");
    (pins, dump)
}

/// The check over the fixture, with the given known differences.
pub fn report_with(known: &str) -> Report {
    let (pins, dump) = dump(&binz(&records(), 2));
    check(&pins, &dump, &planner(), &Known::parse(known).unwrap())
        .expect("the fixture's JSON parses")
}

pub fn report() -> Report {
    report_with("")
}

/// Asserts the report has a finding of this group with these values, and says where it looked.
#[track_caller]
pub fn assert_finding(report: &Report, group: &str, entry: &str, game: &str, planner: &str) {
    let text = report.text();
    let wanted = format!("game: {game} | planner: {planner}");
    let mut in_group = false;
    for line in text.lines() {
        if !line.starts_with("  ") {
            in_group =
                line.starts_with(&format!("{group}:")) || line.starts_with(&format!("{group} ["));
            continue;
        }
        if in_group && line.starts_with(&format!("  {entry} |")) && line.ends_with(&wanted) {
            return;
        }
    }
    panic!("no `{group}` finding for {entry} with `{wanted}` in:\n{text}");
}

/// Asserts no group of `area` names `entry`.
#[track_caller]
pub fn assert_agrees(report: &Report, area: &str, entry: &str) {
    let text = report.text();
    let mut in_area = false;
    for line in text.lines() {
        if !line.starts_with("  ") {
            in_area = line.starts_with(&format!("{area} / "));
            continue;
        }
        assert!(
            !(in_area && line.starts_with(&format!("  {entry} |"))),
            "{entry} differs in {area}: {line}\n{text}"
        );
    }
}

/// A copy of the write fixture's data folder (`tests/fixture/data/`), for a test to write to.
pub fn data_copy() -> tempfile::TempDir {
    fn copy(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), target).unwrap();
            }
        }
    }
    let dir = tempfile::tempdir().unwrap();
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixture/data"),
        dir.path(),
    );
    dir
}
