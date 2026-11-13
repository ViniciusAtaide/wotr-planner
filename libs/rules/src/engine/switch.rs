//! The first ascension, the path and the Path switch at their ranks: which ones the Build takes, the character levels
//! of the Mythic ranks (the rank table's, or the later ones a Build places them at), and what [`Engine::apply`] drops once a rank isn't reached. What a switch takes away is its
//! rule's ([`PathSwitch::drops`]); the walker (`walk::ascension`) takes it away during the walk.

use super::{Engine, FIRST_ASCENSION_RANK, PATH_CHOICE_RANK, PATH_SWITCH_RANK};
use crate::build::{Build, Edit, SlotId};
use crate::choice::{Choice, Issue};
use crate::game_data::{MythicPath, PathSwitch};
use crate::keys::MythicPathKey;

impl Engine {
    /// The Build's path, when it's one that can be chosen at MR 3: a path without a switch rule.
    pub(super) fn chosen_path(&self, build: &Build) -> Option<&MythicPath> {
        self.path_fit(build.mythic_path.as_ref()?).ok()
    }

    /// Whether a path can be chosen at MR 3: [`Issue::UnknownKey`] for a key the data lacks, [`Issue::NotAllowed`]
    /// for a path only a switch takes. The one rule [`chosen_path`](Self::chosen_path) and the walk share.
    pub(super) fn path_fit(&self, key: &MythicPathKey) -> Result<&MythicPath, Issue> {
        let path = self.catalog.path(key).ok_or(Issue::UnknownKey)?;
        match path.switch {
            None => Ok(path),
            Some(_) => Err(Issue::NotAllowed),
        }
    }

    /// A path the first ascension can name: one that can be chosen at MR 3 and has an MR 1 row (its path-linked
    /// ability).
    pub(super) fn first_ascension_option(&self, key: &MythicPathKey) -> Option<&MythicPath> {
        self.first_ascension_fit(key).ok()
    }

    /// Whether the first ascension can name a path: [`Issue::UnknownKey`] for a key the data lacks,
    /// [`Issue::NotAllowed`] for one that can't be chosen at MR 3 or has no MR 1 row. The one rule
    /// [`first_ascension_option`](Self::first_ascension_option) and the walk share.
    pub(super) fn first_ascension_fit(&self, key: &MythicPathKey) -> Result<&MythicPath, Issue> {
        let path = self.path_fit(key)?;
        match path.row(FIRST_ASCENSION_RANK) {
            Some(_) => Ok(path),
            None => Err(Issue::NotAllowed),
        }
    }

    /// The path whose MR 1 and 2 rows the Build takes: its first ascension's, when it can name one, or without one
    /// of its own, the path it chose at MR 3 (once it reaches MR 3: a path set before its rank isn't chosen).
    pub(super) fn first_ascension_path(&self, build: &Build) -> Option<&MythicPath> {
        match &build.first_ascension {
            Some(key) => self.first_ascension_option(key),
            None => self
                .chosen_path(build)
                .filter(|_| self.reaches(build, PATH_CHOICE_RANK)),
        }
    }

    /// The Build's Path switch with its rule, when it's a path with one, open from the Build's path.
    pub(super) fn chosen_switch(&self, build: &Build) -> Option<(&MythicPath, &PathSwitch)> {
        self.switch_fit(build, build.path_switch.as_ref()?).ok()
    }

    /// Whether a Build can take a Path switch: [`Issue::UnknownKey`] for a key the data lacks,
    /// [`Issue::NotAllowed`] for a path with no switch rule, [`Issue::NotFromPath`] for one not open from the Build's
    /// path. The one rule [`chosen_switch`](Self::chosen_switch) and the walk share.
    pub(super) fn switch_fit(
        &self,
        build: &Build,
        key: &MythicPathKey,
    ) -> Result<(&MythicPath, &PathSwitch), Issue> {
        let path = self.catalog.path(key).ok_or(Issue::UnknownKey)?;
        let switch = path.switch.as_ref().ok_or(Issue::NotAllowed)?;
        if switch.open_from(build.mythic_path.as_ref()) {
            Ok((path, switch))
        } else {
            Err(Issue::NotFromPath {
                from: switch.from.clone(),
            })
        }
    }

    /// Applies an edit as the sheet makes it: [`Build::apply`], then whatever the sheet no longer shows goes, so the
    /// Build holds only what the sheet shows:
    /// - the first ascension, the path and the Path switch once their ranks (MR 1, MR 3, MR 8) aren't reached:
    ///   removing level 6 drops the first ascension, level 9 the path and level 17 the switch, and none can be set
    ///   before its rank;
    /// - the picks of the Mythic ranks the Build no longer reaches: removing a level drops the picks of the ranks it
    ///   reached (MR 8 and 9 with level 17), and a switch that ends the ascensions drops the picks of the ranks from
    ///   the switch rank on (Legend: MR 8 to 10);
    /// - a rank's later level ([`Build::rank_levels`]) that isn't one: at or before the rank table's level, past the
    ///   last level, or of a rank after the one a switch ends the ascensions at. Removing the level a rank was
    ///   moved to puts the rank where it comes without that move; when the Build still has that level, the rank
    ///   keeps the picks (and the path or switch chosen at it) that are still legal there, and loses the others.
    ///
    /// The picks a switch takes away at earlier ranks stay, held from their ranks until the switch, as the Mythic
    /// track shows them ([`RankSheet::dropped_slots`](super::RankSheet::dropped_slots)).
    pub fn apply(&self, build: &mut Build, edit: &Edit) {
        build.apply(edit);
        // A rank's later level that isn't one goes first, so each rank is judged by where it comes without it: a
        // rank moved to a level the Build no longer has comes where it would without that move (its earliest level,
        // or a later one a rank below still holds it at).
        let levels = build.levels.len();
        let before: Vec<(u8, u8)> = self.rank_levels(build).collect();
        build.rank_levels.retain(|rank, level| {
            self.catalog
                .ascension_level(*rank)
                .is_some_and(|earliest| *level > earliest)
                && usize::from(*level) <= levels
        });
        if !self.reaches(build, FIRST_ASCENSION_RANK) {
            build.first_ascension = None;
        }
        if !self.reaches(build, PATH_CHOICE_RANK) {
            build.mythic_path = None;
        }
        if !self.reaches(build, PATH_SWITCH_RANK) {
            build.path_switch = None;
        }
        let ascensions_end = self
            .chosen_switch(build)
            .and_then(|(_, switch)| switch.ends_ascensions_at());
        build
            .rank_levels
            .retain(|rank, _| !ascensions_end.is_some_and(|end| *rank > end));
        let placed: Vec<(u8, u8)> = self.rank_levels(build).collect();
        build
            .picks
            .retain(|slot, _| match (slot.level(), slot.rank()) {
                (Some(level), _) => usize::from(level) <= levels,
                (None, Some(rank)) => {
                    !ascensions_end.is_some_and(|end| rank >= end)
                        && placed
                            .iter()
                            .any(|&(placed, level)| placed == rank && usize::from(level) <= levels)
                }
                (None, None) => true,
            });
        // The ranks that came at a level the Build no longer has and still come, earlier: each keeps the picks that
        // are still legal where it comes now, and loses the others (a mythic feat whose feat went with the level).
        // A pick that goes can take another with it, so the walk is asked again until none fails.
        let returned: Vec<u8> = (before.iter().zip(&placed))
            .filter(|((_, was), (_, is))| usize::from(*was) > levels && usize::from(*is) <= levels)
            .map(|((rank, _), _)| *rank)
            .collect();
        while !returned.is_empty() {
            let failing: Vec<SlotId> = (self.failures(build).into_iter())
                .filter_map(|failure| match failure.choice {
                    Choice::Slot(slot) => Some(slot),
                    _ => None,
                })
                .filter(|slot| slot.rank().is_some_and(|rank| returned.contains(&rank)))
                .collect();
            if failing.is_empty() {
                break;
            }
            for slot in failing {
                build.picks.remove(&slot);
            }
        }
    }

    /// The character level each Mythic rank of the data is taken at in a Build, in rank order (ticket 171): the
    /// level the Build places it at ([`Build::rank_levels`]) or else the rank table's, and never before the rank
    /// below it, so a rank taken later takes the ranks above along until they are placed later still.
    pub(super) fn rank_levels<'a>(
        &'a self,
        build: &'a Build,
    ) -> impl Iterator<Item = (u8, u8)> + 'a {
        let mut floor = 0;
        (self.catalog.data().progression.mythic_ascensions.iter()).map(move |ascension| {
            let placed = build.rank_levels.get(&ascension.rank).copied();
            floor = floor
                .max(ascension.character_level)
                .max(placed.unwrap_or(0));
            (ascension.rank, floor)
        })
    }

    /// The character level a Mythic rank is taken at in a Build: the rank table's, or a later one the Build places
    /// it at, or the rank below's when that one comes later still (ticket 171). `None` for a rank the data lacks.
    /// Whether the Build has that level is another question ([`Sheet::ranks`](super::Sheet::ranks)).
    pub fn rank_level(&self, build: &Build, rank: u8) -> Option<u8> {
        self.rank_levels(build)
            .find(|&(placed, _)| placed == rank)
            .map(|(_, level)| level)
    }

    /// The rank table's character level of a Mythic rank: the earliest it is ever taken. `None` for a rank the
    /// data lacks.
    pub fn rank_table_level(&self, rank: u8) -> Option<u8> {
        self.catalog.ascension_level(rank)
    }

    /// The earliest a Mythic rank can be taken in a Build as it is: the rank table's level, or the rank below's
    /// when that one is taken later. `None` for a rank the data lacks.
    pub fn rank_floor(&self, build: &Build, rank: u8) -> Option<u8> {
        let earliest = self.catalog.ascension_level(rank)?;
        let below = self
            .rank_levels(build)
            .take_while(|&(placed, _)| placed < rank)
            .last()
            .map_or(0, |(_, level)| level);
        Some(earliest.max(below))
    }

    /// Whether the Build has the character level a Mythic rank's ascension is taken at (whether or not a Legend
    /// switch ended the ascensions before it).
    pub(super) fn reaches(&self, build: &Build, rank: u8) -> bool {
        self.rank_level(build, rank)
            .is_some_and(|level| usize::from(level) <= build.levels.len())
    }

    /// The character level whose walk first changes with what a Mythic rank holds; 0 (the whole walk) for a rank the
    /// data lacks. The ranks asked about aren't the data's: an edit's slot names whatever rank its Build does, and
    /// the engine's own ranks are constants, so [`GameData::check`](crate::GameData::check) can't vouch for them.
    pub(super) fn rank_walk_level(&self, build: &Build, rank: u8) -> u8 {
        self.rank_level(build, rank).unwrap_or(0)
    }
}
