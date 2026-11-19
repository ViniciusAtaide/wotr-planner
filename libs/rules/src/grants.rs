//! The grant index (ticket 334, design-G §1, its static half): every place the game data gives facts, by the entry
//! that gives them and by the fact. One answer to "what gives this fact" and "what does this entry give", which the
//! engine's catalog, the data check, game-check's carrier guard and the real-data tests read.
//!
//! It says what an entry **can** give, read off the data alone: no Build, no walk. When a grant applies (an option's
//! level, an archetype's replaced row, a switch's drops, a component's class level) stays the walk's question; a
//! [`Site`]'s [`When`] only says which rule it applies by.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::check::DataAt;
use crate::component::{Component, Fact};
use crate::game_data::{
    EXOTIC_WEAPON_PROFICIENCY, FeatValue, GameData, IMPROVED_UNARMED_STRIKE, UNARMED_STRIKE,
    WeaponProficiency,
};
use crate::keys::{ClassKey, FeatKey, FeatureKey, FeatureSelectionKey, WeaponKey};
use crate::parameter::Parameter;

/// Every grant in the game data, by granting entry and by fact. Built once ([`GrantIndex::new`]), independent of any
/// Build, with no IO, so the browser builds it too. It copies no key but each site's entry and grants' facts: both
/// lookups halve sorted lists.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GrantIndex {
    /// Every site, sorted by its entry, its [`When`] and its grants, so the data's order changes nothing; an entry's
    /// sites are a run of it.
    sites: Vec<Site>,
    /// Every grant, as its (site, grant) position, sorted by its fact, then by its position.
    by_fact: Vec<(u32, u16)>,
    /// The selections whose lists offer each fact as a pick, in key order: only an index built
    /// [`with_offers`](GrantIndex::with_offers) has them (game-check's; the engine reads none).
    offered_in: Option<FxHashMap<Fact, Vec<FeatureSelectionKey>>>,
}

/// One place the data gives facts together: a row at a level, a component, an entry's own fields (game-check's
/// carrier guard's "site").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Site {
    /// The granting entry.
    pub by: DataAt,
    /// A class row's class, an archetype row's base class; `None` for every other site.
    pub class: Option<ClassKey>,
    pub when: When,
    /// What it gives, sorted (a row's lists are multisets: one named twice is given twice). Never empty.
    pub grants: Vec<Grant>,
}

/// The rule a site's grants apply by. The walk applies it; the index only names it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum When {
    /// While the entry is held: an option's feats and chosen feat, a feat's or mythic feat's selection, a mythic
    /// ability's features and selection, a race's or subrace's selections, a background's feats, a deity's favoured
    /// weapons' feats, the features a switch to a path grants.
    Always,
    /// A class's or archetype's level row.
    ClassRow { level: u8 },
    /// An option's level row (the option level), or the feature a selection gives in place of a skipped one
    /// ([`FeatureSelection::replaced_by`](crate::FeatureSelection::replaced_by)).
    OptionRow { level: u8 },
    /// A path's rank row.
    Rank { rank: u8 },
    /// A grant component, by its own rule.
    Component(GrantRule),
}

/// A grant component's rule ([`Component`]).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GrantRule {
    /// While the entry is held: [`Component::Grant`], [`Component::GrantWithValue`].
    Held,
    /// From a class level on, or below it: [`Component::GrantAtClassLevel`].
    AtClassLevel { level: u8, before: bool },
    /// From the first moment `has` is held (or, `not`, isn't): [`Component::GrantIf`].
    IfHas { has: Fact, not: bool },
    /// For good, when the entry is first held: [`Component::GrantOnApply`].
    OnApply,
}

/// One fact a site gives.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Grant {
    pub fact: Fact,
    pub value: GrantValue,
    /// Whether it opens a slot: a row's selection, a feat's, mythic feat's or mythic ability's `selection`, a race's
    /// selection. A component's selection opens none (`AddFacts` offers no pick).
    pub opens_slot: bool,
}

/// The value a grant holds its feat for.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GrantValue {
    /// Every value, or none for a fact that takes none: a row's feats, a background's feats, a feature.
    Every,
    /// One value: a row's or a background's `feats_for`, [`Component::GrantWithValue`], an exotic favoured weapon's
    /// proficiency.
    Fixed(Parameter),
    /// The value the entry's pick names: an option's chosen feat, an option row's chosen feats.
    Picked,
}

/// Whether a grant meets a prerequisite's leaf ([`Grant::satisfies`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Satisfies {
    Yes,
    No,
}

impl GrantIndex {
    /// Indexes every grant of `data`, in any order of its lists; never panics on a key it lacks (the data check
    /// builds one before the data is known to be sound).
    pub fn new(data: &GameData) -> Self {
        let mut sites = Sites::new(data);
        sites.read(data);
        let mut sites = sites.out;
        sites
            .sort_unstable_by(|a, b| (&a.by, &a.when, &a.grants).cmp(&(&b.by, &b.when, &b.grants)));
        let mut by_fact: Vec<(u32, u16)> = Vec::new();
        for (s, site) in sites.iter().enumerate() {
            let s = u32::try_from(s).expect("fewer than 4 billion sites");
            let grants =
                0..u16::try_from(site.grants.len()).expect("a site gives fewer than 65,536 facts");
            by_fact.extend(grants.map(|g| (s, g)));
        }
        let fact = |&(s, g): &(u32, u16)| &sites[s as usize].grants[usize::from(g)].fact;
        by_fact.sort_unstable_by(|a, b| fact(a).cmp(fact(b)).then(a.cmp(b)));
        Self {
            sites,
            by_fact,
            offered_in: None,
        }
    }

    /// [`new`](Self::new), with the selections that offer each fact as a pick ([`offered_in`](Self::offered_in)).
    pub fn with_offers(data: &GameData) -> Self {
        Self {
            offered_in: Some(offered_in(data)),
            ..Self::new(data)
        }
    }

    /// What an entry gives, in [`When`] order (rows by level). Empty for an entry that gives nothing or that the data
    /// lacks.
    pub fn gives(&self, by: &DataAt) -> &[Site] {
        let start = self.sites.partition_point(|site| site.by < *by);
        let run = self.sites[start..].partition_point(|site| site.by == *by);
        &self.sites[start..start + run]
    }

    /// Every site that grants `fact`, with the grant, ordered by (`by`, `when`).
    pub fn given_by(&self, fact: &Fact) -> impl Iterator<Item = (&Site, &Grant)> {
        let at = |&(s, g): &(u32, u16)| {
            let site = &self.sites[s as usize];
            (site, &site.grants[usize::from(g)])
        };
        let start = self
            .by_fact
            .partition_point(|position| at(position).1.fact < *fact);
        let run = self.by_fact[start..].partition_point(|position| at(position).1.fact == *fact);
        self.by_fact[start..start + run].iter().map(at)
    }

    /// The selections whose lists offer `fact` as a pick (their own and shared options, their feats and their
    /// mythic abilities), in key order.
    ///
    /// # Panics
    ///
    /// On an index not built [`with_offers`](Self::with_offers).
    pub fn offered_in(&self, fact: &Fact) -> &[FeatureSelectionKey] {
        let offered = (self.offered_in.as_ref()).expect("an index built with_offers");
        offered.get(fact).map_or(&[], Vec::as_slice)
    }

    /// Every site, in (`by`, `when`, grants) order.
    pub fn sites(&self) -> &[Site] {
        &self.sites
    }
}

impl Grant {
    /// Whether this grant meets a prerequisite's leaf on `fact` asking for `value` (`None`: any), `refused` when
    /// the leaf refuses the fact: never a refused leaf; the same fact; a value the leaf asks for that the grant gives
    /// ([`GrantValue::Every`]; [`GrantValue::Fixed`] equal; [`GrantValue::Picked`] with `picked` equal, or no pick
    /// yet: some pick can). A Feature leaf's count isn't read: one grant adds one rank, and "is it met" stays the
    /// walk's question.
    pub fn satisfies(
        &self,
        fact: &Fact,
        value: Option<&Parameter>,
        refused: bool,
        picked: Option<&Parameter>,
    ) -> Satisfies {
        let meets = !refused
            && self.fact == *fact
            && match (value, &self.value) {
                (None, _) | (Some(_), GrantValue::Every) => true,
                (Some(value), GrantValue::Fixed(given)) => given == value,
                (Some(value), GrantValue::Picked) => picked.is_none_or(|picked| picked == value),
            };
        if meets { Satisfies::Yes } else { Satisfies::No }
    }
}

/// The selections whose lists offer each fact as a pick, each list in key order and once.
fn offered_in(data: &GameData) -> FxHashMap<Fact, Vec<FeatureSelectionKey>> {
    let mut out: FxHashMap<Fact, Vec<FeatureSelectionKey>> = FxHashMap::default();
    for selection in &data.selections {
        let picks = (selection.options.iter())
            .chain(&selection.shared_options)
            .cloned()
            .map(Fact::Option)
            .chain(selection.feats.iter().cloned().map(Fact::Feat))
            .chain(
                (selection.mythic_abilities.iter())
                    .cloned()
                    .map(Fact::MythicAbility),
            );
        for fact in picks {
            out.entry(fact).or_default().push(selection.key.clone());
        }
    }
    for selections in out.values_mut() {
        selections.sort_unstable();
        selections.dedup();
    }
    out
}

/// The sites read off the data, in the data's order.
struct Sites<'a> {
    out: Vec<Site>,
    /// The exotic weapons: an exotic favoured weapon's follower holds its proficiency for it.
    exotic: FxHashSet<&'a WeaponKey>,
}

impl<'a> Sites<'a> {
    fn new(data: &'a GameData) -> Self {
        Self {
            out: Vec::new(),
            exotic: (data.weapons.iter())
                .filter(|weapon| weapon.proficiency == Some(WeaponProficiency::Exotic))
                .map(|weapon| &weapon.key)
                .collect(),
        }
    }

    /// A site, when it gives something, its grants sorted (a row's lists are multisets).
    fn push(&mut self, by: &DataAt, class: Option<&ClassKey>, when: When, mut grants: Vec<Grant>) {
        if !grants.is_empty() {
            grants.sort_unstable();
            self.out.push(Site {
                by: by.clone(),
                class: class.cloned(),
                when,
                grants,
            });
        }
    }

    /// One site for each grant component, under its rule.
    fn components(&mut self, by: &DataAt, components: &[Component]) {
        for component in components {
            let (rule, grants) = match component {
                Component::Grant(facts) => {
                    (GrantRule::Held, facts.iter().cloned().map(every).collect())
                }
                Component::GrantAtClassLevel {
                    fact,
                    level,
                    before,
                    ..
                } => (
                    GrantRule::AtClassLevel {
                        level: *level,
                        before: *before,
                    },
                    vec![every(fact.clone())],
                ),
                Component::GrantIf { has, not, fact } => (
                    GrantRule::IfHas {
                        has: has.clone(),
                        not: *not,
                    },
                    vec![every(fact.clone())],
                ),
                Component::GrantOnApply(fact) => (GrantRule::OnApply, vec![every(fact.clone())]),
                Component::GrantWithValue { feat, value } => (
                    GrantRule::Held,
                    vec![fixed(&FeatValue {
                        feat: feat.clone(),
                        value: value.clone(),
                    })],
                ),
                _ => continue,
            };
            self.push(by, None, When::Component(rule), grants);
        }
    }

    /// Every entry's sites.
    fn read(&mut self, data: &GameData) {
        for race in &data.races {
            let by = DataAt::Race(race.key.clone());
            self.push(&by, None, When::Always, slots(&race.selections));
            self.components(&by, &race.components);
        }
        for subrace in &data.subraces {
            let by = DataAt::Subrace(subrace.key.clone());
            self.push(&by, None, When::Always, slots(&subrace.selections));
            self.components(&by, &subrace.components);
        }
        for feat in &data.feats {
            let by = DataAt::Feat(feat.key.clone());
            self.push(&by, None, When::Always, slots(&feat.selection));
            self.components(&by, &feat.components);
        }
        for class in data.all_classes() {
            let by = DataAt::Class(class.key.clone());
            for row in &class.levels {
                let grants = row_grants(&row.features, &row.feats, &row.feats_for, &row.selections);
                self.push(
                    &by,
                    Some(&class.key),
                    When::ClassRow { level: row.level },
                    grants,
                );
            }
            self.components(&by, &class.components);
        }
        for archetype in &data.archetypes {
            let by = DataAt::Archetype(archetype.key.clone());
            for row in &archetype.levels {
                let grants = row_grants(&row.features, &row.feats, &row.feats_for, &row.selections);
                let when = When::ClassRow { level: row.level };
                self.push(&by, Some(&archetype.class), when, grants);
            }
            self.components(&by, &archetype.components);
        }
        for feature in &data.features {
            self.components(&DataAt::Feature(feature.key.clone()), &feature.components);
        }
        for selection in &data.selections {
            let by = DataAt::FeatureSelection(selection.key.clone());
            for (&level, feature) in &selection.replaced_by {
                let grants = vec![every(Fact::Feature(feature.clone()))];
                self.push(&by, None, When::OptionRow { level }, grants);
            }
            self.components(&by, &selection.components);
        }
        for option in &data.options {
            let by = DataAt::SelectionOption(option.key.clone());
            let own = (option
                .feats
                .iter()
                .cloned()
                .map(|feat| every(Fact::Feat(feat))))
            .chain(option.chosen_feat.iter().map(picked))
            .collect();
            self.push(&by, None, When::Always, own);
            for row in &option.levels {
                let grants = features(&row.features)
                    .chain(
                        row.feats
                            .iter()
                            .cloned()
                            .map(|feat| every(Fact::Feat(feat))),
                    )
                    .chain(row.chosen_feats.iter().map(picked))
                    .chain(slots(&row.selections))
                    .collect();
                self.push(&by, None, When::OptionRow { level: row.level }, grants);
            }
            self.components(&by, &option.components);
        }
        for ability in &data.mythic_abilities {
            let by = DataAt::MythicAbility(ability.key.clone());
            let grants = features(&ability.features)
                .chain(slots(&ability.selection))
                .collect();
            self.push(&by, None, When::Always, grants);
            self.components(&by, &ability.components);
        }
        for feat in &data.mythic_feats {
            let by = DataAt::MythicFeat(feat.key.clone());
            self.push(&by, None, When::Always, slots(&feat.selection));
            self.components(&by, &feat.components);
        }
        for path in &data.mythic_paths {
            let by = DataAt::MythicPath(path.key.clone());
            for row in &path.ranks {
                let grants = features(&row.features)
                    .chain(slots(&row.selections))
                    .collect();
                self.push(&by, None, When::Rank { rank: row.rank }, grants);
            }
            if let Some(switch) = &path.switch {
                self.push(
                    &by,
                    None,
                    When::Always,
                    features(&switch.features).collect(),
                );
            }
        }
        for background in &data.backgrounds {
            let by = DataAt::Background(background.key.clone());
            let grants = (background
                .feats
                .iter()
                .cloned()
                .map(|feat| every(Fact::Feat(feat))))
            .chain(background.feats_for.iter().map(fixed))
            .collect();
            self.push(&by, None, When::Always, grants);
            self.components(&by, &background.components);
        }
        for deity in &data.deities {
            let by = DataAt::Deity(deity.key.clone());
            let grants = (deity.favoured_weapons.iter())
                .filter_map(|weapon| self.favoured_weapon_feat(weapon))
                .collect();
            self.push(&by, None, When::Always, grants);
            self.components(&by, &deity.components);
        }
        for item in &data.gear {
            self.components(&DataAt::Gear(item.key.clone()), &item.components);
        }
    }

    /// The feat a follower holds for a favoured weapon while holding a feature that grants them: Improved Unarmed
    /// Strike for the unarmed strike, an exotic weapon's proficiency for that weapon, and none for a simple or martial
    /// weapon (no feat checks proficiency with one weapon).
    fn favoured_weapon_feat(&self, weapon: &WeaponKey) -> Option<Grant> {
        if weapon.as_str() == UNARMED_STRIKE {
            Some(every(Fact::Feat(FeatKey::new(IMPROVED_UNARMED_STRIKE))))
        } else if self.exotic.contains(weapon) {
            Some(fixed(&FeatValue {
                feat: FeatKey::new(EXOTIC_WEAPON_PROFICIENCY),
                value: Parameter::Weapon(weapon.clone()),
            }))
        } else {
            None
        }
    }
}

/// A class's or archetype's row: its features, feats, feats for a value and selections.
fn row_grants<'k>(
    row_features: &'k [FeatureKey],
    feats: impl IntoIterator<Item = &'k FeatKey>,
    feats_for: impl IntoIterator<Item = &'k FeatValue>,
    selections: &'k [FeatureSelectionKey],
) -> Vec<Grant> {
    features(row_features)
        .chain(
            feats
                .into_iter()
                .cloned()
                .map(|feat| every(Fact::Feat(feat))),
        )
        .chain(feats_for.into_iter().map(fixed))
        .chain(slots(selections))
        .collect()
}

fn features<'k>(keys: impl IntoIterator<Item = &'k FeatureKey>) -> impl Iterator<Item = Grant> {
    keys.into_iter()
        .cloned()
        .map(|key| every(Fact::Feature(key)))
}

/// Selections, each the slot it opens.
fn slots<'k>(keys: impl IntoIterator<Item = &'k FeatureSelectionKey>) -> Vec<Grant> {
    keys.into_iter()
        .map(|key| Grant {
            fact: Fact::Selection(key.clone()),
            value: GrantValue::Every,
            opens_slot: true,
        })
        .collect()
}

fn every(fact: Fact) -> Grant {
    Grant {
        fact,
        value: GrantValue::Every,
        opens_slot: false,
    }
}

fn fixed(value: &FeatValue) -> Grant {
    Grant {
        fact: Fact::Feat(value.feat.clone()),
        value: GrantValue::Fixed(value.value.clone()),
        opens_slot: false,
    }
}

fn picked(feat: &FeatKey) -> Grant {
    Grant {
        fact: Fact::Feat(feat.clone()),
        value: GrantValue::Picked,
        opens_slot: false,
    }
}
