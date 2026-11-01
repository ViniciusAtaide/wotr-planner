//! Progression tables: class, archetype and prestige level tables, and mythic path rank tables.
//!
//! The guide's quirks handled here: BAB written as "+6 / +1", a row 0 for what taking the class grants (folded
//! into row 1), a `Feat` column marking a single-class character's general feats (ignored), header spellings that
//! vary, footnote digits glued to cell text, "xN" multipliers, and line breaks inside a cell.

/// One row of a progression table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The Class level or Mythic rank.
    pub level: u8,
    pub bab: Option<u8>,
    pub fort: Option<u8>,
    pub reflex: Option<u8>,
    pub will: Option<u8>,
    /// A prestige table's `CL` column shows "+1": this level advances spellcasting.
    pub advances_spellcasting: bool,
    /// The highest spell level with a spells-per-day entry (a number, even 0); 0 when none.
    pub spell_level: u8,
    /// What the row grants, in column order.
    pub grants: Vec<Grant>,
}

/// A cell's text naming something the level grants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grant {
    pub name: String,
    /// The column's header, such as "Special 1" or "Summons".
    pub column: String,
}

/// Cell values that aren't grants: empty cells, and the picks every character gets by rule (the general feat
/// at odd character levels, a mythic ability or feat at each rank).
const NOT_GRANTS: [&str; 5] = ["", "-", "Feat", "Mythic Ability", "Mythic Feat"];

enum Column {
    Level,
    Bab,
    Fort,
    Reflex,
    Will,
    CasterLevel,
    Spells(u8),
    Grants(String),
}

fn column(header: &str) -> Column {
    let header = header.trim();
    match header.to_ascii_lowercase().as_str() {
        "lvl" | "level" | "rank" => Column::Level,
        "ab" | "bab" => Column::Bab,
        "fort" => Column::Fort,
        "reflex" | "ref" => Column::Reflex,
        "will" => Column::Will,
        "cl" => Column::CasterLevel,
        other => match other.parse() {
            Ok(level) if (1..=9).contains(&level) => Column::Spells(level),
            _ => Column::Grants(header.to_owned()),
        },
    }
}

/// Parses a table given its header row and body rows. Row 0 is folded into row 1.
pub fn parse_table(header: &[String], rows: &[Vec<String>]) -> Result<Vec<Row>, String> {
    let columns: Vec<Column> = header.iter().map(|cell| column(cell)).collect();
    if !matches!(columns.first(), Some(Column::Level)) {
        return Err(format!(
            "the first column is {:?}, not a level",
            header.first()
        ));
    }
    let mut parsed: Vec<Row> = Vec::new();
    for cells in rows {
        let mut row = Row {
            level: 0,
            bab: None,
            fort: None,
            reflex: None,
            will: None,
            advances_spellcasting: false,
            spell_level: 0,
            grants: Vec::new(),
        };
        for (column, cell) in columns.iter().zip(cells) {
            let cell = cell.trim();
            match column {
                Column::Level => {
                    row.level = cell
                        .parse()
                        .map_err(|_| format!("level cell {cell:?} is not a number"))?;
                }
                Column::Bab => row.bab = Some(bonus(cell)?),
                Column::Fort => row.fort = Some(bonus(cell)?),
                Column::Reflex => row.reflex = Some(bonus(cell)?),
                Column::Will => row.will = Some(bonus(cell)?),
                Column::CasterLevel => row.advances_spellcasting = cell.starts_with('+'),
                Column::Spells(level) => {
                    // "3", "0", or "1+1" with a domain slot.
                    if cell.starts_with(|c: char| c.is_ascii_digit()) {
                        row.spell_level = row.spell_level.max(*level);
                    }
                }
                Column::Grants(header) => {
                    for line in cell.lines() {
                        push_grants(&mut row.grants, line, header);
                    }
                }
            }
        }
        if row.level == 1
            && let Some(zero) = parsed.last_mut().filter(|last| last.level == 0)
        {
            let mut grants = std::mem::take(&mut zero.grants);
            grants.append(&mut row.grants);
            row.grants = grants;
            parsed.pop();
        }
        parsed.push(row);
    }
    Ok(parsed)
}

/// "+6 / +1" is 6.
fn bonus(cell: &str) -> Result<u8, String> {
    let first = cell.split('/').next().unwrap_or_default().trim();
    first
        .trim_start_matches('+')
        .parse()
        .map_err(|_| format!("bonus cell {cell:?} is not a number"))
}

fn push_grants(grants: &mut Vec<Grant>, text: &str, column: &str) {
    let text = text.trim();
    let summons = column.to_ascii_lowercase().contains("summon");
    let (name, times) = match text.rsplit_once(" x") {
        Some((base, times)) if !summons => match times.parse::<u8>() {
            Ok(times) => (base.trim(), times),
            Err(_) => (strip_footnote(text), 1),
        },
        Some(_) => (text, 1),
        None => (strip_footnote(text), 1),
    };
    if NOT_GRANTS.contains(&name) {
        return;
    }
    for _ in 0..times {
        grants.push(Grant {
            name: name.to_owned(),
            column: column.to_owned(),
        });
    }
}

/// "Rogue Proficiencies1" is "Rogue Proficiencies": digits glued straight onto a letter or `)`.
fn strip_footnote(text: &str) -> &str {
    let without = text.trim_end_matches(|c: char| c.is_ascii_digit());
    match without.chars().last() {
        Some(c) if without.len() < text.len() && (c.is_alphabetic() || c == ')') => without,
        _ => text,
    }
}
