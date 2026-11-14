//! `compare`: one to four Builds side by side, each in short, then only what differs between them (one Build: its
//! summary alone). Everything is read from each Build's sheet as `sheet` writes it (`sheet::sheet_of` on
//! [`rules::Engine::evaluate`]'s sheet), so a difference here is one the two sheets show. Nothing is ranked by
//! judgement: with `by`, the Builds are ordered by a number the engine computes (`rank.rs`), a fact like the rest.
//!
//! The numbers (the summaries', the differences' and `by`'s) are naked unless `with_gear`: each Build's with no gear
//! at all ([`rules::Engine::naked_sheet`]). A Build with gear also gets the other numbers where its gear changes
//! them. `compare` never adds gear: each Build counts only the gear it holds.

use std::collections::BTreeSet;

use rules::{CharacterState, Sheet};

use crate::attack;
use crate::caster;
use crate::code::open;
use crate::error::ToolError;
use crate::gear::if_differs;
use crate::planner::Planner;
use crate::rank::{self, RankedBy};
use crate::shapes::{
    BuildSummary, ClassSummary, CompareArgs, CompareResult, FeaturesDiff, Gained, LevelDiff,
    LevelLine, Named, NumberDiff, Numbers, OtherNumbers, PickDiff, PickLine, RankDiff, RankLine,
    SheetResult,
};
use crate::sheet::{scores, sheet_of};

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
    let mut builds = args
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
    let spell = caster::caster_argument(args.caster.as_ref())?;
    let number = args
        .by
        .as_ref()
        .map(|by| rank::Number::from_args(engine, by))
        .transpose()?
        // An attack line that names no weapon ranks by the line shown: `weapon`'s, or else each Build's own.
        .map(|number| number.or_weapon(weapon.clone()));
    let counts = Numbers::from_flag(args.with_gear);
    let mut sheets: Vec<Sheet> = builds.iter().map(|build| engine.evaluate(build)).collect();
    // Each Build's numbers as `counts` says (`counted`), and, for a Build with gear, the other ones beside them.
    let naked: Vec<Option<Sheet>> = builds
        .iter()
        .map(|build| engine.naked_sheet(build))
        .collect();
    let (mut counted, mut other): (Vec<Sheet>, Vec<Option<Sheet>>) = (sheets.iter())
        .zip(naked)
        .map(|(sheet, naked)| match (counts, naked) {
            (Numbers::Naked, Some(naked)) => (naked, Some(sheet.clone())),
            (Numbers::WithGear, naked) => (sheet.clone(), naked),
            (Numbers::Naked, None) => (sheet.clone(), None),
        })
        .unzip();
    // With `by`, every per-Build list follows the order of the numbers: the Builds are put in it first.
    let mut ranked: Vec<(usize, Option<i16>)> = Vec::new();
    if let Some(number) = &number {
        let numbers: Vec<Option<i16>> = counted
            .iter()
            .map(|sheet| number.of(engine, sheet))
            .collect();
        let order = rank::order(&numbers);
        ranked = order.iter().map(|&i| (i, numbers[i])).collect();
        builds = order.iter().map(|&i| builds[i].clone()).collect();
        sheets = order.iter().map(|&i| sheets[i].clone()).collect();
        counted = order.iter().map(|&i| counted[i].clone()).collect();
        other = order.iter().map(|&i| other[i].clone()).collect();
    }
    let mut lines: Vec<SheetResult> = builds
        .iter()
        .zip(&sheets)
        .map(|(build, sheet)| sheet_of(planner, build, sheet, false))
        .collect();
    // The differences' numbers are the counted ones; the choices, feats and features are the Build's own.
    for (line, counted) in lines.iter_mut().zip(&counted) {
        for (level, at) in line.levels.iter_mut().zip(&counted.levels) {
            level.scores = scores(at.state.ability_scores);
        }
    }

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

    let summaries: Vec<BuildSummary> = builds
        .iter()
        .zip(&sheets)
        .zip(&lines)
        .enumerate()
        .map(|(i, ((build, sheet), line))| {
            let mut summary = summary(planner, build, sheet, line);
            if let Some(&(given, number)) = ranked.get(i) {
                summary.given = u32::try_from(given).ok();
                summary.number = number;
            }
            // The attack line's weapon is the Build's own Weapon Focus weapon, whichever numbers count.
            let weapon = attack::weapon_or_default(engine, weapon.as_ref(), sheet);
            let numbers_of = |sheet: &Sheet| {
                let last = sheet.levels.last();
                OtherNumbers {
                    scores: last.map(|last| scores(last.state.ability_scores)),
                    attack: (weapon.as_ref().zip(last)).and_then(|(weapon, last)| {
                        attack::attack_line(engine, &last.state, weapon)
                    }),
                    caster: (spell.zip(last))
                        .and_then(|(spell, last)| caster::caster_line(engine, last, spell)),
                    number: number.as_ref().and_then(|number| number.of(engine, sheet)),
                }
            };
            let counted = numbers_of(&counted[i]);
            if let Some(scores) = counted.scores.clone() {
                summary.scores = scores;
            }
            if let Some(other) = &other[i] {
                let other = numbers_of(other);
                let other = OtherNumbers {
                    scores: if_differs(Some(&summary.scores), other.scores),
                    attack: if_differs(counted.attack.as_ref(), other.attack),
                    caster: if_differs(counted.caster.as_ref(), other.caster),
                    number: if_differs(counted.number.as_ref(), other.number),
                };
                let any = other.scores.is_some()
                    || other.attack.is_some()
                    || other.caster.is_some()
                    || other.number.is_some();
                if any {
                    match counts {
                        Numbers::Naked => summary.with_gear = Some(other),
                        Numbers::WithGear => summary.naked = Some(other),
                    }
                }
            }
            summary.attack = counted.attack;
            summary.caster = counted.caster;
            summary
        })
        .collect();
    let gear_lines: Vec<Vec<PickLine>> = lines
        .iter()
        .map(|line| {
            (line.gear.iter())
                .map(|item| PickLine {
                    choice: item.choice.clone(),
                    label: item.label.clone(),
                    value: Some(crate::shapes::ChoiceValue::Key(item.value.clone())),
                    name: Some(item.name.clone()),
                    removable: None,
                    with_gear: None,
                })
                .collect()
        })
        .collect();
    let any_attack_line = summaries.iter().any(|summary| summary.attack.is_some());

    Ok(CompareResult {
        builds: summaries,
        start: pick_diffs(
            &lines
                .iter()
                .map(|line| Some(&line.start[..]))
                .collect::<Vec<_>>(),
        ),
        levels,
        ranks,
        path_features,
        attack_leaves_out: any_attack_line.then(attack::leaves_out),
        caster_leaves_out: spell.is_some().then(caster::leaves_out),
        by: number.map(|number| RankedBy {
            number: number.describe(engine),
            order: rank::ORDER,
            leaves_out: number.leaves_out(),
        }),
        numbers: counts,
        gear: pick_diffs(
            &gear_lines
                .iter()
                .map(|lines| Some(&lines[..]))
                .collect::<Vec<_>>(),
        ),
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
        caster: None,
        number: None,
        given: None,
        gear: build
            .gear
            .items()
            .map(|(_, key)| Named {
                key: key.as_str().to_owned(),
                name: engine.gear_name(key).to_owned(),
            })
            .collect(),
        gear_dependent: line.gear_dependent.clone(),
        with_gear: None,
        naked: None,
    }
}

/// One of the numbers a level line has.
type Number = fn(&LevelLine) -> u8;

/// What differs at one character level, or `None` when nothing does. The features a level's Mythic ranks give through
/// their path rows (`path_features`, by Build) are left to `path_features`, so they aren't listed twice.
fn level_diff(at: &[Option<&LevelLine>], path_features: &[BTreeSet<&str>]) -> Option<LevelDiff> {
    let level = at.iter().flatten().next()?.level;
    let features: Vec<Option<Vec<Gained>>> = at
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
    let levels: Vec<_> = (at.iter())
        .map(|line| line.map(|line| line.character_level))
        .collect();
    let diff = RankDiff {
        rank,
        character_level: differ(&levels).then(|| levels.into_iter().flatten().collect()),
        status: differ(&status).then(|| status.into_iter().flatten().collect()),
        path: differ(&path).then_some(path),
        picks: pick_diffs(
            &at.iter()
                .map(|line| line.map(|line| &line.picks[..]))
                .collect::<Vec<_>>(),
        ),
    };
    let same = diff.character_level.is_none()
        && diff.status.is_none()
        && diff.path.is_none()
        && diff.picks.is_empty();
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
fn only_some<T: PartialEq + Clone>(lists: &[Option<&[T]>]) -> Vec<Vec<T>> {
    let lists: Vec<&[T]> = lists.iter().map(|list| list.unwrap_or(&[])).collect();
    let only: Vec<Vec<T>> = lists
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sneak(times: Option<u8>) -> Gained {
        Gained {
            key: "trickster/sneak-attack".to_owned(),
            name: "Sneak Attack (Trickster)".to_owned(),
            times,
        }
    }

    #[test]
    fn a_feature_gained_twice_differs_from_the_same_one_gained_once() {
        let (twice, once) = ([sneak(Some(2))], [sneak(None)]);
        let only = only_some(&[Some(&twice[..]), Some(&once[..])]);
        assert_eq!(only, [vec![sneak(Some(2))], vec![sneak(None)]]);
        // The same count is no difference.
        assert!(only_some(&[Some(&twice[..]), Some(&twice[..])]).is_empty());
    }
}
