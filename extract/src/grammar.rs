//! The prerequisite-line grammar.

use rules::{Ability, Alignment, SpellType, Story};

/// A condition as the guide writes it, with names not yet resolved to keys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cond {
    AbilityScore(Ability, u8),
    Bab(u8),
    /// "Fighter level: 8": a Class level in the named class.
    ClassLevel(String, u8),
    CharacterLevel(u8),
    /// "Level 10" on a class page: a Class level in that page's class.
    OwnClassLevel(u8),
    MythicRank(u8),
    /// A feat, feature, skill, race, class or selection option, resolved later. `count` is skill ranks or how
    /// many times a feature was granted ("Persuasion: 6", "Sneak Attack (2d6)").
    Named {
        name: String,
        count: Option<u8>,
    },
    All(Vec<Cond>),
    Any(Vec<Cond>),
    AtLeast(u8, Vec<Cond>),
    /// Spell-level access: can cast spells of this type and level.
    SpellLevel(SpellType, u8),
    /// Any one of these alignments.
    Alignment(Vec<Alignment>),
    /// A story condition ("Vow of Dahak"): never checked, shown as a note.
    Story(Story),
}

/// Why a line didn't parse.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unparsed(pub String);

/// The guide's spellings of the prerequisite label, typos included.
const LABELS: [&str; 3] = ["prerequisites", "prerequsites", "prerequsities"];

/// One bullet of a prestige class's characteristics as its entry requirements, ANDed: "Required feats: A, B",
/// "Required skills: Mobility (4)", "Minimum character level: 6", "Alignment must be Lawful". `None` for a
/// bullet that isn't a requirement.
pub fn parse_requirement(item: &str) -> Option<Result<Vec<Cond>, Unparsed>> {
    let item = item.trim().trim_end_matches([';', '.']).trim();
    let lower = item.to_ascii_lowercase();
    if lower.starts_with("alignment must be") {
        return Some(parse_line(item).map(|cond| vec![cond]));
    }
    if let Some(level) = lower.strip_prefix("minimum character level:") {
        return Some(
            level
                .trim()
                .parse()
                .map(|level| vec![Cond::CharacterLevel(level)])
                .map_err(|_| Unparsed(item.to_owned())),
        );
    }
    let (kind, list) = item.split_once(':')?;
    let kind = kind.trim().to_ascii_lowercase();
    let kind = kind.strip_prefix("required ")?;
    let list = list.trim();
    if kind == "bab" {
        return Some(
            list.trim_start_matches('+')
                .parse()
                .map(|bab| vec![Cond::Bab(bab)])
                .map_err(|_| Unparsed(item.to_owned())),
        );
    }
    if kind == "classes" {
        // A list of alternatives, like a prerequisite line.
        return Some(parse_line(list).map(|cond| vec![cond]));
    }
    Some(
        split_top(list, &[","])
            .into_iter()
            .map(|part| match ranks(part) {
                Some((name, ranks)) => Ok(Cond::Named {
                    name: name.to_owned(),
                    count: Some(ranks),
                }),
                None => parse_line(part),
            })
            .collect(),
    )
}

/// `Name (N)`.
fn ranks(text: &str) -> Option<(&str, u8)> {
    let (name, number) = text.strip_suffix(')')?.rsplit_once(" (")?;
    Some((name.trim(), number.parse().ok()?))
}

/// The condition lines of a paragraph's prerequisites, ANDed: the lines after the label, with a line ending
/// in "or" joined to the next. `None` when the paragraph has no prerequisite label.
pub fn prerequisite_lines(lines: &[String]) -> Option<Vec<String>> {
    let (at, rest) = lines.iter().enumerate().find_map(|(index, line)| {
        let trimmed = line.trim();
        LABELS.iter().find_map(|label| {
            (trimmed.len() >= label.len() && trimmed[..label.len()].eq_ignore_ascii_case(label))
                .then(|| (index, trimmed[label.len()..].trim_start_matches(':').trim()))
        })
    })?;
    let mut conditions: Vec<String> = Vec::new();
    let mut continues = false;
    for line in std::iter::once(rest).chain(lines[at + 1..].iter().map(|line| line.trim())) {
        if line.is_empty() {
            continue;
        }
        match conditions.last_mut() {
            Some(last) if continues => {
                last.push(' ');
                last.push_str(line);
            }
            _ => conditions.push(line.to_owned()),
        }
        continues = line.ends_with(" or");
    }
    Some(conditions)
}

pub fn parse_line(line: &str) -> Result<Cond, Unparsed> {
    let line = line.trim();
    if let Some(cond) = spell_level(line)
        .or_else(|| alignment(line))
        .or_else(|| story(line))
    {
        return Ok(cond);
    }
    let conditions = split_top(line, &[";"]);
    if conditions.len() > 1 {
        return Ok(Cond::All(
            conditions
                .into_iter()
                .map(parse_line)
                .collect::<Result<_, _>>()?,
        ));
    }
    for (prefix, n) in [("one of:", 1), ("two of:", 2), ("three of:", 3)] {
        if line.len() > prefix.len() && line[..prefix.len()].eq_ignore_ascii_case(prefix) {
            let items = split_top(&line[prefix.len()..], &[",", " or "])
                .into_iter()
                .map(parse_conjunction)
                .collect::<Result<_, _>>()?;
            return Ok(if n == 1 {
                Cond::Any(items)
            } else {
                Cond::AtLeast(n, items)
            });
        }
    }
    let alternatives = split_top(line, &[",", " or ", "/"]);
    if alternatives.len() > 1 {
        return Ok(Cond::Any(
            alternatives
                .into_iter()
                .map(parse_conjunction)
                .collect::<Result<_, _>>()?,
        ));
    }
    parse_conjunction(line)
}

/// `A + B`.
fn parse_conjunction(text: &str) -> Result<Cond, Unparsed> {
    let parts = split_top(text, &[" + "]);
    if parts.len() > 1 {
        return Ok(Cond::All(
            parts
                .into_iter()
                .map(parse_atom)
                .collect::<Result<_, _>>()?,
        ));
    }
    parse_atom(text)
}

/// Words that a comma attaches to the name before it ("Beast Totem, Lesser").
const NAME_QUALIFIERS: [&str; 4] = ["lesser", "greater", "improved", "mythic"];

/// Splits `text` at any of `separators` outside parentheses. A comma before a name qualifier doesn't split.
fn split_top<'a>(text: &'a str, separators: &[&str]) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0;
    let mut index = 0;
    let bytes = text.as_bytes();
    while index < bytes.len() {
        match bytes[index] {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if depth == 0
            && let Some(separator) = separators
                .iter()
                .find(|separator| text[index..].starts_with(**separator))
        {
            let after = text[index + separator.len()..].trim();
            let qualifier = *separator == ","
                && NAME_QUALIFIERS
                    .iter()
                    .any(|word| after.eq_ignore_ascii_case(word));
            if !qualifier {
                parts.push(text[start..index].trim());
                index += separator.len();
                start = index;
                continue;
            }
        }
        index += 1;
    }
    parts.push(text[start..].trim());
    parts.retain(|part| !part.is_empty());
    parts
}

fn parse_atom(line: &str) -> Result<Cond, Unparsed> {
    let line = line.trim();
    if let Some(cond) = spell_level(line).or_else(|| alignment(line)) {
        return Ok(cond);
    }
    if let Some((key, number)) = key_number(line) {
        if let Some(ability) = ability(key) {
            return Ok(Cond::AbilityScore(ability, number));
        }
        if key.eq_ignore_ascii_case("base attack bonus") || key.eq_ignore_ascii_case("bab") {
            return Ok(Cond::Bab(number));
        }
        let lower = key.to_ascii_lowercase();
        match lower.as_str() {
            "character level" => return Ok(Cond::CharacterLevel(number)),
            "level" => return Ok(Cond::OwnClassLevel(number)),
            "rank" | "mythic rank" => return Ok(Cond::MythicRank(number)),
            _ => {}
        }
        if let Some(class) = lower.strip_suffix(" level") {
            return Ok(Cond::ClassLevel(key[..class.len()].to_owned(), number));
        }
        return Ok(Cond::Named {
            name: key.to_owned(),
            count: Some(number),
        });
    }
    if let Some((name, dice)) = dice(line) {
        return Ok(Cond::Named {
            name: name.to_owned(),
            count: Some(dice),
        });
    }
    if line.is_empty() {
        return Err(Unparsed(line.to_owned()));
    }
    Ok(Cond::Named {
        name: line.to_owned(),
        count: None,
    })
}

/// `Name (Nd6)`.
fn dice(text: &str) -> Option<(&str, u8)> {
    let (name, rest) = text.strip_suffix("d6)")?.rsplit_once(" (")?;
    Some((name.trim(), rest.parse().ok()?))
}

/// "Ability to cast 1st level divine spells", "2nd level arcane spells", "Ability to cast arcane spells".
fn spell_level(line: &str) -> Option<Cond> {
    let lower = line.to_ascii_lowercase();
    let rest = lower.strip_suffix(" spells")?;
    let rest = rest.strip_prefix("ability to cast").unwrap_or(rest).trim();
    let (level, kind) = match rest.rsplit_once(' ') {
        Some((level, kind)) => (Some(level), kind),
        None => (None, rest),
    };
    let spell_type = match kind {
        "arcane" => SpellType::Arcane,
        "divine" => SpellType::Divine,
        _ => return None,
    };
    let level = match level {
        None => 1,
        Some(level) => {
            let digits: String = level.chars().take_while(char::is_ascii_digit).collect();
            let suffix = level[digits.len()..].trim_start_matches(['s', 't', 'n', 'd', 'r', 'h']);
            if !matches!(suffix.trim(), "level" | "-level") {
                return None;
            }
            digits.parse().ok()?
        }
    };
    Some(Cond::SpellLevel(spell_type, level))
}

const ALIGNMENTS: [(Alignment, &str, &str); 9] = [
    (Alignment::LawfulGood, "lawful", "good"),
    (Alignment::NeutralGood, "neutral", "good"),
    (Alignment::ChaoticGood, "chaotic", "good"),
    (Alignment::LawfulNeutral, "lawful", "neutral"),
    (Alignment::TrueNeutral, "neutral", "neutral"),
    (Alignment::ChaoticNeutral, "chaotic", "neutral"),
    (Alignment::LawfulEvil, "lawful", "evil"),
    (Alignment::NeutralEvil, "neutral", "evil"),
    (Alignment::ChaoticEvil, "chaotic", "evil"),
];

/// "Good alignment", "Alignment must be Lawful Good", "Alignment must be neutral or evil".
fn alignment(line: &str) -> Option<Cond> {
    let lower = line.to_ascii_lowercase();
    let lower = lower.trim_end_matches('.');
    if let Some(excluded) = lower
        .strip_prefix("cannot be ")
        .or_else(|| lower.strip_prefix("alignment must be non-"))
    {
        let Some(Cond::Alignment(excluded)) = alignment_list(excluded) else {
            return None;
        };
        let rest = ALIGNMENTS
            .iter()
            .map(|(alignment, _, _)| *alignment)
            .filter(|alignment| !excluded.contains(alignment))
            .collect::<Vec<_>>();
        return Some(Cond::Alignment(sorted(rest)));
    }
    if lower == "alignment must be within one place of neutral" {
        // One step from true neutral on either axis.
        let near = ALIGNMENTS
            .iter()
            .filter(|(_, law, moral)| *law == "neutral" || *moral == "neutral")
            .map(|(alignment, _, _)| *alignment)
            .collect();
        return Some(Cond::Alignment(sorted(near)));
    }
    let rest = lower
        .strip_prefix("alignment must be ")
        .or_else(|| lower.strip_suffix(" alignment"))?;
    alignment_list(rest)
}

fn sorted(mut alignments: Vec<Alignment>) -> Vec<Alignment> {
    alignments.sort();
    alignments.dedup();
    alignments
}

/// "lawful", "neutral or evil", "neutral, neutral evil or chaotic evil".
fn alignment_list(text: &str) -> Option<Cond> {
    let parts: Vec<Vec<&str>> = text
        .split(" or ")
        .flat_map(|part| part.split(','))
        .map(|part| part.split_whitespace().collect())
        .collect();
    // Next to another alignment, a lone "neutral" is true neutral: "neutral, neutral evil or chaotic evil", and the
    // Assassin's "neutral or evil", true neutral or any evil (user, ticket 21 R6).
    let full = parts.len() > 1;
    let mut matching = Vec::new();
    for words in parts {
        let mut found = false;
        for (alignment, law, moral) in ALIGNMENTS {
            let hit = match words.as_slice() {
                ["true", "neutral"] => alignment == Alignment::TrueNeutral,
                ["neutral"] if full => alignment == Alignment::TrueNeutral,
                [first, second] => law == *first && moral == *second,
                [one] => law == *one || moral == *one,
                _ => false,
            };
            if hit {
                found = true;
                if !matching.contains(&alignment) {
                    matching.push(alignment);
                }
            }
        }
        if !found {
            return None;
        }
    }
    Some(Cond::Alignment(sorted(matching)))
}

/// "Vow of Dahak", "Uncorrupted": a story condition, as the guide prints it.
fn story(line: &str) -> Option<Cond> {
    Story::ALL
        .into_iter()
        .find(|story| {
            line.trim_end_matches('.')
                .eq_ignore_ascii_case(story.name())
        })
        .map(Cond::Story)
}

/// `Key: N` or `Key N`.
fn key_number(text: &str) -> Option<(&str, u8)> {
    let (key, number) = text.rsplit_once([':', ' '])?;
    let number = number.trim().parse().ok()?;
    Some((key.trim().trim_end_matches(':').trim(), number))
}

fn ability(name: &str) -> Option<Ability> {
    Some(match name.to_ascii_lowercase().as_str() {
        "strength" => Ability::Strength,
        "dexterity" => Ability::Dexterity,
        "constitution" => Ability::Constitution,
        "intelligence" => Ability::Intelligence,
        "wisdom" => Ability::Wisdom,
        "charisma" => Ability::Charisma,
        _ => return None,
    })
}
