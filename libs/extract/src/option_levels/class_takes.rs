//! How each class and its archetypes take its family ([`Takes`]): the sorcerer's, bloodrager's, wizard's,
//! cavalier's, Magic Deceiver's, shaman's and ranger's.

use super::bloodrager::PRIMAL_CHOICES;

/// How a class and its archetypes take a family.
pub(crate) struct Takes {
    /// The class whose tables take it.
    pub class: &'static str,
    /// The class's level that grants the selection.
    pub(super) level: u8,
    /// The table cell that is the choice itself ("Bloodline"); it becomes the selection.
    pub(super) choices: &'static [&'static str],
    /// Whether the choice cells' features stay defined (off the rows): a line elsewhere resolves to them.
    pub(super) kept_choices: bool,
    /// The table cells the option's rows replace ("Bloodline Ability").
    pub(super) placeholders: &'static [&'static str],
    /// Whether an archetype that takes the family skips the option levels its table has no placeholder for.
    pub(super) skips_without_placeholder: bool,
    /// The tables whose bonus slots' lists are those merged lists: a slot naming one picks from the held option's
    /// list instead.
    pub(super) merged: Merged,
    /// The base class takes the family (the Magic Deceiver's ways are the archetype's alone).
    pub(super) base_takes: bool,
    pub(super) archetypes: &'static [(&'static str, Take)],
    /// A placeholder that is also a choice in an archetype's table (archetype, cell, the selection it grants), at
    /// these levels.
    pub(super) row_choices: &'static [(&'static str, &'static str, &'static str, &'static [u8])],
}

/// Which tables' bonus lists are a family's merged lists.
#[derive(Clone, Copy, Debug)]
pub(super) enum Merged {
    None,
    /// Every table of the class.
    Every,
    /// These tables.
    Tables(&'static [&'static str]),
}

/// How one archetype takes a family, where its table doesn't say by itself.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Take {
    /// Not at all.
    No,
    /// The family, coverage by placeholders (or every level, for a family that doesn't skip).
    Yes,
    /// Only these options (the guide's list), coverage by placeholders.
    Only(&'static [&'static str]),
    /// One option, fixed: its rows where the archetype's table lists the same features, the archetype's own
    /// elsewhere.
    Fixed(&'static str),
    /// Only the feat list of a second option (Crossblooded), from the archetype's own choice cell.
    SecondListOnly(&'static str),
}

pub(super) const SORCERER: Takes = Takes {
    class: "sorcerer",
    level: 1,
    choices: &[],
    kept_choices: false,
    placeholders: &["Bloodline Ability", "Bloodline Arcana"],
    skips_without_placeholder: true,
    merged: Merged::Every,
    base_takes: true,
    // "Note that Empyreal Sorcerers, Sage Sorcerers and Sylvan Sorcerers have fixed bloodlines" (character-creation):
    // "identical to a Sorcerer who has selected the inferior Celestial bloodline", "more or less the same as an
    // Arcane Sorcerer", "the very similar Fey Sorcerer" (sorcerer). Crossblooded: "Bloodline 2 ... Arcana, Bloodline
    // spells, Bloodline feats", of which a second pick keeps only the feats (ticket 29).
    archetypes: &[
        ("sorcerer/crossblooded", Take::SecondListOnly("Bloodline 2")),
        (
            "sorcerer/empyreal-sorcerer",
            Take::Fixed("sorcerer/bloodline/celestial"),
        ),
        ("sorcerer/geomancer", Take::Yes),
        ("sorcerer/nine-tailed-heir", Take::Yes),
        ("sorcerer/overwhelming-mage", Take::Yes),
        (
            "sorcerer/sage-sorcerer",
            Take::Fixed("sorcerer/bloodline/arcane"),
        ),
        ("sorcerer/seeker-sorcerer", Take::Yes),
        (
            "sorcerer/sylvan-sorcerer",
            Take::Fixed("sorcerer/bloodline/fey"),
        ),
    ],
    row_choices: &[],
};

pub(super) const BLOODRAGER: Takes = Takes {
    class: "bloodrager",
    level: 1,
    choices: &["Bloodline"],
    kept_choices: true,
    // A Primalist's "Bloodline Ability or Rage Powers" is the bloodline row unless its choice replaces it.
    placeholders: &["Bloodline Ability", "Bloodline Ability or Rage Powers"],
    skips_without_placeholder: true,
    merged: Merged::Every,
    base_takes: true,
    // "A Hag-Riven can choose from the following bloodlines: Arcane, Elemental, Fey and Hag." "Bloodline must be
    // Abyssal or Infernal" (Reformed Fiend). The Mixed-Blood Rager "Can choose any bloodline each day": a daily
    // choice, no pick (ticket 29). The Primalist's "Bloodline Ability or Rage Powers" is its own choice (below).
    archetypes: &[
        (
            "bloodrager/hag-riven",
            Take::Only(&["arcane", "elemental", "fey", "hag"]),
        ),
        ("bloodrager/mixed-blood-rager", Take::No),
        (
            "bloodrager/reformed-fiend",
            Take::Only(&["abyssal", "infernal"]),
        ),
    ],
    row_choices: &[(
        "bloodrager/primalist",
        "Bloodline Ability or Rage Powers",
        PRIMAL_CHOICES,
        &[4, 8, 12, 16, 20],
    )],
};

pub(super) const WIZARD: Takes = Takes {
    class: "wizard",
    level: 1,
    choices: &[],
    kept_choices: false,
    placeholders: &["Specialisation abilities", "Specialisation ability"],
    skips_without_placeholder: false,
    merged: Merged::None,
    base_takes: true,
    // "Does not get an Arcane Bond ... Must select four Opposition Schools" (Arcane Bomber: no specialisation in
    // its table), "Always a specialist Necromancer" (Cruromancer), "Always a specialist Invoker" (Elemental
    // Specialist), "Cannot choose a specialisation" (Exploiter), "May select a specialisation or no
    // specialisation like a regular Wizard" (Scroll Savant), "May select a specialisation" (Shadowcaster), "Always a
    // Universalist" (Spell Master), the Thassilonian's seven schools (its table of spellbooks).
    archetypes: &[
        ("wizard/arcane-bomber", Take::No),
        (
            "wizard/cruromancer",
            Take::Fixed("wizard/school/necromancer"),
        ),
        (
            "wizard/elemental-specialist",
            Take::Fixed("wizard/school/invoker"),
        ),
        ("wizard/exploiter-wizard", Take::No),
        ("wizard/scroll-savant", Take::Yes),
        ("wizard/shadowcaster", Take::Yes),
        (
            "wizard/spell-master",
            Take::Fixed("wizard/school/universalist"),
        ),
        (
            "wizard/thassilonian-specialist",
            Take::Only(&[
                "abjurer",
                "necromancer",
                "transmuter",
                "enchanter",
                "illusionist",
                "conjurer",
                "invoker",
            ]),
        ),
    ],
    row_choices: &[],
};

/// Notes for an archetype's own selection: the Thassilonian's spellbooks, by the school each specialises in.
pub(super) const SUMMARIES: [(&str, &str); 1] = [(
    "wizard/thassilonian-specialist",
    "Spellbooks: Envy (Abjurer), Gluttony (Necromancer), Greed (Transmuter), Lust (Enchanter), Pride \
     (Illusionist), Sloth (Conjurer), Wrath (Invoker).",
)];

pub(super) const CAVALIER: Takes = Takes {
    class: "cavalier",
    level: 1,
    choices: &["Order"],
    kept_choices: false,
    placeholders: &["Order Ability"],
    skips_without_placeholder: true,
    merged: Merged::None,
    base_takes: true,
    archetypes: &[],
    row_choices: &[],
};

pub(super) const MAGIC_DECEIVER: Takes = Takes {
    class: "arcanist",
    level: 1,
    choices: &["Ways of the Deceiver"],
    kept_choices: false,
    placeholders: &[],
    skips_without_placeholder: false,
    merged: Merged::None,
    base_takes: false,
    archetypes: &[("arcanist/magic-deceiver", Take::Yes)],
    row_choices: &[],
};

pub(super) const SHAMAN: Takes = Takes {
    class: "shaman",
    level: 1,
    choices: &["Spirit"],
    kept_choices: false,
    placeholders: &[
        "Spirit Ability",
        "Greater Spirit Ability",
        "True Spirit Ability",
        "Manifestation",
    ],
    skips_without_placeholder: true,
    merged: Merged::None,
    base_takes: true,
    // "Guiding spirit is the Battle spirit" (Wildland Shaman).
    archetypes: &[(
        "shaman/wildland-shaman",
        Take::Fixed("shaman/spirit/battle"),
    )],
    row_choices: &[],
};

pub(super) const RANGER: Takes = Takes {
    class: "ranger",
    level: 2,
    choices: &[],
    kept_choices: false,
    placeholders: &[],
    skips_without_placeholder: false,
    merged: Merged::Tables(&["ranger"]),
    base_takes: true,
    // Every archetype's table has the "Combat Style Feat" cells but the Nomad's and the Stormwalker's, which take
    // the Archery feats only ("may only select Archery feats").
    archetypes: &[
        ("ranger/demonslayer", Take::Yes),
        ("ranger/espionage-expert", Take::Yes),
        ("ranger/flamewarden", Take::Yes),
        ("ranger/freebooter", Take::Yes),
        ("ranger/nomad", Take::No),
        ("ranger/sable-company-marine", Take::Yes),
        ("ranger/stormwalker", Take::No),
    ],
    row_choices: &[],
};
