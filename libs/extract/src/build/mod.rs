//! Building [`GameData`] from the parsed guide pages.
//!
//! Generic rules do most of the work: anchors give keys, progression tables give level rows, `Prerequisites:`
//! paragraphs give prerequisites, a heading whose name a table grants and whose children it doesn't is a Feature
//! selection. A cell the game makes a pick is the pick's slot from the start, as [`crate::picks`]' providers say. The
//! few rules the guide states only in prose are in [`crate::prose`]. Anything that can't be placed goes to the
//! [`Report`].

use std::collections::{BTreeMap, BTreeSet};

use rules::{
    ArchetypeKey, Class, ClassKey, DataAt, FeatKey, FeatSlot, Feature, FeatureKey,
    FeatureSelectionKey, GameData, MythicAscension, MythicPathKey, Prerequisite, Progression,
};

use crate::feat_lists::{self, PendingLists};
use crate::grammar::{Cond, Unparsed};
use crate::guide::{BASE_CLASSES, Guide};
use crate::html::{Page, Section};
use crate::option_levels::FeatNames;
use crate::picks::Picks;
use crate::progressions;
use crate::prose;
use crate::report::{Kind, Report};
use crate::resolve::{Catalog, Context};

mod base_classes;
mod feats;
mod grants;
mod level_tables;
mod mythic;
mod prestige;
mod resolving;
mod sections;
mod selections;
mod skills_and_races;

pub use mythic::MYTHIC_ASCENSIONS;
pub(crate) use sections::class_tables;
pub use sections::section_text;

/// What the extractor produced.
pub struct Extraction {
    pub data: GameData,
    pub report: Report,
}

/// Builds the game data from the guide.
///
/// First each owner is built where its pages are read: skills, feats, then (once [`crate::picks`]' providers are
/// read) races, the classes and their archetypes (with the picks their tables' cells are, and ticket 30's option
/// families), the prestige classes, the mythic abilities, feats and paths. Then a postlude, in order: each step needs
/// what the steps before it built, as its comment says.
pub fn extract(guide: &Guide) -> Extraction {
    let mut builder = Builder::new(guide);
    let progression = Progression {
        mythic_ascensions: MYTHIC_ASCENSIONS
            .iter()
            .map(|&(rank, character_level)| MythicAscension {
                rank,
                character_level,
            })
            .collect(),
    };
    let skills = builder.skills();
    let mut feats = builder.feats();
    // The providers name feats and skills by the guide's names (the bonus-feat choices' lists, the elements' class
    // skills), so they're read after the feats, and before the races (the half-elf's Adaptability).
    builder.picks = Picks::read(guide, &FeatNames::of(&feats), &skills);
    // A feat's conditions stated beside its lines (Armor Focus's armour proficiency), queued as its lines are.
    for feat in &feats {
        let at = DataAt::Feat(feat.key.clone());
        let stated = builder.picks.stated_for(&at);
        if !stated.is_empty() {
            builder.add_prerequisite(&at, feat.guide.0.clone(), stated);
        }
    }
    let races = builder.races();
    let mut classes: Vec<Class> = BASE_CLASSES
        .iter()
        .map(|slug| builder.base_class(slug))
        .collect();
    let mut prestige_classes = builder.prestige_classes();
    let mythic_abilities = builder.mythic_abilities();
    let mythic_feats = builder.mythic_feats();
    let mut mythic_paths = builder.mythic_paths();
    feats.extend(builder.trickster_feats());

    // The prose rules that join what several owners' pages define (lists folded into another class's selection, the
    // bonus slots and gates the tables can't say, the Trickster's tiers) need every owner built.
    prose::apply(
        &mut builder,
        &mut classes,
        &mut prestige_classes,
        &mut mythic_paths,
    );
    // A feature is defined where a table first names it; each goes to its owner once every table is read.
    builder.place_features(&mut classes, &mut prestige_classes, &mut mythic_paths);
    prose::path_rules(&mut mythic_paths);

    // The weapons the guide's proficiency lists name, with the user's decisions on them.
    let mut weapons = crate::parameters::weapons(guide);
    crate::follow_ups::weapons(guide, &mut weapons);
    let mut data = GameData {
        progression,
        races,
        skills,
        weapons,
        feats,
        classes,
        prestige_classes,
        mythic_abilities,
        mythic_feats,
        mythic_paths,
        backgrounds: Vec::new(),
    };
    // A prerequisite line names what any page defines: resolved once every name is known.
    builder.resolve_prerequisites(&mut data);
    // A cell or a line can name what no page defines: dropped, and reported, once every key is known.
    crate::references::drop_dangling(&mut data, &mut builder.lists, &mut builder.report);
    // An option granting a feat beside that feat in an `Any` adds nothing: needs every option's feats.
    crate::references::drop_redundant_alternatives(&mut data);
    // A list of one feat is a granted feat, known once the dangling feats are gone from the lists.
    feat_lists::grant_lone_feats(&mut data, &builder.lists);
    // A selection that draws on another class's list shares its options; the prerequisites the shared options name
    // are the resolved ones, and the rows it takes over are the full ones.
    crate::shared_options::apply(&mut data);
    // Parameterised and repeatable feats and options, with ticket 32's repeats and mythic values: their
    // prerequisites name values of resolved feats.
    crate::parameters::apply(&mut data);
    // Only now, with the dangling grants gone and the grants complete, is a row that matches its base class's
    // known to.
    for class in &mut data.classes {
        progressions::sparse_archetype_rows(class);
    }
    // And only with the rows sparse is it known which archetype alone uses a list.
    feat_lists::name_lists(&mut data, &builder.lists);
    // The one-feat lists of the bonus feats of the player's choosing aren't the guide's tables' lists.
    crate::parameters::add_lists(&mut data);
    // The user's backgrounds: the guide doesn't list them; each names skills and feats of the finished data.
    data.backgrounds = crate::backgrounds::backgrounds(&data);
    // The feats that give a pick of the selections their "Prerequisites" names: needs every class's, archetype's and
    // path's selection, the option families' included.
    crate::extra_picks::apply(guide, &mut data);
    // What the borrowed picks widen of other owners: the resolved prerequisites of the advanced talents, the hexes
    // and the Extra feats, and the option levels the class stage gave the rogue's and the slayer's talents.
    crate::borrowed_selections::apply(guide, &mut data);
    // Extra Discovery and Extra Arcana reach the Vivisectionist and the Hexcrafter from their first slot: needs the
    // rows in force and ticket 49's extra picks.
    crate::borrowed_extra_picks::apply(guide, &mut data);

    // What the providers can only assert on the finished data, and the report items the finished data settles.
    crate::picks::check(guide, &data);
    let settled = settled();
    let dead = builder.report.settle(&settled);
    assert!(
        dead.is_empty(),
        "report filter entries that match no report item: {dead:?}"
    );
    for problem in data.check() {
        builder.report.add_about(
            Kind::Cell,
            &problem.at,
            "",
            format!("fails the data check: {problem}"),
        );
    }
    Extraction {
        data: data.canonical(),
        report: builder.report,
    }
}

/// The report items the finished data settles: an option family's table, which the family's provider reads (ticket
/// 30), Armor Focus's line (ticket 32), a gap the saved pages fill (tickets 43 and 44), the bonus-feat wild talent
/// (ticket 45). Each entry must settle at least one item.
fn settled() -> Vec<crate::report::Settled> {
    [
        crate::option_levels::settles(),
        crate::follow_ups::settles(),
        crate::report_gaps::settles(),
        crate::wild_talents::settles(),
    ]
    .concat()
}

/// Prerequisite lines waiting for the whole catalog.
struct Pending {
    /// The entity the prerequisite belongs to. Keys carry no type prefix, so the kind is part of the owner.
    owner: DataAt,
    at: String,
    conds: Vec<(String, Result<Cond, Unparsed>)>,
    context: Context,
    kind: Kind,
    /// Conditions already known, ANDed with the parsed ones.
    extra: Vec<Prerequisite>,
}

/// Where a feature is defined.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeatureOwner {
    Class(ClassKey),
    Archetype(ClassKey, ArchetypeKey),
    Path(MythicPathKey),
}

impl FeatureOwner {
    /// The key of the class, archetype or path.
    pub fn key(&self) -> &str {
        match self {
            Self::Class(class) => class.as_str(),
            Self::Archetype(_, archetype) => archetype.as_str(),
            Self::Path(path) => path.as_str(),
        }
    }
}

/// A heading a table cell can name.
#[derive(Clone)]
struct Heading<'g> {
    section: &'g Section,
    page: &'g Page,
    /// The key prefix of what it defines: `rogue`, `rogue/dark-lurker`, `trickster`.
    prefix: String,
    owner: FeatureOwner,
}

/// The headings and selections a class, archetype or path table resolves its cells against, nearest first.
#[derive(Clone, Default)]
pub struct Scope<'g> {
    headings: Vec<Heading<'g>>,
    selections: Vec<(Vec<String>, FeatureSelectionKey)>,
    /// The owner of features with no heading of their own.
    prefix: String,
    owner: Option<FeatureOwner>,
    guide: String,
    /// The archetype's anchor, for headings disambiguated with it ("Sneak Attack (Eldritch Scoundrel)").
    archetype_anchor: Option<String>,
    /// A mythic path's table: a feat it names is granted as a feature, since only classes have bonus lists.
    path: bool,
}

/// What a cell grants.
enum Granted {
    Feature(FeatureKey),
    Selection(FeatureSelectionKey),
    Slots(Vec<FeatSlot>),
    /// A feat the class grants outright.
    Feat(FeatKey),
    /// Nothing on the row: the cell is the held option's row.
    Nothing,
    /// [`Pick::SelectionKeepingFeature`](crate::picks::Pick::SelectionKeepingFeature), before [`Builder::grant`] defines the cell's feature: never on a row.
    SelectionKeepingFeature(FeatureSelectionKey),
    /// Feats granted outright, with the cell's feature when it stays ([`Pick::Feats`](crate::picks::Pick::Feats)).
    Feats(BTreeSet<FeatKey>, Option<FeatureKey>),
    /// [`Pick::FeatureAndFeats`](crate::picks::Pick::FeatureAndFeats), before [`Builder::grant`] defines the cell's feature: never on a row.
    FeatureAndFeats(BTreeSet<FeatKey>),
    /// Feats granted outright and for one value ([`Pick::FeatsFor`](crate::picks::Pick::FeatsFor)).
    FeatsFor(BTreeSet<FeatKey>, BTreeSet<rules::FeatValue>),
}

pub struct Builder<'g> {
    pub guide: &'g Guide,
    pub report: Report,
    pub catalog: Catalog,
    pending: Vec<Pending>,
    /// Every feature, where it's defined, and its definition.
    pub features: BTreeMap<FeatureKey, (FeatureOwner, Feature)>,
    /// Feat names, for bonus lists and granted feats.
    feat_names: Vec<(String, FeatKey)>,
    /// The fixed feat lists of bonus slots, named once the data is complete.
    pub lists: PendingLists,
    /// The cells that are picks, and what comes with them.
    picks: Picks,
    /// How each table of the class being built takes its option family (by the table's key), planned before its
    /// rows are read.
    plans: BTreeMap<String, crate::option_levels::TablePlan>,
}

impl<'g> Builder<'g> {
    fn new(guide: &'g Guide) -> Self {
        Self {
            guide,
            report: Report::default(),
            catalog: Catalog::default(),
            pending: Vec::new(),
            features: BTreeMap::new(),
            feat_names: Vec::new(),
            lists: PendingLists::default(),
            picks: Picks::default(),
            plans: BTreeMap::new(),
        }
    }
}
