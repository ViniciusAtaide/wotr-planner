//! The corpus: about 2,000 Builds of the real game data, which the engine's self-checks run on (ticket 235, which kept
//! them when the behaviour proof that recorded these Builds went).
//!
//! - [`builds::all`]: its own Builds ([`builds::own`], grouped by subject, each named after the ticket that
//!   added it, `t60 …`), the benchmark's (from `bench-links.tsv`), and a full Build of each own
//!   one (every slot, ability increase and skill point filled by a generator seeded by the
//!   Build's name).
//! - [`builds::sample`]: a few of them, for the suite's short runs.
//!
//! The tests beside it are the self-checks, which check the engine against itself:
//!
//! - `tests/agreement.rs`: every picker agrees with the walk ([`rules::agreement`], ticket 150), on the whole corpus;
//!   ignored for its time.
//! - `tests/self_checks.rs`: the withdrawals against `removable`, each start and level option's status against
//!   `allows`, and ticket 48's picker shortcut against full walks; the sample in the suite, the whole corpus ignored.
//!
//! - `tests/form_proof.rs` (ticket 300): `record` and `compare`, for a ticket that changes only the data's or the model's
//!   form and so claims no behaviour moves; ignored, a few seconds in release.
//!
//! The ignored runs take minutes in release (`cargo test --release -p corpus -- --ignored`): a heavy run, once per
//! queue of tickets by the main session (`CLAUDE.md`, "Parallel agents"). A self-check never outranks the game: when
//! one disagrees with it, the engine is fixed (`CLAUDE.md`, "Behaviour checks").

mod bench;
pub mod builds;
mod generate;

use std::sync::atomic::{AtomicUsize, Ordering};

use rules::{Build, Engine, GameData};

/// A Build with its name in the corpus.
pub type Named = (String, Build);

/// The engine on the game data ([`seeds::game_data_dir`]), and that game data in the canonical order the engine holds
/// it in. The corpus keeps its own copy for the lists it walks (every race, skill, class…): the engine doesn't hand out
/// its data. The files reader already returns it canonical. Data that fails [`GameData::check`] is refused.
pub fn engine() -> Result<(Engine, GameData), seeds::Error> {
    let checked = seeds::check_game_data_files(&seeds::game_data_dir())?;
    let data = GameData::clone(&checked);
    Ok((Engine::new(checked), data))
}

/// The threads a long run takes: the environment variable `var` when it holds a number, else 8.
pub fn threads(var: &str) -> usize {
    std::env::var(var)
        .ok()
        .and_then(|threads| threads.parse().ok())
        .unwrap_or(8)
        .max(1)
}

/// Runs `work` on each Build on `threads` threads, keeping the Builds' order.
///
/// Each thread takes the next Build not yet taken, so the threads finish together: a Build's cost varies a
/// hundredfold (a full level-20 Build with ticket 48's shortcut checks against a level-1 one), and the costly ones
/// come last, so fixed chunks left one thread with six times another's work.
pub fn in_parallel<T: Send>(
    builds: &[Named],
    threads: usize,
    work: impl Fn(&Named) -> T + Sync,
) -> Vec<T> {
    let next = AtomicUsize::new(0);
    let (work, next) = (&work, &next);
    let mut done: Vec<(usize, T)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads.clamp(1, builds.len().max(1)))
            .map(|_| {
                scope.spawn(move || {
                    let mut done = Vec::new();
                    loop {
                        // Relaxed: the counter only hands out indices; the scope's join publishes the results.
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        let Some(build) = builds.get(index) else {
                            break done;
                        };
                        done.push((index, work(build)));
                    }
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().expect("a corpus thread panicked"))
            .collect()
    });
    done.sort_unstable_by_key(|(index, _)| *index);
    done.into_iter().map(|(_, result)| result).collect()
}
