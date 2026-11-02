//! Text keys that identify game entities, such as `power-attack` (a feat) or `rogue/dark-lurker` (an archetype).
//!
//! Keys come from the guide's anchors and stay the same across re-seeds, so saved Builds can refer to them.
//! In the data files and in JSON each key is a plain string.
//!
//! A key carries no type prefix: its Rust type (and its table) says what kind of thing it names, so keys of two
//! kinds may be equal (a feat and a feature both `power-attack`). Keys of things that belong to a class or path
//! start with their owner (`rogue/eldritch-scoundrel`), which keeps names unique where the guide reuses them.

use std::fmt;

use serde::{Deserialize, Serialize};

macro_rules! text_key {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn new(key: impl Into<String>) -> Self {
                Self(key.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

text_key!(
    /// `elf`
    RaceKey
);
text_key!(
    /// `elf/blightborn`
    SubraceKey
);
text_key!(
    /// `mobility`
    SkillKey
);
text_key!(
    /// `longsword`: a weapon a parameterised feat is taken for.
    WeaponKey
);
text_key!(
    /// `power-attack`
    FeatKey
);
text_key!(
    /// `rogue` for a base class, `arcane-trickster` for a prestige class.
    ClassKey
);
text_key!(
    /// `rogue/dark-lurker`
    ArchetypeKey
);
text_key!(
    /// `rogue/sneak-attack`, `trickster/trickster-tricks`
    FeatureKey
);
text_key!(
    /// `rogue/rogue-talents`: the list a Feature selection picks from.
    FeatureSelectionKey
);
text_key!(
    /// `rogue/rogue-talents/fast-stealth`: one option of a Feature selection. It extends its selection's key.
    SelectionOptionKey
);
text_key!(
    /// `monk/bonus-feats-6`: a named list of feats that a fixed-list Feat slot picks from.
    FeatListKey
);
text_key!(
    /// `mythic-charge`
    MythicAbilityKey
);
text_key!(
    /// `power-attack-mythic`
    MythicFeatKey
);
text_key!(
    /// `trickster`
    MythicPathKey
);
text_key!(
    /// `shackles-corsair`
    BackgroundKey
);
