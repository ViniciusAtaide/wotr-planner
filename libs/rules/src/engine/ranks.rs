//! Taking a Mythic rank later than the rank table's level (ticket 171): the levels a rank can be taken at in a
//! Build, each with its edit and whether it can be made now ([`Engine::rank_level_options`]).
//!
//! The rank table's level is the earliest a rank comes at. A Build may place a rank at any later level it has
//! ([`Build::rank_levels`]), and ranks keep their order: a rank placed later takes the ranks above it along, until
//! they are placed later still ([`Engine::rank_level`]). A rank's picks are checked at the level it is taken at,
//! after that level's own picks, and what it gives is held from there, so moving a rank can refuse a choice: a
//! level's pick between the two levels that needs what the rank gives, when it moves later; the rank's own pick that
//! needs a later level's, when it moves back. Such a move is locked, naming the choice, as any other edit a later
//! choice depends on is.
//!
//! A rank's slot refuses a pick whose prerequisite comes at a later level; [`Engine::rank_level_that_opens`] says
//! which later level of the rank would open it, for the pickers to say beside the reason.

use super::walk::{Cursor, before_slot, pick_fits};
use super::{Engine, RankStatus, Sheet};
use crate::build::{Build, Edit, Pick, SlotId};
use crate::choice::{Choice, Issue};

/// One option of a Mythic rank's level picker ([`Engine::rank_level_options`]): a character level the rank can be
/// taken at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RankLevelOption {
    pub level: u8,
    /// Whether the rank is taken at this level now.
    pub held: bool,
    /// Whether this is the earliest the rank can be taken in the Build as it is: the rank table's level, or the
    /// rank below's when that one is taken later. Its edit takes the Build's own placement of the rank away.
    pub earliest: bool,
    /// `Ok` when the move can be made now; [`Issue::BreaksLater`] when it would refuse a choice the Build holds.
    /// What the Build holds is always `Ok`.
    pub status: Result<(), Issue>,
    /// The choices the move would refuse, in walk order; empty for an open option.
    pub needed_by: Vec<Choice>,
    /// Why it is locked, naming those choices (the first three, then how many more): "Trick Feat needs MR 3 earlier
    /// than level 12", or, for a move back, "Shield Focus (Mythic) needs MR 10 later than level 19". `None` for an
    /// open option.
    pub reason: Option<String>,
    /// The ranks above that the move takes with it, each with the level it would then be taken at: those it pushes
    /// later, and those that follow it back. Only ranks the Build reaches.
    pub moves: Vec<(u8, u8)>,
    /// The edit that takes the rank at this level ([`Edit::RankLevel`]).
    pub edit: Edit,
}

/// A pick in a slot that a Mythic rank's pick gives at the rank's level, which a move of a rank would shift
/// ([`Engine::shifted_picks`]). Such a slot is keyed by its place among the slots granted at its level, so when a
/// rank is taken at another level the slot goes with it, and the slots of the ranks after it at either level change
/// place: the pick would end up in another rank's slot, or in none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShiftedPick {
    pub slot: SlotId,
    /// The rank whose pick gives the slot now.
    pub from: u8,
    /// The rank whose pick would give the slot after the move; `None` when the level would have no such slot.
    pub to: Option<u8>,
}

/// How many of the choices a move would refuse its reason names; the rest are counted ("and 4 more").
const NAMED: usize = 3;

impl Engine {
    /// A Mythic rank's level picker: every character level the rank can be taken at, from the earliest it comes at
    /// in the Build as it is (the rank table's level, or the rank below's when that one is taken later) to the
    /// Build's last level, each with the edit that takes it there. Empty for a rank the Build doesn't reach (its
    /// earliest level is past the last level, or a switch ended the ascensions before it) and for one the data
    /// lacks.
    ///
    /// A move that would refuse a choice is [`Issue::BreaksLater`], with the choices and a reason naming them; the
    /// same verdict as [`Engine::allows`] of the option's edit, and for the earliest level as the withdrawal of
    /// [`Choice::RankLevel`]. An option also says which ranks above it would move with it.
    ///
    /// Costs a walk from the rank's earliest level for each option but the one held, and for a locked one two full
    /// walks more.
    pub fn rank_level_options(&self, build: &Build, rank: u8) -> Vec<RankLevelOption> {
        let (Some(floor), Some(held)) =
            (self.rank_floor(build, rank), self.rank_level(build, rank))
        else {
            return Vec::new();
        };
        let last = u8::try_from(build.levels.len()).unwrap_or(u8::MAX);
        let ascensions_end = self.ascensions_end(build);
        if held > last || ascensions_end.is_some_and(|end| rank > end) {
            return Vec::new();
        }
        let cursor = Cursor::new(self, build);
        let failing = std::cell::OnceCell::new();
        (floor..=last)
            .map(|level| {
                let edit = Edit::RankLevel {
                    rank,
                    level: (level > floor).then_some(level),
                };
                let is_held = level == held;
                let status = if is_held {
                    Ok(())
                } else {
                    cursor.try_edit(&edit)
                };
                let mut edited = build.clone();
                self.apply(&mut edited, &edit);
                // A move that would shift a pick says so, whatever else it would refuse.
                let shifted = match status {
                    Ok(()) => Vec::new(),
                    Err(_) => self.shifted_between(build, &edited),
                };
                let needed_by = if status.is_ok() {
                    Vec::new()
                } else if !shifted.is_empty() {
                    (shifted.iter())
                        .map(|pick| Choice::Slot(pick.slot.clone()))
                        .collect()
                } else {
                    let failing = failing.get_or_init(|| self.failing_choices(build));
                    self.refused_by(build, failing, &edit)
                };
                let reason = status.is_err().then(|| {
                    if !shifted.is_empty() {
                        return self.shifted_reason(build, &shifted);
                    }
                    // The first few by name: a rank moved past a Legend's level 20 would refuse every level after.
                    let names = self.choice_names_within(build, &needed_by, NAMED);
                    let verb = if needed_by.len() == 1 {
                        "needs"
                    } else {
                        "need"
                    };
                    let way = if level > held { "earlier" } else { "later" };
                    format!("{names} {verb} MR {rank} {way} than level {level}")
                });
                let moves = self.moved_along(build, &edited, rank);
                RankLevelOption {
                    level,
                    held: is_held,
                    earliest: level == floor,
                    status,
                    needed_by,
                    reason,
                    moves,
                    edit,
                }
            })
            .collect()
    }

    /// The picks an edit would shift: each pick the Build keeps in a slot that a Mythic rank's pick gives at the
    /// rank's level (a feat's extra pick, an option row's slot), when after the edit another rank's pick would give
    /// the slot at that place, or none would. Only an edit that changes the level a rank is taken at shifts any
    /// (a rank moved, or the last level removed from under one). The engine refuses such an edit
    /// ([`Engine::allows`], the level picker's and the withdrawals' verdicts), though the walk of the edited Build
    /// may fail no choice: the pick would silently change owner.
    ///
    /// Costs nothing for an edit that moves no rank, and otherwise two walks.
    pub fn shifted_picks(&self, build: &Build, edit: &Edit) -> Vec<ShiftedPick> {
        let mut edited = build.clone();
        self.apply(&mut edited, edit);
        self.shifted_between(build, &edited)
    }

    /// [`shifted_picks`](Self::shifted_picks), for a Build and the Build an edit makes of it.
    pub(super) fn shifted_between(&self, build: &Build, edited: &Build) -> Vec<ShiftedPick> {
        if self.rank_levels(build).eq(self.rank_levels(edited)) {
            return Vec::new();
        }
        let before = self.rank_slots(build);
        if before.is_empty() {
            return Vec::new();
        }
        let after = self.rank_slots(edited);
        before
            .into_iter()
            .filter(|(slot, _)| build.picks.contains_key(slot) && edited.picks.contains_key(slot))
            .filter_map(|(slot, from)| {
                let to = (after.iter())
                    .find(|(other, _)| *other == slot)
                    .map(|(_, rank)| *rank);
                (to != Some(from)).then_some(ShiftedPick { slot, from, to })
            })
            .collect()
    }

    /// Why a move that would shift picks is locked: "Fast Stealth fills a slot MR 9's pick gives at level 17, which
    /// the move would give to MR 4's: clear it first", or "…, which would move with the rank: clear it first" when
    /// the level would have no such slot.
    fn shifted_reason(&self, build: &Build, shifted: &[ShiftedPick]) -> String {
        let first = &shifted[0];
        let names: Vec<Choice> = (shifted.iter())
            .map(|pick| Choice::Slot(pick.slot.clone()))
            .collect();
        let names = self.choice_names(build, &names[..names.len().min(NAMED)]);
        let (verb, level) = (
            if shifted.len() == 1 { "fills" } else { "fill" },
            first.slot.level().unwrap_or(0),
        );
        let what = match (shifted.len(), first.to) {
            (1, Some(to)) => format!("which the move would give to MR {to}'s"),
            (1, None) => "which would move with the rank".to_owned(),
            _ => "which the move would shift".to_owned(),
        };
        let slots = if shifted.len() == 1 {
            format!("a slot MR {}'s pick gives at level {level}", first.from)
        } else {
            "slots that Mythic ranks' picks give at their levels".to_owned()
        };
        format!("{names} {verb} {slots}, {what}: clear it first")
    }

    /// The ranks above `rank` that taking it at the level `edited` holds would move, each with its new level: those
    /// the Build reaches before or after, and that the ascensions get to.
    fn moved_along(&self, build: &Build, edited: &Build, rank: u8) -> Vec<(u8, u8)> {
        let last = u8::try_from(build.levels.len()).unwrap_or(u8::MAX);
        let ascensions_end = self.ascensions_end(build);
        self.rank_levels(edited)
            .zip(self.rank_levels(build))
            .filter(|((other, to), (_, from))| {
                *other > rank
                    && to != from
                    && (*to <= last || *from <= last)
                    && !ascensions_end.is_some_and(|end| *other > end)
            })
            .map(|(moved, _)| moved)
            .collect()
    }

    /// The ranks a move takes along, as a picker and a hint say them: "MR 2 to level 17, MR 3 to level 17".
    pub fn moves_text(moves: &[(u8, u8)]) -> String {
        let moves: Vec<String> = (moves.iter())
            .map(|(rank, level)| format!("MR {rank} to level {level}"))
            .collect();
        moves.join(", ")
    }

    /// Whether a Build takes a Mythic rank at all: it has the character level the rank comes at
    /// ([`Engine::rank_level`]), and no Path switch ended the ascensions before it (Legend: after MR 8). The
    /// rank's level picker ([`Engine::rank_level_options`]) has levels exactly when this holds.
    pub fn rank_reached(&self, build: &Build, rank: u8) -> bool {
        self.reaches(build, rank) && !self.ascensions_end(build).is_some_and(|end| rank > end)
    }

    /// The rank a Build's switch ends the ascensions at (Legend: MR 8, where the switch itself still comes); `None`
    /// while they go on.
    fn ascensions_end(&self, build: &Build) -> Option<u8> {
        self.chosen_switch(build)
            .and_then(|(_, switch)| switch.ends_ascensions_at())
    }

    /// For a Mythic rank's slot and a pick it refuses for an unmet prerequisite: the earliest later level at which
    /// taking the rank opens the pick, with the ranks above that the move takes along, when there is one (Shield Focus (Mythic) at MR 10, with Shield Focus taken at
    /// level 20: level 20). `None` for a level's slot, a pick the slot takes now or refuses for another reason, and
    /// when no later level of the Build opens it. `sheet` is the Build's.
    ///
    /// The level is one whose move the engine allows, with the pick allowed after it: both are walked. The levels
    /// tried are those at whose end the character meets the pick's own check, so a level only a later rank's grant
    /// would close is missed; that costs a hint, never a wrong one.
    ///
    /// Costs no walk for a pick no later level's state takes, and otherwise one to the slot and one for each level
    /// tried.
    pub fn rank_level_that_opens(
        &self,
        build: &Build,
        sheet: &Sheet,
        slot: &SlotId,
        pick: &Pick,
    ) -> Option<(u8, Vec<(u8, u8)>)> {
        let rank = slot.rank()?;
        let held = self.rank_level(build, rank)?;
        let kind = &sheet.slot(slot)?.kind;
        let mut later = (sheet.levels.iter())
            .filter(|level| level.character_level > held)
            .filter(|level| pick_fits(self, &level.state, kind, pick))
            .peekable();
        // Only a pick the slot refuses where the rank is now.
        later.peek()?;
        let (state, _) = before_slot(self, build, slot)?;
        if pick_fits(self, &state, kind, pick) {
            return None;
        }
        for level in later {
            let moved = Edit::RankLevel {
                rank,
                level: Some(level.character_level),
            };
            if !self.allows(build, &moved) {
                continue;
            }
            let mut edited = build.clone();
            self.apply(&mut edited, &moved);
            let moves = self.moved_along(build, &edited, rank);
            self.apply(
                &mut edited,
                &Edit::Pick {
                    slot: slot.clone(),
                    pick: pick.clone(),
                },
            );
            if self.is_possible(&edited) {
                return Some((level.character_level, moves));
            }
        }
        None
    }

    /// Whether a Mythic rank is taken later than the rank table's level in a sheet's Build: what the sheet marks a
    /// moved rank by.
    pub fn rank_is_moved(&self, rank: u8, character_level: u8) -> bool {
        self.rank_table_level(rank)
            .is_some_and(|earliest| character_level > earliest)
    }

    /// Whether a rank's level picker has anywhere to move it: the Build reaches the rank and has a level after the
    /// earliest the rank comes at. What the sheet shows the move control by, at no walk's cost.
    pub fn rank_can_move(&self, build: &Build, rank: u8, status: RankStatus) -> bool {
        let last = u8::try_from(build.levels.len()).unwrap_or(u8::MAX);
        !self.ascensions_end(build).is_some_and(|end| rank > end)
            && (status != RankStatus::NotReached || rank == super::PATH_SWITCH_RANK)
            && self
                .rank_floor(build, rank)
                .zip(self.rank_level(build, rank))
                .is_some_and(|(floor, held)| held <= last && floor < last)
    }
}
