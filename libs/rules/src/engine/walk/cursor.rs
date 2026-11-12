//! The cursor: a Build's walk, paused where a picker asks, which judges each candidate there. The pickers keep only
//! presentation (which candidates, grouped and named); every verdict is the walk's.
//!
//! - [`Cursor::until_slot`] pauses the walk just before a slot is filled ([`AtSlot`]), and
//!   [`Cursor::until_level`] just before a level's class ([`AtLevel`]). Each shows the character's state there,
//!   which the pickers read to present their options.
//! - [`AtSlot::try_pick`], [`AtLevel::try_class`] and [`Cursor::try_edit`] give a candidate's verdict,
//!   `Result<(), Issue>`: the choice's own check at the pause, in the mode it needs (the walk's own checks, in
//!   `judge`), then whether it breaks a later choice ([`Issue::BreaksLater`]).
//! - [`Cursor::kept_advancements`] lists the advancements of a prestige class that keep the Build, which both the
//!   class's verdict and the advancement picker read.
//!
//! The walk can't resume mid-level ([`Walker::level`] works a level's slots out at its start), so the cursor keeps
//! the walk paused before the levels the questions asked need, and a candidate's later choices are
//! walked from the start of the first level it changes, on the edited Build. Those walks keep no sheet and stop at
//! the first choice that fails. [`Engine::allows`] and the withdrawals ask the same walks ([`Cursor::keeps`]).

use std::cell::{OnceCell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashSet};

use super::judge::{check_pick, check_pick_with, class_entry};
use super::level::{advancements, first_taken_at};
use super::{Checkpoint, Probe, RecheckWalk, Walker};
use crate::build::{Build, ClassChoice, Edit, Pick, SlotId};
use crate::choice::{Choice, Issue, LaterPick, Unmet};
use crate::engine::{
    AdvancesOption, Engine, FIRST_ASCENSION_RANK, PATH_CHOICE_RANK, PATH_SWITCH_RANK, SkillPoints,
    SlotKind, class_level_cap, most_advanced,
};
use crate::game_data::{Class, FeatList};
use crate::keys::{ArchetypeKey, ClassKey, FeatKey};
use crate::prerequisite::Prerequisite;
use crate::state::CharacterState;

/// A Build's walk, paused before the character levels the questions asked of it need. Build one per picker (or
/// per question), for the Build it judges.
pub(crate) struct Cursor<'a> {
    engine: &'a Engine,
    build: &'a Build,
    /// The walk just before character level `i + 1` (after the Build start) at key `i`, for each level a question
    /// has asked about; the one after the last level is the last there can be. Only those are kept: a level asked
    /// about is walked on from the closest kept before it, without a copy of the walk at each level between (most
    /// cursors answer one question, at one level: ticket 158).
    paused: RefCell<BTreeMap<usize, Walker<'a>>>,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(engine: &'a Engine, build: &'a Build) -> Self {
        Self {
            engine,
            build,
            paused: RefCell::new(BTreeMap::new()),
        }
    }

    // ---- Pausing ----------------------------------------------------------------------------------------------

    /// The walk paused just before a level's class is taken (`until(level)`): the character after the levels before
    /// it; one past the last level gives the character after the last. `None` for level 0 and past that.
    pub(crate) fn until_level(&self, level: u8) -> Option<AtLevel<'_, 'a>> {
        let index = usize::from(level).checked_sub(1)?;
        let state = if index == self.build.levels.len() {
            self.after_last()
        } else {
            self.with_paused(index, |walker| walker.state.clone())?
        };
        Some(AtLevel {
            cursor: self,
            level,
            state,
        })
    }

    /// The walk paused just before a slot is filled (`until(slot)`), with what the slot accepts. `None` when the
    /// Build has no such slot.
    ///
    /// Costs the walk to the slot's level, which the cursor keeps, then the slot's level.
    pub(crate) fn until_slot(&self, slot: &SlotId) -> Option<AtSlot<'_, 'a>> {
        let level = self.level_of(slot)?;
        let index = usize::from(level).checked_sub(1)?;
        let level_up = self.build.levels.get(index)?;
        let mut walker = self.paused(index)?;
        walker.probe = Some(Probe::Slot(slot));
        walker.level(level, level_up);
        let state = walker.probed.state?;
        let kind = walker.probed.kind?;
        // A feat slot's lists, worked out once for every feat.
        let lists = match &kind {
            SlotKind::Feat(feat_slot) => self.engine.catalog.slot_lists(feat_slot, &state),
            _ => Vec::new(),
        };
        Some(AtSlot {
            cursor: self,
            slot: slot.clone(),
            state,
            kind,
            lists,
            named: OnceCell::new(),
            left_empty: OnceCell::new(),
            later_picks: OnceCell::new(),
        })
    }

    /// The skill points of a character level the Build has, off the walk paused after it.
    pub(crate) fn skill_points(&self, level: u8) -> Option<SkillPoints> {
        if level == 0 {
            return None;
        }
        self.with_paused(usize::from(level), |walker| walker.skill_points)
    }

    /// The character level whose walk fills a slot: its own, or the one its Mythic rank is taken at in the Build.
    /// `None` for a rank the data lacks.
    fn level_of(&self, slot: &SlotId) -> Option<u8> {
        slot.level()
            .or_else(|| self.engine.rank_level(self.build, slot.rank()?))
    }

    /// The character after the last level: the walk kept after it, or else the furthest one kept walked on without
    /// keeping the levels between (a level after the last walks nothing later, so nothing asks for them).
    fn after_last(&self) -> CharacterState {
        let build = self.build;
        let paused = self.paused.borrow();
        if let Some(walker) = paused.get(&build.levels.len()) {
            return walker.state.clone();
        }
        let (mut walker, walked) = match paused.last_key_value() {
            Some((&walked, walker)) => (walker.clone(), walked),
            None => (self.started(), 0),
        };
        for (index, level_up) in build.levels.iter().enumerate().skip(walked) {
            walker.level(u8::try_from(index + 1).unwrap_or(u8::MAX), level_up);
        }
        walker.state
    }

    /// The walk just before character level `index + 1`; `None` past the one after the last level.
    fn paused(&self, index: usize) -> Option<Walker<'a>> {
        self.with_paused(index, Walker::clone)
    }

    /// [`paused`](Self::paused), taken out of the cursor rather than copied, for a question that is the cursor's
    /// last ([`keeps_once`](Self::keeps_once)).
    fn take_paused(&self, index: usize) -> Option<Walker<'a>> {
        self.walk_to(index)
            .then(|| self.paused.borrow_mut().remove(&index))
            .flatten()
    }

    /// What `read` reads off the walk just before character level `index + 1`, without cloning it; `None` past the
    /// one after the last level.
    fn with_paused<T>(&self, index: usize, read: impl FnOnce(&Walker<'a>) -> T) -> Option<T> {
        self.walk_to(index)
            .then(|| self.paused.borrow().get(&index).map(read))
            .flatten()
    }

    /// Keeps the walk just before character level `index + 1`, walked on from the closest one kept before it (or
    /// from the Build start); `false` past the one after the last level.
    fn walk_to(&self, index: usize) -> bool {
        let build = self.build;
        if index > build.levels.len() {
            return false;
        }
        let mut paused = self.paused.borrow_mut();
        if paused.contains_key(&index) {
            return true;
        }
        let (mut walker, walked) = match paused.range(..index).next_back() {
            Some((&walked, walker)) => (walker.clone(), walked),
            None => (self.started(), 0),
        };
        for (at, level_up) in build.levels.iter().enumerate().take(index).skip(walked) {
            walker.level(u8::try_from(at + 1).unwrap_or(u8::MAX), level_up);
        }
        paused.insert(index, walker);
        true
    }

    /// The walk just after the Build start, before the first level, without the sheet.
    fn started(&self) -> Walker<'a> {
        let mut walker = Walker::new(self.engine, self.build).without_sheet();
        walker.start();
        walker
    }

    // ---- Verdicts ---------------------------------------------------------------------------------------------

    /// An edit's verdict (`try(edit)`) for the pickers outside a slot or a level's class (a pick's is
    /// [`AtSlot::try_pick`] and a class's [`AtLevel::try_class`], since a picker pauses once for all its
    /// candidates): its own check, then whether it breaks a later choice. The own check is the walk's: a Build
    /// start's faith and alignment ([`Engine::faith_misfit`]); the first ascension, path and switch the Build can take
    /// ([`Engine::first_ascension_fit`], [`Engine::path_fit`], [`Engine::switch_fit`]). An ability increase and skill
    /// ranks have none beyond what the steppers' range keeps.
    pub(crate) fn try_edit(&self, edit: &Edit) -> Result<(), Issue> {
        let engine = self.engine;
        match edit {
            Edit::Start(start) => {
                if let Some((_, issue)) = engine.faith_misfit(start) {
                    return Err(issue);
                }
            }
            Edit::FirstAscension(Some(key)) => {
                engine.first_ascension_fit(key)?;
            }
            Edit::MythicPath(Some(key)) => {
                engine.path_fit(key)?;
            }
            Edit::PathSwitch(Some(key)) => {
                engine.switch_fit(self.build, key)?;
            }
            _ => {}
        }
        self.later(edit)
    }

    /// Whether `edit` breaks no choice other than the `allowed` ones (an edit's own: [`Edit::choices`]). An edit
    /// that changes nothing breaks nothing.
    /// An edit that moves a Mythic rank and would shift a pick in a slot a rank's pick gives at its level is
    /// refused too, though the walk may fail nothing: the pick would change owner
    /// ([`Engine::shifted_picks`](crate::Engine::shifted_picks)).
    pub(crate) fn keeps(&self, edit: &Edit, allowed: Vec<Choice>) -> bool {
        let edited = self.edited(edit);
        &edited == self.build
            || (self.engine.shifted_between(self.build, &edited).is_empty()
                && self.walks(edit, &edited, allowed))
    }

    /// [`keeps`](Self::keeps), as the cursor's only question: its walk goes on from the paused walk itself rather
    /// than from a copy of it (ticket 158). What [`Engine::allows`] and the withdrawals ask.
    pub(crate) fn keeps_once(self, edit: &Edit, allowed: Vec<Choice>) -> bool {
        let edited = self.edited(edit);
        &edited == self.build
            || (self.engine.shifted_between(self.build, &edited).is_empty()
                && (self.recheck_with(edit, &edited, allowed, Self::take_paused))
                    .broken
                    .is_none())
    }

    /// The Build with `edit` applied.
    fn edited(&self, edit: &Edit) -> Build {
        let mut edited = self.build.clone();
        self.engine.apply(&mut edited, edit);
        edited
    }

    /// Whether the walk of `edited` (the Build with `edit` applied) fails no choice but the `allowed` ones, even
    /// when `edit` changes nothing: then it's the Build's own walk.
    fn walks(&self, edit: &Edit, edited: &Build, allowed: Vec<Choice>) -> bool {
        self.recheck(edit, edited, allowed).broken.is_none()
    }

    /// [`walks`](Self::walks), with why the walk broke, if it did, and whether an `allowed` choice failed.
    fn recheck(&self, edit: &Edit, edited: &Build, allowed: Vec<Choice>) -> RecheckWalk {
        self.recheck_with(edit, edited, allowed, Self::paused)
    }

    /// [`recheck`](Self::recheck), going on from the paused walk `paused` gives: a copy, or the walk itself.
    fn recheck_with(
        &self,
        edit: &Edit,
        edited: &Build,
        allowed: Vec<Choice>,
        paused: impl FnOnce(&Self, usize) -> Option<Walker<'a>>,
    ) -> RecheckWalk {
        let first = self.first_level(edit);
        match first.checked_sub(1) {
            None => Walker::new(self.engine, edited)
                .without_sheet()
                .recheck(0, allowed),
            Some(index) => {
                let Some(mut walker) = paused(self, usize::from(index)) else {
                    // Nothing after the last level depends on an edit past it (Remove level, a name).
                    return RecheckWalk::default();
                };
                walker.on(edited);
                walker.recheck(first, allowed)
            }
        }
    }

    /// The advancements of a prestige class taken at a character level (with `archetype`) that keep the Build, each
    /// with the edit that takes the class with it: the class picker's verdict for a prestige class that asks what it
    /// advances (open when there is one: [`AtLevel::try_class`]) and the advancement picker's list
    /// ([`Engine::advances_options`]), so the two can't disagree. The candidates are [`advancements`]; one keeps the
    /// Build when the walk of the Build with its edit fails no choice, the Build's own walk when the edit changes
    /// nothing (so none keeps a Build that isn't possible).
    ///
    /// `own` (the class option's advancement), or else the first candidate, is walked first, and comes first when it
    /// keeps; the rest follow in the candidates' order, each walked when the iterator gets to it. That first walk
    /// excuses only the advancement itself ([`Choice::Advances`]). When it fails on another choice whose check reads
    /// no spellcasting ([`spellcasting_could_meet`]), no advancement can keep the Build (they differ only in the
    /// spell levels and caster level they give), so no other is walked.
    pub(crate) fn kept_advancements(
        &self,
        level: u8,
        prestige: &'a Class,
        archetype: Option<&ArchetypeKey>,
        own: Option<&BTreeSet<ClassKey>>,
    ) -> impl Iterator<Item = AdvancesOption> + '_ {
        let first_taken = first_taken_at(self.build, &prestige.key, level);
        let archetype = archetype.cloned();
        let edit = move |advances: BTreeSet<ClassKey>| Edit::Class {
            level,
            class: prestige.key.clone(),
            choice: first_taken.then(|| ClassChoice {
                archetype: archetype.clone(),
                advances,
            }),
        };
        let candidates = advancements(self.build, level, prestige);
        let first = own.or(candidates.first()).cloned().unwrap_or_default();
        let advances_choice = Choice::Advances {
            class: prestige.key.clone(),
        };
        let first_edit = edit(first.clone());
        let walked = self.recheck(
            &first_edit,
            &self.edited(&first_edit),
            vec![advances_choice],
        );
        let first_keeps = walked.broken.is_none() && !walked.excused;
        let others_may = walked.broken.as_ref().is_none_or(spellcasting_could_meet);
        let first_kept = first_keeps.then(|| AdvancesOption {
            advances: first.clone(),
            edit: first_edit,
        });
        let others = candidates
            .into_iter()
            .filter(move |advances| others_may && advances != &first)
            .map(move |advances| AdvancesOption {
                edit: edit(advances.clone()),
                advances,
            })
            .filter(|option| self.walks(&option.edit, &self.edited(&option.edit), Vec::new()));
        first_kept.into_iter().chain(others)
    }

    /// An option open at its point of the Build: `Ok`, or [`Issue::BreaksLater`] when its edit breaks a choice but
    /// its own.
    fn later(&self, edit: &Edit) -> Result<(), Issue> {
        status(self.keeps(edit, edit.choices()))
    }

    /// [`later`](Self::later) for an edit's walk even when the edit changes nothing (the walk of the Build itself,
    /// which fails a choice only when the Build isn't possible): what an option gets when its walk is the same as
    /// that one.
    fn later_walked(&self, edit: &Edit) -> Result<(), Issue> {
        status(self.walks(edit, &self.edited(edit), edit.choices()))
    }

    /// The first character level whose walk an edit can change; 0 for the Build start, and past the end for an
    /// edit that changes no level.
    fn first_level(&self, edit: &Edit) -> u8 {
        let (engine, build) = (self.engine, self.build);
        match edit {
            Edit::Pick { slot, .. } | Edit::Clear(slot) => self.level_of(slot).unwrap_or(0),
            // A new archetype or advancement holds from the class's first level.
            Edit::Class {
                level,
                class,
                choice,
            } => {
                let first = choice
                    .as_ref()
                    .and_then(|_| build.levels.iter().position(|l| &l.class == class))
                    .map(|index| u8::try_from(index + 1).unwrap_or(u8::MAX));
                first.map_or(*level, |first| first.min(*level))
            }
            Edit::AbilityIncrease { level, .. } | Edit::SkillRanks { level, .. } => *level,
            Edit::Start(_) => 0,
            Edit::FirstAscension(_) => engine.rank_walk_level(build, FIRST_ASCENSION_RANK),
            // Without a first ascension of its own, the Build takes the MR 3 path's, from MR 1.
            Edit::MythicPath(_) if build.first_ascension.is_none() => {
                engine.rank_walk_level(build, FIRST_ASCENSION_RANK)
            }
            Edit::MythicPath(_) => engine.rank_walk_level(build, PATH_CHOICE_RANK),
            Edit::PathSwitch(_) => engine.rank_walk_level(build, PATH_SWITCH_RANK),
            // A rank moved to the last level goes back to where it comes without the move, and the ranks it took
            // along with it: the walk changes from the earliest any of them can come at. No other removal changes a
            // level that stays.
            Edit::RemoveLevel => (build.rank_levels.iter())
                .filter(|(_, level)| usize::from(**level) >= build.levels.len())
                .filter_map(|(rank, _)| engine.rank_floor(build, *rank))
                .min()
                .unwrap_or(u8::MAX),
            Edit::SetName(_) => u8::MAX,
            // The rank comes no earlier than its floor before the edit or after it, and the ranks it takes along
            // come after it.
            Edit::RankLevel { rank, .. } => engine.rank_floor(build, *rank).unwrap_or(0),
            // Gear counts from the Build start.
            Edit::Wear { .. } | Edit::Use { .. } => 0,
        }
    }

    /// The feats a walk of the Build can look at: each feat picked (whether it's held already), each option's
    /// chosen feat, the feats named by the prerequisites of the picks and of the classes and archetypes the Build
    /// takes, and every feat with an extra pick. Holding any other feat touches nothing but that feat's entries in
    /// the state, so a feat pick outside these changes no later check: the walk with it fails exactly the choices
    /// the walk without it fails. A feat with an extra pick also gives a slot, which takes its level's next
    /// [`SlotId::FeatSelection`] index from a later feat's, so it's always walked.
    fn named_feats(&self) -> HashSet<&'a FeatKey> {
        let (catalog, build) = (&self.engine.catalog, self.build);
        let mut named: HashSet<&FeatKey> = catalog
            .data()
            .feats
            .iter()
            .filter(|feat| !feat.extra_pick_of.is_empty())
            .map(|feat| &feat.key)
            .collect();
        for item in build
            .picks
            .values()
            .filter_map(|pick| catalog.pick_item(pick))
        {
            named.extend(item.feat);
            if let Some(prerequisite) = item.prerequisite {
                prerequisite.collect_feats(&mut named);
            }
        }
        for level in &build.levels {
            if let Some((class, _)) = catalog.class(&level.class) {
                let archetypes = class.archetypes.iter().map(|a| a.prerequisite.as_ref());
                for prerequisite in std::iter::once(class.prerequisite.as_ref())
                    .chain(archetypes)
                    .flatten()
                {
                    prerequisite.collect_feats(&mut named);
                }
            }
        }
        named
    }
}

/// The state just before a slot is filled and what the slot accepts, in a Build: `None` when the Build has no such
/// slot. What tells a pick open only because of gear: the same slot, asked of the Build without it.
pub(crate) fn before_slot(
    engine: &Engine,
    build: &Build,
    slot: &SlotId,
) -> Option<(CharacterState, SlotKind)> {
    let cursor = Cursor::new(engine, build);
    let at = cursor.until_slot(slot)?;
    Some((at.state, at.kind))
}

/// Whether a slot takes a pick at a state, by the walk's own check alone ([`check_pick`]), whatever the pick would
/// break later.
pub(crate) fn pick_fits(
    engine: &Engine,
    state: &CharacterState,
    kind: &SlotKind,
    pick: &Pick,
) -> bool {
    check_pick(&engine.catalog, state, kind, pick).is_ok()
}

/// The walk paused just before a slot is filled ([`Cursor::until_slot`]): the character's state there and what the
/// slot accepts, which the picker reads to list and present its candidates, and each candidate's verdict
/// ([`try_pick`](Self::try_pick)).
pub(crate) struct AtSlot<'c, 'a> {
    cursor: &'c Cursor<'a>,
    slot: SlotId,
    state: CharacterState,
    kind: SlotKind,
    /// A feat slot's lists for this character ([`Catalog::slot_lists`](crate::engine::catalog::Catalog)).
    lists: Vec<&'a FeatList>,
    /// The feats a later check can see ([`Cursor::named_feats`]), once a feat is open.
    named: OnceCell<HashSet<&'a FeatKey>>,
    /// The verdict for the slot left empty, which every feat no later check names shares.
    left_empty: OnceCell<Result<(), Issue>>,
    /// The picks after the slot at its character level, once a pick's prerequisite is unmet.
    later_picks: OnceCell<Vec<Checkpoint>>,
}

impl<'a> AtSlot<'_, 'a> {
    /// The character just before the slot is filled.
    pub(crate) fn state(&self) -> &CharacterState {
        &self.state
    }

    /// What the slot accepts.
    pub(crate) fn kind(&self) -> &SlotKind {
        &self.kind
    }

    /// A feat slot's lists for this character (empty for any other slot), which say which feats it offers.
    pub(crate) fn lists(&self) -> &[&'a FeatList] {
        &self.lists
    }

    /// A pick's verdict in the slot (`try(edit)` for [`Edit::Pick`]): the walk's own check of the pick against the
    /// character here ([`check_pick`]: a known key the slot accepts, a value that fits, not held already,
    /// prerequisites met in the mode the slot needs), with an unmet prerequisite that a later pick at the same level
    /// meets saying so ([`Unmet::MetLater`], ticket 126); then whether it breaks a later choice
    /// ([`Issue::BreaksLater`]).
    ///
    /// The first open feat costs a walk from the slot's level for the slot left empty, which every feat no later
    /// check can see shares ([`Cursor::named_feats`]); every other open pick costs a walk of its own from there.
    pub(crate) fn try_pick(&self, pick: &Pick) -> Result<(), Issue> {
        let catalog = &self.cursor.engine.catalog;
        check_pick_with(catalog, &self.state, &self.kind, &self.lists, pick)
            .map_err(|issue| self.met_later(pick, issue))?;
        match pick {
            // A feat no check names breaks later just what the slot left empty would.
            Pick::Feat { feat, .. }
                if !self
                    .named
                    .get_or_init(|| self.cursor.named_feats())
                    .contains(feat) =>
            {
                self.left_empty
                    .get_or_init(|| self.cursor.later_walked(&Edit::Clear(self.slot.clone())))
                    .clone()
            }
            _ => self.cursor.later(&Edit::Pick {
                slot: self.slot.clone(),
                pick: pick.clone(),
            }),
        }
    }

    /// An unmet prerequisite of `pick` that a later pick at the same level meets: [`Unmet::MetLater`], naming the
    /// first such pick ([`met_later`](Self::met_later)). Any other issue as it is.
    fn met_later(&self, pick: &Pick, issue: Issue) -> Issue {
        let Issue::Unmet(unmet) = issue else {
            return issue;
        };
        match self.later_pick(pick) {
            Some(later) => Issue::Unmet(Unmet::MetLater {
                unmet: Box::new(unmet),
                later: Box::new(later),
            }),
            None => Issue::Unmet(unmet),
        }
    }

    /// The first of the slot's later picks ([`later_picks`](Self::later_picks)) whose own slot makes `pick` open
    /// here: it fails just before that slot and holds just after it. `None` when none does, or when what comes
    /// between picks does (a Mythic rank's row).
    fn later_pick(&self, pick: &Pick) -> Option<LaterPick> {
        let catalog = &self.cursor.engine.catalog;
        let (state, kind, lists) = (&self.state, &self.kind, self.lists.as_slice());
        let open =
            |state: &CharacterState| check_pick_with(catalog, state, kind, lists, pick).is_ok();
        let checkpoints = self.later_picks.get_or_init(|| self.later_picks());
        for checkpoint in checkpoints {
            if open(&checkpoint.before) {
                return None;
            }
            if open(&checkpoint.after) {
                // Each slot takes the other's pick: the later one here now, this one there after it.
                let swap = check_pick_with(catalog, state, kind, lists, &checkpoint.pick).is_ok()
                    && check_pick(catalog, &checkpoint.after, &checkpoint.kind, pick).is_ok();
                return Some(LaterPick {
                    slot: checkpoint.slot.clone(),
                    kind: checkpoint.kind.clone(),
                    pick: checkpoint.pick.clone(),
                    level: state.character_level,
                    swap,
                });
            }
        }
        None
    }

    /// The picks after the slot at its character level, in walk order, each with the state either side of it, in the
    /// Build without the slot's own pick. Empty when no other pick is at that level, without a walk.
    fn later_picks(&self) -> Vec<Checkpoint> {
        let cursor = self.cursor;
        let (build, slot) = (cursor.build, &self.slot);
        let Some(level) = cursor.level_of(slot) else {
            return Vec::new();
        };
        if !build
            .picks
            .keys()
            .any(|other| other != slot && cursor.level_of(other) == Some(level))
        {
            return Vec::new();
        }
        let mut without = build.clone();
        without.picks.remove(slot);
        let index = usize::from(level) - 1;
        // The levels before the slot's are the Build's own: only the slot's pick differs.
        let Some(mut walker) = cursor.paused(index) else {
            return Vec::new();
        };
        walker.on(&without);
        walker.probe = Some(Probe::Later(slot));
        walker.level(level, &without.levels[index]);
        walker.probed.later.unwrap_or_default()
    }
}

/// The walk paused just before a level's class is taken ([`Cursor::until_level`]): the character after the levels
/// before it, which the class picker reads (a class taken keeps its archetype), and each class's verdict
/// ([`try_class`](Self::try_class)).
pub(crate) struct AtLevel<'c, 'a> {
    cursor: &'c Cursor<'a>,
    level: u8,
    state: CharacterState,
}

impl<'a> AtLevel<'_, 'a> {
    /// The character just before the level.
    pub(crate) fn state(&self) -> &CharacterState {
        &self.state
    }

    /// The verdict for taking `class` (`prestige` when it's a prestige class) at the level with `choice`, the
    /// archetype and advancement its edit sets on the class's first level (`try(edit)` for [`Edit::Class`]): the
    /// class table's end ([`Issue::ClassLevelCap`]), then on its first level the walk's class-entry check
    /// ([`class_entry`]: the class's and archetype's prerequisites, one issue), then, at a level the Build has,
    /// whether it breaks a later choice ([`Issue::BreaksLater`]; a level after the last breaks nothing). A prestige
    /// class that asks which classes it advances ([`Engine::asks_advances`]) is open at a level the Build has when
    /// some advancement keeps the Build ([`Cursor::kept_advancements`], which the advancement picker lists). One
    /// past the last level, every class that meets its entry is open without a walk, as before: within the level
    /// cap the advancement picker lists some advancement there too (the agreement test sweeps it); past the cap the
    /// walk refuses the level itself, so it lists none.
    pub(crate) fn try_class(
        &self,
        class: &'a Class,
        prestige: bool,
        choice: Option<&ClassChoice>,
    ) -> Result<(), Issue> {
        let cursor = self.cursor;
        let cap = class_level_cap(class);
        let held = self.state.class_level(&class.key);
        if held >= cap {
            return Err(Issue::ClassLevelCap { cap });
        }
        let archetype = choice.and_then(|choice| choice.archetype.as_ref());
        if held == 0 {
            class_entry(&cursor.engine.catalog, &self.state, class, archetype).issue()?;
        }
        if usize::from(self.level) > cursor.build.levels.len() {
            return Ok(());
        }
        if held == 0 && prestige && most_advanced(class) > 0 {
            let own = choice.map(|choice| &choice.advances);
            let mut kept = cursor.kept_advancements(self.level, class, archetype, own);
            return status(kept.next().is_some());
        }
        cursor.later(&Edit::Class {
            level: self.level,
            class: class.key.clone(),
            choice: choice.cloned(),
        })
    }
}

/// Whether an advancement could meet what `issue` says fails: an unmet prerequisite with a spell level or caster
/// level in it, the only parts of a walk an advancement changes, besides its own check ([`Choice::Advances`]).
fn spellcasting_could_meet(issue: &Issue) -> bool {
    fn reads_spellcasting(unmet: &Unmet) -> bool {
        match unmet {
            Unmet::Condition { needs, .. } => needs.leaves().any(|leaf| {
                matches!(
                    leaf,
                    Prerequisite::SpellLevel { .. }
                        | Prerequisite::SpontaneousSpellLevel { .. }
                        | Prerequisite::CasterLevel(_)
                )
            }),
            Unmet::All(unmet) | Unmet::Any(unmet) | Unmet::AtLeast { failed: unmet, .. } => {
                unmet.iter().any(reads_spellcasting)
            }
            Unmet::MetLater { unmet, .. } => reads_spellcasting(unmet),
        }
    }
    matches!(issue, Issue::Unmet(unmet) if reads_spellcasting(unmet))
}

fn status(keeps: bool) -> Result<(), Issue> {
    if keeps {
        Ok(())
    } else {
        Err(Issue::BreaksLater)
    }
}
