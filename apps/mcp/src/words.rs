//! Small wording helpers shared by the tools' texts.

/// One word or the other, by a count.
pub(crate) fn plural(count: usize, one: &'static str, many: &'static str) -> &'static str {
    match count {
        1 => one,
        _ => many,
    }
}

/// A count of levels: "1 level", "13 levels".
pub(crate) fn levels(count: usize) -> String {
    format!("{count} {}", plural(count, "level", "levels"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_count_of_levels_is_singular_only_for_one() {
        assert_eq!(levels(0), "0 levels");
        assert_eq!(levels(1), "1 level");
        assert_eq!(levels(13), "13 levels");
    }
}
