//! The walk: one pass over a Build, level by level, that gives the sheet and the choices that fail; and the cursor,
//! the walk paused where a picker asks, which judges each candidate there (`cursor`).

use std::collections::{BTreeMap, BTreeSet, HashSet};

use super::{
    BASE_LEVEL_CAP, Engine, FIRST_ASCENSION_RANK, LevelSheet, PATH_CHOICE_RANK, PATH_SWITCH_RANK,
    RankSheet, RankStatus, Sheet, SkillPoints, SlotKind, racial_amounts,
};
use crate::build::{Build, Faith, LevelUp, Pick, SlotId, has_ability_increase};
use crate::choice::{Choice, Failure, Issue};
use crate::codec::{StaleLink, decode_build};
use crate::game_data::Ability;
use crate::keys::{
    ClassKey, FeatKey, FeatureKey, FeatureSelectionKey, MythicAbilityKey, MythicFeatKey,
    SelectionOptionKey, SkillKey,
};
use crate::parameter::Parameter;
use crate::state::CharacterState;

mod ascension;
mod cursor;
mod hold;
mod judge;
mod level;
mod options;

pub(super) use cursor::Cursor;
pub(super) use level::{advancing_levels, first_taken_at};

impl Engine {
    /// Walks the whole Build.
    pub fn evaluate(&self, build: &Build) -> Sheet {
        Walker::new(self, build).run().sheet
    }

    /// Every choice of the Build that fails, with why, in walk order: the Build start's, each level's (its class,
    /// archetype and advancement on first entry, its skill ranks, its picks in slot order, its Mythic ranks'), then
    /// those no level checked (a pick for a slot the walk didn't reach, a class choice for a class no level takes, the
    /// first ascension, path and switch). A failed pick isn't held, so whatever needs it fails too, later in the list.
    /// Empty exactly when the Build [is possible](Self::is_possible).
    ///
    /// One full walk, without the sheet.
    pub fn failures(&self, build: &Build) -> Vec<Failure> {
        Walker::new(self, build).without_sheet().run().failed
    }

    /// Whether every choice in the Build holds up with the current game data: every key known, every pick in a
    /// slot the Build reaches and fitting it there, the path and switch at ranks it reaches, every rule kept. Every
    /// Build the sheet's edits make is.
    ///
    /// Costs at most one walk of the Build, without the sheet; it stops at the first choice that fails.
    pub fn is_possible(&self, build: &Build) -> bool {
        Walker::new(self, build)
            .without_sheet()
            .holds(0, Vec::new())
    }

    /// Opens a link: its Build when it decodes with the game data's keys and [is possible](Self::is_possible), and
    /// otherwise the one generic error, whatever the reason (the game data changed since the link was made, a key
    /// included, or the link is damaged or hand-edited). The sheet then shows a blank Build.
    pub fn open_link(&self, code: &str) -> Result<Build, StaleLink> {
        decode_build(code, &self.link_keys)
            .ok()
            .filter(|build| self.is_possible(build))
            .ok_or(StaleLink)
    }

    /// The state just before a slot is filled, or `None` when the Build has no such slot. Tests only.
    #[cfg(any(test, feature = "testing"))]
    pub fn state_before(&self, build: &Build, slot: &SlotId) -> Option<CharacterState> {
        Cursor::new(self, build)
            .until_slot(slot)
            .map(|at| at.state().clone())
    }
}

/// Something held that a Path switch can take away again.
#[derive(Clone, Debug)]
enum Held {
    /// A feat, for these values (none for an unparameterised feat).
    Feat(FeatKey, Vec<Parameter>),
    Feature(FeatureKey),
    /// An option, through the selection that gave it.
    SelectionOption(SelectionOptionKey, FeatureSelectionKey),
    MythicAbility(MythicAbilityKey),
    /// A mythic feat, for its value when it's parameterised.
    MythicFeat(MythicFeatKey, Option<Parameter>),
    /// A class skill, from a class, the background or an option row.
    ClassSkill(SkillKey),
}

/// Where a Mythic ascension's grant came from, for what a Path switch keeps.
#[derive(Clone, Debug)]
struct GrantSource {
    /// The rank whose row or pick granted it (MR 1 and 2 for the first ascension's rows).
    rank: u8,
    /// The path's own grant (its row's features, the picks in its slots) rather than the general pick.
    from_path: bool,
    /// The Feature selection whose slot was picked.
    selection: Option<FeatureSelectionKey>,
    /// The slot picked, if any.
    slot: Option<SlotId>,
}

impl GrantSource {
    /// A path rank row's own grant (its features).
    fn row(rank: u8) -> Self {
        Self {
            rank,
            from_path: true,
            selection: None,
            slot: None,
        }
    }
}

/// One thing a Mythic ascension granted.
#[derive(Clone, Debug)]
struct RankGrant {
    source: GrantSource,
    item: Held,
}

/// What the cursor has a level's walk note as it goes ([`Cursor::until_slot`] and its later picks): the state just
/// before a slot is filled, or the picks after a slot at its level ([`Probed::later`]).
#[derive(Clone, Copy)]
enum Probe<'a> {
    Slot(&'a SlotId),
    Later(&'a SlotId),
}

/// What a probe found: nothing when the level has no such slot.
#[derive(Clone, Default)]
struct Probed {
    state: Option<CharacterState>,
    /// For a slot probe, what the slot accepts.
    kind: Option<SlotKind>,
    /// For a later probe, once the walk has reached the slot: each pick the slot's character level holds after it,
    /// in walk order.
    later: Option<Vec<Checkpoint>>,
}

/// A pick a later probe passed, with the state just before and just after its slot was filled.
#[derive(Clone)]
struct Checkpoint {
    slot: SlotId,
    kind: SlotKind,
    pick: Pick,
    before: CharacterState,
    after: CharacterState,
}

#[derive(Clone)]
struct Walker<'a> {
    engine: &'a Engine,
    build: &'a Build,
    state: CharacterState,
    /// The choices that don't hold up, in walk order.
    failed: Vec<Failure>,
    /// The slots the walk has reached, so a pick for any other is found at the end.
    reached: HashSet<SlotId>,
    /// What Mythic ascensions granted, by rank and source, so a Path switch can drop it.
    rank_grants: Vec<RankGrant>,
    ranks: Vec<RankSheet>,
    level_cap: u8,
    /// Set by a switch to a path without further ascensions.
    ascensions_over: bool,
    /// The race's ability penalties applied at Build start, which Destiny Beyond Birth gives back.
    racial_penalties: Vec<(Ability, u8)>,
    /// For each class and feature another class's levels count toward (Frost Power: witch, Major Hexes), how many
    /// grants of it those levels hold now: the class's rows past its own level that they reach.
    counted_features: BTreeMap<(ClassKey, FeatureKey), u8>,
    /// How many sources (classes, the background, option rows) give each class skill, so one taken back (a
    /// replaced row) leaves the skill while another gives it.
    class_skill_holds: BTreeMap<SkillKey, u8>,
    /// Each option held through a selection with option levels, in the order taken.
    holdings: Vec<Holding>,
    /// The slots granted at this level that have no slot yet: options' rows' and feats'.
    pending: Vec<Pending>,
    /// The Feature selections the walk has reached a slot of, in the order it first reached each: which one a feat's
    /// extra pick is of ([`Feat::extra_pick_of`](crate::Feat::extra_pick_of)).
    selections_had: Vec<FeatureSelectionKey>,
    /// While an option row's selection slot is filled: the option whose row granted it, for a selection levelled
    /// by [`LevelFrom::GrantingOption`](crate::LevelFrom::GrantingOption).
    granting: Option<SelectionOptionKey>,
    /// How many option-row selection slots this level has so far.
    option_slots: u8,
    /// How many option-row feat slots this level has so far.
    option_feat_slots: u8,
    /// How many selection slots feats have given this level so far.
    feat_selection_slots: u8,
    /// The skill points of the last level walked, which the skill steppers read off a paused walk.
    skill_points: SkillPoints,
    probe: Option<Probe<'a>>,
    probed: Probed,
    /// Whether the walk keeps the sheet's rows; a recheck's walk doesn't ([`without_sheet`](Self::without_sheet)).
    sheet: bool,
    /// Set while the walk only answers whether a choice fails ([`holds`](Self::holds)).
    recheck: Option<RecheckWalk>,
}

/// A slot granted at a level, filled once the level's own slots are.
#[derive(Clone, Debug)]
enum Pending {
    /// A Feature selection or feat slot an option's row granted, with that option.
    Row(SlotKind, SelectionOptionKey),
    /// A Feature selection slot a feat pick gives (Extra Rage Power), of the selection its extra pick is of.
    Feat(FeatKey),
}

/// What a walk that only answers whether a choice fails keeps instead of the failed choices.
#[derive(Clone, Debug, Default)]
struct RecheckWalk {
    /// The choices whose failing doesn't count: the ones an edit sets, and any it may take with it.
    allowed: Vec<Choice>,
    /// Why the first other choice failed, once one has: the walk stops there.
    broken: Option<Issue>,
    /// Whether an `allowed` choice has failed.
    excused: bool,
}

/// One option held through a selection with option levels: which of its rows it has got, and what each holds.
#[derive(Clone, Debug)]
struct Holding {
    /// The selection it was taken through, which says which class levels make its level.
    selection: FeatureSelectionKey,
    option: SelectionOptionKey,
    /// The rows got so far, by option level, with what each holds.
    applied: BTreeMap<u8, Vec<Held>>,
    /// Option levels it never gets the rows of: its selection's skipped levels and the rows a pick replaced.
    skipped: BTreeSet<u8>,
    /// Only its feat lists count (Crossblooded's second bloodline).
    lists_only: bool,
    /// Rows a lists-only holding has in full all the same, which a held feature's option row of the selection it's
    /// held through unlocks (Expanded Defense: an expanded element's defensive talent).
    unlocked: BTreeSet<u8>,
    /// The value its pick named, for the chosen feats of its rows (Weapon Mastery: the weapon).
    parameter: Option<Parameter>,
    /// The Mythic ascension whose pick took it, if any, so a Path switch can drop it.
    source: Option<GrantSource>,
    /// The option whose row granted the slot it was taken in, if any
    /// ([`LevelFrom::GrantingOption`](crate::LevelFrom::GrantingOption)).
    granted_by: Option<SelectionOptionKey>,
}

struct Walk {
    sheet: Sheet,
    failed: Vec<Failure>,
}

impl<'a> Walker<'a> {
    fn new(engine: &'a Engine, build: &'a Build) -> Self {
        let mut ascensions = engine.catalog.data().progression.mythic_ascensions.clone();
        ascensions.sort_by_key(|a| a.rank);
        let ranks = ascensions
            .iter()
            .map(|a| RankSheet {
                rank: a.rank,
                character_level: a.character_level,
                status: RankStatus::NotReached,
                path: None,
                slots: Vec::new(),
                dropped_slots: Vec::new(),
            })
            .collect();
        Self {
            engine,
            build,
            state: CharacterState::default(),
            failed: Vec::new(),
            reached: HashSet::new(),
            rank_grants: Vec::new(),
            ranks,
            level_cap: BASE_LEVEL_CAP,
            ascensions_over: false,
            racial_penalties: Vec::new(),
            counted_features: BTreeMap::new(),
            class_skill_holds: BTreeMap::new(),
            holdings: Vec::new(),
            pending: Vec::new(),
            selections_had: Vec::new(),
            granting: None,
            option_slots: 0,
            option_feat_slots: 0,
            feat_selection_slots: 0,
            skill_points: SkillPoints::default(),
            probe: None,
            probed: Probed::default(),
            sheet: true,
            recheck: None,
        }
    }

    /// The same walk without the sheet's rows (the state at each level), which a recheck doesn't read.
    fn without_sheet(mut self) -> Self {
        self.sheet = false;
        self
    }

    /// Notes a choice that doesn't hold up, and why. A recheck asks only whether one does (it locks an edit, and makes
    /// a link stale); [`Engine::failures`] keeps why.
    fn fail(&mut self, at: Choice, issue: Issue) {
        match &mut self.recheck {
            Some(recheck) if recheck.allowed.contains(&at) => recheck.excused = true,
            Some(recheck) => {
                recheck.broken.get_or_insert(issue);
            }
            None => self.failed.push(Failure { choice: at, issue }),
        }
    }

    /// Whether no choice fails but `allowed` ones: walks the Build start when `first` is 0, then the levels from
    /// `first` on (the ones before it are walked already, and count with what failed in them), and stops at the
    /// first other choice that fails. A pick for a level's slot the level didn't reach fails once the level is
    /// walked, rather than at the end.
    fn holds(self, first: u8, allowed: Vec<Choice>) -> bool {
        self.recheck(first, allowed).broken.is_none()
    }

    /// [`holds`](Self::holds), with why the walk broke, if it did, and whether an `allowed` choice failed.
    fn recheck(mut self, first: u8, allowed: Vec<Choice>) -> RecheckWalk {
        let excused = |failure: &&Failure| allowed.contains(&failure.choice);
        let broken = (self.failed.iter())
            .find(|failure| !excused(failure))
            .map(|failure| failure.issue.clone());
        let excused = self.failed.iter().any(|failure| excused(&failure));
        self.recheck = Some(RecheckWalk {
            allowed,
            broken,
            excused,
        });
        let build = self.build;
        if first == 0 {
            self.start();
        }
        let skipped = usize::from(first).saturating_sub(1);
        for (index, level_up) in build.levels.iter().enumerate().skip(skipped) {
            let level = u8::try_from(index + 1).unwrap_or(u8::MAX);
            if self.broken() {
                break;
            }
            self.level(level, level_up);
            let unreached: Vec<SlotId> = build
                .picks
                .keys()
                .filter(|slot| slot.level() == Some(level) && !self.reached.contains(*slot))
                .cloned()
                .collect();
            for slot in unreached {
                self.fail(Choice::Slot(slot), Issue::NotReached);
            }
        }
        if !self.broken() {
            self.check_unreached();
        }
        self.recheck.unwrap_or_default()
    }

    fn broken(&self) -> bool {
        self.recheck
            .as_ref()
            .is_some_and(|recheck| recheck.broken.is_some())
    }

    /// Walks the whole Build, then reports.
    fn run(mut self) -> Walk {
        self.start();
        let build = self.build;
        let mut levels = Vec::with_capacity(build.levels.len());
        for (index, level_up) in build.levels.iter().enumerate() {
            let level = u8::try_from(index + 1).unwrap_or(u8::MAX);
            levels.extend(self.level(level, level_up));
        }
        self.check_unreached();
        Walk {
            sheet: Sheet {
                levels,
                ranks: self.ranks,
                level_cap: self.level_cap,
            },
            failed: self.failed,
        }
    }

    /// The choices no level checked: a pick for a slot the walk didn't reach (one a level doesn't have, past the
    /// last level, or at a rank not reached), a class choice for a class no level takes, and the first ascension,
    /// path and switch, which must be ones the Build can take (the data's, [`Issue::UnknownKey`]; a path that rank
    /// takes, [`Issue::NotAllowed`]; a switch also open from the Build's path, [`Issue::NotFromPath`]), at ranks it
    /// reaches ([`Issue::NotReached`]), each checked in that order.
    fn check_unreached(&mut self) {
        let build = self.build;
        for slot in build.picks.keys() {
            if !self.reached.contains(slot) {
                self.fail(Choice::Slot(slot.clone()), Issue::NotReached);
            }
        }
        for class in build.class_choices.keys() {
            if !build.levels.iter().any(|level| &level.class == class) {
                self.fail(
                    Choice::Archetype {
                        class: class.clone(),
                    },
                    Issue::NotReached,
                );
            }
        }
        let engine = self.engine;
        let fits = [
            (
                Choice::FirstAscension,
                FIRST_ASCENSION_RANK,
                build
                    .first_ascension
                    .as_ref()
                    .map(|key| engine.first_ascension_fit(key).map(|_| ())),
            ),
            (
                Choice::MythicPath,
                PATH_CHOICE_RANK,
                build
                    .mythic_path
                    .as_ref()
                    .map(|key| engine.path_fit(key).map(|_| ())),
            ),
            (
                Choice::PathSwitch,
                PATH_SWITCH_RANK,
                build
                    .path_switch
                    .as_ref()
                    .map(|key| engine.switch_fit(build, key).map(|_| ())),
            ),
        ];
        for (choice, rank, fit) in fits {
            let issue = match fit {
                None => continue,
                Some(Err(issue)) => issue,
                Some(Ok(())) if !engine.reaches(build, rank) => Issue::NotReached,
                Some(Ok(())) => continue,
            };
            self.fail(choice, issue);
        }
    }

    // ---- Build start --------------------------------------------------------------------------------------

    fn start(&mut self) {
        let catalog = &self.engine.catalog;
        let start = &self.build.start;
        self.state.race = start.race.clone();
        self.state.alignment = start.alignment;

        let race = match &start.race {
            Some(key) => {
                let race = catalog.race(key);
                if race.is_none() {
                    self.fail(Choice::Race, Issue::UnknownKey);
                }
                race
            }
            None => None,
        };
        // A key the data lacks, or a subrace of another race.
        if let Some(key) = &start.subrace {
            let issue = match catalog.subrace(key) {
                None => Some(Issue::UnknownKey),
                Some((owner, _)) if !race.is_some_and(|race| race.key == owner.key) => {
                    Some(Issue::NotAllowed)
                }
                Some(_) => None,
            };
            if let Some(issue) = issue {
                self.fail(Choice::Subrace, issue);
            }
        }

        let mut scores = start.ability_scores.scores();
        for (ability, amount) in racial_amounts(catalog.racial_modifiers(start), start) {
            let score = scores.get_mut(ability);
            *score = score.saturating_add_signed(amount);
            if amount < 0 {
                self.racial_penalties.push((ability, amount.unsigned_abs()));
            }
        }
        self.state.base_ability_scores = scores;
        self.refresh_ability_scores();
        self.background();
        self.faith();
    }

    /// The faith: a deity the data has, which allows the start's alignment (one not chosen yet fits any), or
    /// atheism. The alignment comes after the faith, so an alignment with no faith fails (the rule the start pickers
    /// share: [`Engine::faith_misfit`]). A deity's domains are what deity-limited selections and `DeityOffers` look
    /// at.
    fn faith(&mut self) {
        let start = &self.build.start;
        self.state.faith = start.deity.clone();
        if let Some((choice, issue)) = self.engine.faith_misfit(start) {
            self.fail(choice, issue);
        }
        let Some(key) = start.deity.as_ref().and_then(Faith::deity) else {
            return;
        };
        let Some(deity) = self.engine.catalog.deity(key) else {
            self.fail(Choice::Deity, Issue::UnknownKey);
            return;
        };
        self.state.deity_domains = deity.domains.clone();
    }

    /// The background's skills become class skills and its feats are held from the start, as a class row's
    /// granted feats: no slot, no pick, prerequisites skipped.
    fn background(&mut self) {
        let catalog = &self.engine.catalog;
        let Some(key) = &self.build.start.background else {
            return;
        };
        let Some(background) = catalog.background(key) else {
            self.fail(Choice::Background, Issue::UnknownKey);
            return;
        };
        for skill in &background.class_skills {
            self.hold(Held::ClassSkill(skill.clone()), None);
        }
        for feat in &background.feats {
            let values = catalog.every_value(feat);
            self.hold(Held::Feat(feat.clone(), values), None);
        }
        for granted in &background.feats_for {
            self.hold(
                Held::Feat(granted.feat.clone(), vec![granted.value.clone()]),
                None,
            );
        }
    }

    // ---- Level-up -----------------------------------------------------------------------------------------

    /// Walks one level; its sheet row, unless the walk keeps no sheet.
    fn level(&mut self, level: u8, level_up: &'a LevelUp) -> Option<LevelSheet> {
        if level > self.level_cap {
            self.fail(Choice::CharacterLevel { level }, Issue::NotReached);
        }
        // Entry prerequisites are checked against the character before this level.
        self.check_class_entry(level, level_up);
        self.state.character_level = level;
        let class_level = self.take_class_level(level, level_up);
        self.count_toward_features();

        // Only a level that has one holds an increase (`Build::keeps_fixed_rules`).
        let ability_increase = has_ability_increase(level);
        if ability_increase && let Some(ability) = level_up.ability_increase {
            let score = self.state.base_ability_scores.get_mut(ability);
            *score = score.saturating_add(1);
            self.refresh_ability_scores();
        }
        self.recompute_tables();
        // Options' rows this level reaches, before any slot is filled.
        self.option_slots = 0;
        self.option_feat_slots = 0;
        self.feat_selection_slots = 0;
        self.advance_holdings();
        let skill_points = self.skill_ranks(level, level_up);
        self.skill_points = skill_points;

        let level_slots = self.level_slots(level, level_up);
        let level_slots = self.grant_selections(level_slots);
        let mut slots = self.fill_all(level_slots, None);
        slots.extend(self.fill_pending(level));

        let mut mythic_ranks = Vec::new();
        let due: Vec<u8> = self
            .ranks
            .iter()
            .filter(|rank| rank.character_level == level)
            .map(|rank| rank.rank)
            .collect();
        for rank in due {
            if self.ascend(rank) {
                mythic_ranks.push(rank);
            }
            slots.extend(self.fill_pending(level));
        }

        self.sheet.then(|| LevelSheet {
            character_level: level,
            class: level_up.class.clone(),
            class_level,
            ability_increase,
            skill_points,
            slots,
            mythic_ranks,
            state: self.state.clone(),
        })
    }
}
