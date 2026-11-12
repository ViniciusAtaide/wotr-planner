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
        let now: Vec<(u8, u8)> = self.rank_levels(build).collect();
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
                let needed_by = if status.is_ok() {
                    Vec::new()
                } else {
                    let failing = failing.get_or_init(|| self.failing_choices(build));
                    let mut refused: Vec<Choice> = Vec::new();
                    for failure in self.failures(&edited) {
                        if !failing.contains(&failure.choice) && !refused.contains(&failure.choice)
                        {
                            refused.push(failure.choice);
                        }
                    }
                    refused
                };
                let reason = status.is_err().then(|| {
                    // The first few by name: a rank moved past a Legend's level 20 would refuse every level after.
                    let named = &needed_by[..needed_by.len().min(NAMED)];
                    let mut names = self.choice_names(build, named);
                    if needed_by.len() > NAMED {
                        names = format!("{names} and {} more", needed_by.len() - NAMED);
                    }
                    let verb = if needed_by.len() == 1 {
                        "needs"
                    } else {
                        "need"
                    };
                    let way = if level > held { "earlier" } else { "later" };
                    format!("{names} {verb} MR {rank} {way} than level {level}")
                });
                let moves = self
                    .rank_levels(&edited)
                    .zip(&now)
                    .filter(|((other, to), (_, from))| {
                        *other > rank
                            && to != from
                            && (*to <= last || *from <= last)
                            && !ascensions_end.is_some_and(|end| *other > end)
                    })
                    .map(|(moved, _)| moved)
                    .collect();
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

    /// The rank a Build's switch ends the ascensions at (Legend: MR 8, where the switch itself still comes); `None`
    /// while they go on.
    fn ascensions_end(&self, build: &Build) -> Option<u8> {
        self.chosen_switch(build)
            .and_then(|(_, switch)| switch.ends_ascensions_at())
    }

    /// For a Mythic rank's slot and a pick it refuses for an unmet prerequisite: the earliest later level at which
    /// taking the rank opens the pick, when there is one (Shield Focus (Mythic) at MR 10, with Shield Focus taken at
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
    ) -> Option<u8> {
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
            let mut edited = build.clone();
            self.apply(&mut edited, &moved);
            self.apply(
                &mut edited,
                &Edit::Pick {
                    slot: slot.clone(),
                    pick: pick.clone(),
                },
            );
            if self.is_possible(&edited) {
                return Some(level.character_level);
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
