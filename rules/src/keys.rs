//! Text keys that identify game entities, such as `feat:power-attack` or `archetype:rogue/dark-lurker`.
//!
//! Keys come from the guide's anchors and stay the same across re-seeds, so saved Builds can refer to them.
//! In the data files and in JSON each key is a plain string.

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
    /// `race:elf`
    RaceKey
);
text_key!(
    /// `subrace:elf/blightborn`
    SubraceKey
);
text_key!(
    /// `skill:mobility`
    SkillKey
);
text_key!(
    /// `feat:power-attack`
    FeatKey
);
text_key!(
    /// `class:rogue` for a base class, `class:arcane-trickster` for a prestige class.
    ClassKey
);
text_key!(
    /// `archetype:rogue/dark-lurker`
    ArchetypeKey
);
text_key!(
    /// `feature:rogue/sneak-attack`, `feature:trickster/trickster-tricks`
    FeatureKey
);
text_key!(
    /// `selection:rogue/rogue-talents`: the list a Feature selection picks from.
    FeatureSelectionKey
);
text_key!(
    /// `selection:rogue/rogue-talents/fast-stealth`: one option of a Feature selection.
    SelectionOptionKey
);
text_key!(
    /// `mythic-ability:mythic-charge`
    MythicAbilityKey
);
text_key!(
    /// `mythic-feat:power-attack-mythic`
    MythicFeatKey
);
text_key!(
    /// `path:trickster`
    MythicPathKey
);
