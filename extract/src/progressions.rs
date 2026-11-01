//! A class table's numbers as progressions, and an archetype's full table as the rows where it differs.
//!
//! The guide prints BAB and saves on every row of a class table; the data keeps the progression each column
//! follows and only the save cells it doesn't give. Archetype tables have no number columns at all, and an
//! archetype keeps only the rows where it differs from its base class.

use rules::{
    ArchetypeLevelRow, BabProgression, Class, Save, SaveException, SaveProgression, SpellLevels,
};

/// The BAB and save cells of one table row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Numbers {
    pub bab: u8,
    pub fort: u8,
    pub reflex: u8,
    pub will: u8,
}

impl Numbers {
    fn save(self, save: Save) -> u8 {
        match save {
            Save::Fortitude => self.fort,
            Save::Reflex => self.reflex,
            Save::Will => self.will,
        }
    }
}

/// What a class table's numbers come to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Progressions {
    pub bab: BabProgression,
    pub fort: SaveProgression,
    pub reflex: SaveProgression,
    pub will: SaveProgression,
    /// The save cells the chosen progression doesn't give, as printed.
    pub save_exceptions: Vec<SaveException>,
    /// Levels whose BAB the chosen (closest) progression doesn't give. There's no BAB exception: report them.
    pub bab_misfits: Vec<u8>,
}

/// The progressions a table's `(level, numbers)` rows follow: for each column, the one that misses the fewest
/// cells (on a tie, the first of full/¾/½ and good/poor).
pub fn progressions(rows: &[(u8, Numbers)], prestige: bool) -> Progressions {
    let bab_misses = |progression: BabProgression| -> Vec<u8> {
        rows.iter()
            .filter(|(level, numbers)| numbers.bab != progression.bab(*level))
            .map(|(level, _)| *level)
            .collect()
    };
    let (bab, bab_misfits) = closest(
        [
            BabProgression::Full,
            BabProgression::ThreeQuarters,
            BabProgression::Half,
        ],
        bab_misses,
    );
    let mut save_exceptions = Vec::new();
    let [fort, reflex, will] = Save::ALL.map(|save| {
        let misses = |progression: SaveProgression| -> Vec<u8> {
            rows.iter()
                .filter(|(level, numbers)| numbers.save(save) != progression.save(*level, prestige))
                .map(|(level, _)| *level)
                .collect()
        };
        let (progression, missed) = closest([SaveProgression::Good, SaveProgression::Poor], misses);
        for level in missed {
            let numbers = rows.iter().find(|(l, _)| *l == level).map(|(_, n)| *n);
            if let Some(numbers) = numbers {
                save_exceptions.push(SaveException {
                    level,
                    save,
                    value: numbers.save(save),
                });
            }
        }
        progression
    });
    save_exceptions.sort();
    Progressions {
        bab,
        fort,
        reflex,
        will,
        save_exceptions,
        bab_misfits,
    }
}

/// The candidate whose misses are fewest (the first on a tie), with those misses.
fn closest<P: Copy, const N: usize>(
    candidates: [P; N],
    misses: impl Fn(P) -> Vec<u8>,
) -> (P, Vec<u8>) {
    candidates
        .into_iter()
        .map(|candidate| (candidate, misses(candidate)))
        .reduce(|best, next| {
            if next.1.len() < best.1.len() {
                next
            } else {
                best
            }
        })
        .expect("at least one candidate")
}

/// Turns each archetype's full table (every level, with its spell levels set) into the rows where it differs
/// from the base class: a row with the base row's grants (as multisets) and spell levels goes, and spell levels
/// equal to the base row's are left unset.
pub fn sparse_archetype_rows(class: &mut Class) {
    let base = &class.levels;
    for archetype in &mut class.archetypes {
        archetype.levels.retain_mut(|row| {
            let Some(base) = base.iter().find(|base| base.level == row.level) else {
                return true;
            };
            if row.spell_levels == Some(base.spell_levels()) {
                row.spell_levels = None;
            }
            row.spell_levels.is_some()
                || sorted(&row.features) != sorted(&base.features)
                || row.feats != base.feats
                || sorted(&row.feat_slots) != sorted(&base.feat_slots)
                || sorted(&row.selections) != sorted(&base.selections)
        });
    }
}

/// An archetype's row with everything set, before [`sparse_archetype_rows`].
pub fn full_archetype_row(row: rules::ClassLevelRow) -> ArchetypeLevelRow {
    ArchetypeLevelRow {
        level: row.level,
        spell_levels: Some(SpellLevels {
            arcane: row.arcane_spell_level,
            divine: row.divine_spell_level,
        }),
        features: row.features,
        feats: row.feats,
        feat_slots: row.feat_slots,
        selections: row.selections,
    }
}

fn sorted<T: Ord>(items: &[T]) -> Vec<&T> {
    let mut items: Vec<&T> = items.iter().collect();
    items.sort();
    items
}
