//! `compare`: one to four Builds side by side, each in short, then only what differs between them (one Build: its
//! summary alone). Everything is read from each Build's sheet as `sheet` writes it (`sheet::sheet_of` on
//! [`rules::Engine::evaluate`]'s sheet), so a difference here is one the two sheets show; nothing is ranked.

use std::collections::BTreeSet;

use rules::{CharacterState, Sheet};

use crate::attack;
use crate::code::open;
use crate::error::ToolError;
use crate::planner::Planner;
use crate::shapes::{
    BuildSummary, ClassSummary, CompareArgs, CompareResult, FeaturesDiff, LevelDiff, LevelLine,
    Named, NumberDiff, PickDiff, PickLine, RankDiff, RankLine, SheetResult,
};
use crate::sheet::sheet_of;

/// The fewest and most Builds `compare` takes.
const BUILDS: std::ops::RangeInclusive<usize> = 1..=4;

pub(crate) fn compare(planner: &Planner, args: CompareArgs) -> Result<CompareResult, ToolError> {
    let count = args.builds.len();
    if !BUILDS.contains(&count) {
        return Err(ToolError::invalid_arguments(format!(
            "`compare` takes {} to {} Build codes; {count} given.",
            BUILDS.start(),
            BUILDS.end()
        )));
    }
    let builds = args
        .builds
        .iter()
        .enumerate()
        .map(|(i, code)| {
            open(planner, code).map_err(|error| ToolError {
                reason: format!("builds[{i}]: {}", error.reason),
                ..error
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let engine = &planner.engine;
    let weapon = attack::weapon_argument(engine, args.weapon.as_deref())?;
    let sheets: Vec<Sheet> = builds.iter().map(|build| engine.evaluate(build)).collect();
    let lines: Vec<SheetResult> = builds
        .iter()
        .zip(&sheets)
        .map(|(build, sheet)| sheet_of(planner, build, sheet, false))
        .collect();

    let path_keys: Vec<BTreeSet<&str>> = lines
        .iter()
        .map(|line| {
            line.ranks
                .iter()
                .flat_map(|rank| &rank.features)
                .map(|feature| feature.key.as_str())
                .collect()
        })
        .collect();
    let level_count = lines.iter().map(|line| line.levels.len()).max();
    let levels = (0..level_count.unwrap_or(0))
        .filter_map(|i| level_diff(&per_build(&lines, |line| line.levels.get(i)), &path_keys))
        .collect();
    let rank_count = lines.iter().map(|line| line.ranks.len()).max().unwrap_or(0);
    let ranks = (0..rank_count)
        .filter_map(|i| rank_diff(&per_build(&lines, |line| line.ranks.get(i))))
        .collect();
    let path_features = (0..rank_count)
        .filter_map(|i| {
            let at = per_build(&lines, |line| line.ranks.get(i));
            let rank = at.iter().flatten().next()?.rank;
            let only = only_some(
                &at.iter()
                    .map(|line| line.map(|line| &line.features[..]))
                    .collect::<Vec<_>>(),
            );
            (!only.is_empty()).then_some(FeaturesDiff { rank, only })
        })
        .collect();

    Ok(CompareResult {
        builds: builds
            .iter()
            .zip(&sheets)
            .zip(&lines)
            .map(|((build, sheet), line)| {
                let mut summary = summary(planner, build, sheet, line);
                if let (Some(weapon), Some(last)) = (&weapon, sheet.levels.last()) {
                    summary.attack = attack::attack_line(engine, &last.state, weapon);
                }
                summary
            })
            .collect(),
        start: pick_diffs(
            &lines
                .iter()
                .map(|line| Some(&line.start[..]))
                .collect::<Vec<_>>(),
        ),
        levels,
        ranks,
        path_features,
        attack_leaves_out: weapon.is_some().then(attack::leaves_out),
    })
}

/// One thing of each Build's sheet, `None` where it has none.
fn per_build<'a, T>(
    lines: &'a [SheetResult],
    get: impl Fn(&'a SheetResult) -> Option<&'a T>,
) -> Vec<Option<&'a T>> {
    lines.iter().map(get).collect()
}

fn summary(
    planner: &Planner,
    build: &rules::Build,
    sheet: &Sheet,
    line: &SheetResult,
) -> BuildSummary {
    let engine = &planner.engine;
    let empty = CharacterState::default();
    let last = sheet.levels.last().map_or(&empty, |level| &level.state);
    let path = |key: &Option<rules::MythicPathKey>| {
        key.as_ref().map(|key| Named {
            key: key.to_string(),
            name: engine.path_name(key).to_owned(),
        })
    };
    BuildSummary {
        build: line.envelope.build.clone(),
        link: line.envelope.link.clone(),
        name: line.name.clone(),
        open_choices: u32::try_from(line.envelope.todo.len()).unwrap_or(u32::MAX),
        character_level: last.character_level,
        classes: last
            .classes
            .iter()
            .map(|taken| ClassSummary {
                class: Named {
                    key: taken.class.to_string(),
                    name: engine.class_name(&taken.class).to_owned(),
                },
                archetype: taken.archetype.as_ref().map(|key| Named {
                    key: key.to_string(),
                    name: engine.archetype_name(key).to_owned(),
                }),
                levels: taken.level,
                advances: taken
                    .advances
                    .iter()
                    .map(|class| Named {
                        key: class.to_string(),
                        name: engine.class_name(class).to_owned(),
                    })
                    .collect(),
            })
            .collect(),
        bab: last.bab,
        fort: last.fort,
        reflex: last.reflex,
        will: last.will,
        scores: crate::sheet::scores(last.ability_scores),
        caster_level: line.caster_level,
        spell_levels: crate::shapes::SpellLevels {
            arcane: line.spell_levels.arcane,
            divine: line.spell_levels.divine,
        },
        mythic_rank: last.mythic_rank,
        first_ascension: path(&build.first_ascension),
        path: path(&build.mythic_path),
        switch: path(&build.path_switch),
        level_cap: line.level_cap,
        attack: None,
    }
}

/// One of the numbers a level line has.
type Number = fn(&LevelLine) -> u8;

/// What differs at one character level, or `None` when nothing does. The features a level's Mythic ranks give through
/// their path rows (`path_features`, by Build) are left to `path_features`, so they aren't listed twice.
fn level_diff(at: &[Option<&LevelLine>], path_features: &[BTreeSet<&str>]) -> Option<LevelDiff> {
    let level = at.iter().flatten().next()?.level;
    let features: Vec<Option<Vec<Named>>> = at
        .iter()
        .zip(path_features)
        .map(|(line, path)| {
            line.map(|line| {
                line.features
                    .iter()
                    .filter(|feature| !path.contains(feature.key.as_str()))
                    .cloned()
                    .collect()
            })
        })
        .collect();
    let numbers: [(&str, Number); 11] = [
        ("class_level", |line| line.class_level),
        ("bab", |line| line.bab),
        ("fort", |line| line.fort),
        ("reflex", |line| line.reflex),
        ("will", |line| line.will),
        ("str", |line| line.scores.str),
        ("dex", |line| line.scores.dex),
        ("con", |line| line.scores.con),
        ("int", |line| line.scores.int),
        ("wis", |line| line.scores.wis),
        ("cha", |line| line.scores.cha),
    ];
    let diff = LevelDiff {
        level,
        numbers: numbers
            .iter()
            .filter_map(|(what, number)| {
                let values: Vec<Option<u8>> = at.iter().map(|line| line.map(number)).collect();
                differ(&values).then(|| NumberDiff {
                    what: (*what).to_owned(),
                    values,
                })
            })
            .collect(),
        picks: pick_diffs(
            &at.iter()
                .map(|line| line.map(|line| &line.picks[..]))
                .collect::<Vec<_>>(),
        ),
        feats: only_some(
            &at.iter()
                .map(|line| line.map(|line| &line.feats[..]))
                .collect::<Vec<_>>(),
        ),
        features: only_some(
            &features
                .iter()
                .map(|features| features.as_deref())
                .collect::<Vec<_>>(),
        ),
    };
    let same = diff.numbers.is_empty()
        && diff.picks.is_empty()
        && diff.feats.is_empty()
        && diff.features.is_empty();
    (!same).then_some(diff)
}

/// What differs at one Mythic rank apart from its path features, or `None` when nothing does.
fn rank_diff(at: &[Option<&RankLine>]) -> Option<RankDiff> {
    let rank = at.iter().flatten().next()?.rank;
    let status: Vec<_> = at.iter().map(|line| line.map(|line| line.status)).collect();
    let path: Vec<Option<String>> = at
        .iter()
        .map(|line| {
            line.and_then(|line| line.path.as_ref())
                .map(|path| path.name.clone())
        })
        .collect();
    let diff = RankDiff {
        rank,
        status: differ(&status).then(|| status.into_iter().flatten().collect()),
        path: differ(&path).then_some(path),
        picks: pick_diffs(
            &at.iter()
                .map(|line| line.map(|line| &line.picks[..]))
                .collect::<Vec<_>>(),
        ),
    };
    let same = diff.status.is_none() && diff.path.is_none() && diff.picks.is_empty();
    (!same).then_some(diff)
}

/// Whether the Builds' values aren't all the same.
fn differ<T: PartialEq>(values: &[T]) -> bool {
    values.windows(2).any(|pair| pair[0] != pair[1])
}

/// The choices holding different things in different Builds, by address in the order the Builds list them. A choice
/// whose label differs between the Builds (a path's pick: "Lich Powers", "Enforcing Gaze") gets each label once,
/// joined by " / ".
fn pick_diffs(lists: &[Option<&[PickLine]>]) -> Vec<PickDiff> {
    let mut choices: Vec<&str> = Vec::new();
    for line in lists.iter().flatten().flat_map(|list| list.iter()) {
        if !choices.contains(&line.choice.as_str()) {
            choices.push(&line.choice);
        }
    }
    choices
        .into_iter()
        .filter_map(|choice| {
            let lines: Vec<Option<&PickLine>> = lists
                .iter()
                .map(|list| list.and_then(|list| list.iter().find(|line| line.choice == choice)))
                .collect();
            let names: Vec<Option<String>> = lines
                .iter()
                .map(|line| line.and_then(|line| line.name.clone()))
                .collect();
            let mut labels: Vec<&str> = Vec::new();
            for line in lines.iter().flatten() {
                if !labels.contains(&line.label.as_str()) {
                    labels.push(&line.label);
                }
            }
            differ(&names).then(|| PickDiff {
                choice: choice.to_owned(),
                label: labels.join(" / "),
                names,
            })
        })
        .collect()
}

/// For each Build, the items of its list that some other Build's list lacks (a Build without the list has none); empty
/// when every Build has the same items.
fn only_some(lists: &[Option<&[Named]>]) -> Vec<Vec<Named>> {
    let lists: Vec<&[Named]> = lists.iter().map(|list| list.unwrap_or(&[])).collect();
    let only: Vec<Vec<Named>> = lists
        .iter()
        .map(|list| {
            list.iter()
                .filter(|item| !lists.iter().all(|other| other.contains(item)))
                .cloned()
                .collect()
        })
        .collect();
    if only.iter().all(Vec::is_empty) {
        Vec::new()
    } else {
        only
    }
}
