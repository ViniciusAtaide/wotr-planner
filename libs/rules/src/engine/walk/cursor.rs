//! The cursor: a Build's walk, paused where a picker asks, which judges each candidate there. The pickers keep only
//! presentation (which candidates, grouped and named); every verdict is the walk's.
//!
//! - [`Cursor::until_slot`] pauses the walk just before a slot is filled ([`AtSlot`]), and
//!   [`Cursor::until_level`] just before a level's class ([`AtLevel`]). Each shows the character's state there,
//!   which the pickers read to present their options.
//! - [`AtSlot::try_pick`], [`AtLevel::try_class`] and [`Cursor::try_edit`] give a candidate's verdict,
//!   `Result<(), Issue>`: the choice's own check at the pause, in the mode it needs (the walk's own checks, in
//!   `judge`), then whether it breaks a later choice ([`Issue::BreaksLater`]).
//!
//! The walk can't resume mid-level ([`Walker::level`] works a level's slots out at its start), so the cursor keeps
//! the walk paused before each level, filled as far as the questions asked need, and a candidate's later choices are
//! walked from the start of the first level it changes, on the edited Build. Those walks keep no sheet and stop at
//! the first choice that fails. [`Engine::allows`] and the withdrawals ask the same walks ([`Cursor::keeps`]).

use std::cell::{OnceCell, RefCell};
use std::collections::HashSet;

use super::judge::{check_pick, check_pick_with, class_entry};
use super::level::advancements;
use super::{Checkpoint, Probe, Walker};
use crate::build::{Build, ClassChoice, Edit, Pick, SlotId};
use crate::choice::{Choice, Issue, LaterPick, Unmet};
use crate::engine::{
    Engine, FIRST_ASCENSION_RANK, PATH_CHOICE_RANK, PATH_SWITCH_RANK, SkillPoints, SlotKind,
    class_level_cap,
};
use crate::game_data::FeatList;
use crate::keys::FeatKey;
use crate::state::CharacterState;

/// A Build's walk, paused before each character level as far as the questions asked of it need. Build one per
/// picker (or per question), for the Build it judges.
pub(crate) struct Cursor<'a> {
    engine: &'a Engine,
    build: &'a Build,
    /// `paused[i]` is the walk just before character level `i + 1` (after the Build start); the one after the last
    /// level is the last there can be. Filled as far as a question has needed.
    paused: RefCell<Vec<Walker<'a>>>,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(engine: &'a Engine, build: &'a Build) -> Self {
        Self {
            engine,
            build,
            paused: RefCell::new(Vec::new()),
        }
    }

    // ---- Pausing ----------------------------------------------------------------------------------------------

    /// The walk paused just before a level's class is taken (`until(level)`): the character after the levels before
    /// it; one past the last level gives the character after the last. `None` for level 0 and past that.
    pub(crate) fn until_level(&self, level: u8) -> Option<AtLevel<'_, 'a>> {
        let index = usize::from(level).checked_sub(1)?;
        let state = self.with_paused(index, |walker| walker.state.clone())?;
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

    /// The character level whose walk fills a slot: its own, or its Mythic rank's ascension's. `None` for a rank the
    /// data lacks.
    fn level_of(&self, slot: &SlotId) -> Option<u8> {
        slot.level()
            .or_else(|| self.engine.catalog.ascension_level(slot.rank()?))
    }

    /// The walk just before character level `index + 1`; `None` past the one after the last level.
    fn paused(&self, index: usize) -> Option<Walker<'a>> {
        self.with_paused(index, Walker::clone)
    }

    /// What `read` reads off the walk just before character level `index + 1`, without cloning it; `None` past the
    /// one after the last level.
    fn with_paused<T>(&self, index: usize, read: impl FnOnce(&Walker<'a>) -> T) -> Option<T> {
        self.walk_to(index)
            .then(|| self.paused.borrow().get(index).map(read))
            .flatten()
    }

    /// Walks as far as just before character level `index + 1`; `false` past the one after the last level.
    fn walk_to(&self, index: usize) -> bool {
        let build = self.build;
        if index > build.levels.len() {
            return false;
        }
        let mut paused = self.paused.borrow_mut();
        if paused.is_empty() {
            let mut walker = Walker::new(self.engine, build).without_sheet();
            walker.start();
            paused.push(walker);
        }
        while paused.len() <= index {
            let walked = paused.len() - 1;
            let mut walker = paused[walked].clone();
            walker.level(
                u8::try_from(walked + 1).unwrap_or(u8::MAX),
                &build.levels[walked],
            );
            paused.push(walker);
        }
        true
    }

    // ---- Verdicts ---------------------------------------------------------------------------------------------

    /// An edit's verdict (`try(edit)`): its own check, then whether it breaks a later choice. The own check is the
    /// walk's: a Build start's faith and alignment ([`Engine::faith_misfit`]); the first ascension, path and switch
    /// the Build can take ([`Engine::first_ascension_fit`], [`Engine::path_fit`], [`Engine::switch_fit`]); a pick
    /// in its slot and a class at its level, at the paused walk ([`AtSlot::try_pick`], [`AtLevel::try_class`]: a
    /// picker pauses once for all its candidates). An ability increase and skill ranks have none beyond what the
    /// steppers' range keeps.
    pub(crate) fn try_edit(&self, edit: &Edit) -> Result<(), Issue> {
        let engine = self.engine;
        match edit {
            Edit::Pick { slot, pick } => {
                return self
                    .until_slot(slot)
                    .ok_or(Issue::NotReached)?
                    .try_pick(pick);
            }
            Edit::Class { level, .. } => {
                return self
                    .until_level(*level)
                    .ok_or(Issue::NotReached)?
                    .try_class(edit);
            }
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
    pub(crate) fn keeps(&self, edit: &Edit, allowed: Vec<Choice>) -> bool {
        let mut edited = self.build.clone();
        self.engine.apply(&mut edited, edit);
        &edited == self.build || self.walks(edit, &edited, allowed)
    }

    /// Whether the walk of `edited` (the Build with `edit` applied) fails no choice but the `allowed` ones, even
    /// when `edit` changes nothing: then it's the Build's own walk.
    fn walks(&self, edit: &Edit, edited: &Build, allowed: Vec<Choice>) -> bool {
        let first = self.first_level(edit);
        match first.checked_sub(1) {
            None => Walker::new(self.engine, edited)
                .without_sheet()
                .holds(0, allowed),
            Some(index) => {
                let Some(mut walker) = self.paused(usize::from(index)) else {
                    // Nothing after the last level depends on an edit past it (Remove level, a name).
                    return true;
                };
                walker.build = edited;
                walker.holds(first, allowed)
            }
        }
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
        let mut edited = self.build.clone();
        self.engine.apply(&mut edited, edit);
        status(self.walks(edit, &edited, edit.choices()))
    }

    /// The first character level whose walk an edit can change; 0 for the Build start, and past the end for an
    /// edit that changes no level.
    fn first_level(&self, edit: &Edit) -> u8 {
        let (engine, build) = (self.engine, self.build);
        match edit {
            Edit::Pick { slot, .. } | Edit::Clear(slot) => match (slot.level(), slot.rank()) {
                (Some(level), _) => level,
                (None, Some(rank)) => engine.rank_level(rank),
                (None, None) => 0,
            },
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
            Edit::FirstAscension(_) => engine.rank_level(FIRST_ASCENSION_RANK),
            // Without a first ascension of its own, the Build takes the MR 3 path's, from MR 1.
            Edit::MythicPath(_) if build.first_ascension.is_none() => {
                engine.rank_level(FIRST_ASCENSION_RANK)
            }
            Edit::MythicPath(_) => engine.rank_level(PATH_CHOICE_RANK),
            Edit::PathSwitch(_) => engine.rank_level(PATH_SWITCH_RANK),
            Edit::RemoveLevel | Edit::SetName(_) => u8::MAX,
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
        let level = cursor.level_of(slot);
        if !build
            .picks
            .keys()
            .any(|other| other != slot && cursor.level_of(other) == level)
        {
            return Vec::new();
        }
        let Some(level) = level else {
            return Vec::new();
        };
        let mut without = build.clone();
        without.picks.remove(slot);
        let index = usize::from(level) - 1;
        // The levels before the slot's are the Build's own: only the slot's pick differs.
        let Some(mut walker) = cursor.paused(index) else {
            return Vec::new();
        };
        walker.build = &without;
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

impl AtLevel<'_, '_> {
    /// The character just before the level.
    pub(crate) fn state(&self) -> &CharacterState {
        &self.state
    }

    /// A class's verdict at the level (`try(edit)` for [`Edit::Class`]): the class table's end
    /// ([`Issue::ClassLevelCap`]), then on its first level the walk's class-entry check ([`class_entry`]: the
    /// class's and archetype's prerequisites, one issue), then, at a level the Build has, whether it breaks a later
    /// choice ([`Issue::BreaksLater`]; a level after the last breaks nothing). A prestige class that asks which
    /// classes it advances ([`Engine::asks_advances`]) keeps the Build when some advancement does
    /// ([`advancements`]): the edit's own first, then the rest, each walked with nothing excused, so its advancement
    /// is judged as the walk's class entry judges it. The advancement picker ([`Engine::advances_options`]) then
    /// lists those.
    pub(crate) fn try_class(&self, edit: &Edit) -> Result<(), Issue> {
        let cursor = self.cursor;
        let Edit::Class {
            level,
            class: key,
            choice,
        } = edit
        else {
            return cursor.try_edit(edit);
        };
        if *level != self.level {
            return cursor.try_edit(edit);
        }
        let (engine, build) = (cursor.engine, cursor.build);
        let (class, _) = engine.catalog.class(key).ok_or(Issue::UnknownKey)?;
        let cap = class_level_cap(class);
        let held = self.state.class_level(key);
        if held >= cap {
            return Err(Issue::ClassLevelCap { cap });
        }
        if held == 0 {
            let archetype = choice.as_ref().and_then(|choice| choice.archetype.as_ref());
            class_entry(&engine.catalog, &self.state, class, archetype).issue()?;
        }
        if usize::from(*level) > build.levels.len() {
            return Ok(());
        }
        let Some(choice) = choice
            .as_ref()
            .filter(|_| engine.asks_advances(build, *level, key))
        else {
            return cursor.later(edit);
        };
        let with = |advances| Edit::Class {
            level: *level,
            class: key.clone(),
            choice: Some(ClassChoice {
                archetype: choice.archetype.clone(),
                advances,
            }),
        };
        let others = advancements(build, *level, class)
            .into_iter()
            .filter(|advances| advances != &choice.advances);
        let kept = std::iter::once(choice.advances.clone())
            .chain(others)
            .any(|advances| cursor.keeps(&with(advances), Vec::new()));
        status(kept)
    }
}

fn status(keeps: bool) -> Result<(), Issue> {
    if keeps {
        Ok(())
    } else {
        Err(Issue::BreaksLater)
    }
}
