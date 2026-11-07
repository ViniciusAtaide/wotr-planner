//! The harness's own Builds, grouped by subject. Each Build keeps the name it had in the proofs before the harness
//! was committed (the ticket that added it, `t60 …`): the full Builds are seeded by their names, and records made by
//! those proofs still compare by name.
//!
//! Every group builds what it did then, with the same picks; a few take the game data's lists (every class, every
//! bloodline), so their Builds follow the data.

use rules::{
    Alignment, ArchetypeKey, Build, ClassChoice, ClassKey, Engine, FeatKey, FeatSource, GameData,
    LevelUp, Pick, RaceKey, SelectionOptionKey, SlotId,
};

use crate::Named;
use crate::generate::{faithful, fill_first_choices, full_builds};

mod classes;
mod feats;
mod mythic;
mod selections;
mod talents;

/// Every Build `record` records by default: [`own`], the benchmark's links, and the full Builds of [`own`]'s.
pub fn all(engine: &Engine, data: &GameData, threads: usize) -> Vec<Named> {
    let own = own(engine, data);
    let mut builds = own.clone();
    builds.extend(crate::links::bench_builds(engine));
    builds.extend(full_builds(engine, data, &own, threads));
    builds
}

/// The harness's own Builds (not the benchmark's or the full ones). `data` is the engine's game data.
pub fn own(engine: &Engine, data: &GameData) -> Vec<Named> {
    // Ticket 47's way: these get their listed selections filled with the first open option.
    let mut builds: Vec<Named> = [
        classes::every_class(data),
        classes::prestige(data),
        selections::targeted(),
        selections::options(data),
        selections::cleanup(data),
        talents::totem_lines(),
        feats::sorcerer_and_slayer_slots(),
        talents::wild_talents(),
        feats::repeats(),
        classes::backgrounds(data),
    ]
    .concat();
    for (_, build) in &mut builds {
        // A deity-bound class needs a faith before its domains are filled (ticket 86).
        *build = faithful(engine, data, std::mem::take(build));
        fill_first_choices(engine, build);
    }
    builds.extend(feats::extra_feats(engine));
    builds.extend(talents::mercies_and_exploits(engine));
    builds.extend(mythic::extra_slots(engine));
    builds.extend(talents::greater_exploits(engine));
    builds.extend(talents::ki_powers(engine));
    builds.extend(talents::infusions(engine));
    builds.extend(talents::borrowed_talents(engine));
    builds.extend(feats::class_feat_lists(engine));
    builds.extend(mythic::granted_selections(engine));
    builds.extend(classes::alignment_rules(engine));
    builds.extend(feats::borrowed_extra_feats(engine));
    builds.extend(selections::negated_prerequisites(engine));
    builds.extend(talents::instinctual_warrior(engine));
    builds.extend(selections::dual_curses(engine));
    // The groups that fill as they build: a faith if they still need one.
    for (_, build) in &mut builds {
        *build = faithful(engine, data, std::mem::take(build));
    }
    builds.extend(classes::deities(engine, data));
    builds
}

/// A character of `race` and `alignment` taking each class of `parts` for its levels, in order, with its archetype.
fn character(race: &str, alignment: Alignment, parts: &[(&str, Option<&str>, usize)]) -> Build {
    let mut build = Build::default();
    build.start.race = Some(RaceKey::new(race));
    build.start.alignment = Some(alignment);
    for (class, archetype, levels) in parts {
        build.class_choices.insert(
            ClassKey::new(*class),
            ClassChoice {
                archetype: archetype.map(ArchetypeKey::new),
                ..ClassChoice::default()
            },
        );
        for _ in 0..*levels {
            build.levels.push(LevelUp::new(ClassKey::new(*class)));
        }
    }
    build
}

/// A chaotic neutral human, as most groups make.
fn human(parts: &[(&str, Option<&str>, usize)]) -> Build {
    character("human", Alignment::ChaoticNeutral, parts)
}

/// Level-ups of each class for its count, in order (no class choice: ticket 47's Builds name none for many classes).
fn levels(parts: &[(&str, usize)]) -> Vec<LevelUp> {
    parts
        .iter()
        .flat_map(|(class, count)| std::iter::repeat_n(LevelUp::new(ClassKey::new(*class)), *count))
        .collect()
}

/// The Build with these picks too.
fn with(mut build: Build, picks: Vec<(SlotId, Pick)>) -> Build {
    build.picks.extend(picks);
    build
}

/// The general feat slot of a level.
fn general(level: u8) -> SlotId {
    SlotId::Feat {
        level,
        source: FeatSource::General,
        index: 0,
    }
}

/// A Feature selection slot of a level.
fn selection(level: u8, index: u8) -> SlotId {
    SlotId::Selection { level, index }
}

/// The slot a feat picked at a level gives (Extra Rage Power's rage power).
fn feat_selection(level: u8) -> SlotId {
    SlotId::FeatSelection { level, index: 0 }
}

/// A selection option, by key.
fn option(key: &str) -> Pick {
    Pick::option(SelectionOptionKey::new(key))
}

/// A feat, by key.
fn feat(key: &str) -> Pick {
    Pick::feat(FeatKey::new(key))
}

/// Each Build as it is (`<name> / empty`) and with its selections filled (`<name> / filled`), named `<prefix>
/// <name>`.
fn empty_and_filled(engine: &Engine, prefix: &str, builds: Vec<(String, Build)>) -> Vec<Named> {
    let mut out = Vec::new();
    for (name, build) in builds {
        out.push((format!("{prefix} {name} / empty"), build.clone()));
        out.push((
            format!("{prefix} {name} / filled"),
            crate::generate::with_selections_filled(engine, build),
        ));
    }
    out
}
