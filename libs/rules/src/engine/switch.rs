//! The first ascension, the path and the Path switch at their ranks: which ones the Build takes, the character levels
//! of the Mythic ranks, and what [`Engine::apply`] drops once a rank isn't reached. What a switch takes away is its
//! rule's ([`PathSwitch::drops`]); the walker (`walk::ascension`) takes it away during the walk.

use super::{Engine, FIRST_ASCENSION_RANK, PATH_CHOICE_RANK, PATH_SWITCH_RANK};
use crate::build::{Build, Edit};
use crate::game_data::{MythicPath, PathSwitch};
use crate::keys::MythicPathKey;

impl Engine {
    /// The Build's path, when it's one that can be chosen at MR 3: a path without a switch rule.
    pub(super) fn chosen_path(&self, build: &Build) -> Option<&MythicPath> {
        self.catalog
            .path(build.mythic_path.as_ref()?)
            .filter(|path| path.switch.is_none())
    }

    /// A path the first ascension can name: one that can be chosen at MR 3 and has an MR 1 row (its path-linked
    /// ability).
    pub(super) fn first_ascension_option(&self, key: &MythicPathKey) -> Option<&MythicPath> {
        self.catalog
            .path(key)
            .filter(|path| path.switch.is_none() && path.row(FIRST_ASCENSION_RANK).is_some())
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
        let path = self.catalog.path(build.path_switch.as_ref()?)?;
        let switch = path.switch.as_ref()?;
        switch
            .open_from(build.mythic_path.as_ref())
            .then_some((path, switch))
    }

    /// Applies an edit as the sheet makes it: [`Build::apply`], then whatever the sheet no longer shows goes, so the
    /// Build holds only what the sheet shows:
    /// - the first ascension, the path and the Path switch once their ranks (MR 1, MR 3, MR 8) aren't reached:
    ///   removing level 6 drops the first ascension, level 9 the path and level 17 the switch, and none can be set
    ///   before its rank;
    /// - the picks of the Mythic ranks the Build no longer reaches: removing a level drops the picks of the ranks it
    ///   reached (MR 8 and 9 with level 17), and a switch that ends the ascensions drops the picks of the ranks from
    ///   the switch rank on (Legend: MR 8 to 10).
    ///
    /// The picks a switch takes away at earlier ranks stay, held from their ranks until the switch, as the Mythic
    /// track shows them ([`RankSheet::dropped_slots`](super::RankSheet::dropped_slots)).
    pub fn apply(&self, build: &mut Build, edit: &Edit) {
        build.apply(edit);
        if !self.reaches(build, FIRST_ASCENSION_RANK) {
            build.first_ascension = None;
        }
        if !self.reaches(build, PATH_CHOICE_RANK) {
            build.mythic_path = None;
        }
        if !self.reaches(build, PATH_SWITCH_RANK) {
            build.path_switch = None;
        }
        let levels = build.levels.len();
        let ascensions_end = self
            .chosen_switch(build)
            .and_then(|(_, switch)| switch.ends_ascensions_at());
        build
            .picks
            .retain(|slot, _| match (slot.level(), slot.rank()) {
                (Some(level), _) => usize::from(level) <= levels,
                (None, Some(rank)) => {
                    !ascensions_end.is_some_and(|end| rank >= end)
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

    /// The character level of a Mythic rank's ascension; 0 (the whole walk) for a rank the data lacks. The ranks
    /// asked about aren't the data's: an edit's slot names whatever rank its Build does, and the engine's own ranks
    /// are constants, so [`GameData::check`](crate::GameData::check) can't vouch for them.
    pub(super) fn rank_level(&self, rank: u8) -> u8 {
        self.catalog.ascension_level(rank).unwrap_or(0)
    }
}
