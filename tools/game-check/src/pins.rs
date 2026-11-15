//! The pins of the game's dump (ticket 236): `scripts/game-sources.pins`, plain `KEY=value` lines with `#` comments,
//! which `scripts/game-sources.sh` sources too. The check reads the dump's three pins from it, embedded at build time,
//! so a moved pin rebuilds the check.

use std::collections::BTreeMap;

use crate::Error;

/// The pinned dump: its asset name, its size and its SHA-256.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pins {
    /// The release asset's file name, which is also its name in `blueprints/`.
    pub asset: String,
    pub bytes: u64,
    /// 64 lowercase hex digits.
    pub sha256: String,
}

impl Pins {
    /// The repo's own pins, `scripts/game-sources.pins`.
    pub fn embedded() -> Result<Pins, Error> {
        Pins::parse(include_str!("../../../scripts/game-sources.pins"))
    }

    /// Reads the dump's pins from a pins file's text: `BLUEPRINTS_ASSET`, `BLUEPRINTS_BYTES`, `BLUEPRINTS_SHA256`.
    pub fn parse(text: &str) -> Result<Pins, Error> {
        let mut values = BTreeMap::new();
        for (number, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| Error::Pins(format!("line {} is not KEY=value", number + 1)))?;
            values.insert(key.trim(), value.trim());
        }
        let get = |key: &str| {
            values
                .get(key)
                .map(|value| (*value).to_owned())
                .ok_or_else(|| Error::Pins(format!("no {key}")))
        };
        let bytes = get("BLUEPRINTS_BYTES")?;
        let pins = Pins {
            asset: get("BLUEPRINTS_ASSET")?,
            bytes: bytes
                .parse()
                .map_err(|_| Error::Pins(format!("BLUEPRINTS_BYTES is not a number: {bytes}")))?,
            sha256: get("BLUEPRINTS_SHA256")?,
        };
        if pins.sha256.len() != 64
            || !pins
                .sha256
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(Error::Pins(format!(
                "BLUEPRINTS_SHA256 is not 64 lowercase hex digits: {}",
                pins.sha256
            )));
        }
        Ok(pins)
    }
}
