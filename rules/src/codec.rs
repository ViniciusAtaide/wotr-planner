//! The URL codec: the Build in the link fragment, `/#b=<code>`.
//!
//! `code` is base64url (no padding) over one format-version byte followed by the version's body. Version 1's body
//! is the [`Build`] in postcard (serde's compact binary: varints, length-prefixed strings, enum variants by index),
//! compressed with raw DEFLATE. The version byte stays outside the compression so a later version can change
//! anything after it.
//!
//! Every key is a text key, so a link names things the game data may no longer have; decoding never looks at the
//! game data, and [`Engine::validate`](crate::Engine::validate) reports such keys.
//!
//! Changing [`Build`]'s serde shape (field order, enum variant order, types) changes version 1's format. Don't:
//! add a version 2 with its own frozen types and keep decoding version 1. The frozen v1 test vector guards this.

use std::fmt;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

use crate::build::Build;

/// The format version new links are written in.
pub const CODEC_VERSION: u8 = 1;

/// The most a link may inflate to. A 40-level Build is a few kilobytes at most.
const MAX_INFLATED: usize = 256 * 1024;

/// Why a link couldn't be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    Empty,
    /// Not base64url.
    Base64,
    /// A format version this planner doesn't know (made by a newer one?).
    UnsupportedVersion(u8),
    /// The body isn't valid DEFLATE data, or inflates past the size limit.
    Decompress,
    /// The inflated body isn't a Build of this version.
    Format,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::Empty => f.write_str("the link has no Build in it"),
            DecodeError::Base64 => f.write_str("the link isn't valid base64url"),
            DecodeError::UnsupportedVersion(version) => {
                write!(
                    f,
                    "the link uses Build format version {version}, which this planner doesn't know"
                )
            }
            DecodeError::Decompress => f.write_str("the link's Build data is damaged"),
            DecodeError::Format => f.write_str("the link's Build data doesn't describe a Build"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// The link code for a Build, in the current format version.
pub fn encode_build(build: &Build) -> String {
    let body = postcard::to_allocvec(build).expect("a Build always serialises");
    let mut bytes = vec![CODEC_VERSION];
    bytes.extend(miniz_oxide::deflate::compress_to_vec(&body, 9));
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Reads a link code made by any format version so far.
pub fn decode_build(code: &str) -> Result<Build, DecodeError> {
    if code.is_empty() {
        return Err(DecodeError::Empty);
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(code)
        .map_err(|_| DecodeError::Base64)?;
    let (&version, body) = bytes.split_first().ok_or(DecodeError::Empty)?;
    match version {
        1 => decode_v1(body),
        other => Err(DecodeError::UnsupportedVersion(other)),
    }
}

fn decode_v1(body: &[u8]) -> Result<Build, DecodeError> {
    let inflated = miniz_oxide::inflate::decompress_to_vec_with_limit(body, MAX_INFLATED)
        .map_err(|_| DecodeError::Decompress)?;
    match postcard::take_from_bytes::<Build>(&inflated) {
        Ok((build, [])) => Ok(build),
        _ => Err(DecodeError::Format),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::{ClassChoice, LevelUp, Pick, SlotId};
    use crate::keys::{ClassKey, FeatKey};

    /// Version 1's binary layout, before compression, for a small Build. The compressed bytes may differ between
    /// DEFLATE implementations; this may not.
    #[test]
    fn version_1_body_layout_is_frozen() {
        let mut build = Build {
            name: Some("A".into()),
            ..Build::default()
        };
        build.levels.push(LevelUp::new(ClassKey::new("x")));
        build
            .class_choices
            .insert(ClassKey::new("x"), ClassChoice::default());
        build
            .picks
            .insert(SlotId::Mythic { rank: 1 }, Pick::feat(FeatKey::new("y")));
        let body = postcard::to_allocvec(&build).unwrap();
        assert_eq!(
            body,
            [
                1, 1, b'A', // name: Some("A")
                0, 0, 0, 0,
                0, // start: race, subrace, racial ability, alignment, background all None
                10, 10, 10, 10, 10, 10, // point-buy scores
                1,  // one level
                1, b'x', // class
                0, 0, // no ability increase, no skill ranks
                1, // one class choice
                1, b'x', // for the class
                0, 0, // archetype None, no advances
                0, 0, // mythic path, path switch: None
                1, // one pick
                2, 1, // SlotId::Mythic { rank: 1 }
                0, 1, b'y', 0, // Pick::Feat { feat: "y", parameter: None }
            ]
        );
    }
}
