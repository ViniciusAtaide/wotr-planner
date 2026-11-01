//! Progression tables: a header and rows of cell text in, one row per Class level or Mythic rank out.

use extract::tables::{Row, parse_table};

fn table(header: &[&str], rows: &[&[&str]]) -> Result<Vec<Row>, String> {
    let header: Vec<String> = header.iter().map(|cell| (*cell).to_owned()).collect();
    let rows: Vec<Vec<String>> = rows
        .iter()
        .map(|row| row.iter().map(|cell| (*cell).to_owned()).collect())
        .collect();
    parse_table(&header, &rows)
}

fn grants(rows: &[Row]) -> Vec<(u8, Vec<&str>)> {
    rows.iter()
        .map(|row| {
            (
                row.level,
                row.grants.iter().map(|grant| grant.name.as_str()).collect(),
            )
        })
        .collect()
}

#[test]
fn bab_takes_the_first_of_several_attacks() {
    let rows = table(
        &["Lvl", "AB", "Fort", "Reflex", "Will"],
        &[
            &["1", "+0", "+0", "+2", "+0"],
            &["8", "+6 / +1", "+2", "+6", "+2"],
        ],
    )
    .unwrap();
    assert_eq!(rows[1].bab, Some(6));
    assert_eq!(
        (rows[1].fort, rows[1].reflex, rows[1].will),
        (Some(2), Some(6), Some(2))
    );
}

#[test]
fn row_zero_is_folded_into_row_one_and_the_feat_column_is_ignored() {
    let rows = table(
        &["Lvl", "Feat", "Special 1", "Special 2"],
        &[
            &["0", "-", "Rogue Proficiencies", "Trapfinding"],
            &["1", "Feat", "-", "Sneak Attack"],
            &["2", "-", "Rogue Talent", "Evasion"],
        ],
    )
    .unwrap();
    assert_eq!(
        grants(&rows),
        [
            (
                1,
                vec!["Rogue Proficiencies", "Trapfinding", "Sneak Attack"]
            ),
            (2, vec!["Rogue Talent", "Evasion"]),
        ]
    );
}

#[test]
fn header_spellings_vary() {
    let rows = table(&["Level", "Feats", "Special 1"], &[&["1", "Feat", "Hex"]]).unwrap();
    assert_eq!(grants(&rows), [(1, vec!["Hex"])]);
    let rows = table(
        &["Lvl", "BAB", "CL", "Fort", "Reflex", "Will", "Special 1"],
        &[&["1", "+0", "+1", "+0", "+1", "+1", "Ranged Legerdemain"]],
    )
    .unwrap();
    assert_eq!(rows[0].bab, Some(0));
    assert!(rows[0].advances_spellcasting);
}

#[test]
fn footnote_digits_glued_to_a_cell_are_dropped() {
    let rows = table(
        &["Lvl", "Feat", "Special 1", "Special 2"],
        &[&["1", "Feat", "Rogue Proficiencies1", "Domain Ability (1)2"]],
    )
    .unwrap();
    assert_eq!(
        grants(&rows),
        [(1, vec!["Rogue Proficiencies", "Domain Ability (1)"])]
    );
}

#[test]
fn a_multiplier_repeats_a_grant_except_for_summons() {
    let rows = table(
        &["Rank", "Abilities", "Feats", "Special 1", "Summons"],
        &[&[
            "3",
            "Mythic Ability",
            "Sneak Attack x2",
            "Mythic Trick",
            "Astral Devas x4",
        ]],
    )
    .unwrap();
    // The general mythic picks are rules, not grants.
    assert_eq!(
        grants(&rows),
        [(
            3,
            vec![
                "Sneak Attack",
                "Sneak Attack",
                "Mythic Trick",
                "Astral Devas x4"
            ]
        )]
    );
}

#[test]
fn spells_per_day_give_the_highest_castable_spell_level() {
    let rows = table(
        &["Lvl", "AB", "Fort", "Reflex", "Will", "1", "2", "3"],
        &[
            &["1", "+0", "+0", "+0", "+2", "1", "-", "-"],
            &["3", "+1", "+1", "+1", "+3", "2", "1", "-"],
            &["4", "+2", "+1", "+1", "+4", "3", "2", "0"],
            // A domain slot on top: "1+1".
            &["5", "+2", "+1", "+1", "+4", "3+1", "2+1", "1+1"],
        ],
    )
    .unwrap();
    let levels: Vec<u8> = rows.iter().map(|row| row.spell_level).collect();
    assert_eq!(levels, [1, 2, 3, 3]);
}

#[test]
fn a_cell_with_a_line_break_holds_two_grants() {
    let rows = table(
        &["Lvl", "Feat", "Special 1"],
        &[&["1", "-", "Armor Proficiency\nSturdy"]],
    )
    .unwrap();
    assert_eq!(grants(&rows), [(1, vec!["Armor Proficiency", "Sturdy"])]);
}
