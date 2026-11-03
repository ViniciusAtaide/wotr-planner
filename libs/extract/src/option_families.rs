//! Options with rows by option level, the second group (ticket 31): domains, oracle mysteries and their revelations,
//! warpriest blessings and kineticist elements, on [`option_levels`](crate::option_levels)'s model.
//!
//! The guide gives these families in prose, so each option's rows are read from its section by a table here that
//! names, for each power, the level and a phrase the section must contain (asserted):
//! - **domains** (`character-creation#domains`, 37): a power at 1 and one at 6 or 8 (the War domain's bonus Combat
//!   feat is a slot), the Animal and Scalykind companion at 4, class skills on their row (tickets 37 and 38: the
//!   Animal domain's on its 4th, with the companion). One selection, `cleric/domain` (cleric levels), which the inquisitor, the Divine Hunter, the druid (through Nature
//!   Bond's domain option), the Blight Druid, the Winter Child, the Hellknight's Pentamic Faith and the Living
//!   Deity's Hollow Spark share through selections of their own, levelled by their own class;
//! - **mysteries** (`oracle#oracle-mysteries`, 9): class skills at 1, the Final Revelation at 20, and a pool of
//!   revelations (`oracle/revelation` offers the held mysteries' pools); revelations that grant later are rows;
//! - **blessings** (`warpriest#blessings`): the minor power at 1 and the major at 10, so the level-10 picks go;
//! - **elements** (`kineticist#elemental-focus`, 4): class skills, the wild-talent bonus feat list, the defensive
//!   talent at 2; Expanded Element at 7 and 15 is a plain pick of an element (which may double the first).
//!
//! The classes' placeholder cells ("Domain Abilities", "Revelation", "Defensive Wild Talent" …) are the held
//! option's rows, and the cells that are a choice are the selection. A scaling number stays in the summary (ticket
//! 29), so a domain's resistance growing to an immunity, or a revelation's uses per day, add no rows.
//!
//! The domains, the mysteries, the blessings, the elements and the Hellknight's order and Pentamic Faith are built
//! with their classes ([`provide`], through [`crate::picks`]): no placeholder feature is made, and [`check`] asserts
//! the rows in force. With the domains: the Separatist's second domain and its alignment domains (tickets 33 and 35),
//! the Divine Hunter's every domain power (ticket 33), Domain Zealot's "Domain" and what Boon Companion's and Mythical
//! Beast's "Animal Companion" also accepts. [`apply`] takes the Living Deity's mystery and domain on the Magic
//! Deceiver's ways, and asserts what it finds (why it stays a pass is in [`crate::build::extract`]).

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    Class, ClassKey, DataAt, FeatList, FeatListKey, FeatSlot, Feature, FeatureKey,
    FeatureSelection, FeatureSelectionKey, GameData, LevelFrom, MythicAbilityKey, OptionLevelRow,
    OptionRow, Prerequisite, Repeat, SelectionOption, SelectionOptionKey, Skill, SkillKey,
};

use crate::guide::Guide;
use crate::html::{Page, Section};
use crate::names::{norm, slug};
use crate::option_levels::{
    FeatNames, Log, drop_unused, empty_row, feature, list, name_list, option, page, section,
    selection,
};
use crate::picks::{Pick, Picks, Rule};

/// The Living Deity's mystery, revelations and domains, on the Magic Deceiver's ways (why it stays a pass is in
/// [`crate::build::extract`]); the labels its ways' rows held go.
pub fn apply(guide: &Guide, data: &mut GameData) -> Log {
    let mut log = Log::new();
    let feats = FeatNames::new(data);
    let skills = Skills::new(data);
    let domains = domains(guide, &skills);
    let mysteries = mysteries(guide, &skills, &feats);
    living_deity(data, &domains, &mysteries, &mut log);
    drop_unused(data, &["arcanist"], &mut log);
    log
}

/// The families built with their owners, before any class is: the oracle's mysteries, the warpriest's blessings, the
/// kineticist's elements, the domains and the Hellknight's order and Pentamic Faith.
pub(crate) fn provide(guide: &Guide, feats: &FeatNames, skills: &[Skill], picks: &mut Picks) {
    let skills = Skills::of(skills);
    oracle_mysteries(guide, &mysteries(guide, &skills, feats), picks);
    blessings(guide, picks);
    elements(guide, &skills, feats, picks);
    let domains = domains(guide, &skills);
    provide_domains(guide, &domains, picks);
    hellknight(guide, &domains, picks);
}

/// What the providers can only assert on the finished data: the rows in force. Changes nothing.
pub(crate) fn check(guide: &Guide, data: &GameData) {
    check_mysteries(data);
    check_blessings(guide, data);
    check_elements(data);
    check_hellknight(data);
    check_domains(data);
}

/// Skill names as the guide writes them, to skill keys.
struct Skills(BTreeMap<String, (SkillKey, String)>);

impl Skills {
    fn new(data: &GameData) -> Self {
        Self::of(&data.skills)
    }

    fn of(skills: &[Skill]) -> Self {
        Self(
            skills
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

// ---- rows ------------------------------------------------------------------------------------------------------

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

const DOMAIN: &str = "cleric/domain";
const INQUISITOR_DOMAIN: &str = "inquisitor/domain";
const HUNTSMASTER_DOMAIN: &str = "inquisitor/sacred-huntsmaster/domain";
const DRUID_DOMAIN: &str = "druid/domain";
const BLIGHT_DOMAIN: &str = "druid/blight-druid/domain";
const WINTER_DOMAIN: &str = "druid/winter-child/domain";
const DIVINE_HUNTER_DOMAIN: &str = "hunter/divine-hunter/domain";
const SEPARATIST: &str = "cleric/separatist";
const SEPARATIST_DOMAIN: &str = "cleric/separatist/domain";
const PENTAMIC_DISCIPLINE: &str = "hellknight/disciplines/pentamic-faith";

/// The cells a domain table names its picks and its powers with: "Domain Abilities" at 1 (one or two, "(1)" and
/// "(2)"), "Domain Ability" at 6 and 8.
const PICK_CELLS: [&str; 3] = [
    "domain-abilities",
    "domain-abilities-1",
    "domain-abilities-2",
];
const POWER_CELLS: [&str; 3] = ["domain-ability", "domain-ability-1", "domain-ability-2"];

/// Each cell `{table}/{slug}` of `table` at `levels` is `pick`.
fn cells(picks: &mut Picks, table: &str, slugs: &[&str], levels: &[u8], pick: &Pick) {
    for slug in slugs {
        picks.rule(Rule {
            table: Some(table.to_owned()),
            levels: Some(levels.to_vec()),
            ..Rule::every(&format!("{table}/{slug}"), pick.clone())
        });
    }
}

/// The domains, built with the classes that take them. Each table's domain cells at 1 are the picks, and the cells
/// the domains' rows stand for (the "Animal domain only" companion at 4, the powers at 6 and 8) are the held
/// domain's rows ([`Pick::OptionRow`]), so none of those features is made. [`check_domains`] asserts the rows in
/// force.
fn provide_domains(guide: &Guide, domains: &Family, picks: &mut Picks) {
    let domain = Pick::Selection(FeatureSelectionKey::new(DOMAIN));
    let held = Pick::OptionRow;
    picks.selection("cleric", separatist_alignments(guide, domains));
    // A "Domain" line (Impossible Domain's) names the Divine Hunter's "Domain" feature, as it did when the domains
    // were built after the lines were resolved.
    picks.unnamed(DOMAIN);
    for feature in &domains.features {
        picks.feature("cleric", feature.clone());
    }

    // Cleric: the domain cells at 1 are the picks (two, or one for an archetype that "May only select a single
    // domain"); the cells at 4 ("Animal domain only"), 6 and 8 are the domains' rows.
    for table in crate::build::class_tables(guide, "cleric") {
        cells(picks, &table, &PICK_CELLS, &[1], &domain);
        cells(picks, &table, &["animal-companion"], &[4], &held);
        cells(picks, &table, &POWER_CELLS, &[6, 8], &held);
    }
    // The Separatist (ticket 33's decision on 31-1): "Higher level abilities are delayed by two class levels." Its
    // second domain is its own selection, two levels behind, granted beside its first at 1; that domain's cells (the
    // companion at 6, a Domain Ability at 8 and 10) are its rows.
    crate::follow_ups::says(
        guide,
        "cleric",
        "forbidden-rites",
        "Higher level abilities are delayed by two class levels.",
    );
    assert_eq!(domains.selection.options.len(), 37, "the 37 domains");
    picks.selection(
        SEPARATIST,
        FeatureSelection {
            levels_behind: 2,
            ..sharing(
                SEPARATIST_DOMAIN,
                "Domain (Separatist)",
                "cleric#forbidden-rites".into(),
                &["cleric"],
                domains.all_keys(),
            )
        },
    );
    picks.row_selection(SEPARATIST, 1, SEPARATIST_DOMAIN);
    cells(picks, SEPARATIST, &["animal-companion"], &[6], &held);
    cells(picks, SEPARATIST, &["domain-ability"], &[10], &held);

    // Inquisitor: one domain, levelled by inquisitor levels; the Sacred Huntsmaster "May not select the Animal
    // domain" (and, character-creation says, not Scalykind); the Faith Hunter "Cannot select a domain".
    picks.selection(
        "inquisitor",
        sharing(
            INQUISITOR_DOMAIN,
            "Domain",
            "inquisitor".to_owned(),
            &["inquisitor"],
            domains.all_keys(),
        ),
    );
    picks.selection(
        "inquisitor/sacred-huntsmaster",
        sharing(
            HUNTSMASTER_DOMAIN,
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
    for table in crate::build::class_tables(guide, "inquisitor") {
        let pick = if table == "inquisitor/sacred-huntsmaster" {
            HUNTSMASTER_DOMAIN
        } else {
            INQUISITOR_DOMAIN
        };
        let pick = Pick::Selection(FeatureSelectionKey::new(pick));
        cells(picks, &table, &PICK_CELLS, &[1], &pick);
        cells(picks, &table, &["animal-companion"], &[4], &held);
        cells(picks, &table, &POWER_CELLS, &[6, 8], &held);
    }

    // Druid: Nature Bond's domain option grants the pick (levelled by druid levels; the bond is
    // [`crate::class_rules`]'), the druid's cells at 4 ("Animal domain only"), 6 ("Air, Earth, Fire, Plant and
    // Water") and 8 (Lightning Lord: "Weather domain") are the domains' rows; the Blight Druid picks from its twelve
    // and "Does not receive an Animal Companion" (it skips the companion at 4); the Winter Child "must select the Ice
    // Subdomain".
    picks.selection(
        "druid",
        sharing(
            DRUID_DOMAIN,
            "Domain",
            "druid".to_owned(),
            &["druid"],
            domains.keys(&DRUID_DOMAINS),
        ),
    );
    picks.selection(
        "druid/blight-druid",
        FeatureSelection {
            skipped_levels: BTreeSet::from([4]),
            ..sharing(
                BLIGHT_DOMAIN,
                "Domain (Blight Druid)",
                "druid#blight-druid".to_owned(),
                &["druid"],
                domains.keys(&BLIGHT_DOMAINS),
            )
        },
    );
    picks.selection(
        "druid/winter-child",
        sharing(
            WINTER_DOMAIN,
            "Domain (Winter Child)",
            "druid#winter-child".to_owned(),
            &["druid"],
            domains.keys(&["ice"]),
        ),
    );
    for table in [
        "druid",
        "druid/defender-of-the-true-world",
        "druid/feyspeaker",
    ] {
        cells(picks, table, &["animal-companion"], &[4], &held);
        cells(picks, table, &["domain-ability"], &[6], &held);
        cells(picks, table, &["lightning-lord"], &[8], &held);
    }
    let blight = Pick::Selection(FeatureSelectionKey::new(BLIGHT_DOMAIN));
    cells(
        picks,
        "druid/blight-druid",
        &["domain-abilities"],
        &[1],
        &blight,
    );
    cells(
        picks,
        "druid/blight-druid",
        &["domain-ability"],
        &[6, 8],
        &held,
    );
    // The Winter Child's two cells are the Ice Subdomain's two rows.
    let winter = Pick::Selection(FeatureSelectionKey::new(WINTER_DOMAIN));
    cells(
        picks,
        "druid/winter-child",
        &["domain-ability-icicle"],
        &[1],
        &winter,
    );
    cells(
        picks,
        "druid/winter-child",
        &["domain-ability-body-of-ice"],
        &[8],
        &held,
    );
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

    // Divine Hunter: "May select a domain" (its Domain cell at 1), levelled by hunter levels, every domain power by
    // hunter level (ticket 33's decision on 31-2: no skipped levels). The cell's feature stays defined, off the row:
    // Impossible Domain's "Domain" line names it.
    picks.selection(
        "hunter/divine-hunter",
        sharing(
            DIVINE_HUNTER_DOMAIN,
            "Domain (Divine Hunter)",
            "hunter#domain".to_owned(),
            &["hunter"],
            domains.all_keys(),
        ),
    );
    cells(
        picks,
        "hunter/divine-hunter",
        &["domain"],
        &[1],
        &Pick::SelectionKeepingFeature(FeatureSelectionKey::new(DIVINE_HUNTER_DOMAIN)),
    );

    // Domain Zealot's "Domain" is a domain held (Second Mystery's "Mystery" is stated where the mysteries are built).
    picks.prerequisite(
        DataAt::MythicAbility(MythicAbilityKey::new("domain-zealot")),
        "Domain",
        Prerequisite::Any(
            domains
                .all_keys()
                .into_iter()
                .map(Prerequisite::SelectionOption)
                .collect(),
        ),
    );
    // Boon Companion's and Mythical Beast's "Animal Companion" accepts the domains' companion, whose rows the classes'
    // "Animal domain only" companions are, and the Nature mystery's Animal Companion revelation.
    let companions = vec![
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
    picks.alternatives(
        DataAt::Feat(rules::FeatKey::new("boon-companion")),
        "Animal Companion",
        companions.clone(),
    );
    picks.alternatives(
        DataAt::MythicAbility(MythicAbilityKey::new("mythical-beast")),
        "Animal Companion",
        companions,
    );
}

/// The Separatist (ticket 35's decision on 33-6): "This second domain cannot be an alignment domain that conflicts
/// with the cleric's own alignment." Each alignment domain, picked through the Separatist's second-domain selection,
/// needs an alignment off the opposite side of its axis.
fn separatist_alignments(guide: &Guide, domains: &Family) -> FeatureSelection {
    use rules::Alignment::*;
    crate::follow_ups::says(
        guide,
        "cleric",
        "forbidden-rites",
        "This second domain cannot be an alignment domain that conflicts with the cleric's own alignment.",
    );
    let mut selection = domains.selection.clone();
    for (domain, alignments) in [
        (
            "chaos",
            [
                NeutralGood,
                ChaoticGood,
                TrueNeutral,
                ChaoticNeutral,
                NeutralEvil,
                ChaoticEvil,
            ],
        ),
        (
            "evil",
            [
                LawfulNeutral,
                TrueNeutral,
                ChaoticNeutral,
                LawfulEvil,
                NeutralEvil,
                ChaoticEvil,
            ],
        ),
        (
            "good",
            [
                LawfulGood,
                NeutralGood,
                ChaoticGood,
                LawfulNeutral,
                TrueNeutral,
                ChaoticNeutral,
            ],
        ),
        (
            "law",
            [
                LawfulGood,
                NeutralGood,
                LawfulNeutral,
                TrueNeutral,
                LawfulEvil,
                NeutralEvil,
            ],
        ),
    ] {
        let key = format!("{DOMAIN}/{domain}");
        let option = selection
            .options
            .iter_mut()
            .find(|option| option.key.as_str() == key)
            .unwrap_or_else(|| panic!("no {key}"));
        assert_eq!(option.prerequisite, None, "{key}");
        let mut children: Vec<Prerequisite> = alignments.map(Prerequisite::Alignment).into();
        children.push(Prerequisite::Not(Box::new(Prerequisite::PickedThrough(
            FeatureSelectionKey::new(SEPARATIST_DOMAIN),
        ))));
        children.sort();
        option.prerequisite = Some(Prerequisite::Any(children));
    }
    selection
}

/// The domain cells' features are made nowhere: no cleric, inquisitor or druid feature is a domain cell's or an
/// "Animal domain only" companion (the Sacred Huntsmaster's own companion at 1 aside), and each cleric table picks
/// a domain at 1. Changes nothing.
fn check_domains(data: &GameData) {
    for class in data
        .classes
        .iter()
        .filter(|c| ["cleric", "inquisitor", "druid"].contains(&c.key.as_str()))
    {
        let features = class
            .features
            .iter()
            .chain(class.archetypes.iter().flat_map(|a| &a.features));
        for feature in features {
            let name = norm(&feature.name);
            let companion = name == norm("Animal Companion")
                && feature.key.as_str() != "inquisitor/sacred-huntsmaster/animal-companion"
                && !feature.key.as_str().starts_with("cleric/domain/")
                && !feature.key.as_str().starts_with("druid/drovier/");
            assert!(
                !starts(&name, "Domain Abilit") && !companion,
                "{}: a domain cell's feature",
                feature.key
            );
        }
    }
    let cleric = data
        .classes
        .iter()
        .find(|c| c.key.as_str() == "cleric")
        .expect("the cleric");
    let domain = FeatureSelectionKey::new(DOMAIN);
    let base = cleric
        .levels
        .iter()
        .find(|row| row.level == 1)
        .expect("row 1");
    assert_eq!(
        base.selections.iter().filter(|s| **s == domain).count(),
        2,
        "the cleric's two domains"
    );
    for archetype in &cleric.archetypes {
        let first = archetype
            .levels
            .iter()
            .find(|row| row.level == 1)
            .map_or(&base.selections, |row| &row.selections);
        assert!(first.contains(&domain), "{} picks no domain", archetype.key);
    }
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
            // The mystery's class skills are its row 1's while it's held (ticket 37).
            let mut row = empty_row(1);
            for name in name_list(added) {
                row.class_skills.insert(skills.get(&name).0);
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
            if id == "flame/cinder-dance" {
                // "You cannot select this revelation if you have the Lame curse." (ticket 33)
                contains(
                    &text,
                    "You cannot select this revelation if you have the Lame curse.",
                    &id,
                );
                crate::option_follow_ups::and(
                    &mut taken.prerequisite,
                    Prerequisite::Not(Box::new(Prerequisite::SelectionOption(
                        SelectionOptionKey::new("oracle/oracles-curse/lame"),
                    ))),
                );
            }
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
    // Its maneuver follows its revelation's level, an oracle's or a Living Deity's (ticket 33).
    let mut nested = FeatureSelection {
        level_from: Some(LevelFrom::GrantingOption),
        ..selection(
            MANEUVER,
            "Maneuver Mastery",
            oracle.guide(&revelation.anchor),
            &[],
        )
    };
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

const MYSTERY: &str = "oracle/mystery";
const REVELATION: &str = "oracle/revelation";
const SECOND_MYSTERY: &str = "oracle/second-mystery";
const DUAL_CURSED: &str = "oracle/dual-cursed-oracle";
const DUAL_CURSED_REVELATION: &str = "oracle/dual-cursed-oracle/revelation";

/// The Dual-Cursed Oracle's revelation pick: the held mystery's pool, plus Misfortune and Fortune.
fn dual_cursed_revelations(guide: &Guide) -> FeatureSelection {
    let oracle = page(guide, "oracle");
    contains(
        &section(oracle, "additional-revelations").text(),
        "he can choose two Revelations unique to the archetype",
        "oracle#additional-revelations",
    );
    let own = ["Misfortune", "Fortune"].map(|name| {
        let anchor = name.to_lowercase();
        // The sections exist.
        contains(&section(oracle, &anchor).text(), "this revelation", &anchor);
        option(
            &format!("{DUAL_CURSED_REVELATION}/{anchor}"),
            name,
            format!("oracle#{anchor}"),
            Vec::new(),
        )
    });
    FeatureSelection {
        options: own.into(),
        pool_of: Some(FeatureSelectionKey::new(MYSTERY)),
        ..selection(
            DUAL_CURSED_REVELATION,
            "Revelation (Dual-Cursed Oracle)",
            "oracle#additional-revelations".into(),
            &["oracle"],
        )
    }
}

/// The oracle's tables (the class's and each archetype's), each with the selection its mystery cell at 1 picks.
/// Archetypes that take the mystery their own way: "he does not gain additional class skills from his choice of
/// mystery" (Divine Herbalist: skips the class skills at 1), "Gains a unique Final Revelation" (Enlightened
/// Philosopher: skips 20), "Can choose from a restricted list of Mysteries" (Lone Strider), "Must select the Wind
/// Mystery" (Wind Whisperer: granted with no pick).
const ORACLE_TABLES: [(&str, &str); 9] = [
    ("oracle", MYSTERY),
    ("oracle/divine-herbalist", "oracle/divine-herbalist/mystery"),
    (
        "oracle/enlightened-philosopher",
        "oracle/enlightened-philosopher/mystery",
    ),
    ("oracle/lone-strider", "oracle/lone-strider/mystery"),
    ("oracle/wind-whisperer", "oracle/wind-whisperer/mystery"),
    ("oracle/dual-cursed-oracle", MYSTERY),
    ("oracle/possessed-oracle", MYSTERY),
    ("oracle/purifier", MYSTERY),
    ("oracle/seeker", MYSTERY),
];

/// The oracle's mysteries, built with the class: the selection, its options' rows, the revelation pools and the
/// archetypes' own selections. The "Mystery" cell at 1 (the Lone Strider's "Mystery (Lone Strider)", the Wind
/// Whisperer's "Wind") is the table's mystery pick, every "Revelation" cell a slot of `oracle/revelation`, and the
/// "Final Revelation" at 20 the held mystery's row (the Wind Whisperer's "Wind Final Revelation" too; the Enlightened
/// Philosopher's own stays).
fn oracle_mysteries(guide: &Guide, mysteries: &Family, picks: &mut Picks) {
    for feature in &mysteries.features {
        picks.feature("oracle", feature.clone());
    }
    picks.selection("oracle", mysteries.selection.clone());
    for nested in &mysteries.nested {
        picks.selection("oracle", nested.clone());
    }
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
        .filter(|row| !row.class_skills.is_empty())
        .map(|row| {
            assert!(
                row.features.is_empty(),
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
        picks.selection(archetype, own(archetype, name, skipped, options.clone()));
    }
    // "he can choose two Revelations unique to the archetype" (Dual-Cursed Oracle): its revelations are the
    // mystery's pool plus Misfortune and Fortune, at every revelation level of its table (ticket 33).
    picks.selection(DUAL_CURSED, dual_cursed_revelations(guide));
    // Second Mystery (ticket 33): "You gain a second Oracle Mystery", levelled like the first; it needs a mystery
    // held.
    contains(
        &section(page(guide, "mythic-paths"), "second-mystery").text(),
        "You gain a second Oracle Mystery.",
        "mythic-paths#second-mystery",
    );
    picks.selection(
        "oracle",
        FeatureSelection {
            shared_options: mysteries.all_keys(),
            level_from: Some(LevelFrom::OtherOptions),
            ..selection(
                SECOND_MYSTERY,
                "Second Mystery",
                "mythic-paths#second-mystery".into(),
                &[],
            )
        },
    );
    picks.mythic_selection("second-mystery", FeatureSelectionKey::new(SECOND_MYSTERY));
    picks.prerequisite(
        DataAt::MythicAbility(MythicAbilityKey::new("second-mystery")),
        "Mystery",
        Prerequisite::Any(
            mysteries
                .all_keys()
                .into_iter()
                .map(Prerequisite::SelectionOption)
                .collect(),
        ),
    );

    let rule = |feature: &str, table: Option<&str>, levels: Option<Vec<u8>>, pick: Pick| Rule {
        feature: FeatureKey::new(feature),
        table: table.map(str::to_owned),
        cell: None,
        levels,
        pick,
    };
    // A table's own pick first: [`Picks::pick`] takes the first rule that matches.
    picks.rule(rule(
        REVELATION,
        Some(DUAL_CURSED),
        None,
        Pick::Selection(FeatureSelectionKey::new(DUAL_CURSED_REVELATION)),
    ));
    for (table, pick) in ORACLE_TABLES {
        picks.rule(rule(
            MYSTERY,
            Some(table),
            Some(vec![1]),
            Pick::Selection(FeatureSelectionKey::new(pick)),
        ));
    }
    picks.rule(rule(
        "oracle/lone-strider/mystery-lone-strider",
        Some("oracle/lone-strider"),
        Some(vec![1]),
        Pick::Selection(FeatureSelectionKey::new("oracle/lone-strider/mystery")),
    ));
    // The shaman's Air Barrier, Vortex Spells and Wind Ward resolve their "Wind" line to this feature, so it stays
    // defined (not on the row) until that's settled.
    picks.rule(rule(
        "oracle/wind-whisperer/wind",
        Some("oracle/wind-whisperer"),
        Some(vec![1]),
        Pick::SelectionKeepingFeature(FeatureSelectionKey::new("oracle/wind-whisperer/mystery")),
    ));
    picks.rule(rule(
        REVELATION,
        None,
        None,
        Pick::Selection(FeatureSelectionKey::new(REVELATION)),
    ));
    for final_revelation in [
        "oracle/final-revelation",
        "oracle/wind-whisperer/wind-final-revelation",
    ] {
        picks.rule(rule(
            final_revelation,
            None,
            Some(vec![20]),
            Pick::OptionRow,
        ));
    }
}

/// Every oracle and archetype holds one mystery from level 1, of its table's selection.
fn check_mysteries(data: &GameData) {
    let oracle = data
        .classes
        .iter()
        .find(|class| class.key.as_str() == "oracle")
        .expect("the oracle");
    let archetypes: BTreeSet<&str> = oracle.archetypes.iter().map(|a| a.key.as_str()).collect();
    assert_eq!(
        archetypes,
        ORACLE_TABLES[1..].iter().map(|(table, _)| *table).collect(),
        "the oracle's archetypes"
    );
    let base = oracle
        .levels
        .iter()
        .find(|row| row.level == 1)
        .expect("row 1");
    for (table, pick) in ORACLE_TABLES {
        let own = oracle
            .archetypes
            .iter()
            .find(|a| a.key.as_str() == table)
            .and_then(|a| a.levels.iter().find(|row| row.level == 1));
        let selections = own.map_or(&base.selections, |row| &row.selections);
        let picks: Vec<&str> = selections
            .iter()
            .map(FeatureSelectionKey::as_str)
            .filter(|s| s.ends_with("/mystery"))
            .collect();
        assert_eq!(picks, [pick], "{table}: mystery picks at 1");
    }
    let revelation_levels = |rows: Vec<(u8, &Vec<FeatureSelectionKey>)>, key: &str| -> Vec<u8> {
        rows.into_iter()
            .filter(|(_, selections)| selections.iter().any(|s| s.as_str() == key))
            .map(|(level, _)| level)
            .collect()
    };
    let base_rows: Vec<(u8, &Vec<FeatureSelectionKey>)> = oracle
        .levels
        .iter()
        .map(|row| (row.level, &row.selections))
        .collect();
    assert_eq!(
        revelation_levels(base_rows, REVELATION),
        [1, 3, 7, 11, 15, 19],
        "the oracle's revelations"
    );
    let dual = oracle
        .archetypes
        .iter()
        .find(|a| a.key.as_str() == DUAL_CURSED)
        .expect("the Dual-Cursed Oracle");
    let dual_rows: Vec<(u8, &Vec<FeatureSelectionKey>)> = dual
        .levels
        .iter()
        .map(|row| (row.level, &row.selections))
        .collect();
    assert_eq!(
        revelation_levels(dual_rows, DUAL_CURSED_REVELATION),
        [1, 3, 5, 7, 11, 13, 15, 19],
        "the Dual-Cursed's revelations"
    );
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

const BLESSINGS: &str = "warpriest/blessings";
const FERAL_BLESSING: &str = "warpriest/feral-champion/blessing";
const CHOSEN_BLESSING: &str = "warpriest/champion-of-the-faith/chosen-alignment";

/// The warpriest's blessings, built with the class: each blessing's minor power is its row 1 and its major power its
/// row 10, so the table's level-10 "Blessings" picks are the held blessings' rows. The Feral Champion's "Feral
/// Blessing" cells are its fixed blessing's pick at 1 and that blessing's row at 10; the Champion of the Faith also
/// picks the blessing of his chosen alignment at 1.
fn blessings(guide: &Guide, picks: &mut Picks) {
    let warpriest_page = page(guide, "warpriest");
    let within = section(warpriest_page, "blessings");
    contains(
        &within.text(),
        "Each Blessing grants a minor power at 1st level and a major power at 10th level",
        "warpriest#blessings",
    );
    picks.option_levels(BLESSINGS, &["warpriest"]);
    for heading in &within.children {
        let key = format!("{BLESSINGS}/{}", heading.anchor);
        let text = heading.text();
        contains(&text, "At 1st level", &heading.anchor);
        contains(&text, "At 10th level", &heading.anchor);
        let short = heading.name.trim_end_matches(" Domain").to_owned();
        let mut rows = Vec::new();
        for (level, power) in [(1u8, "Minor"), (10, "Major")] {
            let feature_key = FeatureKey::new(format!("{key}/{}-blessing", power.to_lowercase()));
            picks.feature(
                "warpriest",
                feature(
                    &feature_key,
                    &format!("{power} Blessing ({short})"),
                    warpriest_page.guide(&heading.anchor),
                ),
            );
            rows.push(OptionLevelRow {
                features: vec![feature_key],
                ..empty_row(level)
            });
        }
        picks.option_rows(&key, rows);
    }
    picks.rule(Rule {
        feature: FeatureKey::new(BLESSINGS),
        table: None,
        cell: None,
        levels: Some(vec![10]),
        pick: Pick::OptionRow,
    });

    // "A Feral Champion must take the Animal Domain blessing and does not gain a second." "The Warpriest
    // automatically gains the Blessing for his chosen alignment" (Champion of the Faith).
    picks.selection(
        "warpriest/feral-champion",
        sharing(
            FERAL_BLESSING,
            "Blessing (Feral Champion)",
            "warpriest#feral-blessing".to_owned(),
            &["warpriest"],
            BTreeSet::from([SelectionOptionKey::new(format!(
                "{BLESSINGS}/animal-domain"
            ))]),
        ),
    );
    for (level, pick) in [
        (1, Pick::Selection(FeatureSelectionKey::new(FERAL_BLESSING))),
        (10, Pick::OptionRow),
    ] {
        picks.rule(Rule {
            feature: FeatureKey::new("warpriest/feral-champion/feral-blessing"),
            table: Some("warpriest/feral-champion".to_owned()),
            cell: None,
            levels: Some(vec![level]),
            pick,
        });
    }
    picks.selection(
        "warpriest/champion-of-the-faith",
        sharing(
            CHOSEN_BLESSING,
            "Chosen Alignment Blessing",
            "warpriest#chosen-alignment".to_owned(),
            &["warpriest"],
            ["chaos-domain", "evil-domain", "good-domain", "law-domain"]
                .iter()
                .map(|b| SelectionOptionKey::new(format!("{BLESSINGS}/{b}")))
                .collect(),
        ),
    );
    picks.row_selection("warpriest/champion-of-the-faith", 1, CHOSEN_BLESSING);
}

/// Every blessing has its two rows, and the rows in force pick no blessing at 10: the base class's, and every
/// archetype's own.
fn check_blessings(guide: &Guide, data: &GameData) {
    let within = section(page(guide, "warpriest"), "blessings");
    let warpriest = data
        .classes
        .iter()
        .find(|class| class.key.as_str() == "warpriest")
        .expect("the warpriest");
    let blessings = warpriest
        .selections
        .iter()
        .find(|s| s.key.as_str() == BLESSINGS)
        .expect("the blessings");
    assert_eq!(blessings.options.len(), within.children.len());
    for (option, heading) in blessings.options.iter().zip(&within.children) {
        assert_eq!(
            option.guide.0,
            page(guide, "warpriest").guide(&heading.anchor)
        );
        assert_eq!(
            option
                .levels
                .iter()
                .map(|row| row.level)
                .collect::<Vec<_>>(),
            [1, 10],
            "{}",
            option.key
        );
    }
    let rows = warpriest
        .levels
        .iter()
        .map(|row| (row.level, &row.selections))
        .chain(
            warpriest
                .archetypes
                .iter()
                .flat_map(|a| a.levels.iter().map(|row| (row.level, &row.selections))),
        );
    for (level, selections) in rows {
        assert!(
            level != 10 || !selections.iter().any(|s| s.as_str() == BLESSINGS),
            "a blessing pick at 10"
        );
    }
}

// ---- kineticist elements ---------------------------------------------------------------------------------------

/// The guide's spellings that aren't a feat's name: "Lighting Reflexes".
pub fn element_feat(name: &str) -> String {
    name.replace("Lighting Reflexes", "Lightning Reflexes")
}

/// The kineticist's tables: the class's and each archetype's, each with an "Elemental Focus" choice at 1.
const KINETICIST_TABLES: [&str; 8] = [
    "kineticist",
    "kineticist/blood-kineticist",
    "kineticist/dark-elementalist",
    "kineticist/elemental-engine",
    "kineticist/kinetic-knight",
    "kineticist/kinetic-sharpshooter",
    "kineticist/overwhelming-soul",
    "kineticist/psychokineticist",
];
const ELEMENT: &str = "kineticist/element";
const EXPANDED_ELEMENT: &str = "kineticist/expanded-element";
const BLOOD_ELEMENT: &str = "kineticist/blood-kineticist/element";
const KNIGHT_ELEMENT: &str = "kineticist/kinetic-knight/element";
const KNIGHT_TALENT: &str = "kineticist/kinetic-knight/defensive-wild-talent";

/// The kineticist's elements, built with the class: the selection and its options' rows, the features and lists the
/// rows name, Expanded Element and the archetypes' own selections. Each table's "Class Skills" cell at 1 is the
/// table's element pick, its "Defensive Wild Talent" at 2 the element's row, and "Expanded Element" at 7 and 15 a
/// pick of another element.
fn elements(guide: &Guide, skills: &Skills, feats: &FeatNames, picks: &mut Picks) {
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
    // The 15th-level element isn't the 7th-level one (ticket 33).
    contains(
        &expanded,
        "the one she chose at 1st level, but only if she didn't choose it at 7th",
        "kineticist#expanded-element",
    );
    let mut family = Family {
        selection: selection(
            ELEMENT,
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
        let key = format!("{ELEMENT}/{short}");
        let line = |label: &str| -> String {
            element
                .list_items()
                .find_map(|line| line.strip_prefix(label))
                .unwrap_or_else(|| panic!("{}: no {label:?}", element.anchor))
                .trim()
                .trim_end_matches('.')
                .to_owned()
        };
        // The element's class skills are its row 1's while it's held (ticket 37).
        let mut first = empty_row(1);
        for name in name_list(&line("Additional class skills:")) {
            first.class_skills.insert(skills.get(&name).0);
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

    for feature in &family.features {
        picks.feature("kineticist", feature.clone());
    }
    for list in &family.lists {
        picks.feat_list("kineticist", list.clone());
    }
    // The guide says only that the new element brings its blasts: like Crossblooded's second bloodline, the pick
    // holds the element and gives its feat list, not its class skills or defensive talent.
    picks.selection(
        "kineticist",
        FeatureSelection {
            feat_lists_only: true,
            once_per_option: true,
            ..sharing(
                EXPANDED_ELEMENT,
                "Expanded Element",
                kineticist_page.guide("expanded-element"),
                &["kineticist"],
                family.all_keys(),
            )
        },
    );
    // "Must select Water at 1st and 7th levels" (Blood Kineticist: Water granted with no pick; its 7th level is
    // Blood Blast). The Kinetic Knight's Defensive Wild Talent comes at 4th, not 2nd: its own selection skips row 2
    // and its 4th-level feature grants that row.
    picks.selection(
        "kineticist/blood-kineticist",
        sharing(
            BLOOD_ELEMENT,
            "Elemental Focus (Blood Kineticist)",
            "kineticist#blood-kineticist".to_owned(),
            &["kineticist"],
            family.keys(&["water"]),
        ),
    );
    picks.selection(
        "kineticist/kinetic-knight",
        FeatureSelection {
            skipped_levels: BTreeSet::from([2]),
            ..sharing(
                KNIGHT_ELEMENT,
                "Elemental Focus (Kinetic Knight)",
                "kineticist#kinetic-knight".to_owned(),
                &["kineticist"],
                family.all_keys(),
            )
        },
    );
    picks.rule(Rule {
        feature: FeatureKey::new(KNIGHT_TALENT),
        table: Some("kineticist/kinetic-knight".to_owned()),
        cell: None,
        levels: Some(vec![4]),
        pick: Pick::Feature(Feature {
            option_row: Some(OptionRow {
                selection: FeatureSelectionKey::new(KNIGHT_ELEMENT),
                level: 2,
            }),
            ..feature(
                &FeatureKey::new(KNIGHT_TALENT),
                "Defensive Wild Talent",
                "kineticist#kinetic-knight".to_owned(),
            )
        }),
    });
    for table in KINETICIST_TABLES {
        let pick = match table {
            "kineticist/blood-kineticist" => BLOOD_ELEMENT,
            "kineticist/kinetic-knight" => KNIGHT_ELEMENT,
            _ => ELEMENT,
        };
        picks.rule(Rule {
            feature: FeatureKey::new(format!("{table}/class-skills")),
            table: Some(table.to_owned()),
            cell: None,
            levels: Some(vec![1]),
            pick: Pick::Selection(FeatureSelectionKey::new(pick)),
        });
        picks.rule(Rule {
            feature: FeatureKey::new(format!("{table}/defensive-wild-talent")),
            table: Some(table.to_owned()),
            cell: None,
            levels: Some(vec![2]),
            pick: Pick::OptionRow,
        });
    }
    picks.rule(Rule {
        feature: FeatureKey::new(EXPANDED_ELEMENT),
        table: None,
        cell: None,
        levels: Some(vec![7, 15]),
        pick: Pick::Selection(FeatureSelectionKey::new(EXPANDED_ELEMENT)),
    });
    picks.selection("kineticist", family.selection);
}

/// The kineticist's rows in force hold the element picks where the class's and the archetypes' tables have the
/// cells: every table's choice at 1, Expanded Element at 7 and 15 (the Blood Kineticist's 7th is Blood Blast), and
/// the Kinetic Knight's defensive talent at 4 only.
fn check_elements(data: &GameData) {
    let kineticist = data
        .classes
        .iter()
        .find(|class| class.key.as_str() == "kineticist")
        .expect("the kineticist");
    let archetypes: BTreeSet<&str> = kineticist
        .archetypes
        .iter()
        .map(|a| a.key.as_str())
        .collect();
    assert_eq!(
        archetypes,
        KINETICIST_TABLES[1..].iter().copied().collect(),
        "the kineticist's archetypes"
    );
    let in_force = |archetype: Option<&str>, level: u8| -> (Vec<String>, Vec<String>) {
        let own = archetype.and_then(|key| {
            kineticist
                .archetypes
                .iter()
                .find(|a| a.key.as_str() == key)
                .and_then(|a| a.levels.iter().find(|row| row.level == level))
        });
        let (features, selections) = match own {
            Some(row) => (&row.features, &row.selections),
            None => {
                let row = kineticist
                    .levels
                    .iter()
                    .find(|row| row.level == level)
                    .expect("a kineticist row");
                (&row.features, &row.selections)
            }
        };
        (
            features.iter().map(|f| f.to_string()).collect(),
            selections.iter().map(|s| s.to_string()).collect(),
        )
    };
    for table in KINETICIST_TABLES {
        let archetype = (table != "kineticist").then_some(table);
        let pick = match table {
            "kineticist/blood-kineticist" => BLOOD_ELEMENT,
            "kineticist/kinetic-knight" => KNIGHT_ELEMENT,
            _ => ELEMENT,
        };
        assert!(
            in_force(archetype, 1).1.iter().any(|s| s == pick),
            "{table}: the element pick at 1"
        );
        let expanded_at: &[u8] = if table == "kineticist/blood-kineticist" {
            &[15]
        } else {
            &[7, 15]
        };
        for &level in expanded_at {
            assert!(
                in_force(archetype, level)
                    .1
                    .iter()
                    .any(|s| s == EXPANDED_ELEMENT),
                "{table}: Expanded Element at {level}"
            );
        }
        for level in 1..=20 {
            assert!(
                level == 4 && table == "kineticist/kinetic-knight"
                    || !in_force(archetype, level)
                        .0
                        .iter()
                        .any(|f| f == KNIGHT_TALENT),
                "the Kinetic Knight's talent at 4 only"
            );
        }
    }
}

// ---- Hellknight ------------------------------------------------------------------------------------------------

/// "If the character was not previously an Armiger, they must choose one of the seven Hellknight orders. Otherwise,
/// they retain their current order." (the Armiger's seven, for a character holding none), and Pentamic Faith: "The
/// Hellknight gains access to one of the following domains … She uses her class level to determine which domain
/// abilities she can use." Built with the class: the "Order" cell at 1 is the pick, and the disciplines are levelled
/// by Hellknight levels, Pentamic Faith's row 1 granting its domain.
fn hellknight(guide: &Guide, domains: &Family, picks: &mut Picks) {
    let armiger_orders = section(page(guide, "fighter"), "order");
    let orders: BTreeSet<SelectionOptionKey> = armiger_orders
        .children
        .iter()
        .map(|order| {
            assert!(
                order.anchor.starts_with("order-of-the-"),
                "{}",
                order.anchor
            );
            SelectionOptionKey::new(format!("{ARMIGER_ORDER}/{}", order.anchor))
        })
        .collect();
    assert_eq!(orders.len(), 7);
    picks.selection(
        "hellknight",
        FeatureSelection {
            only_without_option: true,
            ..sharing(
                HELLKNIGHT_ORDER,
                "Order",
                "prestige-classes#order".to_owned(),
                &[],
                orders.clone(),
            )
        },
    );
    // The Signifer's "Order" cell at 1: "The Signifer must choose one of the seven Hellknight orders" (ticket 33),
    // offered, as the Hellknight's, only to a character holding none ("Otherwise, they retain their current order",
    // ticket 35).
    let prestige = page(guide, "prestige-classes");
    contains(
        &section(prestige, "order-hellknight-signifer").text(),
        "The Signifer must choose one of the seven Hellknight orders.",
        "prestige-classes#order-hellknight-signifer",
    );
    contains(
        &section(prestige, "order").text(),
        "Otherwise, they retain their current order.",
        "prestige-classes#order",
    );
    picks.selection(
        "hellknight-signifer",
        FeatureSelection {
            only_without_option: true,
            ..sharing(
                SIGNIFER_ORDER,
                "Order",
                "prestige-classes#order-hellknight-signifer".to_owned(),
                &[],
                orders,
            )
        },
    );
    picks.rule(Rule {
        feature: FeatureKey::new(SIGNIFER_ORDER),
        table: Some("hellknight-signifer".to_owned()),
        cell: None,
        levels: Some(vec![1]),
        pick: Pick::Selection(FeatureSelectionKey::new(SIGNIFER_ORDER)),
    });
    let faith = "hellknight/pentamic-faith";
    picks.selection(
        "hellknight",
        sharing(
            faith,
            "Pentamic Faith",
            "prestige-classes#pentamic-faith".to_owned(),
            &["hellknight"],
            domains.keys(&PENTAMIC_DOMAINS),
        ),
    );
    picks.option_levels("hellknight/disciplines", &["hellknight"]);
    picks.option_rows(
        PENTAMIC_DISCIPLINE,
        vec![OptionLevelRow {
            selections: vec![FeatureSelectionKey::new(faith)],
            ..empty_row(1)
        }],
    );
    picks.rule(Rule {
        feature: FeatureKey::new(HELLKNIGHT_ORDER),
        table: Some("hellknight".to_owned()),
        cell: None,
        levels: Some(vec![1]),
        pick: Pick::Selection(FeatureSelectionKey::new(HELLKNIGHT_ORDER)),
    });
}

const ARMIGER_ORDER: &str = "fighter/armiger/order";
const HELLKNIGHT_ORDER: &str = "hellknight/order";
const SIGNIFER_ORDER: &str = "hellknight-signifer/order";

/// The Hellknight's order shares exactly the Armiger's orders and is its row 1's pick, as the Signifer's is its own;
/// Pentamic Faith's option has its row.
fn check_hellknight(data: &GameData) {
    let armiger: BTreeSet<&SelectionOptionKey> = data
        .classes
        .iter()
        .flat_map(|c| &c.archetypes)
        .flat_map(|a| &a.selections)
        .find(|s| s.key.as_str() == ARMIGER_ORDER)
        .expect("the Armiger's orders")
        .options
        .iter()
        .map(|o| &o.key)
        .collect();
    let hellknight = data
        .prestige_classes
        .iter()
        .find(|c| c.key.as_str() == "hellknight")
        .expect("the Hellknight");
    let order = hellknight
        .selections
        .iter()
        .find(|s| s.key.as_str() == HELLKNIGHT_ORDER)
        .expect("the Hellknight's order");
    assert_eq!(
        order.shared_options.iter().collect::<BTreeSet<_>>(),
        armiger
    );
    let signifer = data
        .prestige_classes
        .iter()
        .find(|c| c.key.as_str() == "hellknight-signifer")
        .expect("the Signifer");
    assert!(
        signifer.levels.iter().any(
            |row| row.level == 1 && row.selections.iter().any(|s| s.as_str() == SIGNIFER_ORDER)
        ),
        "the Signifer's Order cell"
    );
    let first = hellknight
        .levels
        .iter()
        .find(|row| row.level == 1)
        .expect("row 1");
    assert_eq!(
        first
            .selections
            .iter()
            .filter(|s| s.as_str() == HELLKNIGHT_ORDER)
            .count(),
        1,
        "the Hellknight's Order cell"
    );
    let disciplines = hellknight
        .selections
        .iter()
        .find(|s| s.key.as_str() == "hellknight/disciplines")
        .expect("the disciplines");
    assert!(
        disciplines.options.iter().any(|o| o.key.as_str()
            == "hellknight/disciplines/pentamic-faith"
            && !o.levels.is_empty()),
        "Pentamic Faith's row"
    );
}
