//! Options with rows by option level, the second group (ticket 31): domains, oracle mysteries and their revelations,
//! warpriest blessings and kineticist elements, on [`option_levels`](crate::option_levels)'s model.
//!
//! The guide gives these families in prose, so each option's rows are read from its section by a table here that
//! names, for each power, the level and a phrase the section must contain (asserted):
//! - **domains** (`character-creation#domains`, 37): a power at 1 and one at 6 or 8 (the War domain's bonus Combat
//!   feat is a slot), the Animal and Scalykind companion at 4, class skills as "Class Skill: …" features (ticket 30).
//!   One selection, `cleric/domain` (cleric levels), which the inquisitor, the Divine Hunter, the druid (through Nature
//!   Bond's domain option), the Blight Druid, the Winter Child, the Hellknight's Pentamic Faith and the Living
//!   Deity's Hollow Spark share through selections of their own, levelled by their own class;
//! - **mysteries** (`oracle#oracle-mysteries`, 9): class skills at 1, the Final Revelation at 20, and a pool of
//!   revelations (`oracle/revelation` offers the held mysteries' pools); revelations that grant later are rows;
//! - **blessings** (`warpriest#blessings`): the minor power at 1 and the major at 10, so the level-10 picks go;
//! - **elements** (`kineticist#elemental-focus`, 4): class skills, the wild-talent bonus feat list, the defensive
//!   talent at 2; Expanded Element at 7 and 15 is a plain pick of an element (which may double the first).
//!
//! The classes' placeholder cells ("Domain Abilities", "Revelation", "Defensive Wild Talent" …) go, and the cells
//! that are a choice become the selection. A scaling number stays in the summary (ticket 29), so a domain's
//! resistance growing to an immunity, or a revelation's uses per day, add no rows. [`apply`] runs on full rows and
//! asserts what it finds.

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    ArchetypeLevelRow, Class, ClassKey, ClassLevelRow, FeatList, FeatListKey, FeatSlot, Feature,
    FeatureKey, FeatureSelection, FeatureSelectionKey, GameData, OptionLevelRow, OptionRow,
    Prerequisite, Repeat, SelectionOption, SelectionOptionKey, SkillKey,
};

use crate::guide::Guide;
use crate::html::{Page, Section};
use crate::names::{norm, slug};
use crate::option_levels::{
    FeatNames, Log, drop_unused, empty_row, feature, list, name_list, option, page, section,
    selection,
};

/// Reads every family from the guide and gives it to the classes that take it.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    let feats = FeatNames::new(data);
    let skills = Skills::new(data);

    let domains = domains(guide, &skills);
    take_domains(data, &domains, &mut log);
    let mysteries = mysteries(guide, &skills, &feats);
    take_mysteries(data, &mysteries, &mut log);
    living_deity(data, &domains, &mysteries, &mut log);
    blessings(guide, data, &mut log);
    elements(guide, data, &skills, &feats, &mut log);
    hellknight(data, &domains, &mut log);
    prerequisites(data, &domains, &mysteries, &mut log);
    drop_unused(data, &TOUCHED, &mut log);
    normalise(data, &mut log);
    log
}

/// The classes whose features [`apply`] may leave unused.
const TOUCHED: [&str; 9] = [
    "arcanist",
    "cleric",
    "druid",
    "hellknight",
    "hunter",
    "inquisitor",
    "kineticist",
    "oracle",
    "warpriest",
];

/// Skill names as the guide writes them, to skill keys.
struct Skills(BTreeMap<String, (SkillKey, String)>);

impl Skills {
    fn new(data: &GameData) -> Self {
        Self(
            data.skills
                .iter()
                .map(|skill| (norm(&skill.name), (skill.key.clone(), skill.name.clone())))
                .collect(),
        )
    }

    /// The skill a guide name means, with its name as the data writes it.
    fn get(&self, name: &str) -> (SkillKey, String) {
        self.0
            .get(&norm(name))
            .cloned()
            .unwrap_or_else(|| panic!("no skill named {name:?}"))
    }
}

/// One family: its selection (options with rows), the features and lists the rows name, and selections the rows
/// grant.
pub struct Family {
    pub selection: FeatureSelection,
    pub features: Vec<Feature>,
    pub lists: Vec<FeatList>,
    pub nested: Vec<FeatureSelection>,
}

impl Family {
    fn keys(&self, short_names: &[&str]) -> BTreeSet<SelectionOptionKey> {
        short_names
            .iter()
            .map(|name| {
                let key = SelectionOptionKey::new(format!("{}/{name}", self.selection.key));
                assert!(
                    self.selection
                        .options
                        .iter()
                        .any(|option| option.key == key),
                    "no option {key}"
                );
                key
            })
            .collect()
    }

    fn all_keys(&self) -> BTreeSet<SelectionOptionKey> {
        self.selection
            .options
            .iter()
            .map(|option| option.key.clone())
            .collect()
    }
}

/// A "Class Skill: …" feature (ticket 30's shape for an option's class skill).
fn class_skill(under: &str, skills: &Skills, name: &str, guide: String) -> Feature {
    let (key, name) = skills.get(name);
    feature(
        &FeatureKey::new(format!("{under}/class-skill-{key}")),
        &format!("Class Skill: {name}"),
        guide,
    )
}

/// A section's text with its tables' cells too (a wild talent's description is a table row).
fn all_text(section: &Section) -> String {
    let mut text = section.text();
    for table in section.tables() {
        for row in table {
            text.push_str(&row.join(" "));
            text.push(' ');
        }
    }
    text
}

fn contains(text: &str, phrase: &str, at: &str) {
    assert!(text.contains(phrase), "{at}: no {phrase:?} in {text:?}");
}

// ---- domains ---------------------------------------------------------------------------------------------------

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

/// Domains that add class skills: the phrase, then the skills.
const DOMAIN_SKILLS: [(&str, &str, &[&str]); 3] = [
    (
        "animal-domain",
        "Lore (Nature) becomes a Class Skill",
        &["Lore (Nature)"],
    ),
    (
        "knowledge-domain",
        "All Knowledge and Lore skills become class skills",
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

fn domains(guide: &Guide, skills: &Skills) -> Family {
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
        if let Some((_, phrase, names)) = DOMAIN_SKILLS.iter().find(|(of, ..)| *of == anchor) {
            contains(&text, phrase, anchor);
            for name in *names {
                let skill = class_skill(&under, skills, name, creation.guide(anchor));
                rows.entry(1)
                    .or_insert_with(|| empty_row(1))
                    .features
                    .push(skill.key.clone());
                family.features.push(skill);
            }
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

// ---- rows ------------------------------------------------------------------------------------------------------

/// A class's or an archetype's level row.
trait Row {
    fn level(&self) -> u8;
    fn features(&mut self) -> &mut Vec<FeatureKey>;
    fn selections(&mut self) -> &mut Vec<FeatureSelectionKey>;
}

impl Row for ClassLevelRow {
    fn level(&self) -> u8 {
        self.level
    }
    fn features(&mut self) -> &mut Vec<FeatureKey> {
        &mut self.features
    }
    fn selections(&mut self) -> &mut Vec<FeatureSelectionKey> {
        &mut self.selections
    }
}

impl Row for ArchetypeLevelRow {
    fn level(&self) -> u8 {
        self.level
    }
    fn features(&mut self) -> &mut Vec<FeatureKey> {
        &mut self.features
    }
    fn selections(&mut self) -> &mut Vec<FeatureSelectionKey> {
        &mut self.selections
    }
}

fn class_mut<'d>(data: &'d mut GameData, key: &str) -> &'d mut Class {
    data.classes
        .iter_mut()
        .chain(&mut data.prestige_classes)
        .find(|class| class.key.as_str() == key)
        .unwrap_or_else(|| panic!("no class {key}"))
}

/// Every feature name the class and its archetypes define, by key, normalised.
fn names(class: &Class) -> BTreeMap<FeatureKey, String> {
    class
        .features
        .iter()
        .chain(class.archetypes.iter().flat_map(|a| &a.features))
        .map(|feature| (feature.key.clone(), norm(&feature.name)))
        .collect()
}

/// Which rows an edit applies to: the class's own, or one archetype's own.
#[derive(Clone, Copy, Debug)]
enum Rows<'a> {
    Class,
    Archetype(&'a str),
}

/// Runs `edit` over the rows `which` names, with the class's feature names.
fn edit_rows(
    class: &mut Class,
    which: Rows,
    mut edit: impl FnMut(&mut dyn Row, &BTreeMap<FeatureKey, String>),
) {
    let names = names(class);
    match which {
        Rows::Class => {
            for row in &mut class.levels {
                edit(row, &names);
            }
        }
        Rows::Archetype(key) => {
            let archetype = class
                .archetypes
                .iter_mut()
                .find(|a| a.key.as_str() == key)
                .unwrap_or_else(|| panic!("no archetype {key}"));
            for row in &mut archetype.levels {
                edit(row, &names);
            }
        }
    }
}

/// Removes from `row` up to `limit` features whose name `is` names; returns how many it removed.
fn strip(
    row: &mut dyn Row,
    names: &BTreeMap<FeatureKey, String>,
    limit: usize,
    is: impl Fn(&str) -> bool,
) -> usize {
    let mut removed = 0;
    let features = row.features();
    let mut kept = Vec::with_capacity(features.len());
    for feature in features.drain(..) {
        let name = names.get(&feature).map(String::as_str).unwrap_or_default();
        if removed < limit && is(name) {
            removed += 1;
        } else {
            kept.push(feature);
        }
    }
    *row.features() = kept;
    removed
}

fn grant(row: &mut dyn Row, selection: &FeatureSelectionKey, times: usize) {
    for _ in 0..times {
        row.selections().push(selection.clone());
    }
}

/// A selection of `owner`'s that offers `options` of a shared family, levelled by `levels`.
fn sharing(
    key: &str,
    name: &str,
    guide: String,
    levels: &[&str],
    options: BTreeSet<SelectionOptionKey>,
) -> FeatureSelection {
    FeatureSelection {
        shared_options: options,
        ..selection(key, name, guide, levels)
    }
}

fn add_selection(class: &mut Class, archetype: Option<&str>, selection: FeatureSelection) {
    let selections = match archetype {
        None => &mut class.selections,
        Some(key) => {
            &mut class
                .archetypes
                .iter_mut()
                .find(|a| a.key.as_str() == key)
                .unwrap_or_else(|| panic!("no archetype {key}"))
                .selections
        }
    };
    assert!(
        !selections.iter().any(|s| s.key == selection.key),
        "{} is defined already",
        selection.key
    );
    selections.push(selection);
}

fn starts(name: &str, prefix: &str) -> bool {
    name.starts_with(&norm(prefix))
}

/// "Domain Abilities", "Domain Ability (2)", "Domain ability - Icicle".
fn is_domain_cell(name: &str) -> bool {
    starts(name, "Domain Abilit")
}

// ---- taking the domains ----------------------------------------------------------------------------------------

/// The druid's domains: "If you choose a domain, you can choose from: Air, Animal, Earth, Fire, Ice, Plant, Water and
/// Weather."
const DRUID_DOMAINS: [&str; 8] = [
    "air", "animal", "earth", "fire", "ice", "plant", "water", "weather",
];
/// "May choose from the following domains: Air, Animal, Darkness, Death, Destruction, Earth, Fire, Murder, Plant,
/// Undead, Water, Weather." (Blight Druid)
const BLIGHT_DOMAINS: [&str; 12] = [
    "air",
    "animal",
    "darkness",
    "death",
    "destruction",
    "earth",
    "fire",
    "murder",
    "plant",
    "undead",
    "water",
    "weather",
];
/// "The Hellknight gains access to one of the following domains: Artifice, Glory, Knowledge, Law, Magic, Nobility,
/// Protection, Rune, Strength, Travel or War." (Pentamic Faith)
const PENTAMIC_DOMAINS: [&str; 11] = [
    "artifice",
    "glory",
    "knowledge",
    "law",
    "magic",
    "nobility",
    "protection",
    "rune",
    "strength",
    "travel",
    "war",
];

fn take_domains(data: &mut GameData, domains: &Family, log: &mut Log) {
    let domain = domains.selection.key.clone();

    // Cleric: the domain cells at 1 are the picks (two, or one for an archetype that "May only select a single
    // domain"); the cells at 4 ("Animal domain only"), 6 and 8 are the domains' rows now.
    let cleric = class_mut(data, "cleric");
    cleric.features.extend(domains.features.iter().cloned());
    cleric.selections.push(domains.selection.clone());
    let archetypes: Vec<String> = cleric
        .archetypes
        .iter()
        .map(|a| a.key.as_str().to_owned())
        .collect();
    for which in std::iter::once(Rows::Class).chain(archetypes.iter().map(|a| Rows::Archetype(a))) {
        let separatist = matches!(which, Rows::Archetype("cleric/separatist"));
        let mut picks = 0;
        edit_rows(cleric, which, |row, names| match row.level() {
            1 => {
                let cells = strip(row, names, usize::MAX, is_domain_cell);
                grant(row, &domain, cells);
                picks = cells;
            }
            // Forbidden Rites: the Separatist's second domain is delayed by two levels (its companion at 6, its
            // abilities at 8 and 10), which the model can't say: those cells stay (a question in ticket 31).
            4 => {
                strip(row, names, usize::MAX, |n| n == norm("Animal Companion"));
            }
            6 | 8 => {
                let limit = if separatist { 1 } else { usize::MAX };
                strip(row, names, limit, is_domain_cell);
            }
            _ => {}
        });
        log.push(format!("cleric {which:?}: {picks} domain picks"));
        assert!(picks > 0, "{which:?} picks no domain");
    }

    // Inquisitor: one domain, levelled by inquisitor levels; the Sacred Huntsmaster "May not select the Animal
    // domain" (and, character-creation says, not Scalykind); the Faith Hunter "Cannot select a domain".
    let inquisitor = class_mut(data, "inquisitor");
    let own = FeatureSelectionKey::new("inquisitor/domain");
    inquisitor.selections.push(sharing(
        own.as_str(),
        "Domain",
        "inquisitor".to_owned(),
        &["inquisitor"],
        domains.all_keys(),
    ));
    let huntsmaster = FeatureSelectionKey::new("inquisitor/sacred-huntsmaster/domain");
    add_selection(
        inquisitor,
        Some("inquisitor/sacred-huntsmaster"),
        sharing(
            huntsmaster.as_str(),
            "Domain (Sacred Huntsmaster)",
            "inquisitor#sacred-huntsmaster".to_owned(),
            &["inquisitor"],
            domains
                .all_keys()
                .into_iter()
                .filter(|key| {
                    !key.as_str().ends_with("/animal") && !key.as_str().ends_with("/scalykind")
                })
                .collect(),
        ),
    );
    let archetypes: Vec<String> = inquisitor
        .archetypes
        .iter()
        .map(|a| a.key.as_str().to_owned())
        .collect();
    for which in std::iter::once(Rows::Class).chain(archetypes.iter().map(|a| Rows::Archetype(a))) {
        let pick = match which {
            Rows::Archetype("inquisitor/sacred-huntsmaster") => huntsmaster.clone(),
            _ => own.clone(),
        };
        let mut picks = 0;
        edit_rows(inquisitor, which, |row, names| match row.level() {
            1 => {
                let cells = strip(row, names, usize::MAX, is_domain_cell);
                grant(row, &pick, cells);
                picks = cells;
            }
            4 => {
                strip(row, names, usize::MAX, |n| n == norm("Animal Companion"));
            }
            6 | 8 => {
                strip(row, names, usize::MAX, is_domain_cell);
            }
            _ => {}
        });
        log.push(format!("inquisitor {which:?}: {picks} domain picks"));
    }

    // Druid: Nature Bond's domain option grants the pick (levelled by druid levels), the druid's cells at 4
    // ("Animal domain only"), 6 ("Air, Earth, Fire, Plant and Water") and 8 (Lightning Lord: "Weather domain") are
    // the domains' rows; the Blight Druid picks from its twelve and "Does not receive an Animal Companion" (it skips
    // the companion at 4); the Winter Child "must select the Ice Subdomain".
    let druid = class_mut(data, "druid");
    let druid_domain = FeatureSelectionKey::new("druid/domain");
    druid.selections.push(sharing(
        druid_domain.as_str(),
        "Domain",
        "druid".to_owned(),
        &["druid"],
        domains.keys(&DRUID_DOMAINS),
    ));
    let bond = druid
        .selections
        .iter_mut()
        .find(|s| s.key.as_str() == "druid/nature-bond")
        .expect("Nature Bond");
    bond.option_levels = BTreeSet::from([ClassKey::new("druid")]);
    let domain_option = bond
        .options
        .iter_mut()
        .find(|o| o.key.as_str() == "druid/nature-bond/domain")
        .expect("Nature Bond's domain");
    domain_option.levels = vec![OptionLevelRow {
        selections: vec![druid_domain.clone()],
        ..empty_row(1)
    }];
    let blight = FeatureSelectionKey::new("druid/blight-druid/domain");
    add_selection(
        druid,
        Some("druid/blight-druid"),
        FeatureSelection {
            skipped_levels: BTreeSet::from([4]),
            ..sharing(
                blight.as_str(),
                "Domain (Blight Druid)",
                "druid#blight-druid".to_owned(),
                &["druid"],
                domains.keys(&BLIGHT_DOMAINS),
            )
        },
    );
    let winter = FeatureSelectionKey::new("druid/winter-child/domain");
    add_selection(
        druid,
        Some("druid/winter-child"),
        sharing(
            winter.as_str(),
            "Domain (Winter Child)",
            "druid#winter-child".to_owned(),
            &["druid"],
            domains.keys(&["ice"]),
        ),
    );
    for which in [
        Rows::Class,
        Rows::Archetype("druid/defender-of-the-true-world"),
        Rows::Archetype("druid/feyspeaker"),
    ] {
        let mut stripped = 0;
        edit_rows(druid, which, |row, names| {
            stripped += match row.level() {
                4 => strip(row, names, 1, |n| n == norm("Animal Companion")),
                6 => strip(row, names, 1, is_domain_cell),
                8 => strip(row, names, 1, |n| n == norm("Lightning Lord")),
                _ => 0,
            };
        });
        assert_eq!(
            stripped, 3,
            "druid {which:?}: the domain cells at 4, 6 and 8"
        );
    }
    for (which, pick, levels) in [
        ("druid/blight-druid", &blight, [1u8, 6, 8]),
        ("druid/winter-child", &winter, [1, 8, 0]),
    ] {
        let mut stripped = BTreeSet::new();
        edit_rows(druid, Rows::Archetype(which), |row, names| {
            let level = row.level();
            if levels.contains(&level) && strip(row, names, 1, is_domain_cell) == 1 {
                stripped.insert(level);
                if level == 1 {
                    grant(row, pick, 1);
                }
            }
        });
        assert!(stripped.contains(&1), "{which}: no domain cell at 1");
        log.push(format!("druid {which}: domain cells at {stripped:?}"));
    }
    // The Winter Child's two cells are the Ice Subdomain's two rows.
    let ice = domains
        .selection
        .options
        .iter()
        .find(|o| o.key.as_str() == "cleric/domain/ice")
        .expect("Ice");
    assert_eq!(
        ice.levels.iter().map(|row| row.level).collect::<Vec<_>>(),
        [1, 8]
    );

    // Divine Hunter: "May select a domain" (its Domain cell at 0), levelled by hunter levels. Its table has no domain
    // cells after that, so it skips the domains' later rows (a question in ticket 31).
    let hunter = class_mut(data, "hunter");
    let divine = FeatureSelectionKey::new("hunter/divine-hunter/domain");
    let later: BTreeSet<u8> = domains
        .selection
        .options
        .iter()
        .flat_map(|o| o.levels.iter().map(|row| row.level))
        .filter(|level| *level > 1)
        .collect();
    add_selection(
        hunter,
        Some("hunter/divine-hunter"),
        FeatureSelection {
            skipped_levels: later,
            ..sharing(
                divine.as_str(),
                "Domain (Divine Hunter)",
                "hunter#domain".to_owned(),
                &["hunter"],
                domains.all_keys(),
            )
        },
    );
    let mut cells = 0;
    edit_rows(
        hunter,
        Rows::Archetype("hunter/divine-hunter"),
        |row, names| {
            if row.level() == 1 {
                cells = strip(row, names, 1, |n| n == norm("Domain"));
                grant(row, &divine, cells);
            }
        },
    );
    assert_eq!(cells, 1, "the Divine Hunter's Domain cell");
}

// ---- mysteries and revelations ---------------------------------------------------------------------------------

/// The Battle mystery's Maneuver Mastery: "Select one type of combat manoeuvre (Bull Rush, Dirty Trick, Disarm,
/// Sunder Armor, Trip) … At 7th level, you gain the corresponding feat. At 11th level, you gain the corresponding
/// Greater... feat." A selection its first row grants, one option per manoeuvre.
const MANEUVER: &str = "oracle/maneuver-mastery";

/// Revelations that grant later, by `mystery/revelation` anchor: each row with the phrase that says so.
enum Later {
    /// Feats granted at a level (at 0: the option's own feats, held when it's picked).
    Feats(u8, &'static [&'static str], &'static str),
    /// Feats granted at a level for the weapon of the pick's chosen feat.
    Chosen(u8, &'static str, &'static str),
    /// A feature at a level.
    Feature(u8, &'static str, &'static str),
}

const REVELATION_ROWS: [(&str, &[Later]); 5] = [
    (
        "battle/war-sight",
        &[Later::Feature(
            7,
            "Uncanny Dodge",
            "At 7th level, you gain Uncanny Dodge",
        )],
    ),
    (
        "battle/weapon-mastery",
        &[
            Later::Chosen(
                8,
                "Improved Critical",
                "At 8th level, you gain Improved Critical",
            ),
            Later::Chosen(
                12,
                "Greater Weapon Focus",
                "At 12th level, you gain Greater Weapon Focus",
            ),
        ],
    ),
    (
        "stone/stone-stability",
        &[
            Later::Feats(5, &["Trip"], "At 5th level, you gain Trip as a bonus feat"),
            Later::Feats(
                10,
                &["Greater Trip"],
                "At 10th level, you gain Greater Trip",
            ),
        ],
    ),
    (
        "waves/fluid-nature",
        &[Later::Feats(
            5,
            &["Dodge"],
            "At 5th level, you gain Dodge as a bonus feat",
        )],
    ),
    (
        "battle/skill-at-arms",
        &[Later::Feats(
            0,
            // By key: the guide's "Heavy Armor Proficiency" is Armor Proficiency (Heavy Armor).
            &[
                "martial-weapons-proficiency",
                "armor-proficiency-heavy-armor",
            ],
            "You gain the Martial Weapons Proficiency and Heavy Armor Proficiency feats",
        )],
    ),
];

/// "Prerequisites: Level 7" (once "Prequisites:"): the class level whose revelation slot may take it.
pub fn revelation_level(text: &str) -> Option<u8> {
    let rest = text
        .split("Prerequisites: Level ")
        .nth(1)
        .or_else(|| text.split("Prequisites: Level ").nth(1))?;
    rest.split_whitespace().next()?.parse().ok()
}

fn mysteries(guide: &Guide, skills: &Skills, feats: &FeatNames) -> Family {
    let oracle = page(guide, "oracle");
    let within = section(oracle, "oracle-mysteries");
    assert_eq!(within.children.len(), 9, "the oracle's mysteries");
    let mut family = Family {
        selection: selection(
            "oracle/mystery",
            "Mystery",
            oracle.guide("mystery"),
            &["oracle"],
        ),
        features: Vec::new(),
        lists: Vec::new(),
        nested: Vec::new(),
    };
    let mut rows_done = BTreeSet::new();
    for mystery in &within.children {
        let key = format!("oracle/mystery/{}", mystery.anchor);
        let text = mystery.text();
        let mut levels = Vec::new();
        // "The Battle Mystery adds Perception to the list of class skills"
        if let Some(added) = text.split(" adds ").nth(1).and_then(|rest| {
            rest.split(" to the list of class skills")
                .next()
                .filter(|_| text.contains("to the list of class skills"))
        }) {
            let mut row = empty_row(1);
            for name in name_list(added) {
                let skill = class_skill(&key, skills, &name, oracle.guide(&mystery.anchor));
                row.features.push(skill.key.clone());
                family.features.push(skill);
            }
            levels.push(row);
        }
        let pool = FeatureSelectionKey::new(format!("oracle/revelations/{}", mystery.anchor));
        let mut revelations = selection(
            pool.as_str(),
            &format!("{} Revelations", mystery.name),
            oracle.guide(&mystery.anchor),
            &[],
        );
        let mut final_revelation = None;
        for revelation in &mystery.children {
            if revelation.name.ends_with("Final Revelation") {
                let feature_key = FeatureKey::new(format!("{key}/{}", revelation.anchor));
                family.features.push(feature(
                    &feature_key,
                    &revelation.name,
                    oracle.guide(&revelation.anchor),
                ));
                final_revelation = Some(feature_key);
                continue;
            }
            let option_key = format!("{pool}/{}", revelation.anchor);
            let mut taken = option(
                &option_key,
                &revelation.name,
                oracle.guide(&revelation.anchor),
                Vec::new(),
            );
            let text = revelation.text();
            if let Some(level) = revelation_level(&text) {
                taken.prerequisite = Some(Prerequisite::Any(vec![
                    Prerequisite::ClassLevel {
                        class: ClassKey::new("oracle"),
                        level,
                    },
                    // The Living Deity's revelations come by arcanist level.
                    Prerequisite::ClassLevel {
                        class: ClassKey::new("arcanist"),
                        level,
                    },
                ]));
            }
            let id = format!("{}/{}", mystery.anchor, revelation.anchor);
            if let Some((_, later)) = REVELATION_ROWS.iter().find(|(of, _)| *of == id) {
                rows_done.insert(id.clone());
                for row in *later {
                    let (Later::Feats(_, _, phrase)
                    | Later::Chosen(_, _, phrase)
                    | Later::Feature(_, _, phrase)) = row;
                    contains(&text, phrase, &id);
                    match row {
                        Later::Feats(0, names, _) => {
                            let names: Vec<String> =
                                names.iter().map(|n| (*n).to_owned()).collect();
                            taken.feats = feats.all(&names, &id);
                        }
                        Later::Feats(level, names, _) => {
                            let names: Vec<String> =
                                names.iter().map(|n| (*n).to_owned()).collect();
                            taken.levels.push(OptionLevelRow {
                                feats: feats.all(&names, &id),
                                ..empty_row(*level)
                            });
                        }
                        Later::Chosen(level, name, _) => {
                            taken.levels.push(OptionLevelRow {
                                chosen_feats: feats.all(&[(*name).to_owned()], &id),
                                ..empty_row(*level)
                            });
                        }
                        Later::Feature(level, name, _) => {
                            let feature_key =
                                FeatureKey::new(format!("{option_key}/{}", slug(name)));
                            family.features.push(feature(
                                &feature_key,
                                name,
                                oracle.guide(&revelation.anchor),
                            ));
                            taken.levels.push(OptionLevelRow {
                                features: vec![feature_key],
                                ..empty_row(*level)
                            });
                        }
                    }
                }
            }
            if id == "battle/weapon-mastery" {
                contains(&text, "You gain Weapon Focus as a bonus feat", &id);
                taken.chosen_feat = feats.get("Weapon Focus");
            }
            if id == "battle/maneuver-mastery" {
                rows_done.insert(id.clone());
                family
                    .nested
                    .push(maneuver_mastery(oracle, revelation, feats));
                taken.levels.push(OptionLevelRow {
                    selections: vec![FeatureSelectionKey::new(MANEUVER)],
                    ..empty_row(1)
                });
            }
            revelations.options.push(taken);
        }
        assert!(
            revelations.options.len() >= 5,
            "{}: revelations",
            mystery.anchor
        );
        let final_revelation =
            final_revelation.unwrap_or_else(|| panic!("{}: no Final Revelation", mystery.anchor));
        levels.push(OptionLevelRow {
            features: vec![final_revelation],
            ..empty_row(20)
        });
        family.selection.options.push(SelectionOption {
            pool: Some(pool),
            ..option(&key, &mystery.name, oracle.guide(&mystery.anchor), levels)
        });
        family.nested.push(revelations);
    }
    assert_eq!(rows_done.len(), REVELATION_ROWS.len() + 1, "{rows_done:?}");
    // The pool selection the class levels grant: the held mysteries' revelations, rows by oracle level.
    family.nested.push(FeatureSelection {
        pool_of: Some(family.selection.key.clone()),
        ..selection(
            "oracle/revelation",
            "Revelation",
            oracle.guide("revelation"),
            &["oracle"],
        )
    });
    family
}

fn maneuver_mastery(oracle: &Page, revelation: &Section, feats: &FeatNames) -> FeatureSelection {
    let text = revelation.text();
    contains(
        &text,
        "At 7th level, you gain the corresponding feat",
        MANEUVER,
    );
    contains(
        &text,
        "At 11th level, you gain the corresponding Greater... feat",
        MANEUVER,
    );
    let names = text
        .split("Select one type of combat manoeuvre (")
        .nth(1)
        .and_then(|rest| rest.split(')').next())
        .expect("the manoeuvres");
    let mut nested = selection(
        MANEUVER,
        "Maneuver Mastery",
        oracle.guide(&revelation.anchor),
        &["oracle"],
    );
    for name in name_list(names) {
        let feat = feats.all(std::slice::from_ref(&name), MANEUVER);
        let greater = feats.all(&[format!("Greater {name}")], MANEUVER);
        nested.options.push(option(
            &format!("{MANEUVER}/{}", slug(&name)),
            &name,
            oracle.guide(&revelation.anchor),
            vec![
                OptionLevelRow {
                    feats: feat,
                    ..empty_row(7)
                },
                OptionLevelRow {
                    feats: greater,
                    ..empty_row(11)
                },
            ],
        ));
    }
    assert_eq!(nested.options.len(), 5);
    nested
}

/// "You can choose from the Ancestor, Life, Nature, Stone or Waves mysteries." (Lone Strider)
const LONE_STRIDER_MYSTERIES: [&str; 5] = ["ancestor", "life", "nature", "stone", "waves"];

fn take_mysteries(data: &mut GameData, mysteries: &Family, log: &mut Log) {
    let mystery = mysteries.selection.key.clone();
    let revelation = FeatureSelectionKey::new("oracle/revelation");
    let oracle = class_mut(data, "oracle");
    oracle.features.extend(mysteries.features.iter().cloned());
    oracle.selections.push(mysteries.selection.clone());
    oracle.selections.extend(mysteries.nested.iter().cloned());

    // Archetypes that take the mystery their own way: "he does not gain additional class skills from his choice of
    // mystery" (Divine Herbalist: skips the class skills at 1), "Gains a unique Final Revelation" (Enlightened
    // Philosopher: skips 20), "Can choose from a restricted list of Mysteries" (Lone Strider), "Must select the Wind
    // Mystery" (Wind Whisperer: granted with no pick).
    let own =
        |archetype: &str, name: &str, skipped: &[u8], options: BTreeSet<SelectionOptionKey>| {
            FeatureSelection {
                skipped_levels: skipped.iter().copied().collect(),
                ..sharing(
                    &format!("{archetype}/mystery"),
                    &format!("Mystery ({name})"),
                    format!("oracle#{}", archetype.trim_start_matches("oracle/")),
                    &["oracle"],
                    options,
                )
            }
        };
    let class_skill_levels: BTreeSet<u8> = mysteries
        .selection
        .options
        .iter()
        .flat_map(|o| o.levels.iter())
        .filter(|row| {
            row.features
                .iter()
                .any(|f| f.as_str().contains("/class-skill-"))
        })
        .map(|row| {
            assert!(
                row.features
                    .iter()
                    .all(|f| f.as_str().contains("/class-skill-")),
                "a mystery's class skills share their row"
            );
            row.level
        })
        .collect();
    assert_eq!(class_skill_levels, BTreeSet::from([1]));
    let specials = [
        (
            "oracle/divine-herbalist",
            "Divine Herbalist",
            vec![1u8],
            mysteries.all_keys(),
        ),
        (
            "oracle/enlightened-philosopher",
            "Enlightened Philosopher",
            vec![20],
            mysteries.all_keys(),
        ),
        (
            "oracle/lone-strider",
            "Lone Strider",
            vec![],
            mysteries.keys(&LONE_STRIDER_MYSTERIES),
        ),
        (
            "oracle/wind-whisperer",
            "Wind Whisperer",
            vec![],
            mysteries.keys(&["wind"]),
        ),
    ];
    for (archetype, name, skipped, options) in &specials {
        add_selection(
            oracle,
            Some(archetype),
            own(archetype, name, skipped, options.clone()),
        );
    }

    let archetypes: Vec<String> = oracle
        .archetypes
        .iter()
        .map(|a| a.key.as_str().to_owned())
        .collect();
    for which in std::iter::once(Rows::Class).chain(archetypes.iter().map(|a| Rows::Archetype(a))) {
        let pick = match which {
            Rows::Archetype(archetype) if specials.iter().any(|(a, ..)| *a == archetype) => {
                FeatureSelectionKey::new(format!("{archetype}/mystery"))
            }
            _ => mystery.clone(),
        };
        let mut picks = 0;
        let mut slots = 0;
        edit_rows(oracle, which, |row, names| {
            if row.level() == 1 {
                // "Mystery", "Mystery (Lone Strider)", the Wind Whisperer's "Wind".
                let cells = strip(row, names, 1, |n| {
                    n == norm("Mystery") || starts(n, "Mystery (") || n == norm("Wind")
                });
                grant(row, &pick, cells);
                picks += cells;
            }
            let cells = strip(row, names, usize::MAX, |n| n == norm("Revelation"));
            grant(row, &revelation, cells);
            slots += cells;
            if row.level() == 20 {
                // The mystery's Final Revelation is its row 20 (the Wind Whisperer's "Wind Final Revelation" too);
                // the Enlightened Philosopher's own stays.
                strip(row, names, 1, |n| {
                    n == norm("Final Revelation") || n == norm("Wind Final Revelation")
                });
            }
        });
        log.push(format!(
            "oracle {which:?}: {picks} mystery picks, {slots} revelation slots"
        ));
    }
    // Every oracle and archetype holds a mystery from level 1.
    for archetype in &archetypes {
        let row = oracle
            .archetypes
            .iter()
            .find(|a| a.key.as_str() == archetype)
            .and_then(|a| a.levels.iter().find(|row| row.level == 1));
        let picks = row.map_or(1, |row| {
            row.selections
                .iter()
                .filter(|s| s.as_str().ends_with("/mystery"))
                .count()
        });
        assert_eq!(picks, 1, "{archetype}: mystery picks at 1");
    }
}

/// The Living Deity (a way of the Magic Deceiver): "She gains an Oracle's Mystery and Curse class features",
/// "receives Revelations appropriate to the Mystery at 1st, 4th, 12th and 20th levels along with the Final
/// Revelation", and "At 20th level, a Living Deity may select two domains, gaining any abilities granted by these
/// domains" (Hollow Spark). Its mystery, revelations and domains are selections of the Magic Deceiver's, levelled
/// by arcanist levels.
fn living_deity(data: &mut GameData, domains: &Family, mysteries: &Family, log: &mut Log) {
    let arcanist = class_mut(data, "arcanist");
    let deceiver = "arcanist/magic-deceiver";
    let mystery = FeatureSelectionKey::new(format!("{deceiver}/mystery"));
    let revelation = FeatureSelectionKey::new(format!("{deceiver}/revelation"));
    let domain = FeatureSelectionKey::new(format!("{deceiver}/domain"));
    add_selection(
        arcanist,
        Some(deceiver),
        sharing(
            mystery.as_str(),
            "Mystery (Living Deity)",
            "arcanist#mysterious-gift-mystery".to_owned(),
            &["arcanist"],
            mysteries.all_keys(),
        ),
    );
    add_selection(
        arcanist,
        Some(deceiver),
        FeatureSelection {
            pool_of: Some(mystery.clone()),
            ..selection(
                revelation.as_str(),
                "Revelation (Living Deity)",
                "arcanist#mysterious-gift-mystery".to_owned(),
                &["arcanist"],
            )
        },
    );
    add_selection(
        arcanist,
        Some(deceiver),
        sharing(
            domain.as_str(),
            "Domain (Living Deity)",
            "arcanist#hollow-spark".to_owned(),
            &["arcanist"],
            domains.all_keys(),
        ),
    );
    let names = names(arcanist);
    let deity = arcanist
        .archetypes
        .iter_mut()
        .find(|a| a.key.as_str() == deceiver)
        .expect("the Magic Deceiver")
        .selections
        .iter_mut()
        .find(|s| s.key.as_str() == "arcanist/magic-deceiver/ways-of-the-deceiver")
        .expect("the ways")
        .options
        .iter_mut()
        .find(|o| o.key.as_str() == "arcanist/magic-deceiver/ways-of-the-deceiver/living-deity")
        .expect("the Living Deity");
    let curse = FeatureSelectionKey::new("oracle/oracles-curse");
    let mut revelations = 0;
    for row in &mut deity.levels {
        let name = |key: &FeatureKey| names.get(key).cloned().unwrap_or_default();
        let before = row.features.len();
        let mut add = Vec::new();
        row.features.retain(|feature| {
            let n = name(feature);
            if n == norm("Mysterious Gift - Mystery") {
                add.push(mystery.clone());
            } else if n == norm("Mysterious Gift - Curse") {
                add.push(curse.clone());
            } else if n == norm("Revelation") {
                add.push(revelation.clone());
                revelations += 1;
            } else if n == norm("Hollow Spark") {
                add.push(domain.clone());
                add.push(domain.clone());
            } else if n != norm("Final Revelation") {
                return true;
            }
            false
        });
        if row.features.len() != before {
            log.push(format!("living deity {}: {add:?}", row.level));
        }
        row.selections.extend(add);
    }
    assert_eq!(
        revelations, 4,
        "the Living Deity's revelations at 1, 4, 12 and 20"
    );
}

// ---- blessings -------------------------------------------------------------------------------------------------

fn blessings(guide: &Guide, data: &mut GameData, log: &mut Log) {
    let warpriest_page = page(guide, "warpriest");
    let within = section(warpriest_page, "blessings");
    contains(
        &within.text(),
        "Each Blessing grants a minor power at 1st level and a major power at 10th level",
        "warpriest#blessings",
    );
    let warpriest = class_mut(data, "warpriest");
    let key = FeatureSelectionKey::new("warpriest/blessings");
    let mut features = Vec::new();
    {
        let blessings = warpriest
            .selections
            .iter_mut()
            .find(|s| s.key == key)
            .expect("the blessings");
        assert_eq!(blessings.options.len(), within.children.len());
        blessings.option_levels = BTreeSet::from([ClassKey::new("warpriest")]);
        for (option, heading) in blessings.options.iter_mut().zip(&within.children) {
            assert_eq!(option.guide.0, warpriest_page.guide(&heading.anchor));
            let text = heading.text();
            contains(&text, "At 1st level", &heading.anchor);
            contains(&text, "At 10th level", &heading.anchor);
            let short = option.name.trim_end_matches(" Domain").to_owned();
            let mut rows = Vec::new();
            for (level, power) in [(1u8, "Minor"), (10, "Major")] {
                let feature_key =
                    FeatureKey::new(format!("{}/{}-blessing", option.key, power.to_lowercase()));
                features.push(feature(
                    &feature_key,
                    &format!("{power} Blessing ({short})"),
                    warpriest_page.guide(&heading.anchor),
                ));
                rows.push(OptionLevelRow {
                    features: vec![feature_key],
                    ..empty_row(level)
                });
            }
            option.levels = rows;
        }
    }
    warpriest.features.extend(features);

    // "A Feral Champion must take the Animal Domain blessing and does not gain a second." "The Warpriest
    // automatically gains the Blessing for his chosen alignment" (Champion of the Faith).
    let feral = FeatureSelectionKey::new("warpriest/feral-champion/blessing");
    add_selection(
        warpriest,
        Some("warpriest/feral-champion"),
        sharing(
            feral.as_str(),
            "Blessing (Feral Champion)",
            "warpriest#feral-blessing".to_owned(),
            &["warpriest"],
            BTreeSet::from([SelectionOptionKey::new("warpriest/blessings/animal-domain")]),
        ),
    );
    let chosen = FeatureSelectionKey::new("warpriest/champion-of-the-faith/chosen-alignment");
    add_selection(
        warpriest,
        Some("warpriest/champion-of-the-faith"),
        sharing(
            chosen.as_str(),
            "Chosen Alignment Blessing",
            "warpriest#chosen-alignment".to_owned(),
            &["warpriest"],
            ["chaos-domain", "evil-domain", "good-domain", "law-domain"]
                .iter()
                .map(|b| SelectionOptionKey::new(format!("warpriest/blessings/{b}")))
                .collect(),
        ),
    );
    let archetypes: Vec<String> = warpriest
        .archetypes
        .iter()
        .map(|a| a.key.as_str().to_owned())
        .collect();
    for which in std::iter::once(Rows::Class).chain(archetypes.iter().map(|a| Rows::Archetype(a))) {
        let mut dropped = 0;
        edit_rows(warpriest, which, |row, names| {
            if row.level() == 10 {
                let before = row.selections().len();
                row.selections().retain(|s| *s != key);
                dropped += before - row.selections().len();
            }
            if matches!(row.level(), 1 | 10) {
                let cells = strip(row, names, 1, |n| n == norm("Feral Blessing"));
                if row.level() == 1 {
                    grant(row, &feral, cells);
                }
            }
            if row.level() == 1
                && matches!(which, Rows::Archetype("warpriest/champion-of-the-faith"))
            {
                grant(row, &chosen, 1);
            }
        });
        log.push(format!(
            "warpriest {which:?}: {dropped} level-10 picks dropped"
        ));
    }
}

// ---- kineticist elements ---------------------------------------------------------------------------------------

/// The guide's spellings that aren't a feat's name: "Lighting Reflexes".
pub fn element_feat(name: &str) -> String {
    name.replace("Lighting Reflexes", "Lightning Reflexes")
}

fn elements(guide: &Guide, data: &mut GameData, skills: &Skills, feats: &FeatNames, log: &mut Log) {
    let kineticist_page = page(guide, "kineticist");
    let within = section(kineticist_page, "elemental-focus");
    let bonus = all_text(section(kineticist_page, "kineticist-bonus-feat"));
    contains(
        &bonus,
        "You do not need to meet the normal prerequisites for the feat you select",
        "kineticist#kineticist-bonus-feat",
    );
    contains(
        &section(kineticist_page, "elemental-defense").text(),
        "At 2nd level, the Kineticist unlocks the Defensive Wild Talent corresponding to her Elemental Focus",
        "kineticist#elemental-defense",
    );
    let expanded = section(kineticist_page, "expanded-element").text();
    contains(
        &expanded,
        "At 7th level, the Kineticist may choose a second Elemental Focus or increase her primary focus",
        "kineticist#expanded-element",
    );
    let mut family = Family {
        selection: selection(
            "kineticist/element",
            "Elemental Focus",
            kineticist_page.guide("elemental-focus"),
            &["kineticist"],
        ),
        features: Vec::new(),
        lists: Vec::new(),
        nested: Vec::new(),
    };
    assert_eq!(within.children.len(), 4, "the four elements");
    for element in &within.children {
        let short = element.anchor.trim_start_matches("elemental-focus-");
        let key = format!("kineticist/element/{short}");
        let line = |label: &str| -> String {
            element
                .list_items()
                .find_map(|line| line.strip_prefix(label))
                .unwrap_or_else(|| panic!("{}: no {label:?}", element.anchor))
                .trim()
                .trim_end_matches('.')
                .to_owned()
        };
        let mut first = empty_row(1);
        for name in name_list(&line("Additional class skills:")) {
            let skill = class_skill(&key, skills, &name, kineticist_page.guide(&element.anchor));
            first.features.push(skill.key.clone());
            family.features.push(skill);
        }
        let names: Vec<String> = line("Wild Talent bonus feats:")
            .split(", ")
            .map(element_feat)
            .collect();
        let list_key = format!("{key}/wild-talent-feats");
        family.lists.push(list(
            &list_key,
            &format!(
                "Wild Talent Bonus Feats ({})",
                element.name.trim_start_matches("Elemental Focus - ")
            ),
            kineticist_page.guide(&element.anchor),
            true,
            feats.all(&names, &list_key),
        ));
        first.feat_list = Some(FeatListKey::new(&list_key));
        let talent = line("Defensive Wild Talent:");
        let talent_section = section(kineticist_page, &slug(&talent));
        contains(&all_text(talent_section), "Defensive", &talent);
        let talent_key = FeatureKey::new(format!("{key}/{}", slug(&talent)));
        family.features.push(feature(
            &talent_key,
            &talent,
            kineticist_page.guide(&talent_section.anchor),
        ));
        let second = OptionLevelRow {
            features: vec![talent_key],
            ..empty_row(2)
        };
        family.selection.options.push(SelectionOption {
            // "choose a second Elemental Focus or increase her primary focus": held twice at most.
            repeat: Repeat::Times(2),
            ..option(
                &key,
                &element.name,
                kineticist_page.guide(&element.anchor),
                vec![first, second],
            )
        });
    }

    let kineticist = class_mut(data, "kineticist");
    kineticist.features.extend(family.features.iter().cloned());
    kineticist.feat_lists.extend(family.lists.iter().cloned());
    kineticist.selections.push(family.selection.clone());
    let element = family.selection.key.clone();
    let expanded_key = FeatureSelectionKey::new("kineticist/expanded-element");
    // The guide says only that the new element brings its blasts: like Crossblooded's second bloodline, the pick
    // holds the element and gives its feat list, not its class skills or defensive talent.
    kineticist.selections.push(FeatureSelection {
        feat_lists_only: true,
        ..sharing(
            expanded_key.as_str(),
            "Expanded Element",
            kineticist_page.guide("expanded-element"),
            &["kineticist"],
            family.all_keys(),
        )
    });
    // "Must select Water at 1st and 7th levels" (Blood Kineticist: Water granted with no pick; its 7th level is
    // Blood Blast). The Kinetic Knight's Defensive Wild Talent comes at 4th, not 2nd: its own selection skips row 2
    // and its 4th-level feature grants that row.
    let blood = FeatureSelectionKey::new("kineticist/blood-kineticist/element");
    add_selection(
        kineticist,
        Some("kineticist/blood-kineticist"),
        sharing(
            blood.as_str(),
            "Elemental Focus (Blood Kineticist)",
            "kineticist#blood-kineticist".to_owned(),
            &["kineticist"],
            family.keys(&["water"]),
        ),
    );
    let knight = FeatureSelectionKey::new("kineticist/kinetic-knight/element");
    add_selection(
        kineticist,
        Some("kineticist/kinetic-knight"),
        FeatureSelection {
            skipped_levels: BTreeSet::from([2]),
            ..sharing(
                knight.as_str(),
                "Elemental Focus (Kinetic Knight)",
                "kineticist#kinetic-knight".to_owned(),
                &["kineticist"],
                family.all_keys(),
            )
        },
    );
    let knight_talent = kineticist
        .archetypes
        .iter_mut()
        .find(|a| a.key.as_str() == "kineticist/kinetic-knight")
        .expect("the Kinetic Knight")
        .features
        .iter_mut()
        .find(|f| f.name == "Defensive Wild Talent")
        .expect("the Kinetic Knight's Defensive Wild Talent");
    knight_talent.option_row = Some(OptionRow {
        selection: knight.clone(),
        level: 2,
    });
    let knight_talent = knight_talent.key.clone();

    let archetypes: Vec<String> = kineticist
        .archetypes
        .iter()
        .map(|a| a.key.as_str().to_owned())
        .collect();
    for which in std::iter::once(Rows::Class).chain(archetypes.iter().map(|a| Rows::Archetype(a))) {
        let pick = match which {
            Rows::Archetype("kineticist/blood-kineticist") => blood.clone(),
            Rows::Archetype("kineticist/kinetic-knight") => knight.clone(),
            _ => element.clone(),
        };
        let mut picks = 0;
        let mut expanded = 0;
        edit_rows(kineticist, which, |row, names| {
            match row.level() {
                1 => {
                    let cells = strip(row, names, 1, |n| n == norm("Class Skills"));
                    grant(row, &pick, cells);
                    picks += cells;
                }
                2 => {
                    strip(row, names, 1, |n| n == norm("Defensive Wild Talent"));
                }
                7 | 15 => {
                    let cells = strip(row, names, 1, |n| n == norm("Expanded Element"));
                    grant(row, &expanded_key, cells);
                    expanded += cells;
                }
                _ => {}
            }
            assert!(
                row.level() == 4 || !row.features().contains(&knight_talent),
                "the Kinetic Knight's talent at 4 only"
            );
        });
        log.push(format!(
            "kineticist {which:?}: {picks} element picks, {expanded} expanded"
        ));
    }
}

// ---- Hellknight ------------------------------------------------------------------------------------------------

/// "If the character was not previously an Armiger, they must choose one of the seven Hellknight orders. Otherwise,
/// they retain their current order." (the Armiger's seven, for a character holding none), and Pentamic Faith: "The
/// Hellknight gains access to one of the following domains … She uses her class level to determine which domain
/// abilities she can use."
fn hellknight(data: &mut GameData, domains: &Family, log: &mut Log) {
    let orders: BTreeSet<SelectionOptionKey> = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == "fighter")
        .and_then(|c| {
            c.archetypes
                .iter()
                .find(|a| a.key.as_str() == "fighter/armiger")
        })
        .and_then(|a| {
            a.selections
                .iter()
                .find(|s| s.key.as_str() == "fighter/armiger/order")
        })
        .expect("the Armiger's orders")
        .options
        .iter()
        .map(|o| o.key.clone())
        .collect();
    assert_eq!(orders.len(), 7);
    let hellknight = class_mut(data, "hellknight");
    let order = FeatureSelectionKey::new("hellknight/order");
    hellknight.selections.push(FeatureSelection {
        only_without_option: true,
        ..sharing(
            order.as_str(),
            "Order",
            "prestige-classes#order".to_owned(),
            &[],
            orders,
        )
    });
    let faith = FeatureSelectionKey::new("hellknight/pentamic-faith");
    hellknight.selections.push(sharing(
        faith.as_str(),
        "Pentamic Faith",
        "prestige-classes#pentamic-faith".to_owned(),
        &["hellknight"],
        domains.keys(&PENTAMIC_DOMAINS),
    ));
    let disciplines = hellknight
        .selections
        .iter_mut()
        .find(|s| s.key.as_str() == "hellknight/disciplines")
        .expect("the disciplines");
    disciplines.option_levels = BTreeSet::from([ClassKey::new("hellknight")]);
    disciplines
        .options
        .iter_mut()
        .find(|o| o.key.as_str() == "hellknight/disciplines/pentamic-faith")
        .expect("Pentamic Faith")
        .levels = vec![OptionLevelRow {
        selections: vec![faith],
        ..empty_row(1)
    }];
    let mut cells = 0;
    edit_rows(hellknight, Rows::Class, |row, names| {
        if row.level() == 1 {
            cells = strip(row, names, 1, |n| n == norm("Order"));
            grant(row, &order, cells);
        }
    });
    assert_eq!(cells, 1, "the Hellknight's Order cell");
    log.push("hellknight: order and Pentamic Faith's domain".to_owned());
}

// ---- prerequisites ---------------------------------------------------------------------------------------------

/// Domain Zealot's "Domain" and Second Mystery's "Mystery" are an option held; Second Mystery picks another
/// mystery. Boon Companion and Mythical Beast accept the domains' companion where they named the classes' "Animal
/// domain only" companions, and the Nature mystery's Animal Companion revelation.
fn prerequisites(data: &mut GameData, domains: &Family, mysteries: &Family, log: &mut Log) {
    let any_of = |keys: BTreeSet<SelectionOptionKey>| {
        Prerequisite::Any(
            keys.into_iter()
                .map(Prerequisite::SelectionOption)
                .collect(),
        )
    };
    for ability in &mut data.mythic_abilities {
        match ability.key.as_str() {
            "domain-zealot" => {
                ability.prerequisite = Some(any_of(domains.all_keys()));
                log.push("domain-zealot: any domain".to_owned());
            }
            "second-mystery" => {
                ability.prerequisite = Some(any_of(mysteries.all_keys()));
                ability.selections = vec![mysteries.selection.key.clone()];
                log.push("second-mystery: any mystery; picks another".to_owned());
            }
            _ => {}
        }
    }
    // The companions the domains' rows replace: each class's "Animal Companion" cell at 4.
    let gone: Vec<Prerequisite> = [
        "cleric/animal-companion",
        "cleric/angelfire-apostle/animal-companion",
        "cleric/crusader/animal-companion",
        "cleric/demonbane-priest/animal-companion",
        "cleric/ecclesitheurge/animal-companion",
        "cleric/herald-caller/animal-companion",
        "cleric/priest-of-balance/animal-companion",
        "druid/animal-companion",
        "druid/defender-of-the-true-world/animal-companion",
        "druid/feyspeaker/animal-companion",
        "inquisitor/animal-companion",
        "inquisitor/judge/animal-companion",
        "inquisitor/living-grimoire/animal-companion",
        "inquisitor/monster-tactician/animal-companion",
        "inquisitor/sanctified-slayer/animal-companion",
        "inquisitor/tactical-leader/animal-companion",
    ]
    .iter()
    .map(|key| Prerequisite::Feature {
        feature: FeatureKey::new(*key),
        count: 1,
    })
    .collect();
    // The domains' companion, and the Nature mystery's Animal Companion revelation.
    let companions = [
        Prerequisite::Feature {
            feature: FeatureKey::new("cleric/domain/animal/animal-companion"),
            count: 1,
        },
        Prerequisite::Feature {
            feature: FeatureKey::new("cleric/domain/scalykind/animal-companion"),
            count: 1,
        },
        Prerequisite::SelectionOption(SelectionOptionKey::new(
            "oracle/revelations/nature/animal-companion",
        )),
    ];
    // The Separatist's companion at 6 (its second domain's) stays a feature, so its leaf stays too.
    let prerequisites = data
        .feats
        .iter_mut()
        .map(|feat| (feat.key.as_str(), &mut feat.prerequisite))
        .chain(
            data.mythic_abilities
                .iter_mut()
                .map(|ability| (ability.key.as_str(), &mut ability.prerequisite)),
        );
    let mut changed = 0;
    for (key, prerequisite) in prerequisites {
        let Some(Prerequisite::Any(children)) = prerequisite else {
            continue;
        };
        let before = children.len();
        children.retain(|child| !gone.contains(child));
        if children.len() == before {
            continue;
        }
        assert_eq!(before - children.len(), gone.len(), "{key}");
        children.extend(companions.iter().cloned());
        changed += 1;
        log.push(format!("{key}: the domains' companion"));
    }
    assert_eq!(changed, 2, "Boon Companion and Mythical Beast");
}

// ---- tidying ---------------------------------------------------------------------------------------------------

/// An archetype row the edits made the same as the class's row is the class's row.
fn normalise(data: &mut GameData, log: &mut Log) {
    for class in data.classes.iter_mut().chain(&mut data.prestige_classes) {
        if !TOUCHED.contains(&class.key.as_str()) {
            continue;
        }
        let base = class.levels.clone();
        for archetype in &mut class.archetypes {
            archetype.levels.retain(|row| {
                let Some(base) = base.iter().find(|b| b.level == row.level) else {
                    return true;
                };
                let sorted = |items: Vec<String>| {
                    let mut items = items;
                    items.sort();
                    items
                };
                let strings =
                    |items: &[FeatureKey]| sorted(items.iter().map(|k| k.to_string()).collect());
                let slots =
                    |items: &[FeatSlot]| sorted(items.iter().map(|s| format!("{s:?}")).collect());
                let picks = |items: &[FeatureSelectionKey]| {
                    sorted(items.iter().map(|k| k.to_string()).collect())
                };
                let same = row
                    .spell_levels
                    .is_none_or(|spells| spells == base.spell_levels())
                    && strings(&row.features) == strings(&base.features)
                    && row.feats == base.feats
                    && slots(&row.feat_slots) == slots(&base.feat_slots)
                    && picks(&row.selections) == picks(&base.selections);
                if same {
                    log.push(format!(
                        "{} {}: same as the base row now",
                        archetype.key, row.level
                    ));
                }
                !same
            });
        }
    }
}
