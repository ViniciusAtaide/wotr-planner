//! BubblePrints' `.binz` container, as the pinned dump uses it.
//!
//! A run of chunks, each a 16-byte little-endian header and a payload:
//!
//! | offset | type | field |
//! |---|---|---|
//! | 0 | u16 | chunk id |
//! | 2 | u16 | sub id |
//! | 4 | u32 | the payload's length once decoded |
//! | 8 | u32 | its length as stored, an LZ4 block; 0 when stored as is |
//! | 12 | u32 | unused here (0 in the pinned dump) |
//!
//! The blueprints are in the chunks with id 33 and sub id 0, each a run of whole records. A record is four
//! length-prefixed UTF-8 strings (a 7-bit varint length, as .NET's `BinaryWriter` writes them): the GUID, the asset
//! name, the full type name and the blueprint's JSON. The other chunks (the localized strings, reference and type
//! tables) are skipped: the check names blueprints by asset name, never by their display text.

use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::{Dump, Error, Pins};

/// The blueprints chunk: id 33, sub id 0.
const BLUEPRINTS: (u16, u16) = (33, 0);
const HEADER: usize = 16;

/// Reads the dump at `path` once it is the pinned one: its size, then its SHA-256, before anything else.
pub fn read(path: &Path, pins: &Pins) -> Result<Dump, Error> {
    let io = |error| Error::Io {
        path: path.display().to_string(),
        error,
    };
    let size = fs::metadata(path).map_err(io)?.len();
    if size != pins.bytes {
        return Err(Error::Refused {
            what: "size",
            expected: pins.bytes.to_string(),
            found: size.to_string(),
        });
    }
    let bytes = fs::read(path).map_err(io)?;
    let found = sha256(&bytes);
    if found != pins.sha256 {
        return Err(Error::Refused {
            what: "SHA-256",
            expected: pins.sha256.clone(),
            found,
        });
    }
    parse(&bytes)
}

/// The SHA-256 of `bytes`, as 64 lowercase hex digits.
pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The blueprints of a `.binz` file's bytes, unchecked: [`read`] checks them against the pins first.
pub fn parse(bytes: &[u8]) -> Result<Dump, Error> {
    let mut dump = Dump::default();
    let mut chunks = 0;
    let mut pos = 0;
    let mut buffer = Vec::new();
    while pos < bytes.len() {
        let header = bytes
            .get(pos..pos + HEADER)
            .ok_or_else(|| format_error(format!("a chunk header cut short at byte {pos}")))?;
        let id = u16::from_le_bytes([header[0], header[1]]);
        let sub = u16::from_le_bytes([header[2], header[3]]);
        let length = u32_at(header, 4);
        let stored = u32_at(header, 8);
        let payload_length = if stored > 0 { stored } else { length };
        let start = pos + HEADER;
        let payload = bytes.get(start..start + payload_length).ok_or_else(|| {
            format_error(format!("chunk {id}/{sub} at byte {pos} runs past the end"))
        })?;
        pos = start + payload_length;
        if (id, sub) != BLUEPRINTS {
            continue;
        }
        chunks += 1;
        let records = if stored > 0 {
            buffer.resize(length, 0);
            let decoded =
                lz4_flex::block::decompress_into(payload, &mut buffer).map_err(|error| {
                    format_error(format!("chunk at byte {}: {error}", start - HEADER))
                })?;
            if decoded != length {
                return Err(format_error(format!(
                    "chunk at byte {} decodes to {decoded} bytes, not {length}",
                    start - HEADER
                )));
            }
            &buffer[..]
        } else {
            payload
        };
        read_records(records, &mut dump)?;
    }
    if chunks == 0 {
        return Err(format_error("no blueprints chunk".to_owned()));
    }
    Ok(dump)
}

fn u32_at(header: &[u8], at: usize) -> usize {
    u32::from_le_bytes([header[at], header[at + 1], header[at + 2], header[at + 3]]) as usize
}

fn format_error(message: String) -> Error {
    Error::Format(message)
}

fn read_records(mut bytes: &[u8], dump: &mut Dump) -> Result<(), Error> {
    while !bytes.is_empty() {
        let guid = text(read_bytes(&mut bytes)?)?;
        let name = text(read_bytes(&mut bytes)?)?;
        let type_name = text(read_bytes(&mut bytes)?)?;
        // The JSON stays bytes: the dump reads it as text only for the kinds it keeps, a small part of the 542 MB.
        let raw = read_bytes(&mut bytes)?;
        dump.insert(guid, name, type_name, raw)?;
    }
    Ok(())
}

pub(crate) fn text(bytes: &[u8]) -> Result<&str, Error> {
    std::str::from_utf8(bytes).map_err(|_| format_error("a string that is not UTF-8".to_owned()))
}

/// A 7-bit varint length, then that many bytes.
fn read_bytes<'a>(bytes: &mut &'a [u8]) -> Result<&'a [u8], Error> {
    let mut length = 0usize;
    let mut shift = 0;
    loop {
        let (&byte, rest) = bytes
            .split_first()
            .ok_or_else(|| format_error("a record cut short".to_owned()))?;
        *bytes = rest;
        length |= usize::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift > 28 {
            return Err(format_error("a string length over five bytes".to_owned()));
        }
    }
    if length > bytes.len() {
        return Err(format_error("a record cut short".to_owned()));
    }
    let (text, rest) = bytes.split_at(length);
    *bytes = rest;
    Ok(text)
}
