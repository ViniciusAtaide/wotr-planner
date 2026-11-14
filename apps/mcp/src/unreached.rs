//! Why a Mythic rank has no level in a Build, said once for `options`, `choose`, `validate` and `withdraw`: there is
//! no such rank, the Build has too few levels to reach it, or a Legend Path switch ended the ascensions before it.

use rules::{Build, Engine};

/// Why a rank has no level in a Build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Unreached {
    /// The Mythic ranks are 1 to 10.
    NoSuchRank,
    /// The Build has fewer levels than the rank's: it comes at `comes_at` at the earliest.
    TooFewLevels { comes_at: u8, levels: usize },
    /// A Legend switch ends the ascensions at MR 8, before the rank.
    Legend,
}

impl Unreached {
    /// The cause, when the Build doesn't reach the rank; `None` for a rank it reaches. The level the rank comes at
    /// is told from `build` as it is: [`Engine::rank_floor`], the rank table's level or the rank below's.
    pub(crate) fn of(engine: &Engine, build: &Build, rank: u8) -> Option<Self> {
        let Some(table) = engine.rank_table_level(rank) else {
            return Some(Self::NoSuchRank);
        };
        if engine.rank_reached(build, rank) {
            return None;
        }
        let comes_at = engine.rank_floor(build, rank).unwrap_or(table);
        let levels = build.levels.len();
        Some(match usize::from(comes_at) > levels {
            true => Self::TooFewLevels { comes_at, levels },
            false => Self::Legend,
        })
    }

    /// The reason: "The Build doesn't reach MR 5, so {consequence}: MR 5 comes at level 14 at the earliest, and the
    /// Build has 13 levels." (or the Legend cause); "There is no MR 11: the Mythic ranks are 1 to 10." for none.
    pub(crate) fn reason(self, rank: u8, consequence: &str) -> String {
        match self {
            Self::NoSuchRank => format!("There is no MR {rank}: the Mythic ranks are 1 to 10."),
            Self::TooFewLevels { comes_at, levels } => format!(
                "The Build doesn't reach MR {rank}, so {consequence}: MR {rank} comes at level {comes_at} at the \
                 earliest, and the Build has {}.",
                crate::words::levels(levels)
            ),
            Self::Legend => format!(
                "The Build doesn't reach MR {rank}, so {consequence}: its Path switch (Legend) ends the ascensions \
                 at MR {}.",
                rules::PATH_SWITCH_RANK
            ),
        }
    }

    /// The consequence for a rank's level address (`options`, `choose`, `validate`).
    pub(crate) fn no_level(rank: u8) -> String {
        format!("`mythic.{rank}.level` has no level to take it at")
    }
}
