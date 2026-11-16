//! The fillers that turn a Build into a fuller one, and the full Builds.
//!
//! Each keeps the exact behaviour the behaviour proof had when ticket 67 ran it, quirks included: they define the
//! corpus, so a change to one changes the Builds the self-checks run on. The quirks are documented where they live.

use rules::{
    AbilityScores, Build, Edit, Engine, Faith, GameData, Issue, Pick, SkillKey, SlotId, SlotKind,
};

use crate::Named;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// `hash` after FNV-1a over `bytes`.
fn fnv_extend(mut hash: u64, bytes: impl IntoIterator<Item = u8>) -> u64 {
    for byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// The FNV-1a hash of a text: a full Build's seed is its name's.
fn fnv(text: &str) -> u64 {
    fnv_extend(FNV_OFFSET, text.bytes())
}

/// The FNV-1a hash of the parts' bytes, a 0xff byte (never in UTF-8) after each, so no two splits of one text agree.
fn fnv_parts(parts: &[&str]) -> u64 {
    parts.iter().fold(FNV_OFFSET, |hash, part| {
        fnv_extend(hash, part.bytes().chain([0xff]))
    })
}

/// The candidate with the highest hash of (the Build's name, the slot's stable text, its own key): rendezvous
/// hashing (ticket 310). A slot's draw moves only when its own candidates change, and then only if the winner left or
/// a new candidate outranks it, so a change to one slot's offers re-draws no other slot. The hash is FNV-1a over the
/// bytes, finished by a mixer (FNV's last bytes barely reach its high bits), the same on every toolchain.
fn draw_for<'a, T>(
    name: &str,
    slot: &str,
    candidates: &'a [T],
    key: impl Fn(&T) -> String,
) -> Option<&'a T> {
    candidates.iter().max_by_key(|candidate| {
        let key = key(candidate);
        let mut hash = fnv_parts(&[name, slot, &key]);
        hash ^= hash >> 32;
        hash = hash.wrapping_mul(0xd6e8_feb8_6659_fd93);
        hash ^ (hash >> 32)
    })
}

/// A pick's text: its key, and what it's taken for.
fn pick_text(pick: &Pick) -> String {
    format!("{}/{:?}", pick.key(), pick.parameter())
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
/// Each slot's pick, and the path's, is drawn on its own by [`draw_for`] from the Build's `name`, so a change to one
/// slot's offers moves no other pick (ticket 310). The ability increases and skill points are drawn from `rng`.
///
/// Quirks kept:
/// - three rounds of slot filling, each over the slots empty at its start, in sheet order;
/// - Witch Wandering Hex counts among a slot's choices when its only issue is an unmet prerequisite (ticket 62 gave it
///   one), and the slot stays empty when it's drawn then;
/// - the path is drawn only for a Build of 9 levels or more (one that reaches MR 3), in any round where it has none;
/// - at most 40 draws of a skill per level, each kept only when it changes the Build and `allows` keeps it.
pub fn fill_full(engine: &Engine, data: &GameData, build: &mut Build, name: &str, rng: &mut Rng) {
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
                let slot_text = format!("{slot:?}");
                let option = draw_for(name, &slot_text, &open, |option| pick_text(&option.pick))
                    .expect("open isn't empty")
                    .clone();
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
            if build.levels.len() >= 9
                && let Some(option) =
                    draw_for(name, "mythic-path", &open, |option| option.path.to_string())
            {
                engine.apply(build, &Edit::MythicPath(Some(option.path.clone())));
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
        fill_full(engine, data, &mut full, name, &mut Rng(fnv(name) | 1));
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
                    SlotKind::Feat(list) => matches!(
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Twelve slots of twenty options each: the slot's offered keys.
    fn offers() -> Vec<(String, Vec<String>)> {
        (0..12)
            .map(|slot| {
                let keys = (0..20).map(|option| format!("option-{option}")).collect();
                (format!("slot-{slot}"), keys)
            })
            .collect()
    }

    /// The new rule's pick of each slot, none for a slot with no offers.
    fn per_slot(name: &str, offers: &[(String, Vec<String>)]) -> Vec<Option<String>> {
        (offers.iter())
            .map(|(slot, keys)| draw_for(name, slot, keys, |key| key.clone()).cloned())
            .collect()
    }

    /// The old rule's pick of each slot: one stream, one draw per slot that has offers.
    fn stream(name: &str, offers: &[(String, Vec<String>)]) -> Vec<Option<String>> {
        let mut rng = Rng(fnv(name) | 1);
        (offers.iter())
            .map(|(_, keys)| (!keys.is_empty()).then(|| keys[rng.below(keys.len())].clone()))
            .collect()
    }

    /// Ticket 310: closing one option of one slot's offers, or all of them, moves that slot's pick or none, and no
    /// other; the old stream rule re-drew the later slots when a slot went empty.
    #[test]
    fn a_change_to_one_slots_offers_moves_only_its_own_pick() {
        let mut stream_moved_later = 0;
        for name in (0..30).map(|n| format!("full t{n} build")) {
            let before = offers();
            let picks = per_slot(&name, &before);
            let old = stream(&name, &before);
            for changed in 0..before.len() {
                let own = picks[changed].clone().unwrap();
                // The winner closed, another option closed, every option closed.
                for closed in [
                    vec![own.clone()],
                    vec!["option-3".to_string()],
                    before[changed].1.clone(),
                ] {
                    let mut after = before.clone();
                    after[changed].1.retain(|key| !closed.contains(key));
                    let now = per_slot(&name, &after);
                    for slot in (0..before.len()).filter(|&slot| slot != changed) {
                        assert_eq!(now[slot], picks[slot], "{name}: slot {slot} moved");
                    }
                    if !closed.contains(&own) {
                        assert_eq!(now[changed], picks[changed], "{name}: the winner stayed");
                    }
                    let again = stream(&name, &after);
                    stream_moved_later += (changed + 1..before.len())
                        .filter(|&slot| again[slot] != old[slot])
                        .count();
                }
            }
        }
        assert!(stream_moved_later > 0, "the old rule re-drew later slots");
    }
}
