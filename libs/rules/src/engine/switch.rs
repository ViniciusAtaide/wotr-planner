//! The path and the Path switch at their ranks: which ones the Build takes, the character levels of the Mythic
//! ranks, and what [`Engine::apply`] drops once a rank isn't reached. What a switch takes away during the walk is
//! the walker's (`walk::ascension`).

use super::{Engine, PATH_CHOICE_RANK, PATH_SWITCH_RANK};
use crate::build::{Build, Edit};
use crate::game_data::{MythicPath, PathSwitch};

impl Engine {
    /// The Build's path, when it's one that can be chosen at MR 3: a path without a switch rule.
    pub(super) fn chosen_path(&self, build: &Build) -> Option<&MythicPath> {
        self.catalog
            .path(build.mythic_path.as_ref()?)
            .filter(|path| path.switch.is_none())
    }

    /// The Build's Path switch with its rule, when it's a path with one, open from the Build's path.
    pub(super) fn chosen_switch(&self, build: &Build) -> Option<(&MythicPath, &PathSwitch)> {
        let path = self.catalog.path(build.path_switch.as_ref()?)?;
        let switch = path.switch.as_ref()?;
        switch
            .open_from(build.mythic_path.as_ref())
            .then_some((path, switch))
    }

    /// Applies an edit as the sheet makes it: [`Build::apply`], then whatever the sheet no longer shows goes, so the
    /// Build holds only what the sheet shows:
    /// - the path and the Path switch once their ranks (MR 3, MR 8) aren't reached: removing level 9 drops the path
    ///   and level 17 the switch, and neither can be set before its rank;
    /// - the picks of the Mythic ranks the Build no longer reaches: removing a level drops the picks of the ranks it
    ///   reached (MR 8 and 9 with level 17), and a switch that ends the ascensions drops the picks of the ranks from
    ///   the switch rank on (Legend: MR 8 to 10).
    ///
    /// The picks a switch takes away at earlier ranks stay, held from their ranks until the switch, as the Mythic
    /// track shows them ([`RankSheet::dropped_slots`](super::RankSheet::dropped_slots)).
    pub fn apply(&self, build: &mut Build, edit: &Edit) {
        build.apply(edit);
        if !self.reaches(build, PATH_CHOICE_RANK) {
            build.mythic_path = None;
        }
        if !self.reaches(build, PATH_SWITCH_RANK) {
            build.path_switch = None;
        }
        let levels = build.levels.len();
        let ascensions_end = self
            .chosen_switch(build)
            .is_some_and(|(_, switch)| !switch.further_ascensions);
        build
            .picks
            .retain(|slot, _| match (slot.level(), slot.rank()) {
                (Some(level), _) => usize::from(level) <= levels,
                (None, Some(rank)) => {
                    !(ascensions_end && rank >= PATH_SWITCH_RANK)
                        && self
                            .catalog
                            .ascension_level(rank)
                            .is_some_and(|level| usize::from(level) <= levels)
                }
                (None, None) => true,
            });
    }

    /// Whether the Build has the character level of a Mythic rank's ascension (whether or not a Legend switch
    /// ended the ascensions before it).
    pub(super) fn reaches(&self, build: &Build, rank: u8) -> bool {
        self.catalog
            .ascension_level(rank)
            .is_some_and(|level| usize::from(level) <= build.levels.len())
    }

    /// The character level of a Mythic rank's ascension; 0 (the whole walk) for a rank the data lacks.
    pub(super) fn rank_level(&self, rank: u8) -> u8 {
        self.catalog.ascension_level(rank).unwrap_or(0)
    }
}
