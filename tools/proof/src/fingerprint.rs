//! A Build's fingerprint: what the planner answers about it, hashed where it's long. Two sides that answer alike
//! give the same line.
//!
//! The fields, per Build:
//!
//! - `build`: its name;
//! - `who`, `levels`, `picks`: what it is (its classes, each with its archetype; its level and pick counts), for
//!   filtering a comparison;
//! - `possible`: [`Engine::is_possible`];
//! - `sheet`: the sheet ([`Engine::evaluate`]), its Debug text hashed (without a state's empty faith);
//! - `removable`: [`Engine::removable`], listed;
//! - `slots`: per slot of the sheet, by its id's Debug text: `options` (the picker's options with their statuses,
//!   hashed), `open` (how many are open), `total`, `kind`, and the lock-reason texts: `reasons` (those of options
//!   whose issue holds no `Not`, hashed with their keys) and `negated` (the others, listed in full: ticket 65 read
//!   each "Needs not" text, and the split stays so older records still compare);
//! - `classes`: the class picker at each level and one past the end, hashed; `class_reasons`: its lock-reason texts,
//!   hashed;
//! - `paths`, `switches`: the path and switch pickers (Debug); `path_reasons`, `switch_reasons`: their lock-reason
//!   texts, hashed;
//! - `allows`: [`Engine::allows`] for each edit of [`battery`], a `0` or `1` each.
//!
//! Recording also checks, panicking on a difference: [`Engine::is_removable`] against `removable` for every choice
//! the Build can name, [`Engine::removable_classes`] against `removable`'s class levels, [`Engine::allows_many`]
//! against `allows`, and, on the Builds [`SHORTCUT_CHECKED`] names, ticket 48's picker shortcut against full walks.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};

use rules::{
    AbilityScores, Alignment, Build, BuildStart, Choice, Edit, Engine, GameData, Issue, Pick,
    SlotKind, SlotOption,
};
use serde_json::{Value, json};

/// The FNV-1a hash of a text.
pub fn fnv(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// A value's Debug text, hashed.
fn hashed(value: &impl std::fmt::Debug) -> String {
    format!("{:016x}", fnv(&format!("{value:?}")))
}

/// The name prefixes of the Builds whose feat pickers are checked against full walks (ticket 48's shortcut), each
/// also with `full ` before it. The Builds added since ticket 55 (the checks are slow, so not on every Build).
pub const SHORTCUT_CHECKED: [&str; 13] = [
    "t55 ", "t57 ", "t58 ", "t59 ", "t60 ", "t61 ", "t62 ", "t63 ", "t64 ", "t65 ", "t66 ", "t67 ",
    "t86 ",
];

static SHORTCUT_BUILDS: AtomicUsize = AtomicUsize::new(0);
static SHORTCUT_OPTIONS: AtomicUsize = AtomicUsize::new(0);

/// How many Builds and feat options the shortcut checks covered so far, for the log.
pub fn shortcut_counts() -> (usize, usize) {
    (
        SHORTCUT_BUILDS.load(Ordering::Relaxed),
        SHORTCUT_OPTIONS.load(Ordering::Relaxed),
    )
}

/// The Build's fingerprint (see the module docs), after its consistency checks.
pub fn fingerprint(engine: &Engine, data: &GameData, name: &str, build: &Build) -> Value {
    let sheet = engine.evaluate(build);
    let removable = engine.removable(build);
    let mut slots = BTreeMap::new();
    for slot in sheet
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .chain(sheet.ranks.iter().flat_map(|rank| &rank.slots))
    {
        let options = engine.options(build, &slot.id);
        let open = options
            .iter()
            .flatten()
            .filter(|option| option.status.is_ok())
            .count();
        let total = options.as_ref().map_or(0, Vec::len);
        let mut reasons = Vec::new();
        let mut negated = Vec::new();
        for option in options.iter().flatten() {
            let Err(issue) = &option.status else { continue };
            let text = engine.issue_text(issue);
            if format!("{issue:?}").contains("Not(") {
                negated.push(format!(
                    "{}: {}",
                    option.pick.key(),
                    text.unwrap_or_default()
                ));
            } else {
                reasons.push((option.pick.key().to_owned(), text));
            }
        }
        slots.insert(
            format!("{:?}", slot.id),
            json!({
                "options": hashed(&options),
                "open": open,
                "total": total,
                "kind": format!("{:?}", slot.kind),
                "reasons": hashed(&reasons),
                "negated": negated,
            }),
        );
    }
    let past_end = u8::try_from(build.levels.len() + 1).expect("at most 254 levels");
    let classes: Vec<String> = (1..=past_end)
        .map(|level| hashed(&engine.class_options(build, level)))
        .collect();
    let reason = |status: &Result<(), Issue>| {
        status
            .as_ref()
            .err()
            .and_then(|issue| engine.issue_text(issue))
    };
    let class_reasons: Vec<Vec<Option<String>>> = (1..=past_end)
        .map(|level| {
            engine
                .class_options(build, level)
                .unwrap_or_default()
                .iter()
                .map(|option| reason(&option.status))
                .collect()
        })
        .collect();
    let paths = engine.path_options(build);
    let switches = engine.switch_options(build);
    let path_reasons: Vec<Option<String>> =
        paths.iter().map(|option| reason(&option.status)).collect();
    let switch_reasons: Vec<Option<String>> = switches
        .iter()
        .map(|option| reason(&option.status))
        .collect();
    let edits = battery(data, build);
    let allows: Vec<bool> = edits
        .iter()
        .map(|edit| engine.allows(build, edit))
        .collect();

    consistency_checks(engine, build, &removable, &edits, &allows);
    let unfull = name.strip_prefix("full ").unwrap_or(name);
    if SHORTCUT_CHECKED
        .iter()
        .any(|prefix| unfull.starts_with(prefix))
    {
        shortcut_checks(engine, build);
    }

    json!({
        "build": name,
        "who": who(build),
        "levels": build.levels.len(),
        "picks": build.picks.len(),
        "possible": engine.is_possible(build),
        "sheet": sheet_hash(&sheet),
        "removable": removable.iter().map(|choice| format!("{choice:?}")).collect::<Vec<_>>(),
        "slots": slots,
        "classes": classes,
        "class_reasons": hashed(&class_reasons),
        "paths": format!("{paths:?}"),
        "path_reasons": hashed(&path_reasons),
        "switches": format!("{switches:?}"),
        "switch_reasons": hashed(&switch_reasons),
        "allows": allows.iter().map(|allowed| if *allowed { '1' } else { '0' }).collect::<String>(),
    })
}

/// The sheet's Debug text, hashed, without a state's empty faith (ticket 86's `faith: None, deity_domains: {}`): a
/// Build without one walks to the same sheet as on a side before deities, whose states have no such fields.
fn sheet_hash(sheet: &rules::Sheet) -> String {
    let text = format!("{sheet:?}").replace("faith: None, deity_domains: {}, ", "");
    format!("{:016x}", fnv(&text))
}

/// The Build's classes, each with its archetype ("slayer:slayer/arcane-enforcer").
fn who(build: &Build) -> Vec<String> {
    build
        .class_choices
        .iter()
        .map(|(class, choice)| match &choice.archetype {
            Some(archetype) => format!("{class}:{archetype}"),
            None => class.to_string(),
        })
        .collect()
}

/// The edits whose `allows` the fingerprint holds: every Build start the start pickers and point buy offer, each
/// level's ability increases and skill steps, each pick cleared, and the edits past the end.
pub fn battery(data: &GameData, build: &Build) -> Vec<Edit> {
    let current = &build.start;
    let mut starts: Vec<BuildStart> = Vec::new();
    for race in &data.races {
        let subraces =
            std::iter::once(None).chain(race.subraces.iter().map(|s| Some(s.key.clone())));
        for subrace in subraces {
            starts.push(BuildStart {
                race: Some(race.key.clone()),
                subrace,
                ..current.clone()
            });
        }
    }
    for alignment in Alignment::ALL {
        starts.push(BuildStart {
            alignment: Some(alignment),
            ..current.clone()
        });
    }
    for background in &data.backgrounds {
        starts.push(BuildStart {
            background: Some(background.key.clone()),
            ..current.clone()
        });
    }
    for ability in AbilityScores::ABILITIES {
        for delta in [-1i8, 1] {
            let next = current
                .ability_scores
                .get(ability)
                .checked_add_signed(delta);
            if let Some(scores) = next.and_then(|next| current.ability_scores.with(ability, next)) {
                starts.push(BuildStart {
                    ability_scores: scores,
                    ..current.clone()
                });
            }
        }
        starts.push(BuildStart {
            racial_ability: Some(ability),
            ..current.clone()
        });
    }
    starts.push(BuildStart {
        race: None,
        subrace: None,
        racial_ability: None,
        ..current.clone()
    });
    let mut edits: Vec<Edit> = starts.into_iter().map(Edit::Start).collect();
    for (index, level_up) in build.levels.iter().enumerate() {
        let level = u8::try_from(index + 1).expect("at most 255 levels");
        edits.push(Edit::AbilityIncrease {
            level,
            ability: None,
        });
        for ability in AbilityScores::ABILITIES {
            edits.push(Edit::AbilityIncrease {
                level,
                ability: Some(ability),
            });
        }
        for skill in &data.skills {
            let here = level_up.skill_ranks.get(&skill.key).copied().unwrap_or(0);
            for ranks in [here.saturating_sub(1), here + 1] {
                edits.push(Edit::SkillRanks {
                    level,
                    skill: skill.key.clone(),
                    ranks,
                });
            }
        }
    }
    edits.extend(build.picks.keys().cloned().map(Edit::Clear));
    edits.push(Edit::RemoveLevel);
    edits.push(Edit::MythicPath(None));
    edits.push(Edit::PathSwitch(None));
    edits.push(Edit::SetName(Some("x".into())));
    edits
}

/// Every choice a Build can name, for `is_removable`: its own, and some it doesn't make.
fn every_choice(build: &Build) -> Vec<Choice> {
    let mut choices = vec![
        Choice::Race,
        Choice::Subrace,
        Choice::Alignment,
        Choice::Background,
        Choice::MythicPath,
        Choice::PathSwitch,
    ];
    choices.extend(AbilityScores::ABILITIES.map(|ability| Choice::AbilityScore { ability }));
    for (index, level_up) in build.levels.iter().enumerate() {
        let level = u8::try_from(index + 1).expect("at most 255 levels");
        choices.push(Choice::Class { level });
        choices.push(Choice::AbilityIncrease { level });
        choices.push(Choice::SkillPoints { level });
        for skill in level_up.skill_ranks.keys() {
            choices.push(Choice::SkillRanks {
                level,
                skill: skill.clone(),
            });
        }
        choices.push(Choice::Archetype {
            class: level_up.class.clone(),
        });
        choices.push(Choice::Advances {
            class: level_up.class.clone(),
        });
    }
    choices.push(Choice::Class {
        level: u8::try_from(build.levels.len() + 1).expect("at most 254 levels"),
    });
    choices.extend(build.picks.keys().cloned().map(Choice::Slot));
    choices
}

/// The shortcuts against the full answers they stand for: `is_removable` against `removable`, `removable_classes`
/// against `removable`'s class levels, `allows_many` against `allows`. Panics on a difference.
fn consistency_checks(
    engine: &Engine,
    build: &Build,
    removable: &BTreeSet<Choice>,
    edits: &[Edit],
    allows: &[bool],
) {
    for choice in every_choice(build) {
        assert_eq!(
            engine.is_removable(build, &choice),
            removable.contains(&choice),
            "{choice:?}"
        );
    }
    let levels: BTreeSet<u8> = removable
        .iter()
        .filter_map(|choice| match choice {
            Choice::Class { level } => Some(*level),
            _ => None,
        })
        .collect();
    assert_eq!(engine.removable_classes(build), levels);
    assert_eq!(engine.allows_many(build, edits), allows);
}

/// Ticket 48's picker shortcut, checked: every feat a feat slot accepts is open, not `BreaksLater`, exactly when
/// `allows` keeps the Build with it picked there (a full walk of the later choices; the shortcut gives a feat no later
/// check names the slot-left-empty answer). Panics on a difference.
fn shortcut_checks(engine: &Engine, build: &Build) {
    let sheet = engine.evaluate(build);
    for slot in sheet
        .levels
        .iter()
        .flat_map(|level| &level.slots)
        .chain(sheet.ranks.iter().flat_map(|rank| &rank.slots))
        .filter(|slot| matches!(slot.kind, SlotKind::Feat(_)))
    {
        let options: Vec<SlotOption> = engine.options(build, &slot.id).unwrap_or_default();
        for option in options {
            // Only a feat the slot accepts: the shortcut answers whether it breaks a later choice.
            if !matches!(option.pick, Pick::Feat { .. })
                || !matches!(option.status, Ok(()) | Err(Issue::BreaksLater))
            {
                continue;
            }
            SHORTCUT_OPTIONS.fetch_add(1, Ordering::Relaxed);
            let allowed = engine.allows(
                build,
                &Edit::Pick {
                    slot: slot.id.clone(),
                    pick: option.pick.clone(),
                },
            );
            assert_eq!(
                option.status.is_ok(),
                allowed,
                "{:?} {:?}: {:?}",
                slot.id,
                option.pick,
                option.status
            );
        }
    }
    SHORTCUT_BUILDS.fetch_add(1, Ordering::Relaxed);
}
