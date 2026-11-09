//! How close a name an agent gave is to one the planner has, for the hints after a miss: `search`'s note when nothing
//! matches, and `choose`'s closest values for `not_an_option`. Only names at or above [`FLOOR`] are hinted at, so a
//! name that shares one word of two with the given one ("Furious Focus" and "Shield Focus") isn't offered as near.

/// The least closeness a name needs to be hinted at.
pub(crate) const FLOOR: f64 = 0.6;

/// The least likeness by edit distance that reads as a misspelling ("atack" for "attack").
const TYPO: f64 = 0.75;

/// How close `candidate` is to `given`, from 0 to 1: the share of the given words that some word of the candidate
/// starts with or nearly spells, or, when the texts with their spaces dropped are spelt nearly alike ("powerattack"),
/// that likeness. Case and punctuation don't count.
pub(crate) fn closeness(given: &str, candidate: &str) -> f64 {
    let given = words(given);
    let candidate = words(candidate);
    if given.is_empty() || candidate.is_empty() {
        return 0.0;
    }
    let shared = given
        .iter()
        .filter(|word| candidate.iter().any(|other| word_close(word, other)))
        .count();
    let share = shared as f64 / given.len() as f64;
    let likeness = likeness(&given.concat(), &candidate.concat());
    match likeness >= TYPO {
        true => share.max(likeness),
        false => share,
    }
}

/// Whether a candidate's word stands for a given one: it starts with it (a given word of three letters or more), or
/// the two are spelt nearly alike ("atack", "attack").
fn word_close(given: &str, candidate: &str) -> bool {
    given == candidate
        || (given.chars().count() >= 3 && candidate.starts_with(given))
        || likeness(given, candidate) >= TYPO
}

/// 1 minus the edit distance over the longer text's length.
fn likeness(a: &str, b: &str) -> f64 {
    let longer = a.chars().count().max(b.chars().count());
    if longer == 0 {
        return 1.0;
    }
    1.0 - distance(a, b) as f64 / longer as f64
}

/// A text's lower-case words: letters and digits, apostrophes dropped ("Nature's Ally" → `natures`, `ally`).
pub(crate) fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !(c.is_alphanumeric() || c == '\'' || c == '’'))
        .map(|word| {
            word.chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
        .filter(|word| !word.is_empty())
        .collect()
}

/// The edit distance between two texts, ignoring case.
pub(crate) fn distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.to_lowercase().chars().collect();
    let b: Vec<char> = b.to_lowercase().chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (above + 1)
                .min(row[j] + 1)
                .min(diagonal + usize::from(ca != cb));
            diagonal = above;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(given: &str, candidate: &str) -> bool {
        closeness(given, candidate) >= FLOOR
    }

    #[test]
    fn a_name_sharing_one_word_of_two_is_not_close() {
        for candidate in [
            "Shield Focus",
            "Arcane Focus",
            "Armor Focus",
            "weapon-focus",
        ] {
            assert!(!close("Furious Focus", candidate), "{candidate}");
        }
    }

    #[test]
    fn misspellings_spacing_and_word_starts_are_close() {
        assert!(close("improved critcal", "Improved Critical"));
        assert!(close("power-atack", "power-attack"));
        assert!(close("powerattack", "Power Attack"));
        assert!(close("Rogue Talent", "Rogue Talents"));
        assert!(close("improved critical", "Improved Critical (Mythic)"));
    }
}
