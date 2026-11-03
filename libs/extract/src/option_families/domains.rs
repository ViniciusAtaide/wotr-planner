//! The 37 domains: each domain's powers by level with the phrase that says so, the domains' class skills, and the
//! domain family built from `character-creation#domains`.

use std::collections::BTreeMap;

use rules::{FeatSlot, FeatureKey, OptionLevelRow};

use crate::guide::Guide;
use crate::names::slug;
use crate::option_levels::{empty_row, feature, option, page, section, selection};

use super::{Family, Skills, contains};

/// What a domain gives, by level, each with the phrase that says so. An empty name is the War domain's bonus
/// Combat feat slot.
type Powers = &'static [(u8, &'static str, &'static str)];

/// The 37 domains under `character-creation#domains`, in the guide's order. The second power comes at the level its
/// section states (Air at 6th, Nobility at 8th: the cleric table's footnotes disagree on these two, the domain's own
/// section wins).
const DOMAINS: [(&str, Powers); 37] = [
    (
        "air-domain",
        &[
            (1, "Lightning Arc", "You can use Lightning Arc"),
            (
                6,
                "Electrical Resistance",
                "At 6th level, you gain 10 electrical resistance",
            ),
        ],
    ),
    (
        "animal-domain",
        &[(
            4,
            "Animal Companion",
            "At 4th level, you gain an Animal Companion",
        )],
    ),
    (
        "artifice-domain",
        &[
            (
                1,
                "Aura of Efficiency",
                "You can maintain an Aura of Efficiency",
            ),
            (
                8,
                "Aura of Efficiency (Damage Resistance)",
                "At 8th level, this provides 1/- Damage Resistance",
            ),
        ],
    ),
    (
        "chaos-domain",
        &[
            (1, "Touch of Chaos", "You can use Touch of Chaos"),
            (8, "Chaos Blade", "At 8th level, you gain Chaos Blade"),
        ],
    ),
    (
        "charm-domain",
        &[
            (1, "Dazing Touch", "You can use Dazing Touch"),
            (8, "Charming Smile", "At 8th level, you gain Charming Smile"),
        ],
    ),
    (
        "community-domain",
        &[
            (1, "Calming Touch", "You can use Calming Touch"),
            (
                8,
                "Guarded Hearth",
                "At 8th level, you gain the Guarded Hearth ability",
            ),
        ],
    ),
    (
        "darkness-domain",
        &[
            (1, "Touch of Darkness", "You can use Touch of Darkness"),
            (8, "Moonfire", "At 8th level you can use Moonfire"),
        ],
    ),
    (
        "death-domain",
        &[
            (1, "Bleeding Touch", "You can use Bleeding Touch"),
            (
                8,
                "Death's Embrace",
                "At 8th level you gain Death's Embrace",
            ),
        ],
    ),
    (
        "destruction-domain",
        &[
            (1, "Destructive Smite", "You can use Destructive Smite"),
            (
                8,
                "Destructive Aura",
                "At 8th level you gain the Destructive Aura ability",
            ),
        ],
    ),
    (
        "earth-domain",
        &[
            (1, "Acid Dart", "You can use Acid Dart"),
            (
                6,
                "Acid Resistance",
                "At 6th level, you gain 10 acid resistance",
            ),
        ],
    ),
    (
        "evil-domain",
        &[
            (1, "Touch of Evil", "You can use Touch of Evil"),
            (8, "Scythe of Evil", "At 8th level, you gain Scythe of Evil"),
        ],
    ),
    (
        "fire-domain",
        &[
            (1, "Fire Bolt", "You can use Fire Bolt"),
            (
                6,
                "Fire Resistance",
                "At 6th level, you gain 10 fire resistance",
            ),
        ],
    ),
    (
        "glory-domain",
        &[
            (1, "Touch of Glory", "You can use Touch of Glory"),
            (
                8,
                "Aura of Heroism",
                "At 8th level you gain the Aura of Heroism ability",
            ),
        ],
    ),
    (
        "good-domain",
        &[
            (1, "Touch of Good", "You can use Touch of Good"),
            (8, "Holy Lance", "At 8th level, you gain Holy Lance"),
        ],
    ),
    (
        "healing-domain",
        &[
            (1, "Rebuke Death", "You can use Rebuke Death"),
            (
                6,
                "Healer's Blessing",
                "At 6th level you gain Healer's Blessing",
            ),
        ],
    ),
    (
        "ice-subdomain",
        &[
            (1, "Icicle", "You can use Icicle"),
            (
                8,
                "Body of Ice",
                "At 8th level, you gain the Body of Ice ability",
            ),
        ],
    ),
    (
        "knowledge-domain",
        &[
            (1, "Void Form", "you can use Void Form"),
            (
                8,
                "Teaching Moment",
                "At 8th level, you gain the Teaching Moment ability",
            ),
        ],
    ),
    (
        "law-domain",
        &[
            (1, "Touch of Law", "You can use Touch of Law"),
            (8, "Staff of Order", "At 8th level, you gain Staff of Order"),
        ],
    ),
    (
        "liberation-domain",
        &[
            (1, "Freedom of Movement", "You gain Freedom of Movement"),
            (8, "Freedom's Call", "At 8th-level, you gain Freedom's Call"),
        ],
    ),
    (
        "luck-domain",
        &[
            (1, "Bit of Luck", "You can use Bit of Luck"),
            (
                6,
                "Divine Fortune",
                "At 6th level, you may bestow Divine Fortune",
            ),
        ],
    ),
    (
        "madness-domain",
        &[
            (1, "Vision of Madness", "You can use Vision of Madness"),
            (
                8,
                "Aura of Madness",
                "At 8th level you can use Aura of Madness",
            ),
        ],
    ),
    (
        "magic-domain",
        &[
            (1, "Hand of the Acolyte", "You can use Hand of the Acolyte"),
            (
                8,
                "Dispelling Touch",
                "At 8th level, you gain Dispelling Touch",
            ),
        ],
    ),
    (
        "murder-subdomain",
        &[
            (1, "Bleeding Touch", "You can use Bleeding Touch"),
            (
                8,
                "Weapons Infused with Death",
                "At 8th level you can infuse your weapons with death",
            ),
        ],
    ),
    (
        "nobility-domain",
        &[
            (1, "Inspiring Word", "You can use Inspiring Word"),
            (
                8,
                "Inspiring Command",
                "At 8th level, you gain Inspiring Command",
            ),
        ],
    ),
    (
        "plant-domain",
        &[
            (1, "Enlarge", "You can use Enlarge"),
            (
                6,
                "Bramble Armor",
                "At 6th level, you can use Bramble Armor",
            ),
        ],
    ),
    (
        "protection-domain",
        &[
            (1, "Resistant Touch", "You can use Resistant Touch"),
            (
                8,
                "Aura of Protection",
                "At 8th level you gain Aura of Protection",
            ),
        ],
    ),
    (
        "repose-domain",
        &[
            (1, "Gentle Rest", "You can use Gentle Rest"),
            (
                8,
                "Ward Against Death",
                "At 8th level you gain the Ward Against Death ability",
            ),
        ],
    ),
    (
        "rune-domain",
        &[
            (1, "Blast Rune", "You can use Blast Rune"),
            (
                8,
                "Warding Rune",
                "At 8th level, you can inscribe a Warding Rune",
            ),
        ],
    ),
    (
        "scalykind-domain",
        &[
            (1, "Venomous Stare", "You can use Venomous Stare"),
            (
                4,
                "Animal Companion",
                "At 4th level, you gain a Monitor Lizard or Velociraptor Animal Companion",
            ),
        ],
    ),
    (
        "strength-domain",
        &[
            (1, "Strength Surge", "You can use Strength Surge"),
            (
                8,
                "Might of the Gods",
                "At 8th level you gain Might of the Gods",
            ),
        ],
    ),
    (
        "sun-domain",
        &[
            (1, "Sun's Blessing", "You gain Sun's Blessing"),
            (
                8,
                "Nimbus of Light",
                "At 8th level you gain the Nimbus of Light ability",
            ),
        ],
    ),
    (
        "travel-domain",
        &[
            (1, "Agile Feet", "You can use Agile Feet"),
            (
                8,
                "Dimensional Hop",
                "At 8th level you receive Dimensional Hop",
            ),
        ],
    ),
    (
        "trickery-domain",
        &[
            (1, "Copycat", "You can use Copycat"),
            (
                8,
                "Master's Illusion",
                "At 8th level, you can use Master's Illusion",
            ),
        ],
    ),
    (
        "undead-subdomain",
        &[
            (1, "Death's Kiss", "You can use Death's Kiss"),
            (
                8,
                "Death's Embrace",
                "At 8th level you gain Death's Embrace",
            ),
        ],
    ),
    (
        "war-domain",
        &[
            (1, "Battle Rage", "You can use Battle Rage"),
            (8, "", "At 8th level, you can select a bonus combat feat"),
        ],
    ),
    (
        "water-domain",
        &[
            (1, "Icicle", "You can use Icicle"),
            (
                6,
                "Cold Resistance",
                "At 6th level, you gain 10 cold resistance",
            ),
        ],
    ),
    (
        "weather-domain",
        &[
            (1, "Storm Burst", "You can use Storm Burst"),
            (
                8,
                "Lightning Lord",
                "At 8th level, you gain the Lightning Lord ability",
            ),
        ],
    ),
];

/// Domains that add class skills: the phrase, the row the skills are on, then the skills. A domain's class skills
/// are its row's (ticket 37), so no "Class Skill: …" feature is made; the Animal domain's are on its 4th row, with
/// the companion, the level of the sentence they follow (ticket 38).
const DOMAIN_SKILLS: [(&str, &str, u8, &[&str]); 3] = [
    (
        "animal-domain",
        "At 4th level, you gain an Animal Companion. Your class level - 3 is used to determine your companion's \
         effective level. Lore (Nature) becomes a Class Skill.",
        4,
        &["Lore (Nature)"],
    ),
    (
        "knowledge-domain",
        "All Knowledge and Lore skills become class skills",
        1,
        &[
            "Knowledge (Arcana)",
            "Knowledge (World)",
            "Lore (Nature)",
            "Lore (Religion)",
        ],
    ),
    (
        "trickery-domain",
        "Stealth and Trickery become Class Skills",
        1,
        &["Stealth", "Trickery"],
    ),
];

/// A domain's short name: "air-domain" is "air", "ice-subdomain" is "ice".
pub fn domain_short(anchor: &str) -> &str {
    anchor
        .strip_suffix("-subdomain")
        .or_else(|| anchor.strip_suffix("-domain"))
        .expect("a domain anchor")
}

pub(super) fn domains(guide: &Guide, skills: &Skills) -> Family {
    let creation = page(guide, "character-creation");
    let within = section(creation, "domains");
    let headings: Vec<&str> = within
        .children
        .iter()
        .map(|child| child.anchor.as_str())
        .collect();
    let anchors: Vec<&str> = DOMAINS.iter().map(|(anchor, _)| *anchor).collect();
    assert_eq!(headings, anchors, "the guide's domains");
    let mut family = Family {
        selection: selection(
            "cleric/domain",
            "Domain",
            creation.guide("domains"),
            &["cleric"],
        ),
        features: Vec::new(),
        lists: Vec::new(),
        nested: Vec::new(),
    };
    for (anchor, powers) in DOMAINS {
        let domain = section(creation, anchor);
        let text = domain.text();
        let under = format!("cleric/domain/{}", domain_short(anchor));
        let mut rows: BTreeMap<u8, OptionLevelRow> = BTreeMap::new();
        if let Some((_, phrase, level, names)) = DOMAIN_SKILLS.iter().find(|(of, ..)| *of == anchor)
        {
            contains(&text, phrase, anchor);
            rows.entry(*level)
                .or_insert_with(|| empty_row(*level))
                .class_skills
                .extend(names.iter().map(|name| skills.get(name).0));
        }
        for (level, name, phrase) in powers {
            contains(&text, phrase, anchor);
            let row = rows.entry(*level).or_insert_with(|| empty_row(*level));
            if name.is_empty() {
                row.feat_slots.push(FeatSlot::CombatFeat);
                continue;
            }
            let key = FeatureKey::new(format!("{under}/{}", slug(name)));
            row.features.push(key.clone());
            family
                .features
                .push(feature(&key, name, creation.guide(anchor)));
        }
        family.selection.options.push(option(
            &under,
            &domain.name,
            creation.guide(anchor),
            rows.into_values().collect(),
        ));
    }
    family
}
