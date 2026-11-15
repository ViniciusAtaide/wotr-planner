//! The fillers that turn a Build into a fuller one, and the full Builds.
//!
//! Each keeps the exact behaviour the behaviour proof had when ticket 67 ran it, quirks included: they define the
//! corpus, so a change to one changes the Builds the self-checks run on. The quirks are documented where they live.

use rules::{
    AbilityScores, Build, Edit, Engine, Faith, FeatSlot, GameData, Issue, SkillKey, SlotId,
    SlotKind,
};

use crate::Named;

/// The FNV-1a hash of a text: a full Build's seed is its name's.
fn fnv(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// The Build following `faith` (ticket 86).
pub fn with_faith(mut build: Build, faith: Faith) -> Build {
    build.start.deity = Some(faith);
    build
}

/// The Build with the faith a deity-bound class or archetype needs (ticket 86): as it is when it's possible, else
/// the first of atheism and the deities, in key order, it's possible with; as it is when none makes it possible.
pub fn faithful(engine: &Engine, data: &GameData, build: Build) -> Build {
    if engine.is_possible(&build) {
        return build;
    }
    let faiths = std::iter::once(Faith::Atheism).chain(
        data.deities
            .iter()
            .map(|deity| Faith::Deity(deity.key.clone())),
    );
    for faith in faiths {
        let candidate = with_faith(build.clone(), faith);
        if engine.is_possible(&candidate) {
            return candidate;
        }
    }
    build
}

/// A seeded xorshift generator, so a full Build is filled the same way on every run.
pub struct Rng(pub u64);

impl Rng {
    /// A number below `n`.
    pub fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

/// Fills every empty slot, ability increase and skill point with a choice the sheet offers, and the path.
///
/// Quirks kept:
/// - three rounds of slot filling, each over the slots empty at its start, in sheet order;
/// - Witch Wandering Hex counts among a slot's choices when its only issue is an unmet prerequisite (ticket 62 gave it
///   one), and the slot stays empty when it's drawn then;
/// - the path is drawn only for a Build of 9 levels or more (one that reaches MR 3), in any round where it has none;
/// - at most 40 draws of a skill per level, each kept only when it changes the Build and `allows` keeps it.
pub fn fill_full(engine: &Engine, data: &GameData, build: &mut Build, rng: &mut Rng) {
    for _ in 0..3 {
        let sheet = engine.evaluate(build);
        let slots: Vec<SlotId> = sheet
            .levels
            .iter()
            .flat_map(|level| &level.slots)
            .chain(sheet.ranks.iter().flat_map(|rank| &rank.slots))
            .filter(|slot| slot.pick.is_none())
            .map(|slot| slot.id.clone())
            .collect();
        for slot in slots {
            let open: Vec<_> = engine
                .options(build, &slot)
                .unwrap_or_default()
                .into_iter()
                .filter(|option| {
                    option.status.is_ok()
                        || (option.pick.key() == "witch-wandering-hex"
                            && matches!(option.status, Err(Issue::Unmet(_))))
                })
                .collect();
            if !open.is_empty() {
                let option = open[rng.below(open.len())].clone();
                if option.status.is_ok() {
                    engine.apply(
                        build,
                        &Edit::Pick {
                            slot,
                            pick: option.pick,
                        },
                    );
                }
            }
        }
        if build.mythic_path.is_none() {
            let open: Vec<_> = engine
                .path_options(build)
                .into_iter()
                .filter(|option| option.status.is_ok())
                .collect();
            if !open.is_empty() && build.levels.len() >= 9 {
                let path = open[rng.below(open.len())].path.clone();
                engine.apply(build, &Edit::MythicPath(Some(path)));
            }
        }
    }
    let skills: Vec<SkillKey> = data.skills.iter().map(|skill| skill.key.clone()).collect();
    for level in 1..=u8::try_from(build.levels.len()).expect("at most 255 levels") {
        let index = usize::from(level) - 1;
        let sheet = engine.evaluate(build);
        if sheet.levels[index].ability_increase && build.levels[index].ability_increase.is_none() {
            let ability = AbilityScores::ABILITIES[rng.below(6)];
            let edit = Edit::AbilityIncrease {
                level,
                ability: Some(ability),
            };
            if engine.allows(build, &edit) {
                engine.apply(build, &edit);
            }
        }
        let mut points = engine.evaluate(build).levels[index].skill_points;
        for _ in 0..40 {
            if points.spent >= points.available {
                break;
            }
            let skill = &skills[rng.below(skills.len())];
            let here = build.levels[index]
                .skill_ranks
                .get(skill)
                .copied()
                .unwrap_or(0);
            let edit = Edit::SkillRanks {
                level,
                skill: skill.clone(),
                ranks: here + 1,
            };
            let mut after = build.clone();
            engine.apply(&mut after, &edit);
            if after != *build && engine.allows(build, &edit) {
                *build = after;
                points.spent += 1;
            }
        }
    }
    assert!(engine.is_possible(build));
}

/// The full Builds: each possible Build of `base` filled by [`fill_full`] (`full <name>`, seeded by its name's
/// hash, so a renamed Build draws differently), and each of those that reaches MR 8 with each Path switch its picker
/// leaves open (`full <name> / <switch>`), when the switch holds and the Build stays possible.
pub fn full_builds(engine: &Engine, data: &GameData, base: &[Named], threads: usize) -> Vec<Named> {
    crate::in_parallel(base, threads, |(name, build)| {
        let mut out = Vec::new();
        // Only a possible Build: the sheet never holds another.
        if !engine.is_possible(build) {
            return out;
        }
        let mut full = build.clone();
        fill_full(engine, data, &mut full, &mut Rng(fnv(name) | 1));
        out.push((format!("full {name}"), full.clone()));
        for switch in engine.switch_options(&full) {
            if switch.status.is_ok() {
                let mut switched = full.clone();
                engine.apply(&mut switched, &Edit::PathSwitch(Some(switch.path.clone())));
                if switched.path_switch.is_some() && engine.is_possible(&switched) {
                    out.push((format!("full {name} / {}", switch.path), switched));
                }
            }
        }
        out
    })
    .into_iter()
    .flatten()
    .collect()
}

/// Fills every empty Feature selection slot of the Build's levels, in slot order, with the first option its picker
/// leaves open, until none has one. Quirk kept: the rank slots (a Second Bloodline's) are never filled here.
pub fn with_selections_filled(engine: &Engine, mut build: Build) -> Build {
    loop {
        let sheet = engine.evaluate(&build);
        let empty = sheet
            .levels
            .iter()
            .flat_map(|level| &level.slots)
            .filter(|slot| {
                matches!(slot.kind, SlotKind::Selection(_)) && !build.picks.contains_key(&slot.id)
            })
            .map(|slot| slot.id.clone())
            .find(|id| {
                engine
                    .options(&build, id)
                    .is_some_and(|options| options.iter().any(|option| option.status.is_ok()))
            });
        let Some(id) = empty else {
            return build;
        };
        let open = engine
            .options(&build, &id)
            .expect("the slot has a picker")
            .into_iter()
            .find(|option| option.status.is_ok())
            .expect("the slot has an open option");
        build.picks.insert(id, open.pick);
    }
}

/// Fills, twice, the slots ticket 47's proof filled on its Builds: each selection [`first_choice_selection`] names
/// and each slot of the three one-feat lists, with its first open option, so later levels see a held pick.
///
/// Quirk kept: each pass reads the sheet once, before it fills anything, so a slot that a pick of the same pass
/// opens waits for the second pass, and only the levels' slots are filled (never a rank's).
pub fn fill_first_choices(engine: &Engine, build: &mut Build) {
    for _ in 0..2 {
        let sheet = engine.evaluate(build);
        for level in &sheet.levels {
            for slot in &level.slots {
                if build.picks.contains_key(&slot.id) {
                    continue;
                }
                let fill = match &slot.kind {
                    SlotKind::Feat(FeatSlot::OneOf { list }) => matches!(
                        list.as_str(),
                        "warpriest/weapon-focus"
                            | "rogue/master-of-all/skill-focus"
                            | "magus/sword-saint/chosen-weapon"
                    ),
                    SlotKind::Selection(selection) => first_choice_selection(selection.as_str()),
                    _ => false,
                };
                if !fill {
                    continue;
                }
                let open = engine
                    .options(build, &slot.id)
                    .unwrap_or_default()
                    .into_iter()
                    .find(|option| option.status.is_ok());
                if let Some(open) = open {
                    build.picks.insert(slot.id.clone(), open.pick);
                }
            }
        }
    }
}

/// The selections [`fill_first_choices`] fills (ticket 33's, widened by its proofs up to ticket 47's). Wild talents,
/// rage powers, hexes and the like stay empty.
fn first_choice_selection(selection: &str) -> bool {
    selection == "druid/nature-bond"
        || [
            "sorcerer/bloodline",
            "bloodrager/bloodline",
            "wizard/school",
            "cavalier/order",
            "shaman/spirit",
            "arcanist/magic-deceiver/ways-of-the-deceiver",
            "ranger/combat-style-feat",
            "magus/eldritch-scion/bloodline",
            "fighter/dragonheir-scion/bloodline",
            "dragon-disciple/bloodline",
        ]
        .contains(&selection)
        || [
            "/bloodline",
            "/school",
            "/spirit",
            "/primal-choices",
            "/domain",
            "/mystery",
            "/revelation",
            "/element",
        ]
        .iter()
        .any(|suffix| selection.ends_with(suffix))
        || matches!(
            selection,
            "kineticist/expanded-element"
                | "hellknight/order"
                | "hellknight/pentamic-faith"
                | "oracle/maneuver-mastery"
                | "monk/zen-archer/way-of-the-bow"
                | "hellknight-signifer/order"
        )
}
